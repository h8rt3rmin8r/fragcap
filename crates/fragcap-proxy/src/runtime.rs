// SPDX-License-Identifier: Apache-2.0

use std::cmp::min;
use std::net::TcpListener as StdTcpListener;
use std::sync::{mpsc as std_mpsc, Arc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant as StdInstant};

use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Builder;
use tokio::sync::{mpsc, watch, OwnedSemaphorePermit, Semaphore};
use tokio::task::{JoinError, JoinSet};
use tokio::time::{timeout, Instant};

use crate::{CapabilityProof, SessionCapability, CAPABILITY_BYTES};

use crate::{
    BackendCapabilities, BackendIdentity, BackendKind, LifecycleState, NativeProxyConfig,
    ObserveError, RuntimeFailure, RuntimeObservation, ShutdownReport, StartError,
};

pub struct NativeProxyBackend {
    config: NativeProxyConfig,
}

impl NativeProxyBackend {
    pub fn new(config: NativeProxyConfig) -> Self {
        Self { config }
    }

    pub fn identity(&self) -> BackendIdentity {
        BackendIdentity {
            kind: BackendKind::NativeRust,
            name: "fragcap-native",
            version: env!("CARGO_PKG_VERSION"),
            capabilities: BackendCapabilities {
                foundation_listener: true,
                forwards_upstream: false,
                observes_http: false,
                inspects_tls: false,
            },
        }
    }

    pub fn start(&mut self, budget: Duration) -> Result<NativeProxyLease, StartError> {
        let started_at = StdInstant::now();
        if budget.is_zero() {
            return Err(StartError::new(
                "start-budget-exhausted",
                "native proxy start budget was exhausted before binding",
            ));
        }

        let listener = StdTcpListener::bind(self.config.listen).map_err(|error| {
            StartError::new(
                "listener-bind-failed",
                format!("cannot bind {}: {error}", self.config.listen),
            )
        })?;
        listener.set_nonblocking(true).map_err(|error| {
            StartError::new(
                "listener-nonblocking-failed",
                format!("cannot configure {}: {error}", self.config.listen),
            )
        })?;
        let endpoint = listener.local_addr().map_err(|error| {
            StartError::new(
                "listener-address-failed",
                format!("cannot read bound endpoint: {error}"),
            )
        })?;

        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_io()
            .enable_time()
            .build()
            .map_err(|error| {
                StartError::new(
                    "runtime-build-failed",
                    format!("cannot build native proxy runtime: {error}"),
                )
            })?;

        let listener = {
            let _runtime_context = runtime.enter();
            TcpListener::from_std(listener).map_err(|error| {
                StartError::new(
                    "listener-runtime-conversion-failed",
                    format!("cannot attach {endpoint} to the native runtime: {error}"),
                )
            })?
        };

        if started_at.elapsed() >= budget {
            return Err(StartError::new(
                "start-budget-exhausted",
                "native proxy start budget was exhausted before the owner thread started",
            ));
        }

        // The lease is the only sender and the one-slot channel keeps timed-out commands finite.
        let capability = SessionCapability::generate()
            .map_err(|error| StartError::new("capability-generation-failed", error.to_string()))?;
        let capability_proof = capability.proof();
        let (commands, receiver) = mpsc::channel(1);
        let (completion_send, completion) = std_mpsc::sync_channel(1);
        let config = self.config.clone();
        let worker = thread::Builder::new()
            .name("fragcap-native-proxy".to_string())
            .spawn(move || {
                let report =
                    runtime.block_on(run(listener, endpoint, config, capability, receiver));
                drop(runtime);
                let _ = completion_send.send(report.clone());
                report
            })
            .map_err(|error| {
                StartError::new(
                    "runtime-thread-failed",
                    format!("cannot start native proxy owner thread: {error}"),
                )
            })?;

        Ok(NativeProxyLease {
            commands,
            completion,
            worker: Some(worker),
            cached: None,
            pending_report: None,
            last_observation: RuntimeObservation::running(endpoint),
            capability_proof,
        })
    }
}

enum Command {
    Observe(std_mpsc::SyncSender<RuntimeObservation>),
    Stop { budget: Duration },
}

pub struct NativeProxyLease {
    commands: mpsc::Sender<Command>,
    completion: std_mpsc::Receiver<ShutdownReport>,
    worker: Option<JoinHandle<ShutdownReport>>,
    cached: Option<ShutdownReport>,
    pending_report: Option<ShutdownReport>,
    last_observation: RuntimeObservation,
    capability_proof: CapabilityProof,
}

impl NativeProxyLease {
    pub fn capability_proof(&self) -> CapabilityProof {
        self.capability_proof.clone()
    }

