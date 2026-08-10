// SPDX-License-Identifier: Apache-2.0

//! The multi-consumer streaming sink (specification sections 14.3 and 14.4).
//!
//! One [`StreamSink`] serves any number of consumers over one transport. Each
//! connected consumer has its own factory-built encoder (its own header
//! preamble, replayed on connect) and its own bounded queue drained by its own
//! writer thread. [`StreamSink::write`] hands each consumer a clone through a
//! non-blocking [`SyncSender::try_send`]: a full queue drops that packet for
//! that consumer only and counts it; a dead consumer is reaped. `write` never
//! blocks and always returns success, so the pipeline's conservation invariant
//! holds and a slow reader never stalls the capture or any other sink
//! (constitution P-4, P-9).
//!
//! Per-consumer drops are the streaming sink's own accounting. They never
//! advance the pipeline's capture-wide `sink_dropped`, and the sink is never
//! retired for a slow downstream reader: that is the whole point of the design
//! recorded in the slice's clarifications.

use std::sync::mpsc::{sync_channel, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use fragcap_core::packet::CapturedPacket;
use fragcap_core::stats::CaptureStats;
use fragcap_core::traits::Sink;
use fragcap_core::SinkError;

use super::{Acceptor, Connection, SinkFactory};

/// Default per-consumer bounded-queue depth, in packets.
pub const DEFAULT_QUEUE_DEPTH: usize = 1024;

/// Default disconnect timeout: a consumer whose queue stays full this long is
/// disconnected.
pub const DEFAULT_DISCONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Grace at capture end for a keeping-up consumer to drain its queue and exit
/// before any straggler is force-unblocked.
const FINISH_GRACE: Duration = Duration::from_millis(100);

/// Why a consumer left the stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisconnectReason {
    /// The reader closed the connection (its writer thread saw a write error).
    ClientClosed,
    /// The consumer's queue stayed full past the disconnect timeout.
    Timeout,
    /// The capture ended while the consumer was connected.
    CaptureEnded,
    /// A write to the connection failed for a reason other than a clean close.
    WriteError,
}

impl DisconnectReason {
    /// A short, stable label for logs and the CLI event stream.
    pub fn as_str(self) -> &'static str {
        match self {
            DisconnectReason::ClientClosed => "client-closed",
            DisconnectReason::Timeout => "timeout",
            DisconnectReason::CaptureEnded => "capture-ended",
            DisconnectReason::WriteError => "write-error",
        }
    }
}

/// The end-of-connection accounting for one consumer, surfaced separately from
/// [`CaptureStats`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsumerReport {
    /// The consumer's identity (peer address for TCP, an ordinal otherwise).
    pub id: String,
    /// Packets the sink offered this consumer while it was connected.
    pub offered: u64,
    /// Packets refused because the consumer's queue was full (backpressure).
    pub dropped: u64,
    /// Packets its encoder actually wrote to the connection.
    pub written: u64,
    /// Why the consumer left.
    pub reason: DisconnectReason,
}

/// What a consumer's writer thread returns when it exits.
struct ConsumerOutcome {
    written: u64,
    write_failed: bool,
}

/// One connected consumer in the active registry.
struct Consumer {
    id: String,
    sender: SyncSender<CapturedPacket>,
    shutdown: Box<dyn super::ConnShutdown>,
    offered: u64,
    dropped: u64,
    full_since: Option<Instant>,
    thread: JoinHandle<ConsumerOutcome>,
}

/// A consumer removed from the active set, awaiting its thread's outcome.
struct Retired {
    id: String,
    offered: u64,
    dropped: u64,
    reason: DisconnectReason,
    shutdown: Box<dyn super::ConnShutdown>,
    thread: JoinHandle<ConsumerOutcome>,
}

impl Retired {
    fn from_consumer(c: Consumer, reason: DisconnectReason) -> Self {
        Retired {
            id: c.id,
            offered: c.offered,
            dropped: c.dropped,
            reason,
            shutdown: c.shutdown,
            thread: c.thread,
        }
    }

