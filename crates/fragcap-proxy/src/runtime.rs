// SPDX-License-Identifier: Apache-2.0

use std::cmp::min;
use std::net::TcpListener as StdTcpListener;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc as std_mpsc, Arc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant as StdInstant};

use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Builder;
use tokio::sync::{mpsc, watch, Mutex, OwnedSemaphorePermit, Semaphore};
use tokio::task::{JoinError, JoinSet};
use tokio::time::{timeout, Instant};

use crate::http1::{
    read_authenticated_request, read_request, serve_http, HttpRun, ObservationContext,
};
use crate::tls::{
    accept_client_tls, client_server_config, connect_verified_tls, upstream_client_config_for_alpn,
};
use crate::{connect_upstream, ProtocolError};
use crate::{
    CapabilityProof, DestinationPolicy, LeafCache, SessionCapability, SessionCertificateAuthority,
};

static NEXT_AUTHORITY_GENERATION: AtomicU64 = AtomicU64::new(1);

use crate::{
    BackendCapabilities, BackendIdentity, BackendKind, LifecycleState, NativeProxyConfig,
    ObserveError, RuntimeFailure, RuntimeObservation, ShutdownReport, StartError,
};

pub struct NativeProxyBackend {
    config: NativeProxyConfig,
    destination_policy: Option<DestinationPolicy>,
    tls_client_config: Option<Arc<rustls::ClientConfig>>,
    application_sink: crate::application::SharedEventSink,
    key_log: Option<Arc<crate::SessionKeyLog>>,
}

impl NativeProxyBackend {
    pub fn new(config: NativeProxyConfig) -> Self {
        Self {
            config,
            destination_policy: None,
            tls_client_config: None,
            application_sink: None,
            key_log: None,
        }
    }

    pub fn with_destination_policy(mut self, policy: DestinationPolicy) -> Self {
        self.destination_policy = Some(policy);
        self
    }

    pub fn with_tls_client_config(mut self, config: Arc<rustls::ClientConfig>) -> Self {
        self.tls_client_config = Some(config);
        self
    }

    pub fn with_application_event_sink(
        mut self,
        sink: Arc<dyn crate::ApplicationEventSink>,
    ) -> Self {
        self.application_sink = Some(sink);
        self
    }

    pub fn with_key_log(mut self, key_log: Arc<crate::SessionKeyLog>) -> Self {
        self.key_log = Some(key_log);
        self
    }

    pub fn identity(&self) -> BackendIdentity {
        BackendIdentity {
            kind: BackendKind::NativeRust,
            name: "fragcap-native",
            version: env!("CARGO_PKG_VERSION"),
            capabilities: BackendCapabilities {
                foundation_listener: true,
                forwards_upstream: true,
                observes_http: true,
                inspects_tls: true,
            },
        }
    }

