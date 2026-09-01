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
    Error { code: &'static str },
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
}

pub trait ApplicationEventSink: Send + Sync {
    fn try_emit(&self, event: ApplicationEvent) -> EventDisposition;

    fn accounting(&self) -> ApplicationSinkAccounting {
        ApplicationSinkAccounting::default()
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
}
