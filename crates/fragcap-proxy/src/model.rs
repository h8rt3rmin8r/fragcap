// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::time::Duration;

/// Validated, effect-free native proxy configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeProxyConfig {
    pub(crate) listen: SocketAddr,
    pub(crate) max_connections: NonZeroUsize,
    pub(crate) per_connection_buffer_bytes: NonZeroUsize,
    pub(crate) shutdown_timeout: Duration,
    pub(crate) protocol: ProtocolLimits,
    pub(crate) session_id: String,
}

impl NativeProxyConfig {
    pub fn new(
        listen: SocketAddr,
        max_connections: usize,
        per_connection_buffer_bytes: usize,
        shutdown_timeout: Duration,
    ) -> Result<Self, ConfigError> {
        if !listen.ip().is_loopback() {
            return Err(ConfigError::new(
                "non-loopback-listen",
                format!("native proxy endpoint must be loopback, got {listen}"),
            ));
        }
        let max_connections = NonZeroUsize::new(max_connections).ok_or_else(|| {
            ConfigError::new("zero-connection-limit", "connection limit must be non-zero")
        })?;
        let per_connection_buffer_bytes = NonZeroUsize::new(per_connection_buffer_bytes)
            .ok_or_else(|| {
                ConfigError::new("zero-buffer-limit", "connection buffer must be non-zero")
            })?;
        if shutdown_timeout.is_zero() {
            return Err(ConfigError::new(
                "zero-shutdown-timeout",
                "shutdown timeout must be non-zero",
            ));
        }
        Ok(Self {
            listen,
            max_connections,
            per_connection_buffer_bytes,
            shutdown_timeout,
            protocol: ProtocolLimits::default(),
            session_id: "native-session".to_string(),
        })
    }

    pub fn listen(&self) -> SocketAddr {
        self.listen
    }

    pub fn max_connections(&self) -> usize {
        self.max_connections.get()
    }

    pub fn per_connection_buffer_bytes(&self) -> usize {
        self.per_connection_buffer_bytes.get()
    }

    pub fn shutdown_timeout(&self) -> Duration {
        self.shutdown_timeout
    }

    pub fn with_protocol_limits(mut self, protocol: ProtocolLimits) -> Self {
        self.protocol = protocol;
        self
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Result<Self, ConfigError> {
        let session_id = session_id.into();
        if session_id.trim().is_empty() {
            return Err(ConfigError::new(
                "empty-session-id",
                "native proxy session id must not be empty",
            ));
        }
        self.session_id = session_id;
        Ok(self)
    }

    pub fn protocol_limits(&self) -> &ProtocolLimits {
        &self.protocol
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolLimits {
    pub max_header_bytes: usize,
    pub max_headers: usize,
    pub max_body_bytes: u64,
    pub max_requests_per_connection: usize,
    pub max_observations: usize,
    pub header_timeout: Duration,
    pub idle_timeout: Duration,
    pub tls_handshake_timeout: Duration,
    pub upstream: crate::UpstreamBudgets,
    pub leaf_cache_entries: NonZeroUsize,
    pub leaf_cache_bytes: NonZeroUsize,
    pub leaf_lifetime: Duration,
}

impl Default for ProtocolLimits {
    fn default() -> Self {
        Self {
            max_header_bytes: 64 * 1024,
            max_headers: 128,
            max_body_bytes: 16 * 1024 * 1024,
            max_requests_per_connection: 1_024,
            max_observations: 4_096,
            header_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(60),
            tls_handshake_timeout: Duration::from_secs(10),
            upstream: crate::UpstreamBudgets {
                dns: Duration::from_secs(5),
                connect: Duration::from_secs(10),
                read: Duration::from_secs(60),
                write: Duration::from_secs(60),
            },
            leaf_cache_entries: NonZeroUsize::new(256).expect("constant is non-zero"),
            leaf_cache_bytes: NonZeroUsize::new(8 * 1024 * 1024).expect("constant is non-zero"),
            leaf_lifetime: Duration::from_secs(24 * 60 * 60),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigError {
    pub code: &'static str,
    pub detail: String,
}

impl ConfigError {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for ConfigError {}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendKind {
    NativeRust,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackendCapabilities {
    pub foundation_listener: bool,
    pub forwards_upstream: bool,
    pub observes_http: bool,
    pub inspects_tls: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackendIdentity {
    pub kind: BackendKind,
    pub name: &'static str,
    pub version: &'static str,
    pub capabilities: BackendCapabilities,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleState {
    Running,
    Stopping,
    Stopped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeFailure {
    pub code: &'static str,
    pub detail: String,
    pub connection_id: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeObservation {
    pub state: LifecycleState,
    pub endpoint: SocketAddr,
    pub accepted_connections: u64,
    pub authenticated_connections: u64,
    pub authentication_refused: u64,
    pub saturated_connections: u64,
    pub completed_connections: u64,
    pub failed_connections: u64,
    pub forced_connections: u64,
    pub live_connections: usize,
    pub peak_live_connections: usize,
    pub failures: Vec<RuntimeFailure>,
    pub protocol: ProtocolAccounting,
    pub application: Vec<ProxyObservation>,
}

impl RuntimeObservation {
    pub(crate) fn running(endpoint: SocketAddr) -> Self {
        Self {
            state: LifecycleState::Running,
            endpoint,
            accepted_connections: 0,
            authenticated_connections: 0,
            authentication_refused: 0,
            saturated_connections: 0,
            completed_connections: 0,
            failed_connections: 0,
            forced_connections: 0,
            live_connections: 0,
            peak_live_connections: 0,
            failures: Vec::new(),
            protocol: ProtocolAccounting::default(),
            application: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProtocolAccounting {
    pub requests: u64,
    pub responses: u64,
    pub informational_responses: u64,
    pub connect_requests: u64,
    pub client_tls_completed: u64,
    pub upstream_tls_completed: u64,
    pub parse_refused: u64,
    pub policy_refused: u64,
    pub timed_out: u64,
    pub observations_dropped_oldest: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TlsBoundary {
    Client,
    Upstream,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TlsNegotiation {
    pub boundary: TlsBoundary,
    pub requested_identity: String,
    pub version: Option<String>,
    pub alpn: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProxyObservation {
    pub session_id: String,
    pub connection_id: u64,
    pub request_ordinal: u64,
    pub client_peer: SocketAddr,
    pub proxy_local: SocketAddr,
    pub timestamp_ns: u64,
    pub protocol: String,
    pub method: Option<String>,
    pub url: Option<String>,
    pub status: Option<u16>,
    pub inspectability: &'static str,
    pub reason: Option<String>,
    pub tls: Option<TlsNegotiation>,
    pub transformations: Vec<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShutdownReport {
    pub observation: RuntimeObservation,
    pub listener_released: bool,
    pub joined_tasks: u64,
    pub incomplete_tasks: u64,
    pub residue: bool,
}

impl ShutdownReport {
    pub fn is_clean(&self) -> bool {
        self.listener_released
            && self.observation.state == LifecycleState::Stopped
            && self.observation.live_connections == 0
            && self.incomplete_tasks == 0
            && !self.residue
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartError {
    pub code: &'static str,
    pub detail: String,
}

impl StartError {
    pub(crate) fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for StartError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObserveError {
    pub code: &'static str,
    pub detail: String,
}

impl ObserveError {
    pub(crate) fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for ObserveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for ObserveError {}