    fn into_report(self) -> ConsumerReport {
        let outcome = self.thread.join().unwrap_or(ConsumerOutcome {
            written: 0,
            write_failed: true,
        });
        // A consumer that ended the capture healthy but whose final write
        // failed is reported as a write error rather than a clean end.
        let reason = if outcome.write_failed && self.reason == DisconnectReason::CaptureEnded {
            DisconnectReason::WriteError
        } else {
            self.reason
        };
        ConsumerReport {
            id: self.id,
            offered: self.offered,
            dropped: self.dropped,
            written: outcome.written,
            reason,
        }
    }
}

/// The consumer registry, shared between the output thread (which writes) and
/// the acceptor thread (which registers new consumers).
#[derive(Default)]
struct Registry {
    active: Vec<Consumer>,
    retired: Vec<Retired>,
}

/// A multi-consumer streaming sink over one transport.
pub struct StreamSink {
    registry: Arc<Mutex<Registry>>,
    disconnect_timeout: Duration,
    stopper: Arc<dyn Fn() + Send + Sync>,
    acceptor_thread: Option<JoinHandle<()>>,
    reports: Arc<Mutex<Vec<ConsumerReport>>>,
    describe: String,
}

impl std::fmt::Debug for StreamSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamSink")
            .field("transport", &self.describe)
            .finish()
    }
}

impl StreamSink {
    /// Bind a streaming sink over `acceptor`, building each consumer's encoder
    /// with `factory`. The acceptor is driven on its own thread.
    pub fn new(factory: SinkFactory, acceptor: Box<dyn Acceptor>) -> Self {
        Self::build(
            factory,
            acceptor,
            DEFAULT_QUEUE_DEPTH,
            DEFAULT_DISCONNECT_TIMEOUT,
        )
    }

    /// Bind with an explicit queue depth and disconnect timeout.
    pub fn with_settings(
        factory: SinkFactory,
        acceptor: Box<dyn Acceptor>,
        queue_depth: usize,
        disconnect_timeout: Duration,
    ) -> Self {
        Self::build(factory, acceptor, queue_depth, disconnect_timeout)
    }

    fn build(
        factory: SinkFactory,
        acceptor: Box<dyn Acceptor>,
        queue_depth: usize,
        disconnect_timeout: Duration,
    ) -> Self {
        let registry: Arc<Mutex<Registry>> = Arc::new(Mutex::new(Registry::default()));
        let stopper = acceptor.stopper();
        let describe = acceptor.describe();

        let thread_registry = Arc::clone(&registry);
        let acceptor_thread = std::thread::Builder::new()
            .name("fragcap-stream-acceptor".to_string())
            .spawn(move || accept_loop(acceptor, factory, thread_registry, queue_depth))
            .expect("spawn acceptor thread");

        StreamSink {
            registry,
            disconnect_timeout,
            stopper,
            acceptor_thread: Some(acceptor_thread),
            reports: Arc::new(Mutex::new(Vec::new())),
            describe,
        }
    }

    /// A handle the caller keeps to read the per-consumer reports after the
    /// run finishes. Populated by [`Sink::finish`].
    pub fn reports_handle(&self) -> Arc<Mutex<Vec<ConsumerReport>>> {
        Arc::clone(&self.reports)
    }

    /// How many consumers are connected right now. Used by callers (and tests)
    /// to wait for a client to register before writing.
    pub fn active_consumers(&self) -> usize {
        self.registry.lock().unwrap().active.len()
    }

    /// The transport's bound address or name.
    pub fn transport(&self) -> &str {
        &self.describe
    }
}

/// The acceptor thread: register each connection as a consumer.
fn accept_loop(
    mut acceptor: Box<dyn Acceptor>,
    factory: SinkFactory,
    registry: Arc<Mutex<Registry>>,
    queue_depth: usize,
) {
    while let Some(conn) = acceptor.accept() {
        let Connection {
            id,
            writer,
            shutdown,
        } = conn;
        let encoder = match factory.build(writer) {
            Ok(encoder) => encoder,
            // A connection whose header could not be written is dropped; the
            // reader sees a closed connection. Nothing was owed to it.
            Err(_) => continue,
        };
        let (tx, rx) = sync_channel::<CapturedPacket>(queue_depth);
        let thread = std::thread::Builder::new()
            .name("fragcap-stream-consumer".to_string())
            .spawn(move || consumer_loop(encoder, rx))
            .expect("spawn consumer thread");
        registry.lock().unwrap().active.push(Consumer {
            id,
            sender: tx,
            shutdown,
            offered: 0,
            dropped: 0,
            full_since: None,
            thread,
        });
    }
}