    pub fn start(&mut self, budget: Duration) -> Result<NativeProxyLease, StartError> {
        let started_at = StdInstant::now();
        self.config
            .protocol
            .validate()
            .map_err(|error| StartError::new(error.code, error.detail))?;
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
        let generation = NEXT_AUTHORITY_GENERATION.fetch_add(1, Ordering::Relaxed);
        let certificate_authority = Arc::new(
            SessionCertificateAuthority::generate(
                generation,
                std::time::SystemTime::now(),
                Duration::from_secs(24 * 60 * 60),
            )
            .map_err(|error| StartError::new(error.code, error.detail))?,
        );
        let ca_der = certificate_authority.der().to_vec();
        let ca_sha1_thumbprint = certificate_authority.sha1_thumbprint().to_string();
        let ca_sha256_fingerprint = certificate_authority.sha256_fingerprint().to_string();
        let leaf_cache = Arc::new(Mutex::new(
            LeafCache::new(
                self.config.protocol.leaf_cache_entries,
                self.config.protocol.leaf_cache_bytes,
                self.config.protocol.leaf_lifetime,
                generation,
            )
            .map_err(|error| StartError::new(error.code, error.detail))?,
        ));
        let tls_client_config = match self.tls_client_config.clone() {
            Some(config) => config,
            None => crate::native_tls_client_config()
                .map_err(|error| StartError::new(error.code, error.detail))?,
        };
        let (commands, receiver) = mpsc::channel(1);
        let (completion_send, completion) = std_mpsc::sync_channel(1);
        let config = self.config.clone();
        let destination_policy = self
            .destination_policy
            .clone()
            .unwrap_or_else(|| DestinationPolicy::new(endpoint));
        let services = RuntimeServices {
            endpoint,
            config,
            capability,
            destination_policy,
            certificate_authority,
            leaf_cache,
            tls_client_config,
            application_sink: self.application_sink.clone(),
            key_log: self.key_log.clone(),
            body_resources: crate::body::SessionBodyResources::new(
                self.config.protocol.max_concurrent_decoders,
            ),
        };
        let worker = thread::Builder::new()
            .name("fragcap-native-proxy".to_string())
            .spawn(move || {
                let report = runtime.block_on(run(listener, services, receiver));
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
            ca_der,
            ca_sha1_thumbprint,
            ca_sha256_fingerprint,
            authority_generation: generation,
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
    ca_der: Vec<u8>,
    ca_sha1_thumbprint: String,
    ca_sha256_fingerprint: String,
    authority_generation: u64,
}

impl NativeProxyLease {
    pub fn endpoint(&self) -> std::net::SocketAddr {
        self.last_observation.endpoint
    }

    pub fn capability_proof(&self) -> CapabilityProof {
        self.capability_proof.clone()
    }

    pub fn ca_der(&self) -> &[u8] {
        &self.ca_der
    }

    pub fn ca_sha1_thumbprint(&self) -> &str {
        &self.ca_sha1_thumbprint
    }

    pub fn ca_sha256_fingerprint(&self) -> &str {
        &self.ca_sha256_fingerprint
    }

    pub fn authority_generation(&self) -> u64 {
        self.authority_generation
    }

    pub fn proxy_url(&self) -> zeroize::Zeroizing<String> {
        self.capability_proof
            .proxy_url(self.last_observation.endpoint)
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

#[derive(Debug)]
enum ConnectionOutcome {
    Completed(HttpRun),
    Failed(HttpRun),
    AuthenticationRefused(ProtocolError),
}

#[derive(Clone)]
struct RuntimeServices {
    endpoint: std::net::SocketAddr,
    config: NativeProxyConfig,
    capability: SessionCapability,
    destination_policy: DestinationPolicy,
    certificate_authority: Arc<SessionCertificateAuthority>,
    leaf_cache: Arc<Mutex<LeafCache>>,
    tls_client_config: Arc<rustls::ClientConfig>,
    application_sink: crate::application::SharedEventSink,
    key_log: Option<Arc<crate::SessionKeyLog>>,
    body_resources: crate::body::SessionBodyResources,
}

#[derive(Clone, Copy)]
struct ConnectionIdentity {
    id: u64,
    peer: std::net::SocketAddr,
    local: std::net::SocketAddr,
}

async fn connection_task(
    mut stream: TcpStream,
    mut shutdown: watch::Receiver<bool>,
    services: RuntimeServices,
    identity: ConnectionIdentity,
    _permit: OwnedSemaphorePermit,
) -> ConnectionOutcome {
    let config = &services.config;
    let capability = &services.capability;
    let policy = &services.destination_policy;
    let connection_id = identity.id;
    let peer = identity.peer;
    let local = identity.local;
    let cleartext_http2 = tokio::select! {
        _ = shutdown.changed() => false,
        value = has_http2_preface(&stream, config.protocol.header_timeout) => value,
    };
    if cleartext_http2 {
        let run = crate::http2::serve_cleartext_http2(
            stream,
            capability.clone(),
            policy.clone(),
            crate::http2::Http2ConnectionContext {
                limits: config.protocol.clone(),
                session_id: config.session_id.clone(),
                connection_id,
                sink: services.application_sink.clone(),
                body_resources: services.body_resources.clone(),
            },
        )
        .await;
        if let Some(error) = run.failure.as_ref() {
            if error.authentication_refused {
                return ConnectionOutcome::AuthenticationRefused(error.clone());
            }
            return ConnectionOutcome::Failed(HttpRun {
                observations: Vec::new(),
                accounting: run.accounting,
                failure: run.failure,
            });
        }
        return ConnectionOutcome::Completed(HttpRun {
            observations: Vec::new(),
            accounting: run.accounting,
            failure: None,
        });
    }
    let first = tokio::select! {
        _ = shutdown.changed() => {
            return ConnectionOutcome::Failed(HttpRun {
                observations: Vec::new(),
                accounting: Default::default(),
                failure: Some(ProtocolError::new("connection-cancelled", "runtime is stopping")),
            });
        }
        result = read_authenticated_request(&mut stream, capability, config.protocol_limits()) => result,
    };
    let first = match first {
        Ok(Some(first)) => first,
        Ok(None) => {
            return ConnectionOutcome::AuthenticationRefused(ProtocolError::new(
                "proxy-auth-required",
                "client closed before proxy authorization",
            ))
        }
        Err(error) if error.authentication_refused => {
            return ConnectionOutcome::AuthenticationRefused(error)
        }
        Err(error) => {
            return ConnectionOutcome::Failed(HttpRun {
                observations: Vec::new(),
                accounting: Default::default(),
                failure: Some(error),
            })
        }
    };
    crate::application::emit(
        &services.application_sink,
        crate::ApplicationEvent::now(
            &config.session_id,
            connection_id,
            None,
            None,
            crate::ApplicationEventKind::ConnectionOpen,
        ),
    );
    if first.is_connect() {
        let authority = first.authority().clone();
        let upstream = match connect_upstream(&authority, policy, config.protocol.upstream).await {
            Ok(upstream) => upstream,
            Err(error) => {
                let policy_refused = matches!(error.stage, crate::UpstreamStage::Policy);
                let mut protocol = ProtocolError::new(error.code, error.detail);
                protocol.policy_refused = policy_refused;
                return ConnectionOutcome::Failed(HttpRun {
                    observations: vec![connect_observation(
                        &config.session_id,
                        connection_id,
                        peer,
                        local,
                        &first,
                        Some(protocol.code),
                    )],
                    accounting: crate::ProtocolAccounting {
                        requests: 1,
                        connect_requests: 1,
                        policy_refused: u64::from(protocol.policy_refused),
                        ..Default::default()
                    },
                    failure: Some(protocol),
                });
            }
        };
        if let Err(error) = stream
            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await
        {
            return ConnectionOutcome::Failed(HttpRun {
                observations: Vec::new(),
                accounting: Default::default(),
                failure: Some(ProtocolError::new(
                    "connect-response-failed",
                    error.to_string(),
                )),
            });
        }
        let server_config = {
            let mut cache = services.leaf_cache.lock().await;
            match client_server_config(
                &authority,
                &services.certificate_authority,
                &mut cache,
                services.key_log.clone(),
            ) {
                Ok(config) => config,
                Err(error) => {
                    return ConnectionOutcome::Failed(HttpRun {
                        observations: vec![connect_observation(
                            &config.session_id,
                            connection_id,
                            peer,
                            local,
                            &first,
                            Some(error.code),
                        )],
                        accounting: crate::ProtocolAccounting {
                            requests: 1,
                            connect_requests: 1,
                            ..Default::default()
                        },
                        failure: Some(error),
                    });
                }
            }
        };
        let mut client_tls =
            match accept_client_tls(stream, &authority, server_config, config.protocol_limits())
                .await
            {
                Ok(stream) => stream,
                Err(error) => {
                    return ConnectionOutcome::Failed(HttpRun {
                        observations: vec![
                            connect_observation(
                                &config.session_id,
                                connection_id,
                                peer,
                                local,
                                &first,
                                None,
                            ),
                            tls_observation(
                                &config.session_id,
                                connection_id,
                                peer,
                                local,
                                tls_negotiation(crate::TlsBoundary::Client, &authority, None, None),
                                Some(error.code),
                            ),
                        ],
                        accounting: crate::ProtocolAccounting {
                            requests: 1,
                            responses: 1,
                            connect_requests: 1,
                            timed_out: u64::from(error.timed_out),
                            ..Default::default()
                        },
                        failure: Some(error),
                    });
                }
            };
        let client_tls_facts = tls_negotiation(
            crate::TlsBoundary::Client,
            &authority,
            client_tls.get_ref().1.protocol_version(),
            client_tls.get_ref().1.alpn_protocol().map(<[u8]>::to_vec),
        );
        let client_alpn = client_tls
            .get_ref()
            .1
            .alpn_protocol()
            .unwrap_or(b"http/1.1")
            .to_vec();
        let selected_protocol = match crate::protocol::negotiated_protocol(Some(&client_alpn)) {
            Ok(protocol) => protocol,
            Err(code) => {
                return ConnectionOutcome::Failed(HttpRun {
                    observations: vec![connect_observation(
                        &config.session_id,
                        connection_id,
                        peer,
                        local,
                        &first,
                        Some(code),
                    )],
                    accounting: crate::ProtocolAccounting {
                        requests: 1,
                        connect_requests: 1,
                        client_tls_completed: 1,
                        ..Default::default()
                    },
                    failure: Some(ProtocolError::new(
                        code,
                        "client selected an unsupported application protocol",
                    )),
                });
            }
        };
        let upstream_config =
            upstream_client_config_for_alpn(&services.tls_client_config, client_alpn.clone());
        let upstream = match crate::upstream::connect_tls_over_upstream(
            &authority,
            upstream,
            config.protocol.upstream,
            upstream_config,
        )
        .await
        {
            Ok(stream) => stream,
            Err(error) => {
                let protocol = ProtocolError::new(error.code, error.detail);
                return ConnectionOutcome::Failed(HttpRun {
                    observations: vec![connect_observation(
                        &config.session_id,
                        connection_id,
                        peer,
                        local,
                        &first,
                        Some(protocol.code),
                    )],
                    accounting: crate::ProtocolAccounting {
                        requests: 1,
                        connect_requests: 1,
                        client_tls_completed: 1,
                        ..Default::default()
                    },
                    failure: Some(protocol),
                });
            }
        };
        let upstream_alpn = upstream
            .alpn_protocol()
            .unwrap_or_else(|| b"http/1.1".to_vec());
        if client_alpn != upstream_alpn {
            return ConnectionOutcome::Failed(HttpRun {
                observations: vec![connect_observation(
                    &config.session_id,
                    connection_id,
                    peer,
                    local,
                    &first,
                    Some("tls-alpn-mismatch"),
                )],
                accounting: crate::ProtocolAccounting {
                    requests: 1,
                    connect_requests: 1,
                    client_tls_completed: 1,
                    upstream_tls_completed: 1,
                    ..Default::default()
                },
                failure: Some(ProtocolError::new(
                    "tls-alpn-mismatch",
                    "client and origin selected different application protocols",
                )),
            });
        }
        let upstream_tls = tls_negotiation(
            crate::TlsBoundary::Upstream,
            &authority,
            upstream.protocol_version(),
            upstream.alpn_protocol(),
        );
        crate::application::emit(
            &services.application_sink,
            crate::ApplicationEvent::now(
                &config.session_id,
                connection_id,
                None,
                None,
                crate::ApplicationEventKind::TlsNegotiation(client_tls_facts.clone()),
            ),
        );
        crate::application::emit(
            &services.application_sink,
            crate::ApplicationEvent::now(
                &config.session_id,
                connection_id,
                None,
                None,
                crate::ApplicationEventKind::TlsNegotiation(upstream_tls.clone()),
            ),
        );
        if selected_protocol == crate::ProtocolVersion::Http2 {
            let h2 = crate::http2::serve_http2(
                client_tls,
                upstream,
                authority.clone(),
                crate::http2::Http2ConnectionContext {
                    limits: config.protocol.clone(),
                    session_id: config.session_id.clone(),
                    connection_id,
                    sink: services.application_sink.clone(),
                    body_resources: services.body_resources.clone(),
                },
            )
            .await;
            let mut run = HttpRun {
                observations: vec![
                    connect_observation(
                        &config.session_id,
                        connection_id,
                        peer,
                        local,
                        &first,
                        None,
                    ),
                    tls_observation(
                        &config.session_id,
                        connection_id,
                        peer,
                        local,
                        client_tls_facts,
                        None,
                    ),
                    tls_observation(
                        &config.session_id,
                        connection_id,
                        peer,
                        local,
                        upstream_tls,
                        None,
                    ),
                ],
                accounting: h2.accounting,
                failure: h2.failure,
            };
            run.accounting.connect_requests = run.accounting.connect_requests.saturating_add(1);
            run.accounting.client_tls_completed =
                run.accounting.client_tls_completed.saturating_add(1);
            run.accounting.upstream_tls_completed =
                run.accounting.upstream_tls_completed.saturating_add(1);
            return if run.failure.is_some() {
                ConnectionOutcome::Failed(run)
            } else {
                ConnectionOutcome::Completed(run)
            };
        }
        let inner = match read_request(&mut client_tls, config.protocol_limits()).await {
            Ok(Some(request)) if !request.is_connect() => request,
            Ok(Some(_)) => {
                return ConnectionOutcome::Failed(HttpRun {
                    observations: Vec::new(),
                    accounting: Default::default(),
                    failure: Some(ProtocolError::new(
                        "nested-connect-refused",
                        "nested CONNECT is not supported",
                    )),
                })
            }
            Ok(None) => {
                return ConnectionOutcome::Failed(HttpRun {
                    observations: Vec::new(),
                    accounting: Default::default(),
                    failure: Some(ProtocolError::new(
                        "client-tls-no-http",
                        "client TLS ended before an HTTP request",
                    )),
                })
            }
            Err(error) => {
                return ConnectionOutcome::Failed(HttpRun {
                    observations: Vec::new(),
                    accounting: Default::default(),
                    failure: Some(error),
                })
            }
        };
        let tunnel_authority = authority.clone();
        let mut first_upstream = Some(upstream);
        let limits = config.protocol.clone();
        let session_id = config.session_id.clone();
        let tunnel_tls_client_config =
            upstream_client_config_for_alpn(&services.tls_client_config, client_alpn);
        let mut run = serve_http(
            client_tls,
            inner,
            capability,
            &limits,
            ObservationContext {
                session_id: &session_id,
                connection_id,
                client_peer: peer,
                proxy_local: local,
                protocol: "https",
                application_sink: services.application_sink.clone(),
                body_resources: services.body_resources.clone(),
            },
            false,
            |requested| {
                let existing = first_upstream.take();
                let expected = tunnel_authority.clone();
                let policy = policy.clone();
                let limits = limits.clone();
                let tls_client_config = Arc::clone(&tunnel_tls_client_config);
                async move {
                    if requested != expected {
                        return Err(ProtocolError::new(
                            "tls-tunnel-authority-mismatch",
                            "inner HTTP authority differs from CONNECT authority",
                        ));
                    }
                    match existing {
                        Some(stream) => Ok(stream),
                        None => {
                            connect_verified_tls(requested, policy, limits, tls_client_config).await
                        }
                    }
                }
            },
        )
        .await;
        run.accounting.connect_requests = run.accounting.connect_requests.saturating_add(1);
        run.accounting.client_tls_completed = run.accounting.client_tls_completed.saturating_add(1);
        run.accounting.upstream_tls_completed =
            run.accounting.upstream_tls_completed.saturating_add(1);
        run.observations.splice(
            0..0,
            [
                connect_observation(&config.session_id, connection_id, peer, local, &first, None),
                tls_observation(
                    &config.session_id,
                    connection_id,
                    peer,
                    local,
                    client_tls_facts,
                    None,
                ),
                tls_observation(
                    &config.session_id,
                    connection_id,
                    peer,
                    local,
                    upstream_tls,
                    None,
                ),
            ],
        );
        return if run.failure.is_some() {
            ConnectionOutcome::Failed(run)
        } else {
            ConnectionOutcome::Completed(run)
        };
    }
    let session_id = config.session_id.clone();
    let limits = config.protocol.clone();
    let upstream_budgets = limits.upstream;
    let run = serve_http(
        stream,
        first,
        capability,
        &limits,
        ObservationContext {
            session_id: &session_id,
            connection_id,
            client_peer: peer,
            proxy_local: local,
            protocol: "http",
            application_sink: services.application_sink.clone(),
            body_resources: services.body_resources.clone(),
        },
        true,
        |authority| {
            let policy = policy.clone();
            async move {
                connect_upstream(&authority, &policy, upstream_budgets)
                    .await
                    .map_err(|error| {
                        let mut protocol = ProtocolError::new(error.code, error.detail);
                        protocol.policy_refused =
                            matches!(error.stage, crate::UpstreamStage::Policy);
                        protocol
                    })
            }
        },
    )
    .await;
    if run.failure.is_some() {
        ConnectionOutcome::Failed(run)
    } else {
        ConnectionOutcome::Completed(run)
    }
}

async fn has_http2_preface(stream: &TcpStream, budget: Duration) -> bool {
    const PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
    let deadline = Instant::now() + budget;
    let mut observed = [0_u8; 24];
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return false;
        }
        let read = match timeout(remaining, stream.peek(&mut observed)).await {
            Ok(Ok(read)) => read,
            Ok(Err(_)) | Err(_) => return false,
        };
        if read == 0 || observed[..read] != PREFACE[..read] {
            return false;
        }
        if read == PREFACE.len() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}

fn connect_observation(
    session_id: &str,
    connection_id: u64,
    peer: std::net::SocketAddr,
    local: std::net::SocketAddr,
    request: &crate::http1::RequestHead,
    reason: Option<&str>,
) -> crate::ProxyObservation {
    crate::ProxyObservation {
        session_id: session_id.to_string(),
        connection_id,
        request_ordinal: 1,
        client_peer: peer,
        proxy_local: local,
        timestamp_ns: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .try_into()
            .unwrap_or(u64::MAX),
        protocol: "connect".to_string(),
        method: Some(request.method().to_string()),
        url: Some(request.url().to_string()),
        status: reason.is_none().then_some(200),
        inspectability: "metadata-only",
        reason: reason.map(ToOwned::to_owned),
        tls: None,
        transformations: vec!["proxy-authorization-removed"],
    }
}

fn tls_observation(
    session_id: &str,
    connection_id: u64,
    peer: std::net::SocketAddr,
    local: std::net::SocketAddr,
    tls: crate::TlsNegotiation,
    reason: Option<&str>,
) -> crate::ProxyObservation {
    crate::ProxyObservation {
        session_id: session_id.to_string(),
        connection_id,
        request_ordinal: 1,
        client_peer: peer,
        proxy_local: local,
        timestamp_ns: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .try_into()
            .unwrap_or(u64::MAX),
        protocol: "tls".to_string(),
        method: None,
        url: None,
        status: None,
        inspectability: if reason.is_none() {
            "metadata-only"
        } else {
            "inconclusive"
        },
        reason: reason.map(ToOwned::to_owned),
        tls: Some(tls),
        transformations: Vec::new(),
    }
}

fn tls_negotiation(
    boundary: crate::TlsBoundary,
    authority: &crate::DestinationAuthority,
    version: Option<rustls::ProtocolVersion>,
    alpn: Option<Vec<u8>>,
) -> crate::TlsNegotiation {
    crate::TlsNegotiation {
        boundary,
        requested_identity: authority.lookup_host(),
        version: version.map(|value| match value {
            rustls::ProtocolVersion::TLSv1_2 => "TLS1.2".to_string(),
            rustls::ProtocolVersion::TLSv1_3 => "TLS1.3".to_string(),
            other => format!("{other:?}"),
        }),
        alpn,
    }
}

async fn run(
    listener: TcpListener,
    services: RuntimeServices,
    mut commands: mpsc::Receiver<Command>,
) -> ShutdownReport {
    let endpoint = services.endpoint;
    let config = &services.config;
    let permits = Arc::new(Semaphore::new(config.max_connections()));
    let (shutdown_send, shutdown_receive) = watch::channel(false);
    let mut tasks = JoinSet::new();
    let mut observation = RuntimeObservation::running(endpoint);
    let mut next_connection_id = 1_u64;
    let stop_budget = loop {
        while let Some(result) = tasks.try_join_next() {
            account_join(
                &mut observation,
                result,
                config.protocol_limits().max_observations,
            );
        }
        observation.live_connections = config
            .max_connections()
            .saturating_sub(permits.available_permits());

        tokio::select! {
            biased;
            command = commands.recv() => {
                match command {
                    Some(Command::Observe(response)) => {
                        sync_application_accounting(&mut observation, &services.application_sink);
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
                    account_join(
                        &mut observation,
                        result,
                        config.protocol_limits().max_observations,
                    );
                    observation.live_connections = config
                        .max_connections()
                        .saturating_sub(permits.available_permits());
                }
            }
            accepted = listener.accept() => {
                match accepted {
                    Ok((stream, peer)) => {
                        match Arc::clone(&permits).try_acquire_owned() {
                            Ok(permit) => {
                                let connection_id = next_connection_id;
                                let connection_shutdown = shutdown_receive.clone();
                                let connection_services = services.clone();
                                let local = stream.local_addr().unwrap_or(endpoint);
                                next_connection_id = next_connection_id.saturating_add(1);
                                observation.accepted_connections = observation.accepted_connections.saturating_add(1);
                                observation.live_connections = config
                                    .max_connections()
                                    .saturating_sub(permits.available_permits());
                                observation.peak_live_connections = observation
                                    .peak_live_connections
                                    .max(observation.live_connections);
                                tasks.spawn(async move {
                                    let terminal_sink = connection_services.application_sink.clone();
                                    let terminal_session = connection_services.config.session_id.clone();
                                    let outcome = connection_task(
                                        stream,
                                        connection_shutdown,
                                        connection_services,
                                        ConnectionIdentity { id: connection_id, peer, local },
                                        permit,
                                    ).await;
                                    let terminal = match &outcome {
                                        ConnectionOutcome::Completed(_) => crate::StreamTerminal::Complete,
                                        ConnectionOutcome::AuthenticationRefused(_) => crate::StreamTerminal::Refused,
                                        ConnectionOutcome::Failed(run) if run.failure.as_ref().is_some_and(|error| error.timed_out) => crate::StreamTerminal::IdleTimeout,
                                        ConnectionOutcome::Failed(_) => crate::StreamTerminal::ProtocolError,
                                    };
                                    crate::application::emit(
                                        &terminal_sink,
                                        crate::ApplicationEvent::now(
                                            terminal_session,
                                            connection_id,
                                            None,
                                            None,
                                            crate::ApplicationEventKind::ConnectionTerminal(terminal),
                                        ),
                                    );
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
    drain_tasks(
        &mut tasks,
        &mut observation,
        drain_budget,
        config.protocol_limits().max_observations,
    )
    .await;
    observation.live_connections = 0;
    observation.state = LifecycleState::Stopped;
    sync_application_accounting(&mut observation, &services.application_sink);

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

fn sync_application_accounting(
    observation: &mut RuntimeObservation,
    sink: &crate::application::SharedEventSink,
) {
    let accounting = sink
        .as_ref()
        .map_or_else(Default::default, |sink| sink.accounting());
    observation.protocol.application_events_accepted = accounting.accepted_events;
    observation.protocol.application_events_dropped = accounting.dropped_events;
    observation.protocol.body_bytes_queue_dropped = accounting.body_bytes_queue_dropped;
    observation.protocol.streaming_bytes_queue_dropped = accounting.streaming_bytes_queue_dropped;
}

async fn drain_tasks(
    tasks: &mut JoinSet<(u64, ConnectionOutcome)>,
    observation: &mut RuntimeObservation,
    budget: Duration,
    max_observations: usize,
) {
    let deadline = Instant::now() + budget;
    while !tasks.is_empty() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        match timeout(remaining, tasks.join_next()).await {
            Ok(Some(result)) => account_join(observation, result, max_observations),
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
    max_observations: usize,
) {
    observation.live_connections = observation.live_connections.saturating_sub(1);
    match result {
        Ok((_id, ConnectionOutcome::Completed(run))) => {
            observation.authenticated_connections =
                observation.authenticated_connections.saturating_add(1);
            observation.completed_connections = observation.completed_connections.saturating_add(1);
            merge_protocol(observation, run, max_observations);
        }
        Ok((_id, ConnectionOutcome::AuthenticationRefused(error))) => {
            observation.authentication_refused =
                observation.authentication_refused.saturating_add(1);
            observation.completed_connections = observation.completed_connections.saturating_add(1);
            observation.protocol.parse_refused =
                observation.protocol.parse_refused.saturating_add(1);
            observation.failures.push(RuntimeFailure {
                code: error.code,
                detail: error.detail,
                connection_id: None,
            });
        }
        Ok((id, ConnectionOutcome::Failed(run))) => {
            observation.authenticated_connections =
                observation.authenticated_connections.saturating_add(1);
            observation.failed_connections = observation.failed_connections.saturating_add(1);
            let failure = run.failure.clone();
            merge_protocol(observation, run, max_observations);
            let failure = failure
                .unwrap_or_else(|| ProtocolError::new("connection-io-failed", "connection failed"));
            observation.failures.push(RuntimeFailure {
                code: failure.code,
                detail: failure.detail,
                connection_id: Some(id),
            });
        }
        Err(error) => {
            observation.failed_connections = observation.failed_connections.saturating_add(1);
            observation.failures.push(join_failure(error, None));
        }
    }
}

fn merge_protocol(observation: &mut RuntimeObservation, run: HttpRun, max_observations: usize) {
    let source = run.accounting;
    observation.protocol.requests = observation
        .protocol
        .requests
        .saturating_add(source.requests);
    observation.protocol.responses = observation
        .protocol
        .responses
        .saturating_add(source.responses);
    observation.protocol.informational_responses = observation
        .protocol
        .informational_responses
        .saturating_add(source.informational_responses);
    observation.protocol.connect_requests = observation
        .protocol
        .connect_requests
        .saturating_add(source.connect_requests);
    observation.protocol.client_tls_completed = observation
        .protocol
        .client_tls_completed
        .saturating_add(source.client_tls_completed);
    observation.protocol.upstream_tls_completed = observation
        .protocol
        .upstream_tls_completed
        .saturating_add(source.upstream_tls_completed);
    observation.protocol.parse_refused = observation
        .protocol
        .parse_refused
        .saturating_add(source.parse_refused);
    observation.protocol.policy_refused = observation
        .protocol
        .policy_refused
        .saturating_add(source.policy_refused);
    observation.protocol.timed_out = observation
        .protocol
        .timed_out
        .saturating_add(source.timed_out);
    observation.protocol.observations_dropped_oldest = observation
        .protocol
        .observations_dropped_oldest
        .saturating_add(source.observations_dropped_oldest);
    observation.protocol.http2_streams = observation
        .protocol
        .http2_streams
        .saturating_add(source.http2_streams);
    observation.protocol.http2_streams_completed = observation
        .protocol
        .http2_streams_completed
        .saturating_add(source.http2_streams_completed);
    observation.protocol.http2_streams_reset = observation
        .protocol
        .http2_streams_reset
        .saturating_add(source.http2_streams_reset);
    observation.protocol.metadata_blocks = observation
        .protocol
        .metadata_blocks
        .saturating_add(source.metadata_blocks);
    observation.protocol.body_bytes_observed = observation
        .protocol
        .body_bytes_observed
        .saturating_add(source.body_bytes_observed);
    observation.protocol.body_bytes_retained = observation
        .protocol
        .body_bytes_retained
        .saturating_add(source.body_bytes_retained);
    observation.protocol.body_bytes_omitted = observation
        .protocol
        .body_bytes_omitted
        .saturating_add(source.body_bytes_omitted);
    observation.protocol.body_bytes_truncated = observation
        .protocol
        .body_bytes_truncated
        .saturating_add(source.body_bytes_truncated);
    observation.protocol.body_bytes_queue_dropped = observation
        .protocol
        .body_bytes_queue_dropped
        .saturating_add(source.body_bytes_queue_dropped);
    observation.protocol.streaming_bytes_queue_dropped = observation
        .protocol
        .streaming_bytes_queue_dropped
        .saturating_add(source.streaming_bytes_queue_dropped);
    observation.protocol.application_events_accepted = observation
        .protocol
        .application_events_accepted
        .saturating_add(source.application_events_accepted);
    observation.protocol.application_events_dropped = observation
        .protocol
        .application_events_dropped
        .saturating_add(source.application_events_dropped);
    observation.application.extend(run.observations);
    let excess = observation
        .application
        .len()
        .saturating_sub(max_observations);
    if excess > 0 {
        observation.application.drain(..excess);
        observation.protocol.observations_dropped_oldest = observation
            .protocol
            .observations_dropped_oldest
            .saturating_add(excess as u64);
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
                (1, completed_run())
            });
            let mut observed = observation();
            drain_tasks(&mut tasks, &mut observed, Duration::ZERO, 4_096).await;
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
                (1, completed_run())
            });
            let mut observed = observation();
            drain_tasks(&mut tasks, &mut observed, Duration::from_secs(1), 4_096).await;
            assert!(tasks.is_empty());
            assert_eq!(observed.failed_connections, 1);
            assert_eq!(observed.failures[0].code, "connection-task-panicked");
        });
    }

    fn completed_run() -> ConnectionOutcome {
        ConnectionOutcome::Completed(HttpRun {
            observations: Vec::new(),
            accounting: Default::default(),
            failure: None,
        })
    }

    #[test]
    fn application_observations_drop_oldest_and_count_every_eviction() {
        let mut observed = RuntimeObservation::running("127.0.0.1:40002".parse().unwrap());
        let make = |connection_id| crate::ProxyObservation {
            session_id: "bounded".to_string(),
            connection_id,
            request_ordinal: 1,
            client_peer: "127.0.0.1:41000".parse().unwrap(),
            proxy_local: "127.0.0.1:40002".parse().unwrap(),
            timestamp_ns: connection_id,
            protocol: "http".to_string(),
            method: Some("GET".to_string()),
            url: Some("http://example.test/".to_string()),
            status: Some(200),
            inspectability: "full",
            reason: None,
            tls: None,
            transformations: Vec::new(),
        };
        merge_protocol(
            &mut observed,
            HttpRun {
                observations: vec![make(1), make(2), make(3)],
                accounting: Default::default(),
                failure: None,
            },
            2,
        );
        assert_eq!(
            observed
                .application
                .iter()
                .map(|item| item.connection_id)
                .collect::<Vec<_>>(),
            vec![2, 3]
        );
        assert_eq!(observed.protocol.observations_dropped_oldest, 1);
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
            ca_der: Vec::new(),
            ca_sha1_thumbprint: String::new(),
            ca_sha256_fingerprint: String::new(),
            authority_generation: 0,
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
