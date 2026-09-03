// SPDX-License-Identifier: Apache-2.0

//! Bounded, append-only native proxy and cleanup lifecycle streams.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;

use fragcap_proxy::{
    ApplicationEvent, ApplicationEventKind, ApplicationEventSink, ApplicationSinkAccounting,
    EventDisposition,
};
use serde_json::{json, Value};

use super::artifacts::open_sensitive_file;

pub const PROXY_LIFECYCLE: &str = "proxy.jsonl";
pub const CLEANUP_LIFECYCLE: &str = "cleanup.jsonl";
const SCHEMA_VERSION: u64 = 1;
const MAX_STREAM_BYTES: u64 = 16 * 1024 * 1024;
const MAX_STREAM_RECORDS: usize = 65_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleStreamStatus {
    Complete,
    CrashPrefix,
    UnknownVersion,
}

#[derive(Clone, Debug)]
pub struct LifecyclePrefix {
    pub stream: String,
    pub session_id: String,
    pub records: Vec<Value>,
    pub status: LifecycleStreamStatus,
}

pub struct LifecycleWriter {
    path: PathBuf,
    file: File,
    stream: String,
    session_id: String,
    sequence: u64,
    counts: BTreeMap<String, u64>,
    finished: bool,
}

impl LifecycleWriter {
    pub fn create(
        path: impl Into<PathBuf>,
        stream: impl Into<String>,
        session_id: impl Into<String>,
    ) -> io::Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = open_sensitive_file(&path)?;
        let mut writer = Self {
            path,
            file,
            stream: stream.into(),
            session_id: session_id.into(),
            sequence: 0,
            counts: BTreeMap::new(),
            finished: false,
        };
        writer.write_sync(&json!({
            "type": "lifecycle.header",
            "schema_version": SCHEMA_VERSION,
            "stream": writer.stream,
            "session_id": writer.session_id,
        }))?;
        Ok(writer)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn resume(path: &Path) -> io::Result<Self> {
        let prefix = read_lifecycle_prefix(path)?;
        if prefix.status == LifecycleStreamStatus::UnknownVersion {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "an unknown lifecycle stream version cannot resume",
            ));
        }
        if prefix.status == LifecycleStreamStatus::Complete {
            remove_terminal_trailer(path, "lifecycle.trailer")?;
        }
        let mut counts = BTreeMap::new();
        for record in &prefix.records {
            let record_type = required_string(record, "type")?;
            *counts.entry(record_type).or_default() += 1;
        }
        Ok(Self {
            path: path.to_path_buf(),
            file: OpenOptions::new().append(true).read(true).open(path)?,
            stream: prefix.stream,
            session_id: prefix.session_id,
            sequence: prefix.records.len() as u64,
            counts,
            finished: false,
        })
    }

    pub fn append(&mut self, record_type: &str, fields: Value) -> io::Result<u64> {
        if self.finished {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "lifecycle stream is finished",
            ));
        }
        self.sequence = self.sequence.checked_add(1).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "lifecycle sequence overflow")
        })?;
        *self.counts.entry(record_type.to_owned()).or_default() += 1;
        self.write_sync(&json!({
            "type": record_type,
            "schema_version": SCHEMA_VERSION,
            "sequence": self.sequence,
            "session_id": self.session_id,
            "fields": fields,
        }))?;
        Ok(self.sequence)
    }

    pub fn gap(&mut self, scope: &str, reason: &str, lost_records: Option<u64>) -> io::Result<u64> {
        self.append(
            "lifecycle.gap",
            json!({"scope": scope, "reason": reason, "lost_records": lost_records}),
        )
    }

    pub fn finish(&mut self) -> io::Result<()> {
        if self.finished {
            return Ok(());
        }
        self.write_sync(&json!({
            "type": "lifecycle.trailer",
            "schema_version": SCHEMA_VERSION,
            "stream": self.stream,
            "session_id": self.session_id,
            "records": self.sequence,
            "counts": self.counts,
        }))?;
        self.finished = true;
        Ok(())
    }

    fn write_sync(&mut self, value: &Value) -> io::Result<()> {
        serde_json::to_writer(&mut self.file, value).map_err(io::Error::other)?;
        self.file.write_all(b"\n")?;
        self.file.sync_all()
    }
}

