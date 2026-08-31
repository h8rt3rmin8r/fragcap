// SPDX-License-Identifier: Apache-2.0

//! Versioned, append-only Deep Capture application observations.

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;

use base64::Engine;
use fragcap_proxy::{
    ApplicationEvent, ApplicationEventKind, ApplicationEventSink, ApplicationSinkAccounting,
    BodyOutcome, BodyRepresentation, EventDisposition, MetadataOrdering, ProtocolVersion,
};
use serde_json::{json, Value};

const SCHEMA_VERSION: u64 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationStreamStatus {
    Complete,
    Incomplete,
    UnknownVersion,
}

#[derive(Clone, Debug)]
pub struct ApplicationPrefix {
    pub schema_version: u64,
    pub records: Vec<Value>,
    pub status: ApplicationStreamStatus,
}

#[derive(Default)]
struct WriterAccount {
    accepted: AtomicU64,
    dropped: AtomicU64,
    written: AtomicU64,
    serialized_bytes: AtomicU64,
    failures: AtomicU64,
    body_bytes_queue_dropped: AtomicU64,
    body_retained_bytes_queue_dropped: AtomicU64,
    body_queue_losses: Mutex<BTreeMap<BodyDropKey, BodyDropTotals>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct BodyDropKey {
    connection_id: u64,
    stream_id: Option<u64>,
    direction: &'static str,
    representation: &'static str,
}

#[derive(Clone, Copy, Debug, Default)]
struct BodyDropTotals {
    records: u64,
    observed_bytes: u64,
    retained_bytes: u64,
}

impl WriterAccount {
    fn record_queue_drop(&self, event: &ApplicationEvent) {
        let ApplicationEventKind::Body(segment) = &event.kind else {
            return;
        };
        self.body_bytes_queue_dropped
            .fetch_add(segment.observed_len, Ordering::Relaxed);
        self.body_retained_bytes_queue_dropped
            .fetch_add(segment.bytes.len() as u64, Ordering::Relaxed);
        let key = BodyDropKey {
            connection_id: event.connection_id,
            stream_id: event.stream_id,
            direction: match segment.direction {
                fragcap_proxy::BodyDirection::Request => "request",
                fragcap_proxy::BodyDirection::Response => "response",
            },
            representation: match segment.representation {
                BodyRepresentation::Raw => "raw",
                BodyRepresentation::TransferDecoded => "transfer-decoded",
                BodyRepresentation::ContentDecoded => "content-decoded",
            },
        };
        let mut losses = self.body_queue_losses.lock().expect("body queue loss lock");
        let totals = losses.entry(key).or_default();
        totals.records = totals.records.saturating_add(1);
        totals.observed_bytes = totals.observed_bytes.saturating_add(segment.observed_len);
        totals.retained_bytes = totals
            .retained_bytes
            .saturating_add(segment.bytes.len() as u64);
    }
}

struct ChannelSink {
    sender: Mutex<Option<mpsc::SyncSender<ApplicationEvent>>>,
    retired: Arc<AtomicBool>,
    account: Arc<WriterAccount>,
}

impl ApplicationEventSink for ChannelSink {
    fn try_emit(&self, event: ApplicationEvent) -> EventDisposition {
        if self.retired.load(Ordering::Acquire) {
            self.account.dropped.fetch_add(1, Ordering::Relaxed);
            return EventDisposition::Retired;
        }
        let sender = self.sender.lock().expect("application sender lock");
        let Some(sender) = sender.as_ref() else {
            self.account.dropped.fetch_add(1, Ordering::Relaxed);
            return EventDisposition::Retired;
        };
        match sender.try_send(event) {
            Ok(()) => {
                self.account.accepted.fetch_add(1, Ordering::Relaxed);
                EventDisposition::Accepted
            }
            Err(mpsc::TrySendError::Full(event)) => {
                self.account.record_queue_drop(&event);
                self.account.dropped.fetch_add(1, Ordering::Relaxed);
                EventDisposition::QueueFull
            }
            Err(mpsc::TrySendError::Disconnected(_)) => {
                self.retired.store(true, Ordering::Release);
                self.account.dropped.fetch_add(1, Ordering::Relaxed);
                EventDisposition::Retired
            }
        }
    }

