// SPDX-License-Identifier: Apache-2.0

//! Versioned, append-only Deep Capture application observations.

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use base64::Engine;
use fragcap_proxy::{
    ApplicationEvent, ApplicationEventKind, ApplicationEventSink, ApplicationSinkAccounting,
    BodyOutcome, BodyRepresentation, ConnectionDescriptor, EventDisposition, MetadataOrdering,
    ProtocolVersion,
};
use serde_json::{json, Value};

use super::{
    ClassificationReason, ClassificationSummary, DetectionState, InspectabilityState,
    ProtocolClassification, TrafficFamily, CLASSIFICATION_SCHEMA_VERSION,
};

const SCHEMA_VERSION: u64 = 2;
#[cfg(not(test))]
const MAX_CONNECTION_WINDOWS: usize = 65_536;
#[cfg(test)]
const MAX_CONNECTION_WINDOWS: usize = 8;
#[cfg(not(test))]
const MAX_BODY_QUEUE_LOSS_KEYS: usize = 4_096;
#[cfg(test)]
const MAX_BODY_QUEUE_LOSS_KEYS: usize = 8;
#[cfg(not(test))]
const MAX_UDP_QUEUE_LOSS_KEYS: usize = 4_096;
#[cfg(test)]
const MAX_UDP_QUEUE_LOSS_KEYS: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationStreamStatus {
    Complete,
    Incomplete,
    UnknownVersion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClassificationStreamStatus {
    LegacyAbsent,
    Supported,
    UnknownVersion,
}

#[derive(Clone, Debug)]
pub struct ApplicationPrefix {
    pub schema_version: u64,
    pub classification_schema_version: Option<u64>,
    pub classification_status: ClassificationStreamStatus,
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
    queue_capacity: AtomicU64,
    queue_current: AtomicU64,
    queue_peak: AtomicU64,
    body_bytes_queue_dropped: AtomicU64,
    body_retained_bytes_queue_dropped: AtomicU64,
    streaming_bytes_queue_dropped: AtomicU64,
    generic_stream_bytes_queue_dropped: AtomicU64,
    generic_udp_datagrams_queue_dropped: AtomicU64,
    generic_udp_bytes_queue_dropped: AtomicU64,
    generic_udp_datagrams_storage_dropped: AtomicU64,
    generic_udp_bytes_storage_dropped: AtomicU64,
    quic_stream_bytes_queue_dropped: AtomicU64,
    quic_datagrams_queue_dropped: AtomicU64,
    quic_datagram_bytes_queue_dropped: AtomicU64,
    quic_stream_bytes_storage_dropped: AtomicU64,
    quic_datagrams_storage_dropped: AtomicU64,
    quic_datagram_bytes_storage_dropped: AtomicU64,
    generic_udp_queue_losses: Mutex<BTreeMap<UdpDropKey, DatagramDropTotals>>,
    generic_udp_queue_loss_overflow_datagrams: AtomicU64,
    generic_udp_queue_loss_overflow_bytes: AtomicU64,
    body_queue_losses: Mutex<BTreeMap<BodyDropKey, BodyDropTotals>>,
    body_queue_loss_overflow_records: AtomicU64,
    body_queue_loss_overflow_observed_bytes: AtomicU64,
    body_queue_loss_overflow_retained_bytes: AtomicU64,
    connection_windows_unretained: AtomicU64,
    first_unretained_connection_id: AtomicU64,
    external_classification_records_lost: AtomicU64,
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct UdpDropKey {
    connection_id: u64,
    direction: &'static str,
    remote_endpoint: std::net::SocketAddr,
}

#[derive(Clone, Copy, Debug, Default)]
struct DatagramDropTotals {
    datagrams: u64,
    observed_bytes: u64,
}

impl WriterAccount {
    fn reserve_queue_slot(&self) -> bool {
        let capacity = self.queue_capacity.load(Ordering::Acquire);
        let reserved =
            self.queue_current
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                    (current < capacity).then_some(current + 1)
                });
        if let Ok(previous) = reserved {
            self.queue_peak.fetch_max(previous + 1, Ordering::AcqRel);
            true
        } else {
            false
        }
    }

    fn release_queue_slot(&self) {
        self.queue_current.fetch_sub(1, Ordering::AcqRel);
    }

    fn record_storage_drop(&self, event: &ApplicationEvent) {
        if let ApplicationEventKind::GenericUdpDatagram(value) = &event.kind {
            self.generic_udp_datagrams_storage_dropped
                .fetch_add(1, Ordering::Relaxed);
            self.generic_udp_bytes_storage_dropped
                .fetch_add(value.observed_len, Ordering::Relaxed);
        }
        match &event.kind {
            ApplicationEventKind::QuicStream(value) => {
                self.quic_stream_bytes_storage_dropped
                    .fetch_add(value.observed_len, Ordering::Relaxed);
            }
            ApplicationEventKind::QuicDatagram(value) => {
                self.quic_datagrams_storage_dropped
                    .fetch_add(1, Ordering::Relaxed);
                self.quic_datagram_bytes_storage_dropped
                    .fetch_add(value.observed_len, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    fn record_queue_drop(&self, event: &ApplicationEvent) {
        if let ApplicationEventKind::GenericUdpDatagram(value) = &event.kind {
            self.generic_udp_datagrams_queue_dropped
                .fetch_add(1, Ordering::Relaxed);
            self.generic_udp_bytes_queue_dropped
                .fetch_add(value.observed_len, Ordering::Relaxed);
            let key = UdpDropKey {
                connection_id: event.connection_id,
                direction: value.direction.as_str(),
                remote_endpoint: value.remote_endpoint,
            };
            let mut losses = self
                .generic_udp_queue_losses
                .lock()
                .expect("generic UDP queue loss lock");
            if !losses.contains_key(&key) && losses.len() >= MAX_UDP_QUEUE_LOSS_KEYS {
                self.generic_udp_queue_loss_overflow_datagrams
                    .fetch_add(1, Ordering::Relaxed);
                self.generic_udp_queue_loss_overflow_bytes
                    .fetch_add(value.observed_len, Ordering::Relaxed);
                return;
            }
            let totals = losses.entry(key).or_default();
            totals.datagrams = totals.datagrams.saturating_add(1);
            totals.observed_bytes = totals.observed_bytes.saturating_add(value.observed_len);
            return;
        }
        match &event.kind {
            ApplicationEventKind::QuicStream(value) => {
                self.quic_stream_bytes_queue_dropped
                    .fetch_add(value.observed_len, Ordering::Relaxed);
                return;
            }
            ApplicationEventKind::QuicDatagram(value) => {
                self.quic_datagrams_queue_dropped
                    .fetch_add(1, Ordering::Relaxed);
                self.quic_datagram_bytes_queue_dropped
                    .fetch_add(value.observed_len, Ordering::Relaxed);
                return;
            }
            _ => {}
        }
        if let ApplicationEventKind::GenericStreamChunk(value) = &event.kind {
            self.generic_stream_bytes_queue_dropped
                .fetch_add(value.observed_len, Ordering::Relaxed);
            return;
        }
        if let ApplicationEventKind::Streaming(value) = &event.kind {
            let bytes = match value {
                fragcap_proxy::StreamingEvent::WebSocketFrame(value) => value.wire_payload.len(),
                fragcap_proxy::StreamingEvent::WebSocketMessage(value) => value.payload.len(),
                fragcap_proxy::StreamingEvent::SseField(value) => value.value.len(),
                fragcap_proxy::StreamingEvent::SseEvent(value) => value.data.len(),
                fragcap_proxy::StreamingEvent::GrpcMessage(value) => value.payload.len(),
                _ => 0,
            };
            self.streaming_bytes_queue_dropped
                .fetch_add(bytes as u64, Ordering::Relaxed);
            return;
        }
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
        if !losses.contains_key(&key) && losses.len() >= MAX_BODY_QUEUE_LOSS_KEYS {
            self.body_queue_loss_overflow_records
                .fetch_add(1, Ordering::Relaxed);
            self.body_queue_loss_overflow_observed_bytes
                .fetch_add(segment.observed_len, Ordering::Relaxed);
            self.body_queue_loss_overflow_retained_bytes
                .fetch_add(segment.bytes.len() as u64, Ordering::Relaxed);
            return;
        }
        let totals = losses.entry(key).or_default();
        totals.records = totals.records.saturating_add(1);
        totals.observed_bytes = totals.observed_bytes.saturating_add(segment.observed_len);
        totals.retained_bytes = totals
            .retained_bytes
            .saturating_add(segment.bytes.len() as u64);
    }
}

struct ChannelSink {
    sender: Arc<Mutex<Option<mpsc::SyncSender<ApplicationEvent>>>>,
    retired: Arc<AtomicBool>,
    account: Arc<WriterAccount>,
    connections: Arc<Mutex<BTreeMap<u64, ApplicationConnectionWindow>>>,
}

impl ApplicationEventSink for ChannelSink {
    fn try_emit(&self, event: ApplicationEvent) -> EventDisposition {
        // Connection lifetime is correlation control data, not a lossy body or
        // protocol observation. Retain it independently before attempting the
        // bounded event queue so pressure cannot erase the only descriptor for
        // an accepted connection.
        if let ApplicationEventKind::ConnectionOpen(descriptor) = &event.kind {
            let mut connections = self
                .connections
                .lock()
                .expect("application connection lock");
            if connections.len() < MAX_CONNECTION_WINDOWS
                || connections.contains_key(&event.connection_id)
            {
                connections.insert(
                    event.connection_id,
                    ApplicationConnectionWindow {
                        descriptor: *descriptor,
                        opened_at_ns: event.timestamp_ns,
                        closed_at_ns: None,
                    },
                );
            } else {
                // The proxy accept loop emits ConnectionOpen synchronously and
                // assigns contiguous ids before spawning connection work. A
                // first id plus a count therefore retains every overflowed
                // identity in constant memory.
                let _ = self
                    .account
                    .first_unretained_connection_id
                    .compare_exchange(0, event.connection_id, Ordering::AcqRel, Ordering::Acquire);
                self.account
                    .connection_windows_unretained
                    .fetch_add(1, Ordering::Relaxed);
            }
        } else if matches!(event.kind, ApplicationEventKind::ConnectionTerminal(_)) {
            if let Some(window) = self
                .connections
                .lock()
                .expect("application connection lock")
                .get_mut(&event.connection_id)
            {
                window.closed_at_ns = Some(event.timestamp_ns);
            }
        }
        if self.retired.load(Ordering::Acquire) {
            self.account.record_storage_drop(&event);
            self.account.dropped.fetch_add(1, Ordering::Relaxed);
            return EventDisposition::Retired;
        }
        let sender = self.sender.lock().expect("application sender lock");
        let Some(sender) = sender.as_ref() else {
            self.account.record_storage_drop(&event);
            self.account.dropped.fetch_add(1, Ordering::Relaxed);
            return EventDisposition::Retired;
        };
        if !self.account.reserve_queue_slot() {
            self.account.record_queue_drop(&event);
            self.account.dropped.fetch_add(1, Ordering::Relaxed);
            return EventDisposition::QueueFull;
        }
        match sender.try_send(event) {
            Ok(()) => {
                self.account.accepted.fetch_add(1, Ordering::Relaxed);
                EventDisposition::Accepted
            }
            Err(mpsc::TrySendError::Full(event)) => {
                self.account.release_queue_slot();
                self.account.record_queue_drop(&event);
                self.account.dropped.fetch_add(1, Ordering::Relaxed);
                EventDisposition::QueueFull
            }
            Err(mpsc::TrySendError::Disconnected(event)) => {
                self.account.release_queue_slot();
                self.account.record_storage_drop(&event);
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
            queue_capacity: self.account.queue_capacity.load(Ordering::Acquire),
            queue_current: self.account.queue_current.load(Ordering::Acquire),
            queue_peak: self.account.queue_peak.load(Ordering::Acquire),
            body_bytes_queue_dropped: self
                .account
                .body_bytes_queue_dropped
                .load(Ordering::Acquire),
            streaming_bytes_queue_dropped: self
                .account
                .streaming_bytes_queue_dropped
                .load(Ordering::Acquire),
            generic_stream_bytes_queue_dropped: self
                .account
                .generic_stream_bytes_queue_dropped
                .load(Ordering::Acquire),
            generic_udp_datagrams_queue_dropped: self
                .account
                .generic_udp_datagrams_queue_dropped
                .load(Ordering::Acquire),
            generic_udp_bytes_queue_dropped: self
                .account
                .generic_udp_bytes_queue_dropped
                .load(Ordering::Acquire),
            generic_udp_datagrams_storage_dropped: self
                .account
                .generic_udp_datagrams_storage_dropped
                .load(Ordering::Acquire),
            generic_udp_bytes_storage_dropped: self
                .account
                .generic_udp_bytes_storage_dropped
                .load(Ordering::Acquire),
            quic_stream_bytes_queue_dropped: self
                .account
                .quic_stream_bytes_queue_dropped
                .load(Ordering::Acquire),
            quic_datagrams_queue_dropped: self
                .account
                .quic_datagrams_queue_dropped
                .load(Ordering::Acquire),
            quic_datagram_bytes_queue_dropped: self
                .account
                .quic_datagram_bytes_queue_dropped
                .load(Ordering::Acquire),
            quic_stream_bytes_storage_dropped: self
                .account
                .quic_stream_bytes_storage_dropped
                .load(Ordering::Acquire),
            quic_datagrams_storage_dropped: self
                .account
                .quic_datagrams_storage_dropped
                .load(Ordering::Acquire),
            quic_datagram_bytes_storage_dropped: self
                .account
                .quic_datagram_bytes_storage_dropped
                .load(Ordering::Acquire),
        }
    }
}

pub struct ApplicationArtifactLease {
    path: PathBuf,
    sink: Arc<ChannelSink>,
    worker: Option<JoinHandle<io::Result<()>>>,
    classification_summary: Arc<Mutex<Option<ClassificationSummary>>>,
}

#[derive(Clone, Debug, Default)]
pub struct ApplicationCorrelation {
    pub target_id: Option<i64>,
    pub flow_id: Option<fragcap_core::FlowId>,
    pub process_id: Option<u32>,
    pub process_image: Option<String>,
    pub role: Option<String>,
    pub attribution: Option<String>,
    pub packet_observations: u64,
    pub packet_observations_unretained: u64,
    pub state: Option<String>,
    pub reason: Option<String>,
}

/// The proxy-side lifetime used to reconcile one accepted connection against
/// packet evidence after capture has stopped.
#[derive(Clone, Copy, Debug)]
pub struct ApplicationConnectionWindow {
    pub descriptor: ConnectionDescriptor,
    pub opened_at_ns: u64,
    pub closed_at_ns: Option<u64>,
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
            Arc::new(|_| ApplicationCorrelation::default()),
        )
    }

    pub fn open_correlated(
        path: impl Into<PathBuf>,
        session_id: impl Into<String>,
        capacity: usize,
        correlation: Arc<
            dyn Fn(&ApplicationConnectionWindow) -> ApplicationCorrelation + Send + Sync,
        >,
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
        let sender = Arc::new(Mutex::new(Some(sender)));
        let retired = Arc::new(AtomicBool::new(false));
        let account = Arc::new(WriterAccount::default());
        account
            .queue_capacity
            .store(capacity.max(1) as u64, Ordering::Release);
        let connections = Arc::new(Mutex::new(BTreeMap::new()));
        let classification_summary = Arc::new(Mutex::new(None));
        let sink = Arc::new(ChannelSink {
            sender: Arc::clone(&sender),
            retired: Arc::clone(&retired),
            account: Arc::clone(&account),
            connections: Arc::clone(&connections),
        });
        let worker_account = Arc::clone(&account);
        let worker_classification_summary = Arc::clone(&classification_summary);
        let worker = std::thread::Builder::new()
            .name("fragcap-application-writer".to_string())
            .spawn(move || {
                writer_loop(
                    file,
                    &session_id,
                    receiver,
                    sender,
                    retired,
                    worker_account,
                    connections,
                    correlation,
                    worker_classification_summary,
                )
            })?;
        Ok(Self {
            path,
            sink,
            worker: Some(worker),
            classification_summary,
        })
    }

    pub fn sink(&self) -> Arc<dyn ApplicationEventSink> {
        self.sink.clone()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn add_classification_loss(&self, count: u64) {
        self.sink
            .account
            .external_classification_records_lost
            .fetch_add(count, Ordering::Relaxed);
    }

    pub fn classification_summary(&self) -> Option<ClassificationSummary> {
        self.classification_summary
            .lock()
            .expect("application classification summary lock")
            .clone()
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

#[expect(
    clippy::too_many_arguments,
    reason = "the writer must own both queue endpoints to reconcile accepted events after failure"
)]
fn writer_loop(
    file: File,
    session_id: &str,
    receiver: mpsc::Receiver<ApplicationEvent>,
    sender: Arc<Mutex<Option<mpsc::SyncSender<ApplicationEvent>>>>,
    retired: Arc<AtomicBool>,
    account: Arc<WriterAccount>,
    connections: Arc<Mutex<BTreeMap<u64, ApplicationConnectionWindow>>>,
    correlation: Arc<dyn Fn(&ApplicationConnectionWindow) -> ApplicationCorrelation + Send + Sync>,
    reconciled_classification_summary: Arc<Mutex<Option<ClassificationSummary>>>,
) -> io::Result<()> {
    let mut writer = BufWriter::new(file);
    let mut sequence = 0_u64;
    let mut records_by_type = BTreeMap::<String, u64>::new();
    let mut body_observed = 0_u64;
    let mut body_retained = 0_u64;
    let mut body_truncated = 0_u64;
    let mut streaming_observed = 0_u64;
    let mut streaming_retained = 0_u64;
    let mut streaming_truncated = 0_u64;
    let mut streaming_by_outcome = BTreeMap::<String, u64>::new();
    let mut generic_observed = 0_u64;
    let mut generic_retained = 0_u64;
    let mut generic_omitted = 0_u64;
    let mut generic_by_outcome = BTreeMap::<String, u64>::new();
    let mut generic_udp_observed_datagrams = 0_u64;
    let mut generic_udp_observed_bytes = 0_u64;
    let mut generic_udp_retained_bytes = 0_u64;
    let mut generic_udp_omitted_bytes = 0_u64;
    let mut generic_udp_truncated_datagrams = 0_u64;
    let mut generic_udp_by_outcome = BTreeMap::<String, u64>::new();
    let mut quic_stream_observed_bytes = 0_u64;
    let mut quic_stream_retained_bytes = 0_u64;
    let mut quic_datagrams_observed = 0_u64;
    let mut quic_datagrams_capacity_dropped = 0_u64;
    let mut quic_datagram_observed_bytes = 0_u64;
    let mut quic_datagram_retained_bytes = 0_u64;
    let mut classification_summary = ClassificationSummary::default();
    let mut pending_storage = Vec::with_capacity(64);
    let mut pending_serialized_bytes = 0_u64;
    loop {
        let event = match receiver.recv_timeout(Duration::from_millis(10)) {
            Ok(event) => event,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Err(error) = flush_application_batch(
                    &mut writer,
                    &mut pending_storage,
                    &mut pending_serialized_bytes,
                    &account,
                ) {
                    return fail_application_writer(
                        error,
                        &mut pending_storage,
                        &receiver,
                        &sender,
                        &retired,
                        &account,
                    );
                }
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if let Err(error) = flush_application_batch(
                    &mut writer,
                    &mut pending_storage,
                    &mut pending_serialized_bytes,
                    &account,
                ) {
                    return fail_application_writer(
                        error,
                        &mut pending_storage,
                        &receiver,
                        &sender,
                        &retired,
                        &account,
                    );
                }
                break;
            }
        };
        account.release_queue_slot();
        let classification = application_event_classification(&event);
        classification_summary.record(&classification);
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
        if let ApplicationEventKind::Streaming(value) = &event.kind {
            if let Some((outcome, observed, retained)) = streaming_measure(value) {
                streaming_observed = streaming_observed.saturating_add(observed);
                streaming_retained = streaming_retained.saturating_add(retained);
                streaming_truncated =
                    streaming_truncated.saturating_add(observed.saturating_sub(retained));
                *streaming_by_outcome.entry(outcome.to_string()).or_default() += 1;
            }
        }
        if let ApplicationEventKind::GenericStreamChunk(value) = &event.kind {
            generic_observed = generic_observed.saturating_add(value.observed_len);
            generic_retained = generic_retained.saturating_add(value.bytes.len() as u64);
            generic_omitted = generic_omitted
                .saturating_add(value.observed_len.saturating_sub(value.bytes.len() as u64));
            *generic_by_outcome
                .entry(value.outcome.as_str().to_string())
                .or_default() += 1;
        }
        if let ApplicationEventKind::GenericUdpDatagram(value) = &event.kind {
            generic_udp_observed_datagrams = generic_udp_observed_datagrams.saturating_add(1);
            generic_udp_observed_bytes =
                generic_udp_observed_bytes.saturating_add(value.observed_len);
            generic_udp_retained_bytes =
                generic_udp_retained_bytes.saturating_add(value.bytes.len() as u64);
            generic_udp_omitted_bytes = generic_udp_omitted_bytes
                .saturating_add(value.observed_len.saturating_sub(value.bytes.len() as u64));
            if value.outcome == fragcap_proxy::GenericUdpOutcome::RetentionLimit {
                generic_udp_truncated_datagrams = generic_udp_truncated_datagrams.saturating_add(1);
            }
            *generic_udp_by_outcome
                .entry(value.outcome.as_str().to_string())
                .or_default() += 1;
        }
        match &event.kind {
            ApplicationEventKind::QuicStream(value) => {
                quic_stream_observed_bytes =
                    quic_stream_observed_bytes.saturating_add(value.observed_len);
                quic_stream_retained_bytes =
                    quic_stream_retained_bytes.saturating_add(value.bytes.len() as u64);
            }
            ApplicationEventKind::QuicDatagram(value) => {
                quic_datagrams_observed = quic_datagrams_observed.saturating_add(1);
                if value.terminal == "capacity-dropped" {
                    quic_datagrams_capacity_dropped =
                        quic_datagrams_capacity_dropped.saturating_add(1);
                }
                quic_datagram_observed_bytes =
                    quic_datagram_observed_bytes.saturating_add(value.observed_len);
                quic_datagram_retained_bytes =
                    quic_datagram_retained_bytes.saturating_add(value.bytes.len() as u64);
            }
            _ => {}
        }
        sequence = sequence.saturating_add(1);
        // Packet capture and proxy observation run concurrently. Publishing a
        // live answer here would make identity depend on scheduling, so event
        // records remain explicitly deferred until final reconciliation.
        let storage_event = event.clone();
        let value = event_json(event, sequence, ApplicationCorrelation::default());
        match buffer_application_record(
            &mut writer,
            &value,
            storage_event,
            &mut pending_storage,
            &mut pending_serialized_bytes,
            &account,
        ) {
            Ok(()) => {
                if pending_storage.len() == pending_storage.capacity() {
                    if let Err(error) = flush_application_batch(
                        &mut writer,
                        &mut pending_storage,
                        &mut pending_serialized_bytes,
                        &account,
                    ) {
                        return fail_application_writer(
                            error,
                            &mut pending_storage,
                            &receiver,
                            &sender,
                            &retired,
                            &account,
                        );
                    }
                }
            }
            Err(error) => {
                return fail_application_writer(
                    error,
                    &mut pending_storage,
                    &receiver,
                    &sender,
                    &retired,
                    &account,
                );
            }
        }
    }
    let mut correlation_counts = BTreeMap::<String, u64>::new();
    let connections =
        std::mem::take(&mut *connections.lock().expect("application connection lock"));
    for (connection_id, window) in connections {
        sequence = sequence.saturating_add(1);
        let resolved = correlation(&window);
        *correlation_counts
            .entry(
                resolved
                    .state
                    .clone()
                    .unwrap_or_else(|| "unavailable".to_string()),
            )
            .or_default() += 1;
        let value = correlation_record(session_id, sequence, connection_id, resolved);
        write_record(&mut writer, &value)?;
        account.written.fetch_add(1, Ordering::Relaxed);
        account.accepted.fetch_add(1, Ordering::Relaxed);
        account
            .serialized_bytes
            .fetch_add(value.to_string().len() as u64, Ordering::Relaxed);
        *records_by_type
            .entry("application.correlation".to_string())
            .or_default() += 1;
    }
    let unretained_connections = account
        .connection_windows_unretained
        .load(Ordering::Acquire);
    let first_unretained_connection_id = account
        .first_unretained_connection_id
        .load(Ordering::Acquire);
    for offset in 0..unretained_connections {
        *correlation_counts
            .entry("unavailable".to_string())
            .or_default() += 1;
        sequence = sequence.saturating_add(1);
        let connection_id = first_unretained_connection_id.saturating_add(offset);
        let value = correlation_record(
            session_id,
            sequence,
            connection_id,
            ApplicationCorrelation {
                state: Some("unavailable".to_string()),
                reason: Some("connection-history-bound-exceeded".to_string()),
                ..ApplicationCorrelation::default()
            },
        );
        write_record(&mut writer, &value)?;
        account.written.fetch_add(1, Ordering::Relaxed);
        account.accepted.fetch_add(1, Ordering::Relaxed);
        account
            .serialized_bytes
            .fetch_add(value.to_string().len() as u64, Ordering::Relaxed);
        *records_by_type
            .entry("application.correlation".to_string())
            .or_default() += 1;
    }
    let dropped = account.dropped.load(Ordering::Acquire);
    let body_bytes_queue_dropped = account.body_bytes_queue_dropped.load(Ordering::Acquire);
    let streaming_bytes_queue_dropped = account
        .streaming_bytes_queue_dropped
        .load(Ordering::Acquire);
    let generic_stream_bytes_queue_dropped = account
        .generic_stream_bytes_queue_dropped
        .load(Ordering::Acquire);
    let generic_udp_datagrams_queue_dropped = account
        .generic_udp_datagrams_queue_dropped
        .load(Ordering::Acquire);
    let generic_udp_bytes_queue_dropped = account
        .generic_udp_bytes_queue_dropped
        .load(Ordering::Acquire);
    let generic_udp_datagrams_storage_dropped = account
        .generic_udp_datagrams_storage_dropped
        .load(Ordering::Acquire);
    let generic_udp_bytes_storage_dropped = account
        .generic_udp_bytes_storage_dropped
        .load(Ordering::Acquire);
    let quic_stream_bytes_queue_dropped = account
        .quic_stream_bytes_queue_dropped
        .load(Ordering::Acquire);
    let quic_datagrams_queue_dropped = account.quic_datagrams_queue_dropped.load(Ordering::Acquire);
    let quic_datagram_bytes_queue_dropped = account
        .quic_datagram_bytes_queue_dropped
        .load(Ordering::Acquire);
    let quic_stream_bytes_storage_dropped = account
        .quic_stream_bytes_storage_dropped
        .load(Ordering::Acquire);
    let quic_datagrams_storage_dropped = account
        .quic_datagrams_storage_dropped
        .load(Ordering::Acquire);
    let quic_datagram_bytes_storage_dropped = account
        .quic_datagram_bytes_storage_dropped
        .load(Ordering::Acquire);
    let generic_udp_queue_losses = account
        .generic_udp_queue_losses
        .lock()
        .expect("generic UDP queue loss lock")
        .iter()
        .map(|(key, totals)| {
            json!({
                "proxy_connection_id": key.connection_id,
                "direction": key.direction,
                "remote_endpoint": key.remote_endpoint.to_string(),
                "outcome": "queue-dropped",
                "dropped_datagrams": totals.datagrams,
                "observed_bytes": totals.observed_bytes,
            })
        })
        .collect::<Vec<_>>();
    let generic_udp_queue_loss_overflow_datagrams = account
        .generic_udp_queue_loss_overflow_datagrams
        .load(Ordering::Acquire);
    let generic_udp_queue_loss_overflow_bytes = account
        .generic_udp_queue_loss_overflow_bytes
        .load(Ordering::Acquire);
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
    let body_queue_loss_overflow_records = account
        .body_queue_loss_overflow_records
        .load(Ordering::Acquire);
    let body_queue_loss_overflow_observed_bytes = account
        .body_queue_loss_overflow_observed_bytes
        .load(Ordering::Acquire);
    let body_queue_loss_overflow_retained_bytes = account
        .body_queue_loss_overflow_retained_bytes
        .load(Ordering::Acquire);
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
                "streaming_bytes_queue_dropped": streaming_bytes_queue_dropped,
                "generic_stream_bytes_queue_dropped": generic_stream_bytes_queue_dropped,
                "generic_udp_datagrams_queue_dropped": generic_udp_datagrams_queue_dropped,
                "generic_udp_bytes_queue_dropped": generic_udp_bytes_queue_dropped,
                "generic_udp_datagrams_storage_dropped": generic_udp_datagrams_storage_dropped,
                "generic_udp_bytes_storage_dropped": generic_udp_bytes_storage_dropped,
                "quic_stream_bytes_queue_dropped": quic_stream_bytes_queue_dropped,
                "quic_datagrams_queue_dropped": quic_datagrams_queue_dropped,
                "quic_datagram_bytes_queue_dropped": quic_datagram_bytes_queue_dropped,
                "quic_stream_bytes_storage_dropped": quic_stream_bytes_storage_dropped,
                "quic_datagrams_storage_dropped": quic_datagrams_storage_dropped,
                "quic_datagram_bytes_storage_dropped": quic_datagram_bytes_storage_dropped,
                "generic_udp_losses": generic_udp_queue_losses,
                "generic_udp_loss_identity_overflow": {
                    "dropped_datagrams": generic_udp_queue_loss_overflow_datagrams,
                    "observed_bytes": generic_udp_queue_loss_overflow_bytes,
                    "reason": "localized-generic-udp-loss-identity-bound",
                },
                "body_losses": body_queue_losses,
                "body_loss_identity_overflow": {
                    "dropped_records": body_queue_loss_overflow_records,
                    "observed_bytes": body_queue_loss_overflow_observed_bytes,
                    "retained_bytes": body_queue_loss_overflow_retained_bytes,
                    "reason": "localized-body-loss-identity-bound",
                },
            }),
        )?;
    }
    sequence = sequence.saturating_add(1);
    let proxy_observations_dropped_oldest = account
        .external_classification_records_lost
        .load(Ordering::Acquire);
    classification_summary.unclassified_lost =
        dropped.saturating_add(proxy_observations_dropped_oldest);
    let mut trailer = json!({
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
        ,"classification_schema_version": CLASSIFICATION_SCHEMA_VERSION
        ,"classified_records": classification_summary.observations
        ,"classification_records_lost": classification_summary.unclassified_lost
        ,"proxy_observations_dropped_oldest": proxy_observations_dropped_oldest
        ,"classifications_by_family": classification_summary.by_family
        ,"classifications_by_detection": classification_summary.by_detection
        ,"classifications_by_inspectability": classification_summary.by_inspectability
        ,"classifications_by_reason": classification_summary.by_reason
        ,"records_by_type": records_by_type
        ,"body_bytes_observed": body_observed
        ,"body_bytes_retained": body_retained
        ,"body_bytes_truncated": body_truncated
        ,"body_bytes_queue_dropped": body_bytes_queue_dropped
        ,"body_retained_bytes_queue_dropped": body_retained_bytes_queue_dropped
        ,"body_loss_identity_overflow_records": body_queue_loss_overflow_records
        ,"body_loss_identity_overflow_observed_bytes": body_queue_loss_overflow_observed_bytes
        ,"body_loss_identity_overflow_retained_bytes": body_queue_loss_overflow_retained_bytes
        ,"streaming_bytes_queue_dropped": streaming_bytes_queue_dropped
        ,"streaming_bytes_observed": streaming_observed
        ,"streaming_bytes_retained": streaming_retained
        ,"streaming_bytes_truncated": streaming_truncated
        ,"streaming_records_by_outcome": streaming_by_outcome
        ,"generic_stream_bytes_observed": generic_observed
        ,"generic_stream_bytes_retained": generic_retained
        ,"generic_stream_bytes_omitted": generic_omitted
        ,"generic_stream_bytes_queue_dropped": generic_stream_bytes_queue_dropped
        ,"generic_stream_records_by_outcome": generic_by_outcome
        ,"correlation_connections_by_state": correlation_counts
        ,"correlation_connections_total": correlation_counts.values().sum::<u64>()
        ,"correlation_connections_unretained": unretained_connections
    });
    let object = trailer
        .as_object_mut()
        .expect("application trailer is an object");
    for (name, value) in [
        ("quic_stream_bytes_observed", quic_stream_observed_bytes),
        ("quic_stream_bytes_retained", quic_stream_retained_bytes),
        (
            "quic_stream_bytes_omitted",
            quic_stream_observed_bytes.saturating_sub(quic_stream_retained_bytes),
        ),
        (
            "quic_stream_bytes_queue_dropped",
            quic_stream_bytes_queue_dropped,
        ),
        (
            "quic_stream_bytes_storage_dropped",
            quic_stream_bytes_storage_dropped,
        ),
        ("quic_datagrams_observed", quic_datagrams_observed),
        ("quic_datagram_bytes_observed", quic_datagram_observed_bytes),
        ("quic_datagram_bytes_retained", quic_datagram_retained_bytes),
        (
            "quic_datagram_bytes_omitted",
            quic_datagram_observed_bytes.saturating_sub(quic_datagram_retained_bytes),
        ),
        (
            "quic_datagrams_capacity_dropped",
            quic_datagrams_capacity_dropped,
        ),
        ("quic_datagrams_queue_dropped", quic_datagrams_queue_dropped),
        (
            "quic_datagram_bytes_queue_dropped",
            quic_datagram_bytes_queue_dropped,
        ),
        (
            "quic_datagrams_storage_dropped",
            quic_datagrams_storage_dropped,
        ),
        (
            "quic_datagram_bytes_storage_dropped",
            quic_datagram_bytes_storage_dropped,
        ),
    ] {
        object.insert(name.to_string(), json!(value));
    }
    object.insert(
        "generic_udp_datagrams_observed".to_string(),
        json!(generic_udp_observed_datagrams),
    );
    object.insert(
        "generic_udp_bytes_observed".to_string(),
        json!(generic_udp_observed_bytes),
    );
    object.insert(
        "generic_udp_bytes_retained".to_string(),
        json!(generic_udp_retained_bytes),
    );
    object.insert(
        "generic_udp_bytes_omitted".to_string(),
        json!(generic_udp_omitted_bytes),
    );
    object.insert(
        "generic_udp_datagrams_truncated".to_string(),
        json!(generic_udp_truncated_datagrams),
    );
    object.insert(
        "generic_udp_datagrams_queue_dropped".to_string(),
        json!(generic_udp_datagrams_queue_dropped),
    );
    object.insert(
        "generic_udp_bytes_queue_dropped".to_string(),
        json!(generic_udp_bytes_queue_dropped),
    );
    object.insert(
        "generic_udp_datagrams_storage_dropped".to_string(),
        json!(generic_udp_datagrams_storage_dropped),
    );
    object.insert(
        "generic_udp_bytes_storage_dropped".to_string(),
        json!(generic_udp_bytes_storage_dropped),
    );
    object.insert(
        "generic_udp_loss_identity_overflow_datagrams".to_string(),
        json!(generic_udp_queue_loss_overflow_datagrams),
    );
    object.insert(
        "generic_udp_loss_identity_overflow_bytes".to_string(),
        json!(generic_udp_queue_loss_overflow_bytes),
    );
    object.insert(
        "generic_udp_records_by_outcome".to_string(),
        json!(generic_udp_by_outcome),
    );
    write_record(&mut writer, &trailer)?;
    *reconciled_classification_summary
        .lock()
        .expect("application classification summary lock") = Some(classification_summary);
    Ok(())
}

