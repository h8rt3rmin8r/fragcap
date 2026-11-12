// SPDX-License-Identifier: Apache-2.0

//! Bounded selected-flow route evidence, independent of broad packet pass-through.

use std::fs;
use std::io;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::sync::Arc;
use std::thread::JoinHandle;

use serde_json::{json, Value};

use super::{read_lifecycle_prefix, LifecycleStreamStatus, LifecycleWriter};

pub const ROUTE_LIFECYCLE: &str = "route.jsonl";
const MAX_ROUTE_EVENTS: u64 = 8_192;
const MAX_ROUTE_BYTES: usize = 12 * 1024 * 1024;
const MAX_IDENTITY_BYTES: usize = 128;

#[derive(Debug)]
pub enum RouteEventKind {
    Selected {
        protocol: &'static str,
        local: SocketAddr,
        remote: SocketAddr,
        pid: u32,
        process_instance_id: String,
        endpoint_id: u64,
        first_packet_at_ns: i64,
        first_packet_clock_error_ns: u64,
        tcp_flags: Option<u8>,
        packet_flags: u32,
        ip_checksum_valid: bool,
        if_idx: u32,
        sub_if_idx: u32,
        tcp_checksum_valid: bool,
    },
    ProxyAttempt,
    ProxyConnected {
        helper_local: SocketAddr,
        proxy_local: SocketAddr,
    },
    UdpRelayAssociated {
        relay_local: SocketAddr,
        relay_remote: SocketAddr,
    },
    ProxyRefused {
        reason: &'static str,
    },
    SyntheticReplyGenerated {
        packets: u32,
        if_idx: u32,
        sub_if_idx: u32,
    },
    SyntheticReplySent {
        packets: u32,
    },
    SyntheticReplyFailed {
        reason: &'static str,
    },
    ClientResponse {
        ack: bool,
    },
    ReturnedDataAcknowledged,
    Transfer {
        client_to_upstream: u64,
        upstream_to_client: u64,
    },
    DatagramTransfer {
        client_to_upstream: u64,
        upstream_to_client: u64,
        client_datagrams: u64,
        peer_datagrams: u64,
    },
    Terminal {
        reason: &'static str,
    },
}

impl RouteEventKind {
    fn into_parts(self) -> (&'static str, Value) {
        match self {
            Self::Selected {
                protocol,
                local,
                remote,
                pid,
                process_instance_id,
                endpoint_id,
                first_packet_at_ns,
                first_packet_clock_error_ns,
                tcp_flags,
                packet_flags,
                ip_checksum_valid,
                if_idx,
                sub_if_idx,
                tcp_checksum_valid,
            } => (
                "selected",
                json!({"protocol": protocol, "local": local, "remote": remote, "pid": pid,
                    "process_instance_id": process_instance_id, "endpoint_id": endpoint_id,
                    "first_packet_at_ns": first_packet_at_ns, "first_packet_clock_error_ns": first_packet_clock_error_ns, "tcp_flags": tcp_flags,
                    "packet_flags": packet_flags, "ip_checksum_valid": ip_checksum_valid,
                    "if_idx": if_idx, "sub_if_idx": sub_if_idx,
                    "tcp_checksum_valid": tcp_checksum_valid}),
            ),
            Self::ProxyAttempt => ("proxy-attempt", json!({})),
            Self::ProxyConnected {
                helper_local,
                proxy_local,
            } => (
                "proxy-connected",
                json!({"helper_local": helper_local, "proxy_local": proxy_local}),
            ),
            Self::UdpRelayAssociated {
                relay_local,
                relay_remote,
            } => (
                "udp-relay-associated",
                json!({"relay_local": relay_local, "relay_remote": relay_remote}),
            ),
            Self::ProxyRefused { reason } => ("proxy-refused", json!({"reason": reason})),
            Self::SyntheticReplyGenerated {
                packets,
                if_idx,
                sub_if_idx,
            } => (
                "synthetic-reply-generated",
                json!({"packets": packets, "if_idx": if_idx, "sub_if_idx": sub_if_idx}),
            ),
            Self::SyntheticReplySent { packets } => {
                ("synthetic-reply-sent", json!({"packets": packets}))
            }
            Self::SyntheticReplyFailed { reason } => {
                ("synthetic-reply-failed", json!({"reason": reason}))
            }
            Self::ClientResponse { ack } => ("client-response", json!({"ack": ack})),
            Self::ReturnedDataAcknowledged => ("returned-data-acknowledged", json!({})),
            Self::Transfer {
                client_to_upstream,
                upstream_to_client,
            } => (
                "transfer",
                json!({"client_to_upstream": client_to_upstream, "upstream_to_client": upstream_to_client}),
            ),
            Self::DatagramTransfer {
                client_to_upstream,
                upstream_to_client,
                client_datagrams,
                peer_datagrams,
            } => (
                "datagram-transfer",
                json!({"client_to_upstream": client_to_upstream, "upstream_to_client": upstream_to_client, "client_datagrams": client_datagrams, "peer_datagrams": peer_datagrams}),
            ),
            Self::Terminal { reason } => ("terminal", json!({"reason": reason})),
        }
    }

