// SPDX-License-Identifier: Apache-2.0

//! The multi-consumer streaming sink (specification sections 14.3 and 14.4).
//!
//! One [`StreamSink`] serves any number of consumers over one transport. Each
//! connected consumer has its own factory-built encoder (its own header
//! preamble, replayed on connect) and its own bounded queue drained by its own
//! writer thread. [`StreamSink::write`] hands each consumer a clone through a
//! non-blocking [`SyncSender::try_send`]: a full queue withholds that packet
//! from that consumer only; a dead consumer is reaped. `write` never blocks and
//! always returns success, so the pipeline's conservation invariant holds and a
//! slow reader never stalls the capture or any other sink (constitution P-4,
//! P-9).
//!
//! Per-consumer loss is the streaming sink's own accounting, never the
//! pipeline's capture-wide `sink_dropped`, and the sink is never retired for a
//! slow downstream reader. Each consumer's report defines its dropped count as
//! the offered count minus the written count, so the three always reconcile, and
//! a packet its queue accepted but its thread had not yet written when the
//! consumer was disconnected is counted rather than silently lost.
//!
//! Two threads run alongside the pipeline output thread. An acceptor thread
//! registers new consumers. A watchdog thread enforces the disconnect timeout
//! independently of packet arrivals, so a consumer whose queue stays full is
//! disconnected on time even if capture traffic pauses. At capture end each
//! keeping-up consumer is sent a `Finish` message so its encoder writes its
//! trailer (a pcapng Interface Statistics Block, a JSON Lines trailer record)
//! before its connection closes.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
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

/// The watchdog's poll interval, a fraction of the disconnect timeout so a
/// full consumer is disconnected close to when its timeout expires, clamped so
/// a tiny timeout does not spin and a large one is still checked often.
fn watchdog_interval(disconnect_timeout: Duration) -> Duration {
    (disconnect_timeout / 4).clamp(Duration::from_millis(20), Duration::from_millis(250))
}

/// A message to a consumer's writer thread.
enum ToConsumer {
    /// One packet to write.
    Packet(CapturedPacket),
    /// The capture ended: write the trailer with these final statistics.
    Finish(Arc<CaptureStats>),
}

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
    /// Packets the consumer did not receive: refused when its queue was full,
    /// plus any its queue accepted but its thread had not written when it was
    /// disconnected. Always `offered - written`.
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
    sender: SyncSender<ToConsumer>,
    shutdown: Box<dyn super::ConnShutdown>,
    offered: u64,
    full_since: Option<Instant>,
    thread: JoinHandle<ConsumerOutcome>,
}

/// A consumer removed from the active set, awaiting its thread's outcome.
struct Retired {
    id: String,
    offered: u64,
    reason: DisconnectReason,
    shutdown: Box<dyn super::ConnShutdown>,
    thread: JoinHandle<ConsumerOutcome>,
}

impl Retired {
    fn from_consumer(c: Consumer, reason: DisconnectReason) -> Self {
        Retired {
            id: c.id,
            offered: c.offered,
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
        // `offered - written` counts both the backpressure drops during the run
        // and any queued tail unwritten at disconnect, so nothing is silently
        // lost and `offered = written + dropped` holds.
        let dropped = self.offered.saturating_sub(outcome.written);
        ConsumerReport {
            id: self.id,
            offered: self.offered,
            dropped,
            written: outcome.written,
            reason,
        }
    }
}

/// The consumer registry, shared between the output thread (which writes), the
/// acceptor thread (which registers), and the watchdog (which times out).
#[derive(Default)]
struct Registry {
    active: Vec<Consumer>,
    retired: Vec<Retired>,
}

/// A multi-consumer streaming sink over one transport.
pub struct StreamSink {
    registry: Arc<Mutex<Registry>>,
    stopper: Arc<dyn Fn() + Send + Sync>,
    acceptor_thread: Option<JoinHandle<()>>,
    watchdog_stop: Arc<AtomicBool>,
    watchdog_thread: Option<JoinHandle<()>>,
    disconnect_timeout: Duration,
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

        let watchdog_stop = Arc::new(AtomicBool::new(false));
        let watchdog_registry = Arc::clone(&registry);
        let watchdog_flag = Arc::clone(&watchdog_stop);
        let interval = watchdog_interval(disconnect_timeout);
        let watchdog_thread = std::thread::Builder::new()
            .name("fragcap-stream-watchdog".to_string())
            .spawn(move || {
                watchdog_loop(
                    watchdog_registry,
                    watchdog_flag,
                    disconnect_timeout,
                    interval,
                )
            })
            .expect("spawn watchdog thread");

