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
    pub capture_payloads: bool,
    pub max_header_bytes: usize,
    pub max_headers: usize,
    pub max_body_bytes: u64,
    pub max_session_body_bytes: u64,
    pub max_event_queue: usize,
    pub max_event_chunk_bytes: usize,
    pub max_decoded_body_bytes: usize,
    pub max_decode_ratio: usize,
    pub max_concurrent_decoders: usize,
    pub decode_timeout: Duration,
    pub max_concurrent_streams: usize,
    pub max_reset_streams: usize,
    pub max_pending_reset_streams: usize,
    pub http2_stream_window_bytes: usize,
    pub http2_connection_window_bytes: usize,
    pub http2_send_buffer_bytes: usize,
    pub max_requests_per_connection: usize,
    pub max_observations: usize,
    pub max_websocket_frame_bytes: usize,
    pub max_websocket_message_bytes: usize,
    pub max_sse_line_bytes: usize,
    pub max_sse_event_bytes: usize,
    pub max_grpc_message_bytes: usize,
    pub max_socks_udp_datagram_bytes: usize,
    pub max_socks_udp_peers: usize,
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
            capture_payloads: true,
            max_header_bytes: 64 * 1024,
            max_headers: 128,
            max_body_bytes: 16 * 1024 * 1024,
            max_session_body_bytes: 256 * 1024 * 1024,
            max_event_queue: 4_096,
            max_event_chunk_bytes: 16 * 1024,
            max_decoded_body_bytes: 32 * 1024 * 1024,
            max_decode_ratio: 64,
            max_concurrent_decoders: 4,
            decode_timeout: Duration::from_secs(10),
            max_concurrent_streams: 128,
            max_reset_streams: 128,
            max_pending_reset_streams: 128,
            http2_stream_window_bytes: 1024 * 1024,
            http2_connection_window_bytes: 4 * 1024 * 1024,
            http2_send_buffer_bytes: 1024 * 1024,
            max_requests_per_connection: 1_024,
            max_observations: 4_096,
            max_websocket_frame_bytes: 16 * 1024 * 1024,
            max_websocket_message_bytes: 32 * 1024 * 1024,
            max_sse_line_bytes: 64 * 1024,
            max_sse_event_bytes: 1024 * 1024,
            max_grpc_message_bytes: 32 * 1024 * 1024,
            max_socks_udp_datagram_bytes: 65_507,
            max_socks_udp_peers: 256,
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