pub fn read_lifecycle_prefix(path: &Path) -> io::Result<LifecyclePrefix> {
    if fs::metadata(path)?.len() > MAX_STREAM_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "lifecycle stream exceeds byte limit",
        ));
    }
    let ends_with_newline = fs::read(path)?.last() == Some(&b'\n');
    let mut lines = BufReader::new(File::open(path)?).lines();
    let header = lines.next().ok_or_else(|| {
        io::Error::new(io::ErrorKind::UnexpectedEof, "lifecycle stream is empty")
    })??;
    let header: Value = serde_json::from_str(&header).map_err(invalid_json)?;
    if header.get("type").and_then(Value::as_str) != Some("lifecycle.header") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "lifecycle header is missing",
        ));
    }
    if header.get("schema_version").and_then(Value::as_u64) != Some(SCHEMA_VERSION) {
        return Ok(LifecyclePrefix {
            stream: String::new(),
            session_id: String::new(),
            records: Vec::new(),
            status: LifecycleStreamStatus::UnknownVersion,
        });
    }
    let stream = required_string(&header, "stream")?;
    let session_id = required_string(&header, "session_id")?;
    let mut records = Vec::new();
    let mut trailer = false;
    let mut lines = lines.enumerate().peekable();
    while let Some((index, line)) = lines.next() {
        if index >= MAX_STREAM_RECORDS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "lifecycle stream exceeds record limit",
            ));
        }
        let line = line?;
        let value: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) if lines.peek().is_none() && !ends_with_newline => break,
            Err(error) => return Err(invalid_json(error)),
        };
        let record_type = value
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if record_type == "lifecycle.trailer" {
            trailer = true;
            continue;
        }
        if trailer
            || value.get("sequence").and_then(Value::as_u64) != Some(records.len() as u64 + 1)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid lifecycle sequence",
            ));
        }
        records.push(value);
    }
    Ok(LifecyclePrefix {
        stream,
        session_id,
        records,
        status: if trailer {
            LifecycleStreamStatus::Complete
        } else {
            LifecycleStreamStatus::CrashPrefix
        },
    })
}

#[derive(Default)]
struct ProxyAccount {
    accepted: AtomicU64,
    dropped: AtomicU64,
    written: AtomicU64,
    retired: AtomicBool,
    first_dropped_connection_id: AtomicU64,
    dropped_connection_opens: AtomicU64,
}

struct ProxySink {
    sender: Mutex<Option<mpsc::SyncSender<ProxyLifecycleMessage>>>,
    account: Arc<ProxyAccount>,
}

enum ProxyLifecycleMessage {
    Event(Box<ApplicationEvent>),
    ListenerStarted(String),
    ListenerFailed(String),
}

impl ApplicationEventSink for ProxySink {
    fn try_emit(&self, event: ApplicationEvent) -> EventDisposition {
        if self.account.retired.load(Ordering::Acquire) {
            self.account.dropped.fetch_add(1, Ordering::Relaxed);
            return EventDisposition::Retired;
        }
        let sender = self.sender.lock().expect("proxy lifecycle sender lock");
        let Some(sender) = sender.as_ref() else {
            self.account.dropped.fetch_add(1, Ordering::Relaxed);
            return EventDisposition::Retired;
        };
        let connection_open = matches!(event.kind, ApplicationEventKind::ConnectionOpen(_));
        let connection_id = event.connection_id;
        match sender.try_send(ProxyLifecycleMessage::Event(Box::new(event))) {
            Ok(()) => {
                self.account.accepted.fetch_add(1, Ordering::Relaxed);
                EventDisposition::Accepted
            }
            Err(mpsc::TrySendError::Full(_)) => {
                if connection_open {
                    let _ = self.account.first_dropped_connection_id.compare_exchange(
                        0,
                        connection_id,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    );
                    self.account
                        .dropped_connection_opens
                        .fetch_add(1, Ordering::Relaxed);
                }
                self.account.dropped.fetch_add(1, Ordering::Relaxed);
                EventDisposition::QueueFull
            }
            Err(mpsc::TrySendError::Disconnected(_)) => {
                self.account.retired.store(true, Ordering::Release);
                self.account.dropped.fetch_add(1, Ordering::Relaxed);
                EventDisposition::Retired
            }
        }
    }