    pub fn observation(&mut self, budget: Duration) -> Result<RuntimeObservation, ObserveError> {
        if let Some(report) = &self.cached {
            return Ok(report.observation.clone());
        }
        if budget.is_zero() {
            return Err(ObserveError::new(
                "observe-budget-exhausted",
                "native proxy observation budget is zero",
            ));
        }
        let (send, receive) = std_mpsc::sync_channel(1);
        self.commands
            .try_send(Command::Observe(send))
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => ObserveError::new(
                    "runtime-busy",
                    "native proxy runtime still owns an earlier command",
                ),
                mpsc::error::TrySendError::Closed(_) => ObserveError::new(
                    "runtime-unavailable",
                    "native proxy runtime stopped before observation",
                ),
            })?;
        let observation = receive.recv_timeout(budget).map_err(|error| {
            ObserveError::new(
                "observe-timeout",
                format!("native proxy observation did not complete: {error}"),
            )
        })?;
        self.last_observation = observation.clone();
        Ok(observation)
    }

    pub fn stop(&mut self, budget: Duration) -> ShutdownReport {
        if let Some(report) = &self.cached {
            return report.clone();
        }

        let started_at = StdInstant::now();
        if self.pending_report.is_none() {
            let _ = self.commands.try_send(Command::Stop { budget });
            let remaining = budget.saturating_sub(started_at.elapsed());
            if let Ok(report) = self.completion.recv_timeout(remaining) {
                self.last_observation = report.observation.clone();
                self.pending_report = Some(report);
            }
        }

        while self
            .worker
            .as_ref()
            .is_some_and(|worker| !worker.is_finished())
            && started_at.elapsed() < budget
        {
            thread::yield_now();
        }

        if self.worker.as_ref().is_some_and(JoinHandle::is_finished) {
            let joined = self.worker.take().expect("checked owner thread").join();
            let report = match joined {
                Ok(report) => report,
                Err(_) => runtime_thread_failure_report(self.last_observation.clone()),
            };
            self.pending_report = None;
            self.cached = Some(report.clone());
            return report;
        }

        owner_thread_timeout_report(self.pending_report.as_ref(), self.last_observation.clone())
    }

    pub fn cleanup(&mut self, budget: Duration) -> ShutdownReport {
        self.stop(budget)
    }
}