        StreamSink {
            registry,
            stopper,
            acceptor_thread: Some(acceptor_thread),
            watchdog_stop,
            watchdog_thread: Some(watchdog_thread),
            disconnect_timeout,
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
        let (tx, rx) = sync_channel::<ToConsumer>(queue_depth);
        let thread = std::thread::Builder::new()
            .name("fragcap-stream-consumer".to_string())
            .spawn(move || consumer_loop(encoder, rx))
            .expect("spawn consumer thread");
        registry.lock().unwrap().active.push(Consumer {
            id,
            sender: tx,
            shutdown,
            offered: 0,
            full_since: None,
            thread,
        });
    }
}

/// A consumer's writer thread: drain the queue into the encoder until the
/// capture ends (writing the trailer), a write fails, or the channel closes.
fn consumer_loop(mut encoder: Box<dyn Sink>, rx: Receiver<ToConsumer>) -> ConsumerOutcome {
    let mut written = 0u64;
    loop {
        match rx.recv() {
            Ok(ToConsumer::Packet(packet)) => match encoder.write(&packet) {
                Ok(()) => written += 1,
                Err(_) => {
                    return ConsumerOutcome {
                        written,
                        write_failed: true,
                    }
                }
            },
            Ok(ToConsumer::Finish(stats)) => {
                // Clean capture end: write the trailer and close.
                let write_failed = encoder.finish(&stats).is_err();
                return ConsumerOutcome {
                    written,
                    write_failed,
                };
            }
            Err(_) => {
                // The channel closed without a Finish (the consumer was forced,
                // or its sender was dropped). Flush what was written; no trailer.
                let _ = encoder.flush();
                return ConsumerOutcome {
                    written,
                    write_failed: false,
                };
            }
        }
    }
}

/// The watchdog thread: disconnect any consumer whose queue has stayed full
/// past the disconnect timeout, independently of whether packets are arriving.
fn watchdog_loop(
    registry: Arc<Mutex<Registry>>,
    stop: Arc<AtomicBool>,
    disconnect_timeout: Duration,
    interval: Duration,
) {
    while !stop.load(Ordering::Acquire) {
        std::thread::sleep(interval);
        let now = Instant::now();
        let mut reg = registry.lock().unwrap();
        let mut i = 0;
        while i < reg.active.len() {
            let expired = reg.active[i]
                .full_since
                .map(|since| now.duration_since(since) >= disconnect_timeout)
                .unwrap_or(false);
            if expired {
                let consumer = reg.active.remove(i);
                consumer.shutdown.shutdown();
                reg.retired
                    .push(Retired::from_consumer(consumer, DisconnectReason::Timeout));
            } else {
                i += 1;
            }
        }
    }
}

impl Sink for StreamSink {
    fn write(&mut self, packet: &CapturedPacket) -> Result<(), SinkError> {
        let mut reg = self.registry.lock().unwrap();
        let now = Instant::now();
        let mut i = 0;
        while i < reg.active.len() {
            reg.active[i].offered += 1;
            let result = reg.active[i]
                .sender
                .try_send(ToConsumer::Packet(packet.clone()));
            match result {
                Ok(()) => {
                    reg.active[i].full_since = None;
                    i += 1;
                }
                Err(TrySendError::Full(_)) => {
                    // Withheld from this consumer only. The watchdog disconnects
                    // it if it stays full past the timeout; the drop is counted
                    // as offered-minus-written in its report.
                    reg.active[i].full_since.get_or_insert(now);
                    i += 1;
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

    fn finish(self: Box<Self>, stats: &CaptureStats) -> Result<(), SinkError> {
        let this = *self;
        // Stop accepting new consumers and the watchdog, and join both.
        (this.stopper)();
        this.watchdog_stop.store(true, Ordering::Release);
        if let Some(handle) = this.acceptor_thread {
            let _ = handle.join();
        }
        if let Some(handle) = this.watchdog_thread {
            let _ = handle.join();
        }

        let stats = Arc::new(stats.clone());
        let mut reg = this.registry.lock().unwrap();
        // Offer every still-active consumer a Finish so it drains its queue and
        // writes its trailer. A consumer whose queue is full cannot take the
        // Finish; it is retired for forcing below. Retiring drops the sender, so
        // a consumer that took the Finish still sees it before the channel
        // closes.
        let active: Vec<Consumer> = std::mem::take(&mut reg.active);
        for consumer in active {
            let _ = consumer
                .sender
                .try_send(ToConsumer::Finish(Arc::clone(&stats)));
            reg.retired.push(Retired::from_consumer(
                consumer,
                DisconnectReason::CaptureEnded,
            ));
        }
        let retired: Vec<Retired> = std::mem::take(&mut reg.retired);
        drop(reg);

        // Let keeping-up consumers drain and finalize, bounded by the disconnect
        // timeout: a consumer that cannot drain within its own timeout is
        // stalled and is force-unblocked, its unwritten tail counted as dropped.
        let deadline = Instant::now() + this.disconnect_timeout;
        loop {
            if retired.iter().all(|r| r.thread.is_finished()) || Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        for retiree in &retired {
            if !retiree.thread.is_finished() {
                retiree.shutdown.shutdown();
            }
        }

        let reports: Vec<ConsumerReport> = retired.into_iter().map(Retired::into_report).collect();
        *this.reports.lock().unwrap() = reports;
        Ok(())
    }
}