    fn valid(&self) -> bool {
        match self {
            Self::Selected {
                protocol,
                process_instance_id,
                ..
            } => {
                matches!(*protocol, "tcp" | "udp")
                    && process_instance_id.len() <= MAX_IDENTITY_BYTES
                    && !process_instance_id.is_empty()
            }
            _ => true,
        }
    }
}

#[derive(Debug)]
pub struct RouteEvent {
    pub flow_id: u64,
    pub at_ns: u64,
    pub clock_error_ns: Option<u64>,
    pub kind: RouteEventKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouteRecordDisposition {
    Accepted,
    QueueFull,
    Retired,
    Invalid,
}

#[derive(Default)]
struct RouteAccount {
    accepted: AtomicU64,
    written: AtomicU64,
    dropped: AtomicU64,
    retired: AtomicBool,
    writer_failed: AtomicBool,
}

#[derive(Clone)]
pub struct RouteEventSink {
    sender: SyncSender<RouteMessage>,
    account: Arc<RouteAccount>,
}

impl RouteEventSink {
    pub fn record(&self, event: RouteEvent) -> RouteRecordDisposition {
        if event.flow_id == 0 || !event.kind.valid() {
            self.account.dropped.fetch_add(1, Ordering::Relaxed);
            return RouteRecordDisposition::Invalid;
        }
        if self.account.retired.load(Ordering::Acquire) {
            self.account.dropped.fetch_add(1, Ordering::Relaxed);
            return RouteRecordDisposition::Retired;
        }
        match self.sender.try_send(RouteMessage::Event(event)) {
            Ok(()) => {
                self.account.accepted.fetch_add(1, Ordering::Relaxed);
                RouteRecordDisposition::Accepted
            }
            Err(TrySendError::Full(_)) => {
                self.account.dropped.fetch_add(1, Ordering::Relaxed);
                RouteRecordDisposition::QueueFull
            }
            Err(TrySendError::Disconnected(_)) => {
                self.account.retired.store(true, Ordering::Release);
                self.account.dropped.fetch_add(1, Ordering::Relaxed);
                RouteRecordDisposition::Retired
            }
        }
    }
}

enum RouteMessage {
    Event(RouteEvent),
    Finish,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RouteStreamOutcome {
    pub accepted_events: u64,
    pub written_events: u64,
    pub dropped_events: u64,
    pub writer_failed: bool,
}

pub struct RouteEventLease {
    sink: RouteEventSink,
    worker: Option<JoinHandle<io::Result<()>>>,
}

impl RouteEventLease {
    pub fn open(
        path: impl Into<PathBuf>,
        session_id: impl Into<String>,
        capacity: usize,
    ) -> io::Result<Self> {
        if capacity == 0 || capacity > 4096 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "route queue capacity out of bounds",
            ));
        }
        let writer = LifecycleWriter::create(path, "route", session_id)?;
        let (sender, receiver) = mpsc::sync_channel(capacity);
        let account = Arc::new(RouteAccount::default());
        let worker_account = Arc::clone(&account);
        let worker = std::thread::Builder::new()
            .name("fragcap-route-evidence-writer".into())
            .spawn(move || write_route_events(writer, receiver, worker_account))?;
        Ok(Self {
            sink: RouteEventSink { sender, account },
            worker: Some(worker),
        })
    }

    pub fn sink(&self) -> RouteEventSink {
        self.sink.clone()
    }

    pub fn accounting(&self) -> RouteStreamOutcome {
        RouteStreamOutcome {
            accepted_events: self.sink.account.accepted.load(Ordering::Acquire),
            written_events: self.sink.account.written.load(Ordering::Acquire),
            dropped_events: self.sink.account.dropped.load(Ordering::Acquire),
            writer_failed: self.sink.account.writer_failed.load(Ordering::Acquire),
        }
    }

    pub fn finish(&mut self) -> io::Result<RouteStreamOutcome> {
        let Some(worker) = self.worker.take() else {
            return Ok(self.accounting());
        };
        self.sink.account.retired.store(true, Ordering::Release);
        let _ = self.sink.sender.send(RouteMessage::Finish);
        let result = worker
            .join()
            .map_err(|_| io::Error::other("route writer panicked"))?;
        result?;
        Ok(self.accounting())
    }
}