fn header_record(session_id: &str) -> Value {
    json!({
        "type": "application.header",
        "schema_version": SCHEMA_VERSION,
        "classification_schema_version": CLASSIFICATION_SCHEMA_VERSION,
        "session_id": session_id,
        "sequence": 0,
        "event_time_ns": 0,
        "exports": ["connection", "tls", "http", "metadata", "body", "transformation", "websocket", "sse", "grpc", "socks5", "generic-stream", "generic-udp", "quic", "http3"],
        "non_exports": {
            "unrouted-udp": "packet-only-unsupported",
            "unknown-quic-alpn": "explicitly-refused"
        }
    })
}

fn correlation_record(
    session_id: &str,
    sequence: u64,
    connection_id: u64,
    value: ApplicationCorrelation,
) -> Value {
    json!({
        "type": "application.correlation",
        "schema_version": SCHEMA_VERSION,
        "session_id": session_id,
        "sequence": sequence,
        "event_time_ns": 0,
        "proxy_connection_id": connection_id,
        "flow_id": value.flow_id.map(|id| id.to_string()),
        "target_id": value.target_id,
        "process_id": value.process_id,
        "process_image": value.process_image,
        "role": value.role,
        "attribution": value.attribution,
        "packet_observations": value.packet_observations,
        "packet_observations_unretained": value.packet_observations_unretained,
        "correlation_state": value.state.unwrap_or_else(|| "unavailable".to_string()),
        "correlation_reason": value.reason.unwrap_or_else(|| "correlation-context-unavailable".to_string()),
    })
}