    fn accounting(&self) -> ApplicationSinkAccounting {
        ApplicationSinkAccounting {
            accepted_events: self.account.accepted.load(Ordering::Acquire),
            dropped_events: self.account.dropped.load(Ordering::Acquire),
            body_bytes_queue_dropped: self
                .account
                .body_bytes_queue_dropped
                .load(Ordering::Acquire),
        }
    }
}

pub struct ApplicationArtifactLease {
    path: PathBuf,
    sink: Arc<ChannelSink>,
    worker: Option<JoinHandle<io::Result<()>>>,
}

#[derive(Clone, Debug, Default)]
pub struct ApplicationCorrelation {
    pub target_id: Option<i64>,
    pub process_id: Option<u32>,
    pub process_image: Option<String>,
    pub role: Option<String>,
    pub attribution: Option<String>,
}

impl ApplicationArtifactLease {
    pub fn open(
        path: impl Into<PathBuf>,
        session_id: impl Into<String>,
        capacity: usize,
    ) -> io::Result<Self> {
        Self::open_correlated(
            path,
            session_id,
            capacity,
            Arc::new(ApplicationCorrelation::default),
        )
    }

    pub fn open_correlated(
        path: impl Into<PathBuf>,
        session_id: impl Into<String>,
        capacity: usize,
        correlation: Arc<dyn Fn() -> ApplicationCorrelation + Send + Sync>,
    ) -> io::Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)?;
        let session_id = session_id.into();
        write_record(&mut file, &header_record(&session_id))?;
        let (sender, receiver) = mpsc::sync_channel(capacity.max(1));
        let retired = Arc::new(AtomicBool::new(false));
        let account = Arc::new(WriterAccount::default());
        let sink = Arc::new(ChannelSink {
            sender: Mutex::new(Some(sender)),
            retired: Arc::clone(&retired),
            account: Arc::clone(&account),
        });
        let worker_account = Arc::clone(&account);
        let worker = std::thread::Builder::new()
            .name("fragcap-application-writer".to_string())
            .spawn(move || {
                writer_loop(
                    file,
                    &session_id,
                    receiver,
                    retired,
                    worker_account,
                    correlation,
                )
            })?;
        Ok(Self {
            path,
            sink,
            worker: Some(worker),
        })
    }

    pub fn sink(&self) -> Arc<dyn ApplicationEventSink> {
        self.sink.clone()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn finish(&mut self) -> io::Result<()> {
        self.sink.retired.store(true, Ordering::Release);
        let sender = self
            .sink
            .sender
            .lock()
            .expect("application sender lock")
            .take();
        drop(sender);
        match self.worker.take() {
            Some(worker) => worker
                .join()
                .map_err(|_| io::Error::other("application writer panicked"))?,
            None => Ok(()),
        }
    }
}