    fn accounting(&self) -> ApplicationSinkAccounting {
        ApplicationSinkAccounting {
            accepted_events: self.account.accepted.load(Ordering::Acquire),
            dropped_events: self.account.dropped.load(Ordering::Acquire),
            ..ApplicationSinkAccounting::default()
        }
    }
}

pub struct ProxyLifecycleLease {
    sink: Arc<ProxySink>,
    worker: Option<JoinHandle<io::Result<()>>>,
}

impl ProxyLifecycleLease {
    pub fn open(
        path: impl Into<PathBuf>,
        session_id: impl Into<String>,
        capacity: usize,
    ) -> io::Result<Self> {
        Self::open_with_listener(path, session_id, capacity, "loopback-listener")
    }

    pub fn open_with_listener(
        path: impl Into<PathBuf>,
        session_id: impl Into<String>,
        capacity: usize,
        listener: impl Into<String>,
    ) -> io::Result<Self> {
        let mut writer = LifecycleWriter::create(path, "proxy", session_id)?;
        writer.append(
            "proxy.listener-attempt",
            json!({"listener": listener.into(), "scope": "loopback-only"}),
        )?;
        writer.gap(
            "dns.success",
            "the native runtime does not expose successful resolver answers separately from upstream connection evidence",
            None,
        )?;
        let (sender, receiver) = mpsc::sync_channel(capacity.max(1));
        let account = Arc::new(ProxyAccount::default());
        let sink = Arc::new(ProxySink {
            sender: Mutex::new(Some(sender)),
            account: Arc::clone(&account),
        });
        let worker = std::thread::Builder::new()
            .name("fragcap-proxy-lifecycle-writer".into())
            .spawn(move || proxy_writer_loop(writer, receiver, account))?;
        Ok(Self {
            sink,
            worker: Some(worker),
        })
    }

    pub fn sink(&self) -> Arc<dyn ApplicationEventSink> {
        self.sink.clone()
    }

    pub fn listener_started(&self, listener: impl Into<String>) -> io::Result<()> {
        self.sink
            .sender
            .lock()
            .expect("proxy lifecycle sender lock")
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "lifecycle is retired"))?
            .send(ProxyLifecycleMessage::ListenerStarted(listener.into()))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "lifecycle writer retired"))
    }

    pub fn listener_failed(&self, detail: impl Into<String>) -> io::Result<()> {
        self.sink
            .sender
            .lock()
            .expect("proxy lifecycle sender lock")
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "lifecycle is retired"))?
            .send(ProxyLifecycleMessage::ListenerFailed(detail.into()))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "lifecycle writer retired"))
    }

    pub fn finish(&mut self) -> io::Result<()> {
        self.sink.account.retired.store(true, Ordering::Release);
        let sender = self
            .sink
            .sender
            .lock()
            .expect("proxy lifecycle sender lock")
            .take();
        drop(sender);
        match self.worker.take() {
            Some(worker) => worker
                .join()
                .map_err(|_| io::Error::other("proxy lifecycle writer panicked"))?,
            None => Ok(()),
        }
    }
}