fn write_record(writer: &mut impl Write, value: &Value) -> io::Result<()> {
    write_record_buffered(writer, value)?;
    writer.flush()
}

fn write_record_buffered(writer: &mut impl Write, value: &Value) -> io::Result<u64> {
    let (wire, serialized_bytes) = serialized_record(value)?;
    writer.write_all(&wire)?;
    Ok(serialized_bytes)
}

fn serialized_record(value: &Value) -> io::Result<(Vec<u8>, u64)> {
    let mut record = serde_json::to_vec(value).map_err(io::Error::other)?;
    let serialized_bytes = record.len() as u64;
    record.push(b'\n');
    Ok((record, serialized_bytes))
}

fn buffer_application_record<W: Write>(
    writer: &mut BufWriter<W>,
    value: &Value,
    event: ApplicationEvent,
    pending: &mut Vec<ApplicationEvent>,
    pending_serialized_bytes: &mut u64,
    account: &WriterAccount,
) -> io::Result<()> {
    let (wire, serialized_bytes) = match serialized_record(value) {
        Ok(record) => record,
        Err(error) => {
            pending.push(event);
            return Err(error);
        }
    };
    if !pending.is_empty() && writer.buffer().len().saturating_add(wire.len()) > writer.capacity() {
        if let Err(error) =
            flush_application_batch(writer, pending, pending_serialized_bytes, account)
        {
            pending.push(event);
            return Err(error);
        }
    }
    if wire.len() >= writer.capacity() {
        if let Err(error) = writer.write_all(&wire) {
            pending.push(event);
            return Err(error);
        }
        account.written.fetch_add(1, Ordering::Relaxed);
        account
            .serialized_bytes
            .fetch_add(serialized_bytes, Ordering::Relaxed);
    } else {
        if let Err(error) = writer.write_all(&wire) {
            pending.push(event);
            return Err(error);
        }
        pending.push(event);
        *pending_serialized_bytes = pending_serialized_bytes.saturating_add(serialized_bytes);
    }
    Ok(())
}