/// A consumer's writer thread: drain the queue into the encoder until the
/// channel closes or a write fails.
fn consumer_loop(
    mut encoder: Box<dyn Sink>,
    rx: std::sync::mpsc::Receiver<CapturedPacket>,
) -> ConsumerOutcome {
    let mut written = 0u64;
    let mut write_failed = false;
    while let Ok(packet) = rx.recv() {
        match encoder.write(&packet) {
            Ok(()) => written += 1,
            Err(_) => {
                write_failed = true;
                break;
            }
        }
    }
    let _ = encoder.flush();
    // The encoder (and the connection it owns) is dropped here, closing it.
    ConsumerOutcome {
        written,
        write_failed,
    }
}

impl Sink for StreamSink {
    fn write(&mut self, packet: &CapturedPacket) -> Result<(), SinkError> {
        let now = Instant::now();
        let mut reg = self.registry.lock().unwrap();
        let mut i = 0;
        while i < reg.active.len() {
            reg.active[i].offered += 1;
            let result = reg.active[i].sender.try_send(packet.clone());
            match result {
                Ok(()) => {
                    reg.active[i].full_since = None;
                    i += 1;
                }
                Err(TrySendError::Full(_)) => {
                    reg.active[i].dropped += 1;
                    let since = *reg.active[i].full_since.get_or_insert(now);
                    if now.duration_since(since) >= self.disconnect_timeout {
                        let consumer = reg.active.remove(i);
                        consumer.shutdown.shutdown();
                        reg.retired
                            .push(Retired::from_consumer(consumer, DisconnectReason::Timeout));
                    } else {
                        i += 1;
                    }
                }
                Err(TrySendError::Disconnected(_)) => {
                    let consumer = reg.active.remove(i);
                    reg.retired.push(Retired::from_consumer(
                        consumer,
                        DisconnectReason::ClientClosed,
                    ));
                }
            }
        }
        // Always Ok: a streaming sink accepts every packet from the pipeline's
        // point of view, so the conservation invariant holds and the sink is
        // never retired for a slow downstream reader.
        Ok(())
    }

    fn flush(&mut self) -> Result<(), SinkError> {
        // Each consumer thread flushes its own encoder; there is nothing to
        // flush at the sink level.
        Ok(())
    }

    fn finish(self: Box<Self>, _stats: &CaptureStats) -> Result<(), SinkError> {
        let this = *self;
        // Stop accepting new consumers and join the acceptor thread.
        (this.stopper)();
        if let Some(handle) = this.acceptor_thread {
            let _ = handle.join();
        }

        let mut reg = this.registry.lock().unwrap();
        // Retire every still-active consumer. Retiring drops its channel sender,
        // so a keeping-up consumer drains its queue, writes its tail, and exits
        // on its own.
        let active: Vec<Consumer> = std::mem::take(&mut reg.active);
        for consumer in active {
            reg.retired.push(Retired::from_consumer(
                consumer,
                DisconnectReason::CaptureEnded,
            ));
        }
        let retired: Vec<Retired> = std::mem::take(&mut reg.retired);
        drop(reg);

        // Give keeping-up consumers a brief grace to drain and exit, then
        // force-unblock any straggler so the join is bounded. A consumer that
        // has kept up has already exited by now and is not truncated; one that
        // has stalled is unblocked and ends where it stalled.
        std::thread::sleep(FINISH_GRACE);
        for retiree in &retired {
            retiree.shutdown.shutdown();
        }

        let reports: Vec<ConsumerReport> = retired.into_iter().map(Retired::into_report).collect();
        *this.reports.lock().unwrap() = reports;
        Ok(())
    }
}