impl Drop for ApplicationArtifactLease {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

fn writer_loop(
    file: File,
    session_id: &str,
    receiver: mpsc::Receiver<ApplicationEvent>,
    retired: Arc<AtomicBool>,
    account: Arc<WriterAccount>,
    correlation: Arc<dyn Fn() -> ApplicationCorrelation + Send + Sync>,
) -> io::Result<()> {
    let mut writer = BufWriter::new(file);
    let mut sequence = 0_u64;
    let mut records_by_type = BTreeMap::<String, u64>::new();
    let mut body_observed = 0_u64;
    let mut body_retained = 0_u64;
    let mut body_truncated = 0_u64;
    for event in receiver {
        let record_type = event_type(&event.kind).to_string();
        *records_by_type.entry(record_type).or_default() += 1;
        if let ApplicationEventKind::Body(segment) = &event.kind {
            body_observed = body_observed.saturating_add(segment.observed_len);
            body_retained = body_retained.saturating_add(segment.bytes.len() as u64);
            if segment.outcome == BodyOutcome::RetentionLimit {
                body_truncated = body_truncated.saturating_add(
                    segment
                        .observed_len
                        .saturating_sub(segment.bytes.len() as u64),
                );
            }
        }
        sequence = sequence.saturating_add(1);
        let value = event_json(event, sequence, correlation());
        match write_record(&mut writer, &value) {
            Ok(()) => {
                account.written.fetch_add(1, Ordering::Relaxed);
                account
                    .serialized_bytes
                    .fetch_add(value.to_string().len() as u64, Ordering::Relaxed);
            }
            Err(error) => {
                account.failures.fetch_add(1, Ordering::Relaxed);
                retired.store(true, Ordering::Release);
                return Err(error);
            }
        }
    }
    let dropped = account.dropped.load(Ordering::Acquire);
    let body_bytes_queue_dropped = account.body_bytes_queue_dropped.load(Ordering::Acquire);
    let body_retained_bytes_queue_dropped = account
        .body_retained_bytes_queue_dropped
        .load(Ordering::Acquire);
    let body_queue_losses = account
        .body_queue_losses
        .lock()
        .expect("body queue loss lock")
        .iter()
        .map(|(key, totals)| {
            json!({
                "proxy_connection_id": key.connection_id,
                "http_stream_id": key.stream_id,
                "direction": key.direction,
                "representation": key.representation,
                "outcome": "queue-dropped",
                "dropped_records": totals.records,
                "observed_bytes": totals.observed_bytes,
                "retained_bytes": totals.retained_bytes,
            })
        })
        .collect::<Vec<_>>();
    if dropped > 0 {
        sequence = sequence.saturating_add(1);
        write_record(
            &mut writer,
            &json!({
                "type": "application.gap",
                "schema_version": SCHEMA_VERSION,
                "session_id": session_id,
                "sequence": sequence,
                "event_time_ns": 0,
                "dropped_records": dropped,
                "reason": "event-queue-or-retired-writer",
                "body_bytes_queue_dropped": body_bytes_queue_dropped,
                "body_retained_bytes_queue_dropped": body_retained_bytes_queue_dropped,
                "body_losses": body_queue_losses,
            }),
        )?;
    }
    sequence = sequence.saturating_add(1);
    write_record(
        &mut writer,
        &json!({
            "type": "application.trailer",
            "schema_version": SCHEMA_VERSION,
            "session_id": session_id,
            "sequence": sequence,
            "event_time_ns": 0,
            "writer_status": "complete",
            "accepted_records": account.accepted.load(Ordering::Acquire),
            "written_records": account.written.load(Ordering::Acquire),
            "dropped_records": dropped,
            "serialized_bytes": account.serialized_bytes.load(Ordering::Acquire),
            "writer_failures": account.failures.load(Ordering::Acquire)
            ,"records_by_type": records_by_type
            ,"body_bytes_observed": body_observed
            ,"body_bytes_retained": body_retained
            ,"body_bytes_truncated": body_truncated
            ,"body_bytes_queue_dropped": body_bytes_queue_dropped
            ,"body_retained_bytes_queue_dropped": body_retained_bytes_queue_dropped
        }),
    )
}

fn header_record(session_id: &str) -> Value {
    json!({
        "type": "application.header",
        "schema_version": SCHEMA_VERSION,
        "session_id": session_id,
        "sequence": 0,
        "event_time_ns": 0,
        "exports": ["connection", "tls", "http", "metadata", "body", "transformation"],
        "non_exports": {
            "websocket": "deferred-issue-295",
            "sse": "deferred-issue-298",
            "grpc": "deferred-issue-299",
            "tcp": "deferred-issue-306",
            "udp": "deferred-issue-307",
            "quic": "deferred-issue-305"
        }
    })
}

fn write_record(writer: &mut impl Write, value: &Value) -> io::Result<()> {
    serde_json::to_writer(&mut *writer, value).map_err(io::Error::other)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

fn event_json(
    event: ApplicationEvent,
    sequence: u64,
    correlation: ApplicationCorrelation,
) -> Value {
    let protocol = event.protocol.map(|value| match value {
        ProtocolVersion::Http11 => "http/1.1",
        ProtocolVersion::Http2 => "h2",
    });
    let common = json!({
        "schema_version": SCHEMA_VERSION,
        "session_id": event.session_id,
        "sequence": sequence,
        "event_time_ns": event.timestamp_ns,
        "proxy_connection_id": event.connection_id,
        "http_stream_id": event.stream_id,
        "protocol": protocol,
        "target_id": correlation.target_id,
        "process_id": correlation.process_id,
        "process_image": correlation.process_image,
        "role": correlation.role,
        "attribution": correlation.attribution,
    });
    let mut object = common.as_object().cloned().unwrap_or_default();
    let (kind, detail) = match event.kind {
        ApplicationEventKind::ConnectionOpen => ("connection.open", json!({})),
        ApplicationEventKind::ConnectionTerminal(outcome) => (
            "connection.terminal",
            json!({"outcome": format!("{outcome:?}").to_ascii_lowercase()}),
        ),
        ApplicationEventKind::TlsNegotiation(value) => (
            "tls.negotiation",
            json!({
                "boundary": format!("{:?}", value.boundary).to_ascii_lowercase(),
                "requested_identity": value.requested_identity,
                "tls_version": value.version,
                "alpn_encoding": "base64",
                "alpn": value.alpn.map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes)),
            }),
        ),
        ApplicationEventKind::TlsTerminal(outcome) => (
            "tls.terminal",
            json!({"outcome": format!("{outcome:?}").to_ascii_lowercase()}),
        ),
        ApplicationEventKind::HttpStreamOpen => {
            ("http.stream.open", json!({"inspectability": "full"}))
        }
        ApplicationEventKind::HttpStreamTerminal(outcome) => (
            "http.stream.terminal",
            json!({"outcome": format!("{outcome:?}").to_ascii_lowercase()}),
        ),
        ApplicationEventKind::Metadata(block) => (
            "http.metadata",
            json!({
                "inspectability": "full",
                "kind": format!("{:?}", block.kind).to_ascii_lowercase(),
                "ordering": match block.ordering { MetadataOrdering::Wire => "wire", MetadataOrdering::DecodedPerName => "decoded-per-name" },
                "pseudo_fields": block.pseudo_fields.into_iter().map(field_json).collect::<Vec<_>>(),
                "fields": block.fields.into_iter().map(field_json).collect::<Vec<_>>(),
                "method": block.method.map(binary_json),
                "target": block.target.map(binary_json),
                "status": block.status,
                "reason": block.reason.map(binary_json),
                "query": block.query.into_iter().map(derived_json).collect::<Vec<_>>(),
                "cookies": block.cookies.into_iter().map(derived_json).collect::<Vec<_>>(),
                "unavailable": block.unavailable,
            }),
        ),
        ApplicationEventKind::Body(segment) => ("http.body_segment", body_json(segment)),
        ApplicationEventKind::Transformation(value) => (
            "http.transformation",
            json!({
                "encoding": value.encoding,
                "input_bytes": value.input_bytes,
                "output_bytes": value.output_bytes,
                "outcome": body_outcome(value.outcome),
            }),
        ),
        ApplicationEventKind::Error { code } => ("application.error", json!({"code": code})),
    };
    object.insert("type".to_string(), Value::String(kind.to_string()));
    if let Some(detail) = detail.as_object() {
        object.extend(detail.clone());
    }
    Value::Object(object)
}

