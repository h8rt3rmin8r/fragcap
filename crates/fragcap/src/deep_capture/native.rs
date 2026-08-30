// SPDX-License-Identifier: Apache-2.0

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use fragcap_proxy::{
    tls_client_config_with_roots, DestinationPolicy, NativeProxyBackend as RuntimeBackend,
    NativeProxyConfig, ShutdownReport,
};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName};

use fragcap_core::{Fidelity, FlowKey, FlowRegistry, Proto};

pub use fragcap_proxy::{
    CertificateStore, NativeCertificateStore, TrustController, TrustError, TrustMutation,
    TrustState, CURRENT_USER_ROOT, LOCAL_MACHINE_ROOT,
};

use super::{
    BackendDescriptor, Budget, CleanupResult, CleanupStatus, CompatibilityObservation,
    Inspectability, LoopbackEndpoint, ProxyBackend, ProxyLease, ProxyRoute, SessionPlan, Stage,
    StageFailure,
};

/// Finite native runtime limits selected by the library consumer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeProxyLimits {
    pub max_connections: usize,
    pub per_connection_buffer_bytes: usize,
    pub shutdown_timeout: Duration,
}

impl Default for NativeProxyLimits {
    fn default() -> Self {
        Self {
            max_connections: 128,
            per_connection_buffer_bytes: 16 * 1024,
            shutdown_timeout: Duration::from_secs(10),
        }
    }
}

/// Library-owned native implementation of the Deep Capture proxy seam.
///
/// Production native implementation of the Deep Capture proxy seam.
pub struct NativeProxyAdapter {
    limits: NativeProxyLimits,
    observation_context: NativeObservationContext,
}

/// Session-owned bridge between packet truth and native proxy observations.
#[derive(Clone, Debug, Default)]
pub struct NativeObservationContext {
    flow_registry: Arc<FlowRegistry>,
    controlled_process_id: Arc<AtomicU32>,
}

impl NativeObservationContext {
    /// The registry ordinary Capture must populate for this session.
    pub fn flow_registry(&self) -> Arc<FlowRegistry> {
        Arc::clone(&self.flow_registry)
    }

    /// Record the exact controlled child process after it has launched.
    pub fn record_controlled_process_id(&self, process_id: u32) {
        self.controlled_process_id
            .store(process_id, Ordering::Release);
    }

    fn controlled_process_id(&self) -> Option<u32> {
        match self.controlled_process_id.load(Ordering::Acquire) {
            0 => None,
            process_id => Some(process_id),
        }
    }
}

impl NativeProxyAdapter {
    pub fn new(limits: NativeProxyLimits) -> Self {
        Self {
            limits,
            observation_context: NativeObservationContext::default(),
        }
    }

    pub fn limits(&self) -> NativeProxyLimits {
        self.limits
    }

    /// Use packet and process truth shared with this session's capture runner.
    pub fn with_observation_context(mut self, context: NativeObservationContext) -> Self {
        self.observation_context = context;
        self
    }
}

impl Default for NativeProxyAdapter {
    fn default() -> Self {
        Self::new(NativeProxyLimits::default())
    }
}

impl ProxyBackend for NativeProxyAdapter {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            name: "fragcap-native".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn start(
        &mut self,
        plan: &SessionPlan,
        budget: Budget,
    ) -> Result<Box<dyn ProxyLease>, StageFailure> {
        let endpoint = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), plan.endpoint.port);
        let config = NativeProxyConfig::new(
            endpoint,
            self.limits.max_connections,
            self.limits.per_connection_buffer_bytes,
            self.limits.shutdown_timeout,
        )
        .map_err(|error| StageFailure::new(Stage::ProxyStart, error.code, error.detail))?
        .with_session_id(plan.session_id.clone())
        .map_err(|error| StageFailure::new(Stage::ProxyStart, error.code, error.detail))?;
        let controlled_lab = plan
            .controlled
            .then(ControlledLab::start)
            .transpose()
            .map_err(|error| {
                StageFailure::new(Stage::ProxyStart, "controlled-lab-failed", error)
            })?;
        let mut backend = RuntimeBackend::new(config);
        if let Some(lab) = &controlled_lab {
            let mut policy = DestinationPolicy::new(endpoint);
            policy.grant_for_test(lab.http_origin);
            policy.grant_for_test(lab.https_origin);
            let mut roots = rustls::RootCertStore::empty();
            roots.add(lab.origin_certificate.clone()).map_err(|error| {
                StageFailure::new(
                    Stage::ProxyStart,
                    "controlled-root-failed",
                    error.to_string(),
                )
            })?;
            let tls = tls_client_config_with_roots(roots)
                .map_err(|error| StageFailure::new(Stage::ProxyStart, error.code, error.detail))?;
            backend = backend
                .with_destination_policy(policy)
                .with_tls_client_config(tls);
        }
        let lease = backend
            .start(budget.remaining())
            .map_err(|error| StageFailure::new(Stage::ProxyStart, error.code, error.detail))?;
        Ok(Box::new(NativeProxyLease {
            lease,
            controlled: plan.controlled,
            controlled_lab,
            observation_context: self.observation_context.clone(),
        }))
    }
}