fn flush_application_batch(
    writer: &mut impl Write,
    pending: &mut Vec<ApplicationEvent>,
    serialized_bytes: &mut u64,
    account: &WriterAccount,
) -> io::Result<()> {
    if pending.is_empty() {
        return Ok(());
    }
    writer.flush()?;
    account
        .written
        .fetch_add(pending.len() as u64, Ordering::Relaxed);
    account
        .serialized_bytes
        .fetch_add(*serialized_bytes, Ordering::Relaxed);
    pending.clear();
    *serialized_bytes = 0;
    Ok(())
}

fn fail_application_writer(
    error: io::Error,
    pending: &mut Vec<ApplicationEvent>,
    receiver: &mpsc::Receiver<ApplicationEvent>,
    sender: &Arc<Mutex<Option<mpsc::SyncSender<ApplicationEvent>>>>,
    retired: &AtomicBool,
    account: &WriterAccount,
) -> io::Result<()> {
    account.failures.fetch_add(1, Ordering::Relaxed);
    retired.store(true, Ordering::Release);
    for event in pending.drain(..) {
        account.record_storage_drop(&event);
        account.dropped.fetch_add(1, Ordering::Relaxed);
    }
    drop(sender.lock().expect("application sender lock").take());
    while let Ok(event) = receiver.recv() {
        account.release_queue_slot();
        account.record_storage_drop(&event);
        account.dropped.fetch_add(1, Ordering::Relaxed);
    }
    Err(error)
}

fn event_json(
    event: ApplicationEvent,
    sequence: u64,
    correlation: ApplicationCorrelation,
) -> Value {
    let classification = application_event_classification(&event);
    let protocol = event.protocol.map(|value| match value {
        ProtocolVersion::Http11 => "http/1.1",
        ProtocolVersion::Http2 => "h2",
        ProtocolVersion::Http3 => "h3",
    });
    let common = json!({
        "schema_version": SCHEMA_VERSION,
        "session_id": event.session_id,
        "sequence": sequence,
        "event_time_ns": event.timestamp_ns,
        "proxy_connection_id": event.connection_id,
        "http_stream_id": event.stream_id,
        "protocol": protocol,
        "classification": classification_json(&classification),
        "target_id": correlation.target_id,
        "flow_id": correlation.flow_id.map(|id| id.to_string()),
        "process_id": correlation.process_id,
        "process_image": correlation.process_image,
        "role": correlation.role,
        "attribution": correlation.attribution,
        "correlation_state": correlation.state.unwrap_or_else(|| "deferred".to_string()),
        "correlation_reason": correlation.reason.unwrap_or_else(|| "final-reconciliation-pending".to_string()),
    });
    let mut object = common.as_object().cloned().unwrap_or_default();
    let (kind, detail) = match event.kind {
        ApplicationEventKind::ConnectionOpen(value) => (
            "connection.open",
            json!({
                "transport": value.transport,
                "client_peer": value.client_peer.to_string(),
                "proxy_local": value.proxy_local.to_string(),
                "correlation_state": "deferred",
                "correlation_reason": "final-reconciliation-pending",
            }),
        ),
        ApplicationEventKind::UpstreamSocket(value) => (
            "connection.upstream_socket",
            json!({
                "upstream_protocol": value.protocol,
                "upstream_local": value.local.to_string(),
                "upstream_peer": value.peer.to_string(),
            }),
        ),
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
        ApplicationEventKind::HttpTiming(value) => (
            "http.timing",
            json!({"send_ns": value.send_ns, "wait_ns": value.wait_ns, "receive_ns": value.receive_ns}),
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
                "url": block.url.map(binary_json),
                "status": block.status,
                "reason": block.reason.map(binary_json),
                "head_bytes": block.head_bytes,
                "query": block.query.into_iter().map(derived_json).collect::<Vec<_>>(),
                "cookies": block.cookies.into_iter().map(derived_json).collect::<Vec<_>>(),
                "unavailable": block.unavailable,
            }),
        ),
        ApplicationEventKind::Body(segment) => ("http.body_segment", body_json(segment)),
        ApplicationEventKind::Transformation(value) => (
            "http.transformation",
            json!({
                "direction": direction(value.direction),
                "encoding": value.encoding,
                "input_bytes": value.input_bytes,
                "output_bytes": value.output_bytes,
                "outcome": body_outcome(value.outcome),
            }),
        ),
        ApplicationEventKind::Streaming(value) => streaming_json(value),
        ApplicationEventKind::SocksNegotiation(value) => (
            "socks5.negotiation",
            json!({"authenticated": value.authenticated}),
        ),
        ApplicationEventKind::SocksConnect(value) => (
            "socks5.connect",
            json!({
                "authority": value.authority,
                "upstream_local": value.upstream_local.map(|address| address.to_string()),
                "selected_peer": value.selected_peer.map(|address| address.to_string()),
                "address_type": value.address_type,
                "dns_owner": value.dns_owner,
                "outcome": value.outcome,
                "classification": value.classification.map(|item| item.as_str()),
                "inspectability": "metadata-only",
            }),
        ),
        ApplicationEventKind::SocksTransfer(value) => (
            "socks5.transfer",
            json!({
                "client_to_upstream_bytes": value.client_to_upstream_bytes,
                "upstream_to_client_bytes": value.upstream_to_client_bytes,
                "payload_retained": false,
            }),
        ),
        ApplicationEventKind::SocksUdp(value) => (
            "socks5.udp",
            json!({
                "action": value.action,
                "outcome": value.outcome,
                "address_type": value.address_type,
                "remote": value.remote.map(|address| address.to_string()),
                "payload_bytes": value.payload_bytes,
                "active_peers": value.active_peers,
                "payload_retained": false,
            }),
        ),
        ApplicationEventKind::GenericStreamChunk(value) => {
            let mut detail = json!({
                "direction": value.direction.as_str(),
                "provenance": value.provenance.as_str(),
                "offset": value.offset,
                "observed_len": value.observed_len,
                "retained_len": value.bytes.len(),
                "outcome": value.outcome.as_str(),
                "inspectability": match value.provenance {
                    fragcap_proxy::GenericStreamProvenance::TlsEncrypted => "opaque",
                    fragcap_proxy::GenericStreamProvenance::TcpPlaintext
                    | fragcap_proxy::GenericStreamProvenance::TlsDecrypted => "protocol-unknown",
                },
            });
            if !value.bytes.is_empty() {
                let object = detail.as_object_mut().expect("generic detail is an object");
                object.insert("payload_encoding".to_string(), json!("base64"));
                object.insert(
                    "payload".to_string(),
                    json!(base64::engine::general_purpose::STANDARD.encode(value.bytes)),
                );
            }
            ("generic.stream_chunk", detail)
        }
        ApplicationEventKind::GenericUdpDatagram(value) => {
            let mut detail = json!({
                "direction": value.direction.as_str(),
                "datagram_sequence": value.sequence,
                "client_endpoint": value.client_endpoint.to_string(),
                "remote_endpoint": value.remote_endpoint.to_string(),
                "observed_len": value.observed_len,
                "retained_len": value.bytes.len(),
                "outcome": value.outcome.as_str(),
                "inspectability": "protocol-unknown",
            });
            if !value.bytes.is_empty() {
                let object = detail
                    .as_object_mut()
                    .expect("generic UDP detail is an object");
                object.insert("payload_encoding".to_string(), json!("base64"));
                object.insert(
                    "payload".to_string(),
                    json!(base64::engine::general_purpose::STANDARD.encode(value.bytes)),
                );
            }
            ("generic.udp_datagram", detail)
        }
        ApplicationEventKind::UdpSocketError(value) => (
            "generic.udp_socket_error",
            json!({
                "direction": value.direction.as_str(),
                "operation": value.operation,
                "failure_code": value.failure_code,
                "endpoint": value.endpoint.map(|address| address.to_string()),
                "error_kind": value.error_kind,
                "visibility": value.visibility,
                "icmp": "unavailable",
            }),
        ),
        ApplicationEventKind::QuicConnection(value) => (
            "quic.connection",
            json!({
                "pair_id": value.pair_id,
                "half": value.half.as_str(),
                "connection_identity": format!("{}-{}", value.pair_id, value.half.as_str()),
                "peer": value.peer.to_string(),
                "origin": value.origin.to_string(),
                "server_name": value.server_name,
                "alpn_encoding": "base64",
                "alpn": value.alpn.map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes)),
                "zero_rtt": value.zero_rtt,
                "migration": value.migration,
                "outcome": value.outcome,
            }),
        ),
        ApplicationEventKind::QuicStream(value) => {
            let mut detail = json!({
                "pair_id": value.pair_id,
                "direction": value.direction.as_str(),
                "quic_stream_id": value.stream_id,
                "stream_kind": value.stream_kind,
                "stream_sequence": value.sequence,
                "offset": value.offset,
                "observed_len": value.observed_len,
                "retained_len": value.bytes.len(),
                "outcome": value.outcome.as_str(),
                "terminal": value.terminal,
                "inspectability": "protocol-unknown",
            });
            if !value.bytes.is_empty() {
                let object = detail
                    .as_object_mut()
                    .expect("QUIC stream detail is an object");
                object.insert("payload_encoding".to_string(), json!("base64"));
                object.insert(
                    "payload".to_string(),
                    json!(base64::engine::general_purpose::STANDARD.encode(value.bytes)),
                );
            }
            ("quic.stream", detail)
        }
        ApplicationEventKind::QuicDatagram(value) => {
            let mut detail = json!({
                "pair_id": value.pair_id,
                "direction": value.direction.as_str(),
                "datagram_sequence": value.sequence,
                "observed_len": value.observed_len,
                "retained_len": value.bytes.len(),
                "outcome": value.outcome.as_str(),
                "terminal": value.terminal,
                "inspectability": "protocol-unknown",
            });
            if !value.bytes.is_empty() {
                let object = detail
                    .as_object_mut()
                    .expect("QUIC datagram detail is an object");
                object.insert("payload_encoding".to_string(), json!("base64"));
                object.insert(
                    "payload".to_string(),
                    json!(base64::engine::general_purpose::STANDARD.encode(value.bytes)),
                );
            }
            ("quic.datagram", detail)
        }
        ApplicationEventKind::QuicRefusal(value) => (
            "quic.refusal",
            json!({"pair_id": value.pair_id, "code": value.code, "transparent_fallback": false}),
        ),
        ApplicationEventKind::Error { code } => ("application.error", json!({"code": code})),
    };
    object.insert("type".to_string(), Value::String(kind.to_string()));
    if let Some(detail) = detail.as_object() {
        object.extend(detail.clone());
    }
    Value::Object(object)
}

fn classification_json(value: &ProtocolClassification) -> Value {
    json!({
        "schema_version": value.schema_version(),
        "family": value.family().as_str(),
        "detection": value.detection().as_str(),
        "inspectability": value.inspectability().as_str(),
        "reason": value.reason().map(ClassificationReason::as_str),
    })
}