fn event_type(kind: &ApplicationEventKind) -> &'static str {
    match kind {
        ApplicationEventKind::ConnectionOpen => "connection.open",
        ApplicationEventKind::ConnectionTerminal(_) => "connection.terminal",
        ApplicationEventKind::TlsNegotiation(_) => "tls.negotiation",
        ApplicationEventKind::TlsTerminal(_) => "tls.terminal",
        ApplicationEventKind::HttpStreamOpen => "http.stream.open",
        ApplicationEventKind::HttpStreamTerminal(_) => "http.stream.terminal",
        ApplicationEventKind::Metadata(_) => "http.metadata",
        ApplicationEventKind::Body(_) => "http.body_segment",
        ApplicationEventKind::Transformation(_) => "http.transformation",
        ApplicationEventKind::Error { .. } => "application.error",
    }
}

fn body_json(segment: fragcap_proxy::BodySegment) -> Value {
    let mut value = json!({
        "inspectability": "full",
        "direction": format!("{:?}", segment.direction).to_ascii_lowercase(),
        "representation": match segment.representation { BodyRepresentation::Raw => "raw", BodyRepresentation::TransferDecoded => "transfer-decoded", BodyRepresentation::ContentDecoded => "content-decoded" },
        "offset": segment.offset,
        "observed_len": segment.observed_len,
        "retained_len": segment.bytes.len(),
        "outcome": body_outcome(segment.outcome),
    });
    if segment.outcome != BodyOutcome::IntentionallyOmitted {
        let object = value.as_object_mut().expect("body record is an object");
        object.insert("payload_encoding".to_string(), json!("base64"));
        object.insert(
            "payload".to_string(),
            json!(base64::engine::general_purpose::STANDARD.encode(segment.bytes)),
        );
    }
    value
}