impl Drop for ProxyLifecycleLease {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

fn proxy_writer_loop(
    mut writer: LifecycleWriter,
    receiver: mpsc::Receiver<ProxyLifecycleMessage>,
    account: Arc<ProxyAccount>,
) -> io::Result<()> {
    let mut listener_started = false;
    for message in receiver {
        match message {
            ProxyLifecycleMessage::ListenerStarted(listener) => {
                listener_started = true;
                writer.append(
                    "proxy.listener-started",
                    json!({"listener": listener, "scope": "loopback-only"}),
                )?;
            }
            ProxyLifecycleMessage::ListenerFailed(detail) => {
                writer.append("proxy.listener-failed", json!({"detail": detail}))?;
            }
            ProxyLifecycleMessage::Event(event) => {
                writer.append(
                    proxy_event_type(&event.kind),
                    json!({
                        "connection_id": event.connection_id,
                        "stream_id": event.stream_id,
                        "timestamp_ns": event.timestamp_ns,
                        "protocol": event.protocol.map(|value| format!("{value:?}")),
                        "detail": proxy_event_detail(&event.kind),
                    }),
                )?;
                account.written.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    let dropped = account.dropped.load(Ordering::Acquire);
    let dropped_connection_opens = account.dropped_connection_opens.load(Ordering::Acquire);
    if dropped_connection_opens > 0 {
        writer.append(
            "proxy.connection-gap",
            json!({
                "first_connection_id": account.first_dropped_connection_id.load(Ordering::Acquire),
                "connection_count": dropped_connection_opens,
                "reason": "bounded-lifecycle-queue",
            }),
        )?;
    }
    if dropped > 0 {
        writer.gap(
            "proxy.events",
            "bounded lifecycle queue dropped records",
            Some(dropped),
        )?;
    }
    writer.append(
        "proxy.writer-accounting",
        json!({
            "accepted": account.accepted.load(Ordering::Acquire),
            "written": account.written.load(Ordering::Acquire),
            "dropped": dropped,
        }),
    )?;
    if listener_started {
        writer.append("proxy.listener-stopped", json!({"status": "released"}))?;
    }
    writer.append("proxy.drain", json!({"status": "complete"}))?;
    writer.finish()
}

fn proxy_event_type(kind: &ApplicationEventKind) -> &'static str {
    match kind {
        ApplicationEventKind::ConnectionOpen(_) => "proxy.connection-open",
        ApplicationEventKind::UpstreamSocket(_) => "proxy.upstream-socket",
        ApplicationEventKind::ConnectionTerminal(_) => "proxy.connection-terminal",
        ApplicationEventKind::TlsNegotiation(_) => "proxy.tls-negotiation",
        ApplicationEventKind::TlsTerminal(_) => "proxy.tls-terminal",
        ApplicationEventKind::HttpStreamOpen => "proxy.protocol-open",
        ApplicationEventKind::HttpStreamTerminal(_) => "proxy.protocol-terminal",
        ApplicationEventKind::HttpTiming(_) => "proxy.protocol-timing",
        ApplicationEventKind::Metadata(_) => "proxy.metadata",
        ApplicationEventKind::Body(_) => "proxy.body",
        ApplicationEventKind::Transformation(_) => "proxy.transformation",
        ApplicationEventKind::Streaming(_) => "proxy.streaming",
        ApplicationEventKind::SocksNegotiation(_) => "proxy.socks5-negotiation",
        ApplicationEventKind::SocksConnect(_) => "proxy.socks5-connect",
        ApplicationEventKind::SocksTransfer(_) => "proxy.socks5-transfer",
        ApplicationEventKind::SocksUdp(_) => "proxy.socks5-udp",
        ApplicationEventKind::GenericStreamChunk(_) => "proxy.generic-stream-chunk",
        ApplicationEventKind::GenericUdpDatagram(_) => "proxy.generic-udp-datagram",
        ApplicationEventKind::UdpSocketError(_) => "proxy.generic-udp-socket-error",
        ApplicationEventKind::QuicConnection(_) => "proxy.quic-connection",
        ApplicationEventKind::QuicStream(_) => "proxy.quic-stream",
        ApplicationEventKind::QuicDatagram(_) => "proxy.quic-datagram",
        ApplicationEventKind::QuicRefusal(_) => "proxy.quic-refusal",
        ApplicationEventKind::Error { .. } => "proxy.error",
    }
}

fn proxy_event_detail(kind: &ApplicationEventKind) -> Value {
    match kind {
        ApplicationEventKind::ConnectionOpen(value) => json!({
            "transport": value.transport,
            "client_peer": value.client_peer.to_string(),
            "proxy_local": value.proxy_local.to_string(),
        }),
        ApplicationEventKind::UpstreamSocket(value) => json!({
            "protocol": value.protocol,
            "upstream_local": value.local.to_string(),
            "upstream_peer": value.peer.to_string(),
        }),
        ApplicationEventKind::SocksNegotiation(value) => json!({
            "authenticated": value.authenticated,
        }),
        ApplicationEventKind::SocksConnect(value) => json!({
            "authority": value.authority,
            "upstream_local": value.upstream_local.map(|address| address.to_string()),
            "selected_peer": value.selected_peer.map(|address| address.to_string()),
            "address_type": value.address_type,
            "dns_owner": value.dns_owner,
            "outcome": value.outcome,
            "classification": value.classification.map(|classification| classification.as_str()),
        }),
        ApplicationEventKind::SocksTransfer(value) => json!({
            "client_to_upstream_bytes": value.client_to_upstream_bytes,
            "upstream_to_client_bytes": value.upstream_to_client_bytes,
        }),
        ApplicationEventKind::SocksUdp(value) => json!({
            "action": value.action,
            "outcome": value.outcome,
            "address_type": value.address_type,
            "remote": value.remote.map(|address| address.to_string()),
            "payload_bytes": value.payload_bytes,
            "active_peers": value.active_peers,
            "payload_retained": false,
        }),
        ApplicationEventKind::GenericStreamChunk(value) => json!({
            "direction": value.direction.as_str(),
            "provenance": value.provenance.as_str(),
            "offset": value.offset,
            "observed_len": value.observed_len,
            "retained_len": value.bytes.len(),
            "outcome": value.outcome.as_str(),
        }),
        ApplicationEventKind::GenericUdpDatagram(value) => json!({
            "direction": value.direction.as_str(),
            "datagram_sequence": value.sequence,
            "client_endpoint": value.client_endpoint.to_string(),
            "remote_endpoint": value.remote_endpoint.to_string(),
            "observed_len": value.observed_len,
            "retained_len": value.bytes.len(),
            "outcome": value.outcome.as_str(),
        }),
        ApplicationEventKind::UdpSocketError(value) => json!({
            "direction": value.direction.as_str(),
            "operation": value.operation,
            "failure_code": value.failure_code,
            "endpoint": value.endpoint.map(|address| address.to_string()),
            "error_kind": value.error_kind,
            "visibility": value.visibility,
            "icmp": "unavailable",
        }),
        ApplicationEventKind::Error { code } => json!({"code": code}),
        _ => json!({"kind": format!("{kind:?}")}),
    }
}

fn required_string(value: &Value, field: &str) -> io::Result<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("lifecycle field `{field}` is missing"),
            )
        })
}

fn invalid_json(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

fn remove_terminal_trailer(path: &Path, expected_type: &str) -> io::Result<()> {
    let bytes = fs::read(path)?;
    let mut end = bytes.len();
    while end > 0 && matches!(bytes[end - 1], b'\n' | b'\r') {
        end -= 1;
    }
    let start = bytes[..end]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |position| position + 1);
    let trailer: Value = serde_json::from_slice(&bytes[start..end]).map_err(invalid_json)?;
    if trailer.get("type").and_then(Value::as_str) != Some(expected_type) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "complete lifecycle stream does not end in its trailer",
        ));
    }
    let file = OpenOptions::new().write(true).open(path)?;
    file.set_len(start as u64)?;
    file.sync_all()
}