fn application_event_classification(event: &ApplicationEvent) -> ProtocolClassification {
    use fragcap_proxy::{
        GenericStreamOutcome, GenericStreamProvenance, GenericUdpOutcome, StreamingOutcome,
    };

    let identified = |family, inspectability, reason| {
        if family == TrafficFamily::Unknown {
            ProtocolClassification::new(
                family,
                DetectionState::Unknown,
                InspectabilityState::Unavailable,
                None,
            )
            .expect("unknown application event classification is valid")
        } else {
            ProtocolClassification::new(family, DetectionState::Identified, inspectability, reason)
                .expect("application event classification matrix is valid")
        }
    };
    let protocol_family = || match event.protocol {
        Some(ProtocolVersion::Http11) => TrafficFamily::Http1,
        Some(ProtocolVersion::Http2) => TrafficFamily::Http2,
        Some(ProtocolVersion::Http3) => TrafficFamily::Http3,
        None => TrafficFamily::Unknown,
    };
    match &event.kind {
        ApplicationEventKind::Streaming(value) => {
            let family = match value {
                fragcap_proxy::StreamingEvent::WebSocketFrame(_)
                | fragcap_proxy::StreamingEvent::WebSocketMessage(_)
                | fragcap_proxy::StreamingEvent::WebSocketTerminal { .. } => {
                    TrafficFamily::WebSocket
                }
                fragcap_proxy::StreamingEvent::SseField(_)
                | fragcap_proxy::StreamingEvent::SseEvent(_)
                | fragcap_proxy::StreamingEvent::SseTerminal { .. } => TrafficFamily::Sse,
                fragcap_proxy::StreamingEvent::GrpcCall { .. }
                | fragcap_proxy::StreamingEvent::GrpcMessage(_)
                | fragcap_proxy::StreamingEvent::GrpcTerminal { .. } => TrafficFamily::Grpc,
            };
            match streaming_event_outcome(value) {
                None | Some(StreamingOutcome::Complete) => {
                    identified(family, InspectabilityState::Full, None)
                }
                Some(StreamingOutcome::IntentionallyOmitted) => {
                    identified(family, InspectabilityState::MetadataOnly, None)
                }
                Some(StreamingOutcome::Malformed | StreamingOutcome::InvalidUtf8) => {
                    ProtocolClassification::new(
                        family,
                        DetectionState::Failed,
                        InspectabilityState::MetadataOnly,
                        Some(ClassificationReason::ParserFailed),
                    )
                    .expect("streaming parser failure classification is valid")
                }
                Some(StreamingOutcome::UnsupportedCompression) => ProtocolClassification::new(
                    family,
                    DetectionState::Unsupported,
                    InspectabilityState::Unavailable,
                    Some(ClassificationReason::UnsupportedVersion),
                )
                .expect("unsupported streaming classification is valid"),
                Some(
                    StreamingOutcome::Partial
                    | StreamingOutcome::Limit
                    | StreamingOutcome::Cancelled,
                ) => identified(
                    family,
                    InspectabilityState::MetadataOnly,
                    Some(ClassificationReason::Truncated),
                ),
            }
        }
        ApplicationEventKind::Body(value) => {
            let family = protocol_family();
            let reason = match value.outcome {
                BodyOutcome::RetentionLimit | BodyOutcome::ExpansionLimit => {
                    Some(ClassificationReason::Truncated)
                }
                BodyOutcome::StorageFailed => Some(ClassificationReason::WriterFailed),
                BodyOutcome::MalformedEncoding => Some(ClassificationReason::ParserFailed),
                _ => None,
            };
            if family == TrafficFamily::Unknown {
                identified(family, InspectabilityState::Unavailable, None)
            } else if reason == Some(ClassificationReason::ParserFailed) {
                ProtocolClassification::new(
                    family,
                    DetectionState::Failed,
                    InspectabilityState::MetadataOnly,
                    reason,
                )
                .expect("body parser classification is valid")
            } else {
                identified(family, InspectabilityState::Full, reason)
            }
        }
        ApplicationEventKind::SocksNegotiation(_)
        | ApplicationEventKind::SocksConnect(_)
        | ApplicationEventKind::SocksTransfer(_) => identified(
            TrafficFamily::Socks5Tcp,
            InspectabilityState::MetadataOnly,
            None,
        ),
        ApplicationEventKind::SocksUdp(_) => identified(
            TrafficFamily::Socks5Udp,
            InspectabilityState::MetadataOnly,
            None,
        ),
        ApplicationEventKind::GenericStreamChunk(value) => {
            let (family, inspectability, reason) = match value.provenance {
                GenericStreamProvenance::TcpPlaintext => (
                    TrafficFamily::GenericTcp,
                    InspectabilityState::DecryptedUnknown,
                    (value.outcome == GenericStreamOutcome::RetentionLimit)
                        .then_some(ClassificationReason::Truncated),
                ),
                GenericStreamProvenance::TlsDecrypted => (
                    TrafficFamily::NonHttpTls,
                    InspectabilityState::DecryptedUnknown,
                    (value.outcome == GenericStreamOutcome::RetentionLimit)
                        .then_some(ClassificationReason::Truncated),
                ),
                GenericStreamProvenance::TlsEncrypted => (
                    TrafficFamily::NonHttpTls,
                    InspectabilityState::EncryptedOpaque,
                    Some(ClassificationReason::EncryptedOpaque),
                ),
            };
            identified(family, inspectability, reason)
        }
        ApplicationEventKind::GenericUdpDatagram(value) => identified(
            TrafficFamily::GenericUdp,
            InspectabilityState::MetadataOnly,
            (value.outcome == GenericUdpOutcome::RetentionLimit)
                .then_some(ClassificationReason::Truncated),
        ),
        ApplicationEventKind::UdpSocketError(_) => identified(
            TrafficFamily::GenericUdp,
            InspectabilityState::MetadataOnly,
            None,
        ),
        ApplicationEventKind::QuicConnection(_) => {
            let family = if event.protocol == Some(ProtocolVersion::Http3) {
                TrafficFamily::Http3
            } else {
                TrafficFamily::Quic
            };
            let inspectability = if family == TrafficFamily::Http3 {
                InspectabilityState::Full
            } else {
                InspectabilityState::DecryptedUnknown
            };
            identified(family, inspectability, None)
        }
        ApplicationEventKind::QuicStream(value) => {
            let family = if event.protocol == Some(ProtocolVersion::Http3) {
                TrafficFamily::Http3
            } else {
                TrafficFamily::Quic
            };
            identified(
                family,
                if family == TrafficFamily::Http3 {
                    InspectabilityState::Full
                } else {
                    InspectabilityState::DecryptedUnknown
                },
                (value.outcome == GenericStreamOutcome::RetentionLimit)
                    .then_some(ClassificationReason::Truncated),
            )
        }
        ApplicationEventKind::QuicDatagram(value) => identified(
            TrafficFamily::Quic,
            InspectabilityState::DecryptedUnknown,
            (value.outcome == GenericUdpOutcome::RetentionLimit)
                .then_some(ClassificationReason::Truncated),
        ),
        ApplicationEventKind::QuicRefusal(value) if value.code.contains("alpn-unsupported") => {
            ProtocolClassification::new(
                TrafficFamily::Quic,
                DetectionState::Unsupported,
                InspectabilityState::Unavailable,
                Some(ClassificationReason::UnsupportedVersion),
            )
            .expect("QUIC unsupported classification is valid")
        }
        ApplicationEventKind::QuicRefusal(value) if value.code.contains("protocol-failed") => {
            ProtocolClassification::new(
                TrafficFamily::Quic,
                DetectionState::Failed,
                InspectabilityState::MetadataOnly,
                Some(ClassificationReason::ParserFailed),
            )
            .expect("QUIC parser classification is valid")
        }
        ApplicationEventKind::QuicRefusal(value) => {
            let reason = ClassificationReason::from_raw(value.code);
            identified(
                TrafficFamily::Quic,
                InspectabilityState::Unavailable,
                reason,
            )
        }
        ApplicationEventKind::HttpStreamTerminal(outcome) => match outcome {
            fragcap_proxy::StreamTerminal::Complete => {
                identified(protocol_family(), InspectabilityState::Full, None)
            }
            fragcap_proxy::StreamTerminal::ProtocolError => ProtocolClassification::new(
                protocol_family(),
                DetectionState::Failed,
                InspectabilityState::MetadataOnly,
                Some(ClassificationReason::ParserFailed),
            )
            .expect("HTTP protocol terminal classification is valid"),
            _ => identified(
                protocol_family(),
                InspectabilityState::MetadataOnly,
                Some(ClassificationReason::Truncated),
            ),
        },
        ApplicationEventKind::Error { code }
            if event.protocol.is_some()
                && (code.contains("parse") || code.contains("protocol")) =>
        {
            ProtocolClassification::new(
                protocol_family(),
                DetectionState::Failed,
                InspectabilityState::MetadataOnly,
                Some(ClassificationReason::ParserFailed),
            )
            .expect("known protocol parser classification is valid")
        }
        _ if event.protocol.is_some() => {
            identified(protocol_family(), InspectabilityState::Full, None)
        }
        ApplicationEventKind::ConnectionOpen(value) => {
            let family = TrafficFamily::from_proxy_label(value.transport);
            if family == TrafficFamily::Unknown {
                ProtocolClassification::new(
                    family,
                    DetectionState::Unknown,
                    InspectabilityState::Unavailable,
                    None,
                )
                .expect("unknown connection classification is valid")
            } else {
                identified(family, InspectabilityState::MetadataOnly, None)
            }
        }
        ApplicationEventKind::TlsNegotiation(_) | ApplicationEventKind::TlsTerminal(_) => {
            identified(
                TrafficFamily::NonHttpTls,
                InspectabilityState::MetadataOnly,
                None,
            )
        }
        _ => ProtocolClassification::new(
            TrafficFamily::Unknown,
            DetectionState::Unknown,
            InspectabilityState::Unavailable,
            None,
        )
        .expect("unknown application classification is valid"),
    }
}

fn streaming_event_outcome(
    event: &fragcap_proxy::StreamingEvent,
) -> Option<fragcap_proxy::StreamingOutcome> {
    use fragcap_proxy::StreamingEvent;
    match event {
        StreamingEvent::WebSocketFrame(value) => Some(value.outcome),
        StreamingEvent::WebSocketMessage(value) => Some(value.outcome),
        StreamingEvent::WebSocketTerminal { outcome, .. }
        | StreamingEvent::SseTerminal { outcome }
        | StreamingEvent::GrpcTerminal { outcome, .. } => Some(*outcome),
        StreamingEvent::SseField(value) => Some(value.outcome),
        StreamingEvent::SseEvent(value) => Some(value.outcome),
        StreamingEvent::GrpcMessage(value) => Some(value.outcome),
        StreamingEvent::GrpcCall { .. } => None,
    }
}

fn event_type(kind: &ApplicationEventKind) -> &'static str {
    match kind {
        ApplicationEventKind::ConnectionOpen(_) => "connection.open",
        ApplicationEventKind::UpstreamSocket(_) => "connection.upstream_socket",
        ApplicationEventKind::ConnectionTerminal(_) => "connection.terminal",
        ApplicationEventKind::TlsNegotiation(_) => "tls.negotiation",
        ApplicationEventKind::TlsTerminal(_) => "tls.terminal",
        ApplicationEventKind::HttpStreamOpen => "http.stream.open",
        ApplicationEventKind::HttpStreamTerminal(_) => "http.stream.terminal",
        ApplicationEventKind::HttpTiming(_) => "http.timing",
        ApplicationEventKind::Metadata(_) => "http.metadata",
        ApplicationEventKind::Body(_) => "http.body_segment",
        ApplicationEventKind::Transformation(_) => "http.transformation",
        ApplicationEventKind::Streaming(value) => streaming_type(value),
        ApplicationEventKind::SocksNegotiation(_) => "socks5.negotiation",
        ApplicationEventKind::SocksConnect(_) => "socks5.connect",
        ApplicationEventKind::SocksTransfer(_) => "socks5.transfer",
        ApplicationEventKind::SocksUdp(_) => "socks5.udp",
        ApplicationEventKind::GenericStreamChunk(_) => "generic.stream_chunk",
        ApplicationEventKind::GenericUdpDatagram(_) => "generic.udp_datagram",
        ApplicationEventKind::UdpSocketError(_) => "generic.udp_socket_error",
        ApplicationEventKind::QuicConnection(_) => "quic.connection",
        ApplicationEventKind::QuicStream(_) => "quic.stream",
        ApplicationEventKind::QuicDatagram(_) => "quic.datagram",
        ApplicationEventKind::QuicRefusal(_) => "quic.refusal",
        ApplicationEventKind::Error { .. } => "application.error",
    }
}