fn field_json(field: fragcap_proxy::MetadataField) -> Value {
    json!({
        "name_encoding": "base64",
        "name": base64::engine::general_purpose::STANDARD.encode(field.name),
        "value_encoding": "base64",
        "value": base64::engine::general_purpose::STANDARD.encode(field.value),
        "original_index": field.original_index,
        "sensitive": field.sensitive,
    })
}

fn binary_json(bytes: Vec<u8>) -> Value {
    json!({
        "encoding": "base64",
        "value": base64::engine::general_purpose::STANDARD.encode(bytes),
    })
}

fn derived_json(value: fragcap_proxy::DerivedMetadataValue) -> Value {
    json!({
        "name": binary_json(value.name),
        "value": binary_json(value.value),
        "source_field_index": value.source_field_index,
        "source_component": value.source_component,
        "decode_valid": value.decode_valid,
    })
}

fn body_outcome(value: BodyOutcome) -> &'static str {
    match value {
        BodyOutcome::Complete => "complete",
        BodyOutcome::Partial => "partial",
        BodyOutcome::IntentionallyOmitted => "intentionally-omitted",
        BodyOutcome::RetentionLimit => "retention-limit",
        BodyOutcome::QueueDropped => "queue-dropped",
        BodyOutcome::StorageFailed => "storage-failed",
        BodyOutcome::UnsupportedEncoding => "unsupported-encoding",
        BodyOutcome::MalformedEncoding => "malformed-encoding",
        BodyOutcome::ExpansionLimit => "expansion-limit",
        BodyOutcome::TimeLimit => "time-limit",
        BodyOutcome::Cancelled => "cancelled",
    }
}