struct NativeProxyLease {
    lease: fragcap_proxy::NativeProxyLease,
    controlled: bool,
    controlled_lab: Option<ControlledLab>,
    observation_context: NativeObservationContext,
}

impl ProxyLease for NativeProxyLease {
    fn route(&self) -> Result<ProxyRoute, StageFailure> {
        let endpoint = self.lease.endpoint();
        Ok(ProxyRoute::new(
            LoopbackEndpoint {
                port: endpoint.port(),
            },
            self.lease.proxy_url(),
            self.lease.capability_proof().proxy_authorization(),
            self.lease.ca_der().to_vec(),
            self.lease.ca_sha1_thumbprint().to_string(),
            self.lease.authority_generation(),
            self.controlled_lab
                .as_ref()
                .map(|lab| (lab.http_origin, lab.https_origin)),
        ))
    }

    fn observations(
        &mut self,
        budget: Budget,
    ) -> Result<Vec<CompatibilityObservation>, StageFailure> {
        let observation = self
            .lease
            .observation(budget.remaining())
            .map_err(|error| StageFailure::new(Stage::Observe, error.code, error.detail))?;
        Ok(observation
            .application
            .into_iter()
            .map(|value| {
                let (flow_id, process_id, process_image, role, attribution) = if self.controlled {
                    (
                        fragcap_core::FlowId::new(value.connection_id),
                        self.observation_context.controlled_process_id(),
                        Some("client.exe".to_string()),
                        Some("client".to_string()),
                        Some("controlled-harness".to_string()),
                    )
                } else {
                    correlate_native_observation(
                        &self.observation_context.flow_registry,
                        value.client_peer,
                        value.proxy_local,
                    )
                };
                CompatibilityObservation {
                    flow_id,
                    proxy_connection_id: value.connection_id.to_string(),
                    client_peer: Some(value.client_peer),
                    proxy_local: Some(value.proxy_local),
                    observed_at: value.timestamp_ns.to_string(),
                    process_id,
                    process_image,
                    role,
                    attribution,
                    protocol: match value.protocol.as_str() {
                        "connect" | "tls" => "https".to_string(),
                        _ => value.protocol,
                    },
                    inspectability: Inspectability::from_label(value.inspectability),
                    method: value.method,
                    url: value.url,
                    status: value.status,
                    reason: value.reason,
                }
            })
            .collect())
    }

    fn stop(&mut self, budget: Budget) -> CleanupResult {
        let report = self.lease.stop(budget.remaining());
        cleanup_result("native-proxy-listener", &report)
    }

    fn cleanup(&mut self, budget: Budget) -> Vec<CleanupResult> {
        let report = self.lease.cleanup(budget.remaining());
        let mut results = vec![cleanup_result("native-proxy-runtime", &report)];
        if let Some(mut lab) = self.controlled_lab.take() {
            results.push(lab.cleanup());
        }
        results
    }
}

type NativeCorrelation = (
    Option<fragcap_core::FlowId>,
    Option<u32>,
    Option<String>,
    Option<String>,
    Option<String>,
);

fn correlate_native_observation(
    registry: &FlowRegistry,
    client_peer: SocketAddr,
    proxy_local: SocketAddr,
) -> NativeCorrelation {
    let (local, remote) = if client_peer <= proxy_local {
        (client_peer, proxy_local)
    } else {
        (proxy_local, client_peer)
    };
    let key = FlowKey::new(Proto::Tcp, local, remote);
    let flow_id = registry.lookup(&key);
    let Some(attribution) = registry.attribution(&key) else {
        return (flow_id, None, None, None, None);
    };
    (
        flow_id,
        Some(attribution.pid),
        Some(attribution.process.to_string()),
        attribution.role.map(|role| role.to_string()),
        Some(
            match attribution.fidelity {
                Fidelity::Live => "live",
                Fidelity::Retained => "retained",
                Fidelity::None => "none",
            }
            .to_string(),
        ),
    )
}