fn streaming_type(value: &fragcap_proxy::StreamingEvent) -> &'static str {
    use fragcap_proxy::StreamingEvent::*;
    match value {
        WebSocketFrame(_) => "websocket.frame",
        WebSocketMessage(_) => "websocket.message",
        WebSocketTerminal { .. } => "websocket.terminal",
        SseField(_) => "sse.field",
        SseEvent(_) => "sse.event",
        SseTerminal { .. } => "sse.terminal",
        GrpcCall { .. } => "grpc.call",
        GrpcMessage(_) => "grpc.message",
        GrpcTerminal { .. } => "grpc.terminal",
    }
}

fn streaming_json(value: fragcap_proxy::StreamingEvent) -> (&'static str, Value) {
    use fragcap_proxy::StreamingEvent::*;
    match value {
        WebSocketFrame(frame) => {
            let omitted = frame.payload_omitted;
            let mut body = json!({
                "direction": direction(frame.direction), "frame_sequence": frame.sequence,
                "fin": frame.fin, "rsv1": frame.rsv1, "opcode": frame.opcode,
                "masked": frame.masked, "masking_key": frame.masking_key.map(|value| base64::engine::general_purpose::STANDARD.encode(value)),
                "declared_len": frame.declared_len, "retained_len": frame.wire_payload.len(),
            "payload_omitted": frame.payload_omitted,
            "payload_encoding": "base64", "payload": base64::engine::general_purpose::STANDARD.encode(frame.wire_payload),
            "close_code": frame.close_code, "close_reason": binary_json(frame.close_reason),
            "close_reason_utf8": frame.close_reason_utf8,
                "outcome": streaming_outcome(frame.outcome),
            });
            if omitted {
                let object = body.as_object_mut().expect("streaming record is an object");
                object.remove("payload_encoding");
                object.remove("payload");
                object.remove("close_reason");
                object.remove("close_reason_utf8");
            }
            ("websocket.frame", body)
        }
        WebSocketMessage(message) => {
            let omitted = message.payload_omitted;
            let mut body = json!({
                "direction": direction(message.direction), "message_sequence": message.sequence,
                "first_frame": message.first_frame, "frame_count": message.frame_count,
                "kind": format!("{:?}", message.kind).to_ascii_lowercase(), "compressed": message.compressed,
                "observed_len": message.observed_len, "retained_len": message.payload.len(),
                "payload_omitted": message.payload_omitted,
                "payload_encoding": "base64", "payload": base64::engine::general_purpose::STANDARD.encode(message.payload),
                "outcome": streaming_outcome(message.outcome),
            });
            if omitted {
                let object = body.as_object_mut().expect("streaming record is an object");
                object.remove("payload_encoding");
                object.remove("payload");
            }
            ("websocket.message", body)
        }
        WebSocketTerminal {
            direction: value,
            outcome,
        } => (
            "websocket.terminal",
            json!({"direction": direction(value), "outcome": streaming_outcome(outcome)}),
        ),
        SseField(field) => {
            let omitted = field.payload_omitted;
            let retained_len = field.value.len();
            let mut body = json!({
                "field_sequence": field.sequence, "line": field.line, "comment": field.comment,
                "name": binary_json(field.name), "value": binary_json(field.value),
                "observed_len": field.observed_len, "retained_len": retained_len,
                "payload_omitted": field.payload_omitted,
                "outcome": streaming_outcome(field.outcome),
            });
            if omitted {
                let object = body.as_object_mut().expect("streaming record is an object");
                object.remove("name");
                object.remove("value");
            }
            ("sse.field", body)
        }
        SseEvent(event) => {
            let omitted = event.payload_omitted;
            let retained_len = event.data.len();
            let mut body = json!({
                "event_sequence": event.sequence, "first_line": event.first_line, "last_line": event.last_line,
                "event_type": binary_json(event.event_type), "data": binary_json(event.data),
                "last_event_id": binary_json(event.last_event_id), "retry_ms": event.retry_ms,
                "observed_len": event.observed_len, "retained_len": retained_len,
                "payload_omitted": event.payload_omitted,
                "outcome": streaming_outcome(event.outcome),
            });
            if omitted {
                let object = body.as_object_mut().expect("streaming record is an object");
                object.remove("event_type");
                object.remove("data");
                object.remove("last_event_id");
                object.remove("retry_ms");
            }
            ("sse.event", body)
        }
        SseTerminal { outcome } => (
            "sse.terminal",
            json!({"outcome": streaming_outcome(outcome)}),
        ),
        GrpcCall {
            method,
            content_type,
            encoding,
        } => (
            "grpc.call",
            json!({
                "method": binary_json(method), "content_type": binary_json(content_type),
                "encoding": encoding.map(binary_json),
            }),
        ),
        GrpcMessage(message) => {
            let omitted = message.payload_omitted;
            let mut body = json!({
                "direction": direction(message.direction), "message_sequence": message.sequence,
            "compressed": message.compressed, "declared_len": message.declared_len,
            "encoding": message.encoding.map(binary_json),
                "retained_len": message.payload.len(), "payload_encoding": "base64",
                "payload_omitted": message.payload_omitted,
                "payload": base64::engine::general_purpose::STANDARD.encode(message.payload),
                "outcome": streaming_outcome(message.outcome),
            });
            if omitted {
                let object = body.as_object_mut().expect("streaming record is an object");
                object.remove("payload_encoding");
                object.remove("payload");
            }
            ("grpc.message", body)
        }
        GrpcTerminal {
            direction: value,
            status,
            message,
            status_details,
            outcome,
        } => (
            "grpc.terminal",
            json!({
                "direction": direction(value),
                "status": status.map(binary_json), "message": message.map(binary_json),
                "status_details": status_details.map(binary_json),
                "outcome": streaming_outcome(outcome),
            }),
        ),
    }
}

fn direction(value: fragcap_proxy::BodyDirection) -> &'static str {
    match value {
        fragcap_proxy::BodyDirection::Request => "request",
        fragcap_proxy::BodyDirection::Response => "response",
    }
}

fn streaming_outcome(value: fragcap_proxy::StreamingOutcome) -> &'static str {
    use fragcap_proxy::StreamingOutcome::*;
    match value {
        Complete => "complete",
        IntentionallyOmitted => "intentionally-omitted",
        Partial => "partial",
        Malformed => "malformed",
        Limit => "limit",
        UnsupportedCompression => "unsupported-compression",
        InvalidUtf8 => "invalid-utf8",
        Cancelled => "cancelled",
    }
}

fn streaming_measure(value: &fragcap_proxy::StreamingEvent) -> Option<(&'static str, u64, u64)> {
    use fragcap_proxy::StreamingEvent::*;
    let (outcome, observed, retained) = match value {
        WebSocketFrame(value) => (
            value.outcome,
            value.declared_len,
            value.wire_payload.len() as u64,
        ),
        WebSocketMessage(value) => (
            value.outcome,
            value.observed_len,
            value.payload.len() as u64,
        ),
        SseField(value) => (value.outcome, value.observed_len, value.value.len() as u64),
        SseEvent(value) => (value.outcome, value.observed_len, value.data.len() as u64),
        GrpcMessage(value) => (
            value.outcome,
            u64::from(value.declared_len),
            value.payload.len() as u64,
        ),
        _ => return None,
    };
    Some((streaming_outcome(outcome), observed, retained))
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
    parse_application_prefix(BufReader::new(File::open(path)?))
}

pub(crate) fn read_application_prefix_bytes(bytes: &[u8]) -> io::Result<ApplicationPrefix> {
    parse_application_prefix(BufReader::new(Cursor::new(bytes)))
}