impl Drop for NativeProxyLease {
    fn drop(&mut self) {
        if self.cached.is_none() {
            let _ = self.stop(Duration::from_secs(1));
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ConnectionOutcome {
    Completed,
    Failed,
    AuthenticationRefused,
}

async fn connection_task(
    mut stream: TcpStream,
    mut shutdown: watch::Receiver<bool>,
    buffer_bytes: usize,
    capability: SessionCapability,
    _permit: OwnedSemaphorePermit,
) -> ConnectionOutcome {
    let mut proof = [0_u8; CAPABILITY_BYTES];
    let authenticated = tokio::select! {
        _ = shutdown.changed() => false,
        result = timeout(Duration::from_secs(1), stream.read_exact(&mut proof)) => {
            matches!(result, Ok(Ok(_))) && capability.authenticates(&proof)
        }
    };
    if !authenticated {
        return ConnectionOutcome::AuthenticationRefused;
    }
    let mut buffer = vec![0_u8; buffer_bytes];
    tokio::select! {
        changed = shutdown.changed() => {
            if changed.is_err() {
                ConnectionOutcome::Failed
            } else {
                ConnectionOutcome::Completed
            }
        }
        result = stream.read(&mut buffer) => {
            match result {
                Ok(_) => ConnectionOutcome::Completed,
                Err(_) => ConnectionOutcome::Failed,
            }
        }
    }
}

async fn run(
    listener: TcpListener,
    endpoint: std::net::SocketAddr,
    config: NativeProxyConfig,
    capability: SessionCapability,
    mut commands: mpsc::Receiver<Command>,
) -> ShutdownReport {
    let permits = Arc::new(Semaphore::new(config.max_connections()));
    let (shutdown_send, shutdown_receive) = watch::channel(false);
    let mut tasks = JoinSet::new();
    let mut observation = RuntimeObservation::running(endpoint);
    let mut next_connection_id = 1_u64;
    let stop_budget = loop {
        while let Some(result) = tasks.try_join_next() {
            account_join(&mut observation, result);
        }
        observation.live_connections = config
            .max_connections()
            .saturating_sub(permits.available_permits());

        tokio::select! {
            biased;
            command = commands.recv() => {
                match command {
                    Some(Command::Observe(response)) => {
                        let _ = response.send(observation.clone());
                    }
                    Some(Command::Stop { budget }) => {
                        observation.state = LifecycleState::Stopping;
                        break budget;
                    }
                    None => {
                        observation.state = LifecycleState::Stopping;
                        break config.shutdown_timeout();
                    }
                }
            }
            joined = tasks.join_next(), if !tasks.is_empty() => {
                if let Some(result) = joined {
                    account_join(&mut observation, result);
                    observation.live_connections = config
                        .max_connections()
                        .saturating_sub(permits.available_permits());
                }
            }
            accepted = listener.accept() => {
                match accepted {
                    Ok((stream, _peer)) => {
                        match Arc::clone(&permits).try_acquire_owned() {
                            Ok(permit) => {
                                let connection_id = next_connection_id;
                                let connection_shutdown = shutdown_receive.clone();
                                let buffer_bytes = config.per_connection_buffer_bytes();
                                let connection_capability = capability.clone();
                                next_connection_id = next_connection_id.saturating_add(1);
                                observation.accepted_connections = observation.accepted_connections.saturating_add(1);
                                observation.live_connections = config
                                    .max_connections()
                                    .saturating_sub(permits.available_permits());
                                observation.peak_live_connections = observation
                                    .peak_live_connections
                                    .max(observation.live_connections);
                                tasks.spawn(async move {
                                    let outcome = connection_task(
                                        stream,
                                        connection_shutdown,
                                        buffer_bytes,
                                        connection_capability,
                                        permit,
                                    ).await;
                                    (connection_id, outcome)
                                });
                            }
                            Err(_) => {
                                observation.saturated_connections = observation.saturated_connections.saturating_add(1);
                                drop(stream);
                            }
                        }
                    }
                    Err(error) => {
                        observation.failures.push(RuntimeFailure {
                            code: "listener-accept-failed",
                            detail: error.to_string(),
                            connection_id: None,
                        });
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }
            }
        }
    };

    drop(listener);
    let _ = shutdown_send.send(true);
    let drain_budget = min(stop_budget, config.shutdown_timeout());
    drain_tasks(&mut tasks, &mut observation, drain_budget).await;
    observation.live_connections = 0;
    observation.state = LifecycleState::Stopped;

    let accounted = observation.completed_connections
        + observation.failed_connections
        + observation.forced_connections;
    let incomplete = observation.accepted_connections.saturating_sub(accounted);
    if incomplete > 0 {
        observation.failures.push(RuntimeFailure {
            code: "connection-accounting-incomplete",
            detail: format!("{incomplete} accepted connection task(s) have no terminal outcome"),
            connection_id: None,
        });
    }
    ShutdownReport {
        joined_tasks: observation.accepted_connections.saturating_sub(incomplete),
        incomplete_tasks: incomplete,
        residue: incomplete > 0,
        listener_released: true,
        observation,
    }
}

async fn drain_tasks(
    tasks: &mut JoinSet<(u64, ConnectionOutcome)>,
    observation: &mut RuntimeObservation,
    budget: Duration,
) {
    let deadline = Instant::now() + budget;
    while !tasks.is_empty() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        match timeout(remaining, tasks.join_next()).await {
            Ok(Some(result)) => account_join(observation, result),
            Ok(None) => break,
            Err(_) => break,
        }
    }

    let forced = tasks.len() as u64;
    observation.forced_connections = observation.forced_connections.saturating_add(forced);
    tasks.abort_all();
    while let Some(result) = tasks.join_next().await {
        if let Err(error) = result {
            if !error.is_cancelled() {
                observation.failures.push(join_failure(error, None));
                observation.failed_connections = observation.failed_connections.saturating_add(1);
                observation.forced_connections = observation.forced_connections.saturating_sub(1);
            }
        }
    }
}

fn account_join(
    observation: &mut RuntimeObservation,
    result: Result<(u64, ConnectionOutcome), JoinError>,
) {
    observation.live_connections = observation.live_connections.saturating_sub(1);
    match result {
        Ok((_id, ConnectionOutcome::Completed)) => {
            observation.authenticated_connections =
                observation.authenticated_connections.saturating_add(1);
            observation.completed_connections = observation.completed_connections.saturating_add(1);
        }
        Ok((_id, ConnectionOutcome::AuthenticationRefused)) => {
            observation.authentication_refused =
                observation.authentication_refused.saturating_add(1);
            observation.completed_connections = observation.completed_connections.saturating_add(1);
        }
        Ok((id, ConnectionOutcome::Failed)) => {
            observation.failed_connections = observation.failed_connections.saturating_add(1);
            observation.failures.push(RuntimeFailure {
                code: "connection-io-failed",
                detail: "connection closed after an I/O failure".to_string(),
                connection_id: Some(id),
            });
        }
        Err(error) => {
            observation.failed_connections = observation.failed_connections.saturating_add(1);
            observation.failures.push(join_failure(error, None));
        }
    }
}

fn join_failure(error: JoinError, connection_id: Option<u64>) -> RuntimeFailure {
    RuntimeFailure {
        code: if error.is_panic() {
            "connection-task-panicked"
        } else {
            "connection-task-cancelled"
        },
        detail: error.to_string(),
        connection_id,
    }
}

fn runtime_thread_failure_report(mut observation: RuntimeObservation) -> ShutdownReport {
    observation.state = LifecycleState::Stopped;
    observation.failures.push(RuntimeFailure {
        code: "runtime-thread-failed",
        detail: "native proxy owner thread ended without a terminal report".to_string(),
        connection_id: None,
    });
    ShutdownReport {
        observation,
        listener_released: false,
        joined_tasks: 0,
        incomplete_tasks: 0,
        residue: true,
    }
}

fn owner_thread_timeout_report(
    pending: Option<&ShutdownReport>,
    mut observation: RuntimeObservation,
) -> ShutdownReport {
    let (listener_released, joined_tasks, incomplete_tasks) = pending
        .map(|report| {
            (
                report.listener_released,
                report.joined_tasks,
                report.incomplete_tasks,
            )
        })
        .unwrap_or((false, 0, observation.live_connections as u64));
    observation.state = LifecycleState::Stopping;
    observation.failures.push(RuntimeFailure {
        code: "owner-thread-join-timeout",
        detail: "native proxy owner thread did not finish within the cleanup budget".to_string(),
        connection_id: None,
    });
    ShutdownReport {
        observation,
        listener_released,
        joined_tasks,
        incomplete_tasks,
        residue: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation() -> RuntimeObservation {
        let endpoint = "127.0.0.1:40000".parse().unwrap();
        let mut observation = RuntimeObservation::running(endpoint);
        observation.accepted_connections = 1;
        observation.live_connections = 1;
        observation.peak_live_connections = 1;
        observation
    }

    #[test]
    fn forced_timeout_aborts_and_joins_every_task() {
        let runtime = Builder::new_current_thread().enable_time().build().unwrap();
        runtime.block_on(async {
            let mut tasks = JoinSet::new();
            tasks.spawn(async {
                std::future::pending::<()>().await;
                (1, ConnectionOutcome::Completed)
            });
            let mut observed = observation();
            drain_tasks(&mut tasks, &mut observed, Duration::ZERO).await;
            assert!(tasks.is_empty());
            assert_eq!(observed.forced_connections, 1);
            assert!(observed.failures.is_empty());
        });
    }

    #[test]
    fn panicked_task_is_preserved_and_accounted() {
        let runtime = Builder::new_current_thread().enable_time().build().unwrap();
        runtime.block_on(async {
            let mut tasks = JoinSet::new();
            tasks.spawn(async {
                panic!("controlled connection panic");
                #[allow(unreachable_code)]
                (1, ConnectionOutcome::Completed)
            });
            let mut observed = observation();
            drain_tasks(&mut tasks, &mut observed, Duration::from_secs(1)).await;
            assert!(tasks.is_empty());
            assert_eq!(observed.failed_connections, 1);
            assert_eq!(observed.failures[0].code, "connection-task-panicked");
        });
    }

    #[test]
    fn stop_returns_at_its_budget_and_a_later_cleanup_joins_the_owner() {
        let endpoint = "127.0.0.1:40001".parse().unwrap();
        let running = RuntimeObservation::running(endpoint);
        let (commands, receiver) = mpsc::channel(1);
        let (completion_send, completion) = std_mpsc::sync_channel(1);
        let worker_running = running.clone();
        let worker = thread::spawn(move || {
            let _receiver = receiver;
            thread::sleep(Duration::from_millis(100));
            let mut observation = worker_running;
            observation.state = LifecycleState::Stopped;
            let report = ShutdownReport {
                observation,
                listener_released: true,
                joined_tasks: 0,
                incomplete_tasks: 0,
                residue: false,
            };
            let _ = completion_send.send(report.clone());
            report
        });
        let mut lease = NativeProxyLease {
            commands,
            completion,
            worker: Some(worker),
            cached: None,
            pending_report: None,
            last_observation: running,
            capability_proof: SessionCapability::generate().unwrap().proof(),
        };

        let started_at = StdInstant::now();
        let timed_out = lease.stop(Duration::from_millis(1));
        assert!(started_at.elapsed() < Duration::from_millis(50));
        assert!(timed_out.residue);
        assert_eq!(
            timed_out.observation.failures.last().unwrap().code,
            "owner-thread-join-timeout"
        );

        thread::sleep(Duration::from_millis(125));
        let cleaned = lease.cleanup(Duration::from_secs(1));
        assert!(cleaned.is_clean(), "{cleaned:?}");
        assert!(lease.worker.is_none());
    }
}