impl Drop for RouteEventLease {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

fn write_route_events(
    mut writer: LifecycleWriter,
    receiver: mpsc::Receiver<RouteMessage>,
    account: Arc<RouteAccount>,
) -> io::Result<()> {
    let result = (|| {
        let mut bytes = 0usize;
        for message in receiver {
            let RouteMessage::Event(event) = message else {
                break;
            };
            let (kind, fields) = event.kind.into_parts();
            let value = json!({"flow_id": event.flow_id, "at_ns": event.at_ns,
                "clock_error_ns": event.clock_error_ns, "kind": kind, "fields": fields});
            let size = serde_json::to_vec(&value)
                .map_err(io::Error::other)?
                .len()
                .saturating_add(128);
            if account.written.load(Ordering::Relaxed) >= MAX_ROUTE_EVENTS
                || bytes.saturating_add(size) > MAX_ROUTE_BYTES
            {
                account.dropped.fetch_add(1, Ordering::Relaxed);
                continue;
            }
            writer.append("route.event", value)?;
            bytes += size;
            account.written.fetch_add(1, Ordering::Release);
        }
        let dropped = account.dropped.load(Ordering::Acquire);
        if dropped > 0 {
            writer.gap("route", "queue-or-capacity-loss", Some(dropped))?;
        }
        writer.finish()
    })();
    if result.is_err() {
        account.writer_failed.store(true, Ordering::Release);
    }
    result
}

#[derive(Clone, Debug)]
pub struct RouteRecord {
    pub flow_id: u64,
    pub at_ns: u64,
    pub clock_error_ns: Option<u64>,
    pub kind: String,
    pub fields: Value,
}

#[derive(Clone, Debug)]
pub struct RoutePrefix {
    pub session_id: String,
    pub events: Vec<RouteRecord>,
    pub status: LifecycleStreamStatus,
    pub dropped_events: Option<u64>,
}

impl RoutePrefix {
    pub fn complete(&self) -> bool {
        self.status == LifecycleStreamStatus::Complete && self.dropped_events == Some(0)
    }
}

pub fn read_route_prefix(path: &Path) -> io::Result<RoutePrefix> {
    let prefix = read_lifecycle_prefix(path)?;
    if prefix.stream != "route" || prefix.status == LifecycleStreamStatus::UnknownVersion {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "not a supported route stream",
        ));
    }
    let mut events = Vec::new();
    let mut dropped = Some(0u64);
    for record in prefix.records {
        match record.get("type").and_then(Value::as_str) {
            Some("route.event") => {
                let fields = record.get("fields").ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "route event missing fields")
                })?;
                let required = |key| {
                    fields.get(key).and_then(Value::as_u64).ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "route event missing numeric identity",
                        )
                    })
                };
                let kind = fields.get("kind").and_then(Value::as_str).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "route event missing kind")
                })?;
                events.push(RouteRecord {
                    flow_id: required("flow_id")?,
                    at_ns: required("at_ns")?,
                    clock_error_ns: fields.get("clock_error_ns").and_then(Value::as_u64),
                    kind: kind.to_owned(),
                    fields: fields.get("fields").cloned().ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "route event missing body")
                    })?,
                });
            }
            Some("lifecycle.gap") => {
                dropped = record
                    .get("fields")
                    .and_then(|value| value.get("lost_records"))
                    .and_then(Value::as_u64);
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unknown route record",
                ))
            }
        }
    }
    Ok(RoutePrefix {
        session_id: prefix.session_id,
        events,
        status: prefix.status,
        dropped_events: dropped,
    })
}

pub fn route_stream_size(path: &Path) -> io::Result<u64> {
    Ok(fs::metadata(path)?.len())
}