impl ProtocolLimits {
    pub(crate) fn validate(&self) -> Result<(), ConfigError> {
        let nonzero = [
            ("max-header-bytes", self.max_header_bytes),
            ("max-headers", self.max_headers),
            ("max-event-queue", self.max_event_queue),
            ("max-event-chunk-bytes", self.max_event_chunk_bytes),
            ("max-decoded-body-bytes", self.max_decoded_body_bytes),
            ("max-decode-ratio", self.max_decode_ratio),
            ("max-concurrent-decoders", self.max_concurrent_decoders),
            ("max-concurrent-streams", self.max_concurrent_streams),
            ("max-reset-streams", self.max_reset_streams),
            ("max-pending-reset-streams", self.max_pending_reset_streams),
            ("http2-stream-window-bytes", self.http2_stream_window_bytes),
            (
                "http2-connection-window-bytes",
                self.http2_connection_window_bytes,
            ),
            ("http2-send-buffer-bytes", self.http2_send_buffer_bytes),
            ("max-websocket-frame-bytes", self.max_websocket_frame_bytes),
            (
                "max-websocket-message-bytes",
                self.max_websocket_message_bytes,
            ),
            ("max-sse-line-bytes", self.max_sse_line_bytes),
            ("max-sse-event-bytes", self.max_sse_event_bytes),
            ("max-grpc-message-bytes", self.max_grpc_message_bytes),
            (
                "max-socks-udp-datagram-bytes",
                self.max_socks_udp_datagram_bytes,
            ),
            ("max-socks-udp-peers", self.max_socks_udp_peers),
        ];
        if let Some((name, _)) = nonzero.into_iter().find(|(_, value)| *value == 0) {
            return Err(ConfigError::new(
                "zero-protocol-limit",
                format!("{name} must be non-zero"),
            ));
        }
        if self.max_body_bytes == 0 || self.max_session_body_bytes == 0 {
            return Err(ConfigError::new(
                "zero-protocol-limit",
                "body retention limits must be non-zero",
            ));
        }
        if !(23..=65_507).contains(&self.max_socks_udp_datagram_bytes) {
            return Err(ConfigError::new(
                "socks-udp-datagram-limit-invalid",
                "max-socks-udp-datagram-bytes must be between 23 and 65507",
            ));
        }
        if self.max_concurrent_streams > u32::MAX as usize
            || self.max_header_bytes > u32::MAX as usize
            || self.http2_stream_window_bytes > (i32::MAX as usize)
            || self.http2_connection_window_bytes > (i32::MAX as usize)
        {
            return Err(ConfigError::new(
                "protocol-limit-out-of-range",
                "HTTP/2 settings exceed their protocol representation",
            ));
        }
        if [
            self.header_timeout,
            self.idle_timeout,
            self.tls_handshake_timeout,
            self.decode_timeout,
            self.upstream.dns,
            self.upstream.connect,
            self.upstream.read,
            self.upstream.write,
        ]
        .into_iter()
        .any(|value| value.is_zero())
        {
            return Err(ConfigError::new(
                "zero-protocol-timeout",
                "protocol timeouts must be non-zero",
            ));
        }
        Ok(())
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
    pub http2_streams: u64,
    pub http2_streams_completed: u64,
    pub http2_streams_reset: u64,
    pub metadata_blocks: u64,
    pub body_bytes_observed: u64,
    pub body_bytes_retained: u64,
    pub body_bytes_omitted: u64,
    pub body_bytes_truncated: u64,
    pub body_bytes_queue_dropped: u64,
    pub application_events_accepted: u64,
    pub application_events_dropped: u64,
    pub streaming_bytes_queue_dropped: u64,
    pub socks_negotiations: u64,
    pub socks_auth_succeeded: u64,
    pub socks_auth_refused: u64,
    pub socks_connect_requested: u64,
    pub socks_connect_succeeded: u64,
    pub socks_connect_refused: u64,
    pub socks_dns_owned: u64,
    pub socks_ipv4: u64,
    pub socks_ipv6: u64,
    pub socks_domain: u64,
    pub socks_http: u64,
    pub socks_tls: u64,
    pub socks_tcp_opaque: u64,
    pub socks_client_bytes: u64,
    pub socks_upstream_bytes: u64,
    pub socks_udp_associate_requested: u64,
    pub socks_udp_associate_succeeded: u64,
    pub socks_udp_associate_refused: u64,
    pub socks_udp_client_datagrams: u64,
    pub socks_udp_client_forwarded: u64,
    pub socks_udp_upstream_datagrams: u64,
    pub socks_udp_upstream_forwarded: u64,
    pub socks_udp_malformed_dropped: u64,
    pub socks_udp_fragment_dropped: u64,
    pub socks_udp_source_dropped: u64,
    pub socks_udp_policy_dropped: u64,
    pub socks_udp_resolution_dropped: u64,
    pub socks_udp_peer_limit_dropped: u64,
    pub socks_udp_oversized_dropped: u64,
    pub socks_udp_unsolicited_dropped: u64,
    pub socks_udp_transport_dropped: u64,
    pub socks_udp_owner_dropped: u64,
    pub socks_udp_client_bytes: u64,
    pub socks_udp_upstream_bytes: u64,
    pub socks_udp_peak_peers: u64,
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
    pub connection_opened_at_ns: u64,
    pub connection_closed_at_ns: u64,
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