pub fn read_application_prefix(path: &Path) -> io::Result<ApplicationPrefix> {
    let reader = BufReader::new(File::open(path)?);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(&line) {
            Ok(value) => records.push(value),
            Err(_) => break,
        }
    }
    let first = records.first().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "application stream has no header",
        )
    })?;
    let schema_version = first
        .get("schema_version")
        .and_then(Value::as_u64)
        .unwrap_or_else(|| {
            if first.get("manifest_version").and_then(Value::as_u64) == Some(1) {
                1
            } else {
                0
            }
        });
    let header_session = first.get("session_id").and_then(Value::as_str);
    let valid_v2 = schema_version != 2
        || records.iter().enumerate().all(|(index, record)| {
            record.get("schema_version").and_then(Value::as_u64) == Some(2)
                && record.get("sequence").and_then(Value::as_u64) == Some(index as u64)
                && record.get("session_id").and_then(Value::as_str) == header_session
        });
    let trailers = records
        .iter()
        .filter(|value| value.get("type").and_then(Value::as_str) == Some("application.trailer"))
        .count();
    let status = if !matches!(schema_version, 1 | 2) {
        ApplicationStreamStatus::UnknownVersion
    } else if valid_v2
        && trailers == 1
        && records
            .last()
            .and_then(|value| value.get("type"))
            .and_then(Value::as_str)
            == Some("application.trailer")
        && trailer_reconciles(&records)
    {
        ApplicationStreamStatus::Complete
    } else {
        ApplicationStreamStatus::Incomplete
    };
    Ok(ApplicationPrefix {
        schema_version,
        records,
        status,
    })
}

fn trailer_reconciles(records: &[Value]) -> bool {
    let Some(trailer) = records.last() else {
        return false;
    };
    let mut records_by_type = BTreeMap::<String, u64>::new();
    let mut written = 0_u64;
    let mut serialized_bytes = 0_u64;
    let mut dropped = 0_u64;
    let mut body_observed = 0_u64;
    let mut body_retained = 0_u64;
    let mut body_truncated = 0_u64;
    let mut body_queue_dropped = 0_u64;
    let mut body_retained_queue_dropped = 0_u64;
    for record in records.iter().skip(1).take(records.len().saturating_sub(2)) {
        let Some(kind) = record.get("type").and_then(Value::as_str) else {
            return false;
        };
        if kind == "application.gap" {
            dropped = dropped.saturating_add(
                record
                    .get("dropped_records")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
            );
            body_queue_dropped = body_queue_dropped.saturating_add(
                record
                    .get("body_bytes_queue_dropped")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
            );
            body_retained_queue_dropped = body_retained_queue_dropped.saturating_add(
                record
                    .get("body_retained_bytes_queue_dropped")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
            );
            continue;
        }
        written = written.saturating_add(1);
        serialized_bytes = serialized_bytes.saturating_add(record.to_string().len() as u64);
        *records_by_type.entry(kind.to_string()).or_default() += 1;
        if kind == "http.body_segment" {
            let observed = record
                .get("observed_len")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let retained = record
                .get("retained_len")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            body_observed = body_observed.saturating_add(observed);
            body_retained = body_retained.saturating_add(retained);
            if record.get("outcome").and_then(Value::as_str) == Some("retention-limit") {
                body_truncated = body_truncated.saturating_add(observed.saturating_sub(retained));
            }
        }
    }
    trailer.get("writer_status").and_then(Value::as_str) == Some("complete")
        && trailer.get("writer_failures").and_then(Value::as_u64) == Some(0)
        && trailer.get("accepted_records").and_then(Value::as_u64) == Some(written)
        && trailer.get("written_records").and_then(Value::as_u64) == Some(written)
        && trailer.get("dropped_records").and_then(Value::as_u64) == Some(dropped)
        && trailer.get("serialized_bytes").and_then(Value::as_u64) == Some(serialized_bytes)
        && trailer.get("records_by_type") == Some(&json!(records_by_type))
        && trailer.get("body_bytes_observed").and_then(Value::as_u64) == Some(body_observed)
        && trailer.get("body_bytes_retained").and_then(Value::as_u64) == Some(body_retained)
        && trailer.get("body_bytes_truncated").and_then(Value::as_u64) == Some(body_truncated)
        && trailer
            .get("body_bytes_queue_dropped")
            .and_then(Value::as_u64)
            == Some(body_queue_dropped)
        && trailer
            .get("body_retained_bytes_queue_dropped")
            .and_then(Value::as_u64)
            == Some(body_retained_queue_dropped)
}