fn parse_application_prefix(reader: impl BufRead) -> io::Result<ApplicationPrefix> {
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
    let classification_schema_version = first
        .get("classification_schema_version")
        .and_then(Value::as_u64);
    let classification_status = match classification_schema_version {
        None => ClassificationStreamStatus::LegacyAbsent,
        Some(version) if version == u64::from(CLASSIFICATION_SCHEMA_VERSION) => {
            ClassificationStreamStatus::Supported
        }
        Some(_) => ClassificationStreamStatus::UnknownVersion,
    };
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
        classification_schema_version,
        classification_status,
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
    let mut streaming_queue_dropped = 0_u64;
    let mut streaming_observed = 0_u64;
    let mut streaming_retained = 0_u64;
    let mut streaming_truncated = 0_u64;
    let mut streaming_by_outcome = BTreeMap::<String, u64>::new();
    let mut generic_queue_dropped = 0_u64;
    let mut generic_observed = 0_u64;
    let mut generic_retained = 0_u64;
    let mut generic_omitted = 0_u64;
    let mut generic_by_outcome = BTreeMap::<String, u64>::new();
    let mut generic_udp_queue_dropped_datagrams = 0_u64;
    let mut generic_udp_queue_dropped_bytes = 0_u64;
    let mut generic_udp_storage_dropped_datagrams = 0_u64;
    let mut generic_udp_storage_dropped_bytes = 0_u64;
    let mut generic_udp_observed_datagrams = 0_u64;
    let mut generic_udp_observed_bytes = 0_u64;
    let mut generic_udp_retained_bytes = 0_u64;
    let mut generic_udp_omitted_bytes = 0_u64;
    let mut generic_udp_truncated_datagrams = 0_u64;
    let mut generic_udp_by_outcome = BTreeMap::<String, u64>::new();
    let mut correlation_total = 0_u64;
    let mut correlation_by_state = BTreeMap::<String, u64>::new();
    let classification_version = records
        .first()
        .and_then(|record| record["classification_schema_version"].as_u64());
    let mut classifications_by_family = BTreeMap::<String, u64>::new();
    let mut classifications_by_detection = BTreeMap::<String, u64>::new();
    let mut classifications_by_inspectability = BTreeMap::<String, u64>::new();
    let mut classifications_by_reason = BTreeMap::<String, u64>::new();
    let mut classified_records = 0_u64;
    for record in records.iter().skip(1).take(records.len().saturating_sub(2)) {
        let Some(kind) = record.get("type").and_then(Value::as_str) else {
            return false;
        };
        let classification_required = classification_version
            == Some(u64::from(CLASSIFICATION_SCHEMA_VERSION))
            && !matches!(kind, "application.correlation" | "application.gap");
        if let Some(classification) = record.get("classification") {
            let (Some(family), Some(detection), Some(inspectability)) = (
                classification["family"].as_str(),
                classification["detection"].as_str(),
                classification["inspectability"].as_str(),
            ) else {
                return false;
            };
            if classification["schema_version"].as_u64() != classification_version {
                return false;
            }
            classified_records = classified_records.saturating_add(1);
            *classifications_by_family
                .entry(family.to_string())
                .or_default() += 1;
            *classifications_by_detection
                .entry(detection.to_string())
                .or_default() += 1;
            *classifications_by_inspectability
                .entry(inspectability.to_string())
                .or_default() += 1;
            if let Some(reason) = classification["reason"].as_str() {
                *classifications_by_reason
                    .entry(reason.to_string())
                    .or_default() += 1;
            }
        } else if classification_required {
            return false;
        }
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
            streaming_queue_dropped = streaming_queue_dropped.saturating_add(
                record
                    .get("streaming_bytes_queue_dropped")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
            );
            generic_queue_dropped = generic_queue_dropped.saturating_add(
                record
                    .get("generic_stream_bytes_queue_dropped")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
            );
            generic_udp_queue_dropped_datagrams = generic_udp_queue_dropped_datagrams
                .saturating_add(
                    record
                        .get("generic_udp_datagrams_queue_dropped")
                        .and_then(Value::as_u64)
                        .unwrap_or(0),
                );
            generic_udp_queue_dropped_bytes = generic_udp_queue_dropped_bytes.saturating_add(
                record
                    .get("generic_udp_bytes_queue_dropped")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
            );
            generic_udp_storage_dropped_datagrams = generic_udp_storage_dropped_datagrams
                .saturating_add(
                    record
                        .get("generic_udp_datagrams_storage_dropped")
                        .and_then(Value::as_u64)
                        .unwrap_or(0),
                );
            generic_udp_storage_dropped_bytes = generic_udp_storage_dropped_bytes.saturating_add(
                record
                    .get("generic_udp_bytes_storage_dropped")
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
        if kind == "application.correlation" {
            let Some(state) = record.get("correlation_state").and_then(Value::as_str) else {
                return false;
            };
            correlation_total = correlation_total.saturating_add(1);
            *correlation_by_state.entry(state.to_string()).or_default() += 1;
        }
        if matches!(
            kind,
            "websocket.frame" | "websocket.message" | "sse.field" | "sse.event" | "grpc.message"
        ) {
            let observed = record
                .get("observed_len")
                .or_else(|| record.get("declared_len"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let retained = record
                .get("retained_len")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            streaming_observed = streaming_observed.saturating_add(observed);
            streaming_retained = streaming_retained.saturating_add(retained);
            streaming_truncated =
                streaming_truncated.saturating_add(observed.saturating_sub(retained));
            let outcome = record
                .get("outcome")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            *streaming_by_outcome.entry(outcome.to_string()).or_default() += 1;
        }
        if kind == "generic.stream_chunk" {
            let observed = record
                .get("observed_len")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let retained = record
                .get("retained_len")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            generic_observed = generic_observed.saturating_add(observed);
            generic_retained = generic_retained.saturating_add(retained);
            generic_omitted = generic_omitted.saturating_add(observed.saturating_sub(retained));
            let outcome = record
                .get("outcome")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            *generic_by_outcome.entry(outcome.to_string()).or_default() += 1;
        }
        if kind == "generic.udp_datagram" {
            let observed = record
                .get("observed_len")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let retained = record
                .get("retained_len")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            generic_udp_observed_datagrams = generic_udp_observed_datagrams.saturating_add(1);
            generic_udp_observed_bytes = generic_udp_observed_bytes.saturating_add(observed);
            generic_udp_retained_bytes = generic_udp_retained_bytes.saturating_add(retained);
            generic_udp_omitted_bytes =
                generic_udp_omitted_bytes.saturating_add(observed.saturating_sub(retained));
            let outcome = record
                .get("outcome")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            if outcome == "retention-limit" {
                generic_udp_truncated_datagrams = generic_udp_truncated_datagrams.saturating_add(1);
            }
            *generic_udp_by_outcome
                .entry(outcome.to_string())
                .or_default() += 1;
        }
    }
    trailer.get("writer_status").and_then(Value::as_str) == Some("complete")
        && trailer.get("writer_failures").and_then(Value::as_u64) == Some(0)
        && trailer.get("accepted_records").and_then(Value::as_u64) == Some(written)
        && trailer.get("written_records").and_then(Value::as_u64) == Some(written)
        && trailer.get("dropped_records").and_then(Value::as_u64) == Some(dropped)
        && trailer.get("serialized_bytes").and_then(Value::as_u64) == Some(serialized_bytes)
        && trailer.get("records_by_type") == Some(&json!(records_by_type))
        && (classification_version != Some(u64::from(CLASSIFICATION_SCHEMA_VERSION))
            || (trailer.get("classification_schema_version")
                == Some(&json!(CLASSIFICATION_SCHEMA_VERSION))
                && trailer.get("classified_records") == Some(&json!(classified_records))
                && trailer
                    .get("classification_records_lost")
                    .and_then(Value::as_u64)
                    == Some(
                        dropped.saturating_add(
                            trailer
                                .get("proxy_observations_dropped_oldest")
                                .and_then(Value::as_u64)
                                .unwrap_or(0),
                        ),
                    )
                && trailer.get("classifications_by_family")
                    == Some(&json!(classifications_by_family))
                && trailer.get("classifications_by_detection")
                    == Some(&json!(classifications_by_detection))
                && trailer.get("classifications_by_inspectability")
                    == Some(&json!(classifications_by_inspectability))
                && trailer.get("classifications_by_reason")
                    == Some(&json!(classifications_by_reason))))
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
        && trailer
            .get("streaming_bytes_queue_dropped")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == streaming_queue_dropped
        && trailer
            .get("streaming_bytes_observed")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == streaming_observed
        && trailer
            .get("streaming_bytes_retained")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == streaming_retained
        && trailer
            .get("streaming_bytes_truncated")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == streaming_truncated
        && trailer
            .get("streaming_records_by_outcome")
            .cloned()
            .unwrap_or_else(|| json!({}))
            == json!(streaming_by_outcome)
        && trailer
            .get("generic_stream_bytes_queue_dropped")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == generic_queue_dropped
        && trailer
            .get("generic_stream_bytes_observed")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == generic_observed
        && trailer
            .get("generic_stream_bytes_retained")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == generic_retained
        && trailer
            .get("generic_stream_bytes_omitted")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == generic_omitted
        && trailer
            .get("generic_stream_records_by_outcome")
            .cloned()
            .unwrap_or_else(|| json!({}))
            == json!(generic_by_outcome)
        && trailer
            .get("generic_udp_datagrams_observed")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == generic_udp_observed_datagrams
        && trailer
            .get("generic_udp_bytes_observed")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == generic_udp_observed_bytes
        && trailer
            .get("generic_udp_bytes_retained")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == generic_udp_retained_bytes
        && trailer
            .get("generic_udp_bytes_omitted")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == generic_udp_omitted_bytes
        && trailer
            .get("generic_udp_datagrams_truncated")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == generic_udp_truncated_datagrams
        && trailer
            .get("generic_udp_datagrams_queue_dropped")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == generic_udp_queue_dropped_datagrams
        && trailer
            .get("generic_udp_bytes_queue_dropped")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == generic_udp_queue_dropped_bytes
        && trailer
            .get("generic_udp_datagrams_storage_dropped")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == generic_udp_storage_dropped_datagrams
        && trailer
            .get("generic_udp_bytes_storage_dropped")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == generic_udp_storage_dropped_bytes
        && trailer
            .get("generic_udp_records_by_outcome")
            .cloned()
            .unwrap_or_else(|| json!({}))
            == json!(generic_udp_by_outcome)
        && trailer
            .get("correlation_connections_total")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == correlation_total
        && trailer
            .get("correlation_connections_by_state")
            .cloned()
            .unwrap_or_else(|| json!({}))
            == json!(correlation_by_state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap_proxy::{
        ApplicationEvent, ApplicationEventKind, BodyDirection, ProtocolVersion, StreamTerminal,
        StreamingEvent, StreamingOutcome,
    };

    #[test]
    fn streaming_failures_and_limits_are_not_classified_as_full() {
        let classification = |outcome| {
            application_event_classification(&ApplicationEvent::now(
                "session",
                7,
                Some(1),
                Some(ProtocolVersion::Http11),
                ApplicationEventKind::Streaming(StreamingEvent::WebSocketTerminal {
                    direction: BodyDirection::Response,
                    outcome,
                }),
            ))
        };

        let malformed = classification(StreamingOutcome::Malformed);
        assert_eq!(malformed.detection(), DetectionState::Failed);
        assert_eq!(
            malformed.inspectability(),
            InspectabilityState::MetadataOnly
        );
        assert_eq!(malformed.reason(), Some(ClassificationReason::ParserFailed));

        let limited = classification(StreamingOutcome::Limit);
        assert_eq!(limited.detection(), DetectionState::Identified);
        assert_eq!(limited.inspectability(), InspectabilityState::MetadataOnly);
        assert_eq!(limited.reason(), Some(ClassificationReason::Truncated));

        let protocol_error = application_event_classification(&ApplicationEvent::now(
            "session",
            7,
            Some(1),
            Some(ProtocolVersion::Http2),
            ApplicationEventKind::HttpStreamTerminal(StreamTerminal::ProtocolError),
        ));
        assert_eq!(protocol_error.detection(), DetectionState::Failed);
        assert_eq!(
            protocol_error.reason(),
            Some(ClassificationReason::ParserFailed)
        );

        let reset = application_event_classification(&ApplicationEvent::now(
            "session",
            7,
            Some(1),
            Some(ProtocolVersion::Http3),
            ApplicationEventKind::HttpStreamTerminal(StreamTerminal::Reset),
        ));
        assert_eq!(reset.inspectability(), InspectabilityState::MetadataOnly);
        assert_eq!(reset.reason(), Some(ClassificationReason::Truncated));
    }

    #[test]
    fn generic_udp_queue_loss_identity_is_bounded_with_exact_overflow() {
        let account = WriterAccount::default();
        for port in 42_000..42_000 + MAX_UDP_QUEUE_LOSS_KEYS as u16 + 1 {
            account.record_queue_drop(&ApplicationEvent::now(
                "session",
                7,
                None,
                None,
                ApplicationEventKind::GenericUdpDatagram(fragcap_proxy::GenericUdpDatagram {
                    direction: fragcap_proxy::GenericUdpDirection::ClientToUpstream,
                    sequence: u64::from(port),
                    client_endpoint: "127.0.0.1:41000".parse().unwrap(),
                    remote_endpoint: std::net::SocketAddr::from(([127, 0, 0, 1], port)),
                    observed_len: 4,
                    bytes: bytes::Bytes::from_static(b"data"),
                    outcome: fragcap_proxy::GenericUdpOutcome::Complete,
                }),
            ));
        }
        assert_eq!(
            account.generic_udp_queue_losses.lock().unwrap().len(),
            MAX_UDP_QUEUE_LOSS_KEYS
        );
        assert_eq!(
            account
                .generic_udp_queue_loss_overflow_datagrams
                .load(Ordering::Acquire),
            1
        );
        assert_eq!(
            account
                .generic_udp_queue_loss_overflow_bytes
                .load(Ordering::Acquire),
            4
        );
    }

    #[test]
    fn retired_writer_counts_generic_udp_storage_loss() {
        let (sender, _receiver) = mpsc::sync_channel(1);
        let account = Arc::new(WriterAccount::default());
        let sink = ChannelSink {
            sender: Arc::new(Mutex::new(Some(sender))),
            retired: Arc::new(AtomicBool::new(true)),
            account: Arc::clone(&account),
            connections: Arc::new(Mutex::new(BTreeMap::new())),
        };
        let disposition = sink.try_emit(ApplicationEvent::now(
            "session",
            7,
            None,
            None,
            ApplicationEventKind::GenericUdpDatagram(fragcap_proxy::GenericUdpDatagram {
                direction: fragcap_proxy::GenericUdpDirection::UpstreamToClient,
                sequence: 0,
                client_endpoint: "127.0.0.1:41000".parse().unwrap(),
                remote_endpoint: "127.0.0.1:42000".parse().unwrap(),
                observed_len: 4,
                bytes: bytes::Bytes::from_static(b"data"),
                outcome: fragcap_proxy::GenericUdpOutcome::Complete,
            }),
        ));
        assert_eq!(disposition, EventDisposition::Retired);
        let accounting = sink.accounting();
        assert_eq!(accounting.generic_udp_datagrams_storage_dropped, 1);
        assert_eq!(accounting.generic_udp_bytes_storage_dropped, 4);
    }

    #[test]
    fn writer_queue_reports_capacity_current_and_peak() {
        let (sender, receiver) = mpsc::sync_channel(1);
        let account = Arc::new(WriterAccount::default());
        account.queue_capacity.store(1, Ordering::Release);
        let sink = ChannelSink {
            sender: Arc::new(Mutex::new(Some(sender))),
            retired: Arc::new(AtomicBool::new(false)),
            account: Arc::clone(&account),
            connections: Arc::new(Mutex::new(BTreeMap::new())),
        };
        let event = ApplicationEvent::now(
            "session",
            7,
            None,
            None,
            ApplicationEventKind::ConnectionOpen(ConnectionDescriptor {
                transport: "tcp",
                client_peer: "127.0.0.1:41000".parse().unwrap(),
                proxy_local: "127.0.0.1:42000".parse().unwrap(),
            }),
        );
        assert_eq!(sink.try_emit(event.clone()), EventDisposition::Accepted);
        assert_eq!(sink.try_emit(event), EventDisposition::QueueFull);
        let accounting = sink.accounting();
        assert_eq!(accounting.queue_capacity, 1);
        assert_eq!(accounting.queue_current, 1);
        assert_eq!(accounting.queue_peak, 1);
        let _ = receiver.recv().unwrap();
        account.queue_current.fetch_sub(1, Ordering::AcqRel);
        assert_eq!(sink.accounting().queue_current, 0);
    }

    #[test]
    fn write_failure_counts_the_triggering_and_buffered_quic_records() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("read-only.jsonl");
        drop(File::create(&path).unwrap());
        let file = OpenOptions::new().read(true).open(&path).unwrap();
        let (sender, receiver) = mpsc::sync_channel(2);
        let sender = Arc::new(Mutex::new(Some(sender)));
        let retired = Arc::new(AtomicBool::new(false));
        let account = Arc::new(WriterAccount::default());
        account.queue_capacity.store(2, Ordering::Release);
        let sink = ChannelSink {
            sender: Arc::clone(&sender),
            retired: Arc::clone(&retired),
            account: Arc::clone(&account),
            connections: Arc::new(Mutex::new(BTreeMap::new())),
        };
        assert_eq!(
            sink.try_emit(ApplicationEvent::now(
                "session",
                7,
                Some(0),
                Some(ProtocolVersion::Http3),
                ApplicationEventKind::QuicStream(fragcap_proxy::QuicStreamEvent {
                    pair_id: 9,
                    direction: fragcap_proxy::QuicDirection::ClientToUpstream,
                    stream_id: 0,
                    stream_kind: "http3-request",
                    sequence: 0,
                    offset: 0,
                    observed_len: 4,
                    bytes: bytes::Bytes::from_static(b"data"),
                    outcome: fragcap_proxy::GenericStreamOutcome::Complete,
                    terminal: "forwarded",
                }),
            )),
            EventDisposition::Accepted
        );
        assert_eq!(
            sink.try_emit(ApplicationEvent::now(
                "session",
                7,
                None,
                None,
                ApplicationEventKind::QuicDatagram(fragcap_proxy::QuicDatagramEvent {
                    pair_id: 9,
                    direction: fragcap_proxy::QuicDirection::ClientToUpstream,
                    sequence: 0,
                    observed_len: 4,
                    bytes: bytes::Bytes::from_static(b"data"),
                    outcome: fragcap_proxy::GenericUdpOutcome::Complete,
                    terminal: "forwarded",
                }),
            )),
            EventDisposition::Accepted
        );

        assert!(writer_loop(
            file,
            "session",
            receiver,
            sender,
            Arc::clone(&retired),
            Arc::clone(&account),
            Arc::new(Mutex::new(BTreeMap::new())),
            Arc::new(|_| ApplicationCorrelation::default()),
            Arc::new(Mutex::new(None)),
        )
        .is_err());
        assert!(retired.load(Ordering::Acquire));
        assert_eq!(account.accepted.load(Ordering::Acquire), 2);
        assert_eq!(account.written.load(Ordering::Acquire), 0);
        assert_eq!(account.dropped.load(Ordering::Acquire), 2);
        assert_eq!(
            account
                .quic_stream_bytes_storage_dropped
                .load(Ordering::Acquire),
            4
        );
        assert_eq!(
            account
                .quic_datagrams_storage_dropped
                .load(Ordering::Acquire),
            1
        );
        assert_eq!(
            account
                .quic_datagram_bytes_storage_dropped
                .load(Ordering::Acquire),
            4
        );
    }

    #[test]
    fn buffered_writer_commits_prior_records_before_a_later_flush_failure() {
        #[derive(Default)]
        struct FailSecondWrite {
            calls: usize,
            bytes: Vec<u8>,
        }
        impl Write for FailSecondWrite {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                self.calls += 1;
                if self.calls == 2 {
                    return Err(io::Error::other("second write failed"));
                }
                self.bytes.extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let event = ApplicationEvent::now(
            "session",
            7,
            None,
            None,
            ApplicationEventKind::ConnectionOpen(ConnectionDescriptor {
                transport: "tcp",
                client_peer: "127.0.0.1:41000".parse().unwrap(),
                proxy_local: "127.0.0.1:42000".parse().unwrap(),
            }),
        );
        let value = event_json(event.clone(), 1, ApplicationCorrelation::default());
        let wire_length = serialized_record(&value).unwrap().0.len();
        let mut writer = BufWriter::with_capacity(wire_length + 1, FailSecondWrite::default());
        let account = WriterAccount::default();
        let mut pending = Vec::with_capacity(64);
        let mut serialized_bytes = 0;

        buffer_application_record(
            &mut writer,
            &value,
            event.clone(),
            &mut pending,
            &mut serialized_bytes,
            &account,
        )
        .unwrap();
        buffer_application_record(
            &mut writer,
            &value,
            event,
            &mut pending,
            &mut serialized_bytes,
            &account,
        )
        .unwrap();

        assert_eq!(account.written.load(Ordering::Acquire), 1);
        assert_eq!(pending.len(), 1);
        assert!(flush_application_batch(
            &mut writer,
            &mut pending,
            &mut serialized_bytes,
            &account,
        )
        .is_err());
        assert_eq!(account.written.load(Ordering::Acquire), 1);
        assert_eq!(pending.len(), 1);
        assert_eq!(
            writer
                .get_ref()
                .bytes
                .iter()
                .filter(|byte| **byte == b'\n')
                .count(),
            1
        );
    }

    #[test]
    fn queue_pressure_cannot_erase_a_connection_descriptor() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("application.jsonl");
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .unwrap();
        write_record(&mut file, &header_record("session")).unwrap();
        let (sender, receiver) = mpsc::sync_channel(1);
        let retired = Arc::new(AtomicBool::new(false));
        let account = Arc::new(WriterAccount::default());
        account.queue_capacity.store(1, Ordering::Release);
        let connections = Arc::new(Mutex::new(BTreeMap::new()));
        let sink = ChannelSink {
            sender: Arc::new(Mutex::new(Some(sender))),
            retired: Arc::clone(&retired),
            account: Arc::clone(&account),
            connections: Arc::clone(&connections),
        };
        assert_eq!(
            sink.try_emit(ApplicationEvent::now(
                "session",
                99,
                Some(1),
                Some(ProtocolVersion::Http2),
                ApplicationEventKind::HttpStreamOpen,
            )),
            EventDisposition::Accepted
        );
        let descriptor = ConnectionDescriptor {
            transport: "tcp",
            client_peer: "127.0.0.1:41000".parse().unwrap(),
            proxy_local: "127.0.0.1:42000".parse().unwrap(),
        };
        assert_eq!(
            sink.try_emit(ApplicationEvent::now(
                "session",
                7,
                None,
                None,
                ApplicationEventKind::ConnectionOpen(descriptor),
            )),
            EventDisposition::QueueFull
        );
        assert_eq!(
            sink.try_emit(ApplicationEvent::now(
                "session",
                7,
                None,
                None,
                ApplicationEventKind::ConnectionTerminal(StreamTerminal::Complete),
            )),
            EventDisposition::QueueFull
        );
        let sender = Arc::clone(&sink.sender);
        drop(sink.sender.lock().unwrap().take());
        writer_loop(
            file,
            "session",
            receiver,
            sender,
            retired,
            account,
            connections,
            Arc::new(|_| ApplicationCorrelation {
                state: Some("unavailable".to_string()),
                reason: Some("packet-evidence-unavailable".to_string()),
                ..ApplicationCorrelation::default()
            }),
            Arc::new(Mutex::new(None)),
        )
        .unwrap();
        let prefix = read_application_prefix(&path).unwrap();
        assert_eq!(prefix.status, ApplicationStreamStatus::Complete);
        let correlation = prefix
            .records
            .iter()
            .find(|record| record["type"] == "application.correlation")
            .expect("the queue-dropped descriptor still produces final correlation");
        assert_eq!(correlation["proxy_connection_id"], 7);
        assert_eq!(correlation["correlation_state"], "unavailable");
    }

    #[test]
    fn connection_history_bound_is_reported_as_unavailable() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("application.jsonl");
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .unwrap();
        write_record(&mut file, &header_record("session")).unwrap();
        let (sender, receiver) = mpsc::sync_channel(1);
        let retired = Arc::new(AtomicBool::new(false));
        let account = Arc::new(WriterAccount::default());
        account.queue_capacity.store(1, Ordering::Release);
        let connections = Arc::new(Mutex::new(BTreeMap::new()));
        let sink = ChannelSink {
            sender: Arc::new(Mutex::new(Some(sender))),
            retired: Arc::clone(&retired),
            account: Arc::clone(&account),
            connections: Arc::clone(&connections),
        };
        for connection_id in 1..=(MAX_CONNECTION_WINDOWS as u64 + 1) {
            let descriptor = ConnectionDescriptor {
                transport: "tcp",
                client_peer: format!("127.0.0.1:{}", 41_000 + connection_id)
                    .parse()
                    .unwrap(),
                proxy_local: "127.0.0.1:42000".parse().unwrap(),
            };
            let _ = sink.try_emit(ApplicationEvent::now(
                "session",
                connection_id,
                None,
                None,
                ApplicationEventKind::ConnectionOpen(descriptor),
            ));
        }
        assert_eq!(connections.lock().unwrap().len(), MAX_CONNECTION_WINDOWS);
        assert_eq!(
            account
                .connection_windows_unretained
                .load(Ordering::Acquire),
            1
        );
        let sender = Arc::clone(&sink.sender);
        drop(sink.sender.lock().unwrap().take());
        writer_loop(
            file,
            "session",
            receiver,
            sender,
            retired,
            account,
            connections,
            Arc::new(|_| ApplicationCorrelation {
                state: Some("unavailable".to_string()),
                reason: Some("packet-evidence-unavailable".to_string()),
                ..ApplicationCorrelation::default()
            }),
            Arc::new(Mutex::new(None)),
        )
        .unwrap();
        let prefix = read_application_prefix(&path).unwrap();
        assert_eq!(prefix.status, ApplicationStreamStatus::Complete);
        let overflow = prefix
            .records
            .iter()
            .find(|record| {
                record["type"] == "application.correlation"
                    && record["proxy_connection_id"] == MAX_CONNECTION_WINDOWS as u64 + 1
            })
            .expect("overflow must produce a per-connection correlation result");
        assert_eq!(overflow["correlation_state"], "unavailable");
        assert_eq!(
            overflow["correlation_reason"],
            "connection-history-bound-exceeded"
        );
        let trailer = prefix.records.last().unwrap();
        assert_eq!(
            trailer["correlation_connections_total"],
            MAX_CONNECTION_WINDOWS as u64 + 1
        );
        assert_eq!(trailer["correlation_connections_unretained"], 1);
    }

    #[test]
    fn connection_correlation_is_resolved_once_after_event_collection() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("application.jsonl");
        let ready = Arc::new(AtomicBool::new(false));
        let resolver_ready = Arc::clone(&ready);
        let mut lease = ApplicationArtifactLease::open_correlated(
            &path,
            "session",
            8,
            Arc::new(move |window| {
                assert!(resolver_ready.load(Ordering::Acquire));
                assert!(window.closed_at_ns.is_some());
                ApplicationCorrelation {
                    packet_observations: 2,
                    state: Some("matched".to_string()),
                    reason: Some("exact-flow-and-owner".to_string()),
                    ..ApplicationCorrelation::default()
                }
            }),
        )
        .unwrap();
        let descriptor = ConnectionDescriptor {
            transport: "tcp",
            client_peer: "127.0.0.1:41000".parse().unwrap(),
            proxy_local: "127.0.0.1:42000".parse().unwrap(),
        };
        let sink = lease.sink();
        assert_eq!(
            sink.try_emit(ApplicationEvent::now(
                "session",
                1,
                None,
                None,
                ApplicationEventKind::ConnectionOpen(descriptor),
            )),
            EventDisposition::Accepted
        );
        for stream_id in [1, 3] {
            assert_eq!(
                sink.try_emit(ApplicationEvent::now(
                    "session",
                    1,
                    Some(stream_id),
                    Some(ProtocolVersion::Http2),
                    ApplicationEventKind::HttpStreamOpen,
                )),
                EventDisposition::Accepted
            );
        }
        assert_eq!(
            sink.try_emit(ApplicationEvent::now(
                "session",
                1,
                None,
                None,
                ApplicationEventKind::ConnectionTerminal(StreamTerminal::Complete),
            )),
            EventDisposition::Accepted
        );
        ready.store(true, Ordering::Release);
        lease.finish().unwrap();

        let prefix = read_application_prefix(&path).unwrap();
        assert_eq!(prefix.status, ApplicationStreamStatus::Complete);
        let correlation = prefix
            .records
            .iter()
            .find(|record| record["type"] == "application.correlation")
            .unwrap();
        assert_eq!(correlation["correlation_state"], "matched");
        assert_eq!(correlation["packet_observations"], 2);
        let stream_ids = prefix
            .records
            .iter()
            .filter(|record| record["type"] == "http.stream.open")
            .filter_map(|record| record["http_stream_id"].as_u64())
            .collect::<Vec<_>>();
        assert_eq!(stream_ids, [1, 3]);
        assert!(prefix
            .records
            .iter()
            .filter(|record| {
                matches!(
                    record["type"].as_str(),
                    Some("connection.open" | "connection.terminal")
                )
            })
            .all(|record| record["correlation_state"].is_null()
                || record["correlation_state"] == "deferred"));
        let trailer = prefix.records.last().unwrap();
        assert_eq!(trailer["correlation_connections_total"], 1);
        assert_eq!(trailer["correlation_connections_by_state"]["matched"], 1);
    }
}