struct ControlledLab {
    http_origin: SocketAddr,
    https_origin: SocketAddr,
    origin_certificate: CertificateDer<'static>,
    shutdown: Arc<AtomicBool>,
    workers: Vec<JoinHandle<Result<(), String>>>,
}

impl ControlledLab {
    fn start() -> Result<Self, String> {
        let http =
            TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).map_err(|error| error.to_string())?;
        let https =
            TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).map_err(|error| error.to_string())?;
        http.set_nonblocking(true)
            .map_err(|error| error.to_string())?;
        https
            .set_nonblocking(true)
            .map_err(|error| error.to_string())?;
        let http_origin = http.local_addr().map_err(|error| error.to_string())?;
        let https_origin = https.local_addr().map_err(|error| error.to_string())?;
        let generated = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])
            .map_err(|error| error.to_string())?;
        let origin_certificate = CertificateDer::from(generated.cert);
        let private = PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der());
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let mut tls_config = rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .map_err(|error| error.to_string())?
            .with_no_client_auth()
            .with_single_cert(vec![origin_certificate.clone()], private.into())
            .map_err(|error| error.to_string())?;
        tls_config.alpn_protocols = vec![b"http/1.1".to_vec()];
        let shutdown = Arc::new(AtomicBool::new(false));
        let http_stop = Arc::clone(&shutdown);
        let http_worker = std::thread::spawn(move || serve_origin(http, http_stop, None));
        let https_stop = Arc::clone(&shutdown);
        let https_worker =
            std::thread::spawn(move || serve_origin(https, https_stop, Some(Arc::new(tls_config))));
        Ok(Self {
            http_origin,
            https_origin,
            origin_certificate,
            shutdown,
            workers: vec![http_worker, https_worker],
        })
    }

    fn cleanup(&mut self) -> CleanupResult {
        self.shutdown.store(true, Ordering::Release);
        let mut failed = Vec::new();
        for worker in self.workers.drain(..) {
            match worker.join() {
                Ok(Ok(())) => {}
                Ok(Err(error)) => failed.push(error),
                Err(_) => failed.push("controlled origin thread panicked".to_string()),
            }
        }
        CleanupResult {
            resource: "controlled-protocol-lab".to_string(),
            status: if failed.is_empty() {
                CleanupStatus::Released
            } else {
                CleanupStatus::Failed
            },
            reason: if failed.is_empty() {
                "released".to_string()
            } else {
                failed.join("; ")
            },
        }
    }
}

fn serve_origin(
    listener: TcpListener,
    shutdown: Arc<AtomicBool>,
    tls: Option<Arc<rustls::ServerConfig>>,
) -> Result<(), String> {
    while !shutdown.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(false).map_err(|e| e.to_string())?;
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .map_err(|e| e.to_string())?;
                stream
                    .set_write_timeout(Some(Duration::from_secs(2)))
                    .map_err(|e| e.to_string())?;
                if let Some(config) = &tls {
                    let connection = rustls::ServerConnection::new(Arc::clone(config))
                        .map_err(|e| e.to_string())?;
                    let mut stream = rustls::StreamOwned::new(connection, stream);
                    answer_origin(&mut stream, b"SECURE")?;
                } else {
                    let mut stream = stream;
                    answer_origin(&mut stream, b"PLAIN")?;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}

fn answer_origin(stream: &mut (impl Read + Write), body: &[u8]) -> Result<(), String> {
    let mut head = Vec::new();
    let mut byte = [0_u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        stream
            .read_exact(&mut byte)
            .map_err(|error| error.to_string())?;
        head.push(byte[0]);
        if head.len() > 16 * 1024 {
            return Err("controlled origin request head exceeded limit".to_string());
        }
    }
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .map_err(|error| error.to_string())?;
    stream.write_all(body).map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())
}

