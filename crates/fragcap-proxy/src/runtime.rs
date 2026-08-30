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

        // The sender is private and every command waits synchronously for its response, so safe
        // callers can have at most one queued command despite the channel's unbounded primitive.
        let (commands, receiver) = mpsc::unbounded_channel();
        let config = self.config.clone();
        let worker = thread::Builder::new()
            .name("fragcap-native-proxy".to_string())
            .spawn(move || runtime.block_on(run(listener, endpoint, config, receiver)))
            .map_err(|error| {
                StartError::new(
                    "runtime-thread-failed",
                    format!("cannot start native proxy owner thread: {error}"),
                )
            })?;

        Ok(NativeProxyLease {
            commands,
            worker: Some(worker),
            cached: None,
        })
    }
}

enum Command {
    Observe(std_mpsc::SyncSender<RuntimeObservation>),
    Stop {
        budget: Duration,
        response: std_mpsc::SyncSender<ShutdownReport>,
    },
}

pub struct NativeProxyLease {
    commands: mpsc::UnboundedSender<Command>,
    worker: Option<JoinHandle<ShutdownReport>>,
    cached: Option<ShutdownReport>,
}

impl NativeProxyLease {
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
        self.commands.send(Command::Observe(send)).map_err(|_| {
            ObserveError::new(
                "runtime-unavailable",
                "native proxy runtime stopped before observation",
            )
        })?;
        receive.recv_timeout(budget).map_err(|error| {
            ObserveError::new(
                "observe-timeout",
                format!("native proxy observation did not complete: {error}"),
            )
        })
    }

    pub fn stop(&mut self, budget: Duration) -> ShutdownReport {
        if let Some(report) = &self.cached {
            return report.clone();
        }

        let (send, receive) = std_mpsc::sync_channel(1);
        let command_sent = self
            .commands
            .send(Command::Stop {
                budget,
                response: send,
            })
            .is_ok();

        let wait = if budget.is_zero() {
            Duration::from_millis(1)
        } else {
            budget
        };
        let reported = command_sent
            .then(|| receive.recv_timeout(wait).ok())
            .flatten();
        let joined = self.worker.take().and_then(|worker| worker.join().ok());
        let report = reported
            .or(joined)
            .unwrap_or_else(runtime_thread_failure_report);
        self.cached = Some(report.clone());
        report
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
}

async fn connection_task(
    mut stream: TcpStream,
    mut shutdown: watch::Receiver<bool>,
    buffer_bytes: usize,
    _permit: OwnedSemaphorePermit,
) -> ConnectionOutcome {
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
    mut commands: mpsc::UnboundedReceiver<Command>,
) -> ShutdownReport {
    let permits = Arc::new(Semaphore::new(config.max_connections()));
    let (shutdown_send, shutdown_receive) = watch::channel(false);
    let mut tasks = JoinSet::new();
    let mut observation = RuntimeObservation::running(endpoint);
    let mut next_connection_id = 1_u64;
    let (stop_budget, stop_response) = loop {
        tokio::select! {
            biased;
            command = commands.recv() => {
                match command {
                    Some(Command::Observe(response)) => {
                        let _ = response.send(observation.clone());
                    }
                    Some(Command::Stop { budget, response }) => {
                        observation.state = LifecycleState::Stopping;
                        break (budget, Some(response));
                    }
                    None => {
                        observation.state = LifecycleState::Stopping;
                        break (config.shutdown_timeout(), None);
                    }
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
                                next_connection_id = next_connection_id.saturating_add(1);
                                observation.accepted_connections = observation.accepted_connections.saturating_add(1);
                                observation.live_connections += 1;
                                observation.peak_live_connections = observation
                                    .peak_live_connections
                                    .max(observation.live_connections);
                                tasks.spawn(async move {
                                    let outcome = connection_task(
                                        stream,
                                        connection_shutdown,
                                        buffer_bytes,
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
            joined = tasks.join_next(), if !tasks.is_empty() => {
                if let Some(result) = joined {
                    account_join(&mut observation, result);
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
    let report = ShutdownReport {
        joined_tasks: observation.accepted_connections.saturating_sub(incomplete),
        incomplete_tasks: incomplete,
        residue: incomplete > 0,
        listener_released: true,
        observation,
    };
    if let Some(response) = stop_response {
        let _ = response.send(report.clone());
    }
    report
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

fn runtime_thread_failure_report() -> ShutdownReport {
    let endpoint = "127.0.0.1:0".parse().expect("literal loopback endpoint");
    let mut observation = RuntimeObservation::running(endpoint);
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
}
