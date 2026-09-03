// SPDX-License-Identifier: Apache-2.0

//! Typed, nonblocking application observation delivery.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    BodySegment, MetadataBlock, ProtocolVersion, StreamingEvent, TlsNegotiation, Transformation,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamTerminal {
    Complete,
    Reset,
    Cancelled,
    Refused,
    ProtocolError,
    TransportError,
    GoAway,
    IdleTimeout,
    Shutdown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationEventKind {
    ConnectionOpen(ConnectionDescriptor),
    ConnectionTerminal(StreamTerminal),
    TlsNegotiation(TlsNegotiation),
    TlsTerminal(StreamTerminal),
    HttpStreamOpen,
    HttpStreamTerminal(StreamTerminal),
    HttpTiming(HttpTiming),
    Metadata(MetadataBlock),
    Body(BodySegment),
    Transformation(Transformation),
    Streaming(StreamingEvent),
    SocksNegotiation(SocksNegotiationEvent),
    SocksConnect(SocksConnectEvent),
    SocksTransfer(SocksTransferEvent),
    SocksUdp(SocksUdpEvent),
    GenericStreamChunk(GenericStreamChunk),
    GenericUdpDatagram(GenericUdpDatagram),
    UdpSocketError(UdpSocketError),
    Error { code: &'static str },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenericStreamDirection {
    ClientToUpstream,
    UpstreamToClient,
}

impl GenericStreamDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ClientToUpstream => "client-to-upstream",
            Self::UpstreamToClient => "upstream-to-client",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenericStreamProvenance {
    TcpPlaintext,
    TlsEncrypted,
    TlsDecrypted,
}

impl GenericStreamProvenance {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TcpPlaintext => "tcp-plaintext",
            Self::TlsEncrypted => "tls-encrypted",
            Self::TlsDecrypted => "tls-decrypted",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenericStreamOutcome {
    Complete,
    IntentionallyOmitted,
    RetentionLimit,
}

impl GenericStreamOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::IntentionallyOmitted => "intentionally-omitted",
            Self::RetentionLimit => "retention-limit",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericStreamChunk {
    pub direction: GenericStreamDirection,
    pub provenance: GenericStreamProvenance,
    pub offset: u64,
    pub observed_len: u64,
    pub bytes: bytes::Bytes,
    pub outcome: GenericStreamOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenericUdpDirection {
    ClientToUpstream,
    UpstreamToClient,
}

impl GenericUdpDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ClientToUpstream => "client-to-upstream",
            Self::UpstreamToClient => "upstream-to-client",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenericUdpOutcome {
    Complete,
    IntentionallyOmitted,
    RetentionLimit,
}

impl GenericUdpOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::IntentionallyOmitted => "intentionally-omitted",
            Self::RetentionLimit => "retention-limit",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericUdpDatagram {
    pub direction: GenericUdpDirection,
    pub sequence: u64,
    pub client_endpoint: SocketAddr,
    pub remote_endpoint: SocketAddr,
    pub observed_len: u64,
    pub bytes: bytes::Bytes,
    pub outcome: GenericUdpOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UdpSocketError {
    pub direction: GenericUdpDirection,
    pub operation: &'static str,
    pub failure_code: &'static str,
    pub endpoint: Option<SocketAddr>,
    pub error_kind: &'static str,
    pub visibility: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SocksClassification {
    Http,
    Tls,
    OpaqueTcp,
}

impl SocksClassification {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Tls => "tls",
            Self::OpaqueTcp => "tcp-opaque",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SocksNegotiationEvent {
    pub authenticated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SocksConnectEvent {
    pub authority: String,
    pub address_type: &'static str,
    pub dns_owner: &'static str,
    pub outcome: &'static str,
    pub classification: Option<SocksClassification>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SocksTransferEvent {
    pub client_to_upstream_bytes: u64,
    pub upstream_to_client_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SocksUdpEvent {
    pub action: &'static str,
    pub outcome: &'static str,
    pub address_type: Option<&'static str>,
    pub remote: Option<SocketAddr>,
    pub payload_bytes: u64,
    pub active_peers: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HttpTiming {
    pub send_ns: u64,
    pub wait_ns: u64,
    pub receive_ns: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConnectionDescriptor {
    pub transport: &'static str,
    pub client_peer: SocketAddr,
    pub proxy_local: SocketAddr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationEvent {
    pub session_id: String,
    pub connection_id: u64,
    pub stream_id: Option<u64>,
    pub timestamp_ns: u64,
    pub protocol: Option<ProtocolVersion>,
    pub kind: ApplicationEventKind,
}

impl ApplicationEvent {
    pub fn now(
        session_id: impl Into<String>,
        connection_id: u64,
        stream_id: Option<u64>,
        protocol: Option<ProtocolVersion>,
        kind: ApplicationEventKind,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            connection_id,
            stream_id,
            timestamp_ns: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                .try_into()
                .unwrap_or(u64::MAX),
            protocol,
            kind,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventDisposition {
    Accepted,
    QueueFull,
    Retired,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ApplicationSinkAccounting {
    pub accepted_events: u64,
    pub dropped_events: u64,
    pub body_bytes_queue_dropped: u64,
    pub streaming_bytes_queue_dropped: u64,
    pub generic_stream_bytes_queue_dropped: u64,
    pub generic_udp_datagrams_queue_dropped: u64,
    pub generic_udp_bytes_queue_dropped: u64,
    pub generic_udp_datagrams_storage_dropped: u64,
    pub generic_udp_bytes_storage_dropped: u64,
}

pub trait ApplicationEventSink: Send + Sync {
    fn try_emit(&self, event: ApplicationEvent) -> EventDisposition;

    fn accounting(&self) -> ApplicationSinkAccounting {
        ApplicationSinkAccounting::default()
    }
}

/// Delivers each event to every configured observer without making one
/// observer's queue pressure block or retire the others.
pub struct FanoutApplicationEventSink {
    sinks: Vec<Arc<dyn ApplicationEventSink>>,
}

impl FanoutApplicationEventSink {
    pub fn new(sinks: Vec<Arc<dyn ApplicationEventSink>>) -> Self {
        Self { sinks }
    }
}

impl ApplicationEventSink for FanoutApplicationEventSink {
    fn try_emit(&self, event: ApplicationEvent) -> EventDisposition {
        let mut accepted = false;
        let mut full = false;
        for sink in &self.sinks {
            match sink.try_emit(event.clone()) {
                EventDisposition::Accepted => accepted = true,
                EventDisposition::QueueFull => full = true,
                EventDisposition::Retired => {}
            }
        }
        if accepted {
            EventDisposition::Accepted
        } else if full {
            EventDisposition::QueueFull
        } else {
            EventDisposition::Retired
        }
    }

    fn accounting(&self) -> ApplicationSinkAccounting {
        self.sinks
            .iter()
            .fold(ApplicationSinkAccounting::default(), |mut total, sink| {
                let value = sink.accounting();
                total.accepted_events = total.accepted_events.saturating_add(value.accepted_events);
                total.dropped_events = total.dropped_events.saturating_add(value.dropped_events);
                total.body_bytes_queue_dropped = total
                    .body_bytes_queue_dropped
                    .saturating_add(value.body_bytes_queue_dropped);
                total.streaming_bytes_queue_dropped = total
                    .streaming_bytes_queue_dropped
                    .saturating_add(value.streaming_bytes_queue_dropped);
                total.generic_stream_bytes_queue_dropped = total
                    .generic_stream_bytes_queue_dropped
                    .saturating_add(value.generic_stream_bytes_queue_dropped);
                total.generic_udp_datagrams_queue_dropped = total
                    .generic_udp_datagrams_queue_dropped
                    .saturating_add(value.generic_udp_datagrams_queue_dropped);
                total.generic_udp_bytes_queue_dropped = total
                    .generic_udp_bytes_queue_dropped
                    .saturating_add(value.generic_udp_bytes_queue_dropped);
                total.generic_udp_datagrams_storage_dropped = total
                    .generic_udp_datagrams_storage_dropped
                    .saturating_add(value.generic_udp_datagrams_storage_dropped);
                total.generic_udp_bytes_storage_dropped = total
                    .generic_udp_bytes_storage_dropped
                    .saturating_add(value.generic_udp_bytes_storage_dropped);
                total
            })
    }
}

pub(crate) type SharedEventSink = Option<Arc<dyn ApplicationEventSink>>;

pub(crate) fn emit(sink: &SharedEventSink, event: ApplicationEvent) -> EventDisposition {
    sink.as_ref()
        .map_or(EventDisposition::Retired, |sink| sink.try_emit(event))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_connection_and_phase_timing_are_typed_without_placeholder_values() {
        let descriptor = ConnectionDescriptor {
            transport: "tcp",
            client_peer: "[::1]:41000".parse().unwrap(),
            proxy_local: "[::1]:42000".parse().unwrap(),
        };
        let event = ApplicationEvent::now(
            "session",
            7,
            None,
            None,
            ApplicationEventKind::ConnectionOpen(descriptor),
        );
        assert_eq!(event.connection_id, 7);
        assert!(event.timestamp_ns > 0);
        assert_eq!(event.kind, ApplicationEventKind::ConnectionOpen(descriptor));

        let timing = HttpTiming {
            send_ns: 1,
            wait_ns: 2,
            receive_ns: 3,
        };
        assert_eq!(timing.send_ns + timing.wait_ns + timing.receive_ns, 6);
    }

    #[test]
    fn generic_udp_keeps_one_boundary_and_exact_endpoints() {
        let client = "127.0.0.1:41000".parse().unwrap();
        let remote = "127.0.0.1:42000".parse().unwrap();
        let datagram = GenericUdpDatagram {
            direction: GenericUdpDirection::ClientToUpstream,
            sequence: 3,
            client_endpoint: client,
            remote_endpoint: remote,
            observed_len: 4,
            bytes: bytes::Bytes::from_static(&[0, 1, 2]),
            outcome: GenericUdpOutcome::RetentionLimit,
        };
        assert_eq!(datagram.observed_len, 4);
        assert_eq!(datagram.bytes.len(), 3);
        assert_eq!(datagram.direction.as_str(), "client-to-upstream");
        assert_eq!(datagram.outcome.as_str(), "retention-limit");
        assert_eq!(datagram.client_endpoint, client);
        assert_eq!(datagram.remote_endpoint, remote);
    }
}