/// Exercise the native route from the hidden controlled child process.
#[doc(hidden)]
pub fn run_controlled_native_requests(
    endpoint: SocketAddr,
    authorization: &str,
    http_origin: SocketAddr,
    https_origin: SocketAddr,
    ca_der: Vec<u8>,
    include_tls: bool,
) -> Result<(), String> {
    let mut plain =
        TcpStream::connect_timeout(&endpoint, Duration::from_secs(2)).map_err(|e| e.to_string())?;
    plain
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    write!(plain, "GET http://{http_origin}/plain HTTP/1.1\r\nHost: {http_origin}\r\nProxy-Authorization: {authorization}\r\nConnection: close\r\n\r\n").map_err(|e| e.to_string())?;
    let mut response = String::new();
    plain
        .read_to_string(&mut response)
        .map_err(|e| e.to_string())?;
    if !response.starts_with("HTTP/1.1 200") || !response.ends_with("PLAIN") {
        return Err("native HTTP controlled request returned an unexpected response".to_string());
    }
    if !include_tls {
        return Ok(());
    }
    let mut tcp =
        TcpStream::connect_timeout(&endpoint, Duration::from_secs(2)).map_err(|e| e.to_string())?;
    tcp.set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    write!(tcp, "CONNECT localhost:{} HTTP/1.1\r\nHost: localhost:{}\r\nProxy-Authorization: {authorization}\r\n\r\n", https_origin.port(), https_origin.port()).map_err(|e| e.to_string())?;
    let head = read_head(&mut tcp)?;
    if head != b"HTTP/1.1 200 Connection Established\r\n\r\n" {
        return Err("native CONNECT returned an unexpected response".to_string());
    }
    let mut roots = rustls::RootCertStore::empty();
    roots
        .add(CertificateDer::from(ca_der))
        .map_err(|e| e.to_string())?;
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| e.to_string())?
        .with_root_certificates(roots)
        .with_no_client_auth();
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    let connection = rustls::ClientConnection::new(
        Arc::new(config),
        ServerName::try_from("localhost")
            .map_err(|e| e.to_string())?
            .to_owned(),
    )
    .map_err(|e| e.to_string())?;
    let mut tls = rustls::StreamOwned::new(connection, tcp);
    write!(
        tls,
        "GET /secure HTTP/1.1\r\nHost: localhost:{}\r\nConnection: close\r\n\r\n",
        https_origin.port()
    )
    .map_err(|e| e.to_string())?;
    let mut response = String::new();
    tls.read_to_string(&mut response)
        .map_err(|e| e.to_string())?;
    if !response.starts_with("HTTP/1.1 200") || !response.ends_with("SECURE") {
        return Err("native HTTPS controlled request returned an unexpected response".to_string());
    }
    Ok(())
}

fn read_head(stream: &mut impl Read) -> Result<Vec<u8>, String> {
    let mut head = Vec::new();
    let mut byte = [0_u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        stream
            .read_exact(&mut byte)
            .map_err(|error| error.to_string())?;
        head.push(byte[0]);
        if head.len() > 16 * 1024 {
            return Err("proxy response head exceeded limit".to_string());
        }
    }
    Ok(head)
}

fn cleanup_result(resource: &str, report: &ShutdownReport) -> CleanupResult {
    let status = if report.is_clean() {
        CleanupStatus::Released
    } else if report.residue || report.incomplete_tasks > 0 {
        CleanupStatus::TimedOut
    } else {
        CleanupStatus::Failed
    };
    CleanupResult {
        resource: resource.to_string(),
        status,
        reason: format!(
            "accepted={}, completed={}, failed={}, forced={}, incomplete={}, failures={}",
            report.observation.accepted_connections,
            report.observation.completed_connections,
            report.observation.failed_connections,
            report.observation.forced_connections,
            report.incomplete_tasks,
            report.observation.failures.len()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap_core::Attribution;

    #[test]
    fn native_observation_correlation_restores_packet_and_process_truth() {
        let registry = FlowRegistry::default();
        let client: SocketAddr = "127.0.0.1:41000".parse().unwrap();
        let proxy: SocketAddr = "127.0.0.1:42000".parse().unwrap();
        let key = FlowKey::new(Proto::Tcp, client, proxy);
        registry.observe(
            key,
            Some(&Attribution::new(77, "game.exe", Fidelity::Live).with_role("client")),
        );

        let correlated = correlate_native_observation(&registry, client, proxy);
        assert_eq!(correlated.0, registry.lookup(&key));
        assert_eq!(correlated.1, Some(77));
        assert_eq!(correlated.2.as_deref(), Some("game.exe"));
        assert_eq!(correlated.3.as_deref(), Some("client"));
        assert_eq!(correlated.4.as_deref(), Some("live"));
    }

    #[test]
    fn unmatched_native_observation_does_not_invent_identity() {
        let registry = FlowRegistry::default();
        let correlated = correlate_native_observation(
            &registry,
            "127.0.0.1:41000".parse().unwrap(),
            "127.0.0.1:42000".parse().unwrap(),
        );
        assert_eq!(correlated, (None, None, None, None, None));
    }

    #[test]
    fn controlled_process_identity_is_the_explicit_child() {
        let context = NativeObservationContext::default();
        assert_eq!(context.controlled_process_id(), None);
        context.record_controlled_process_id(4242);
        assert_eq!(context.controlled_process_id(), Some(4242));
    }
}
