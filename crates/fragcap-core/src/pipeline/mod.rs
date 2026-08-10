// SPDX-License-Identifier: Apache-2.0

//! The capture pipeline of specification sections 8.6 and 12.4.
//!
//! This is the module that composes the seams. A [`PacketSource`] supplies
//! frames, the S03 header parser derives a flow key and a direction, a
//! [`FlowAttributor`] answers who owned the flow at the instant the packet was
//! observed, and a set of [`Sink`] values write the result. Between acquisition
//! and output sits the bounded buffer of section 12.4, which drops the oldest
//! packet rather than making the acquisition side wait.
//!
//! # Threads
//!
//! Section 8.6 puts the pipeline on three threads. This slice builds two of
//! them. The control thread owns the [`ProcessWatcher`](crate::traits::
//! ProcessWatcher), the process tree, and the filter manager, none of which
//! exist before slices S11 and S13; its seam is named at
//! [`Pipeline::new`] and deliberately left unfilled.
//!
//! Of the two that are built, the caller's own thread is the acquisition
//! thread and the output thread is spawned. Section 8.6 does not say which is
//! which, and the choice is forced by the seams: [`PacketSource`] carries no
//! `Send` bound while [`FlowAttributor`], `ProcessWatcher`, and [`Sink`] all
//! do. Moving the source to a spawned thread would mean adding `Send` to a
//! trait that [`crate::traits`] documents as intended to reach 1.0.0 unchanged,
//! and nothing here needs it.
//!
//! That deferral has an owner. Specification section 12.1 requires one capture
//! handle and one capture thread per interface, which this arrangement cannot
//! express, so slice S09 will need `PacketSource: Send` and should carry the
//! trait change with the slice that first requires it. Recorded for promotion
//! to specification section 29.
//!
//! # Accounting
//!
//! Constitution P-4 is the reason this slice exists. [`CaptureStats`] has
//! carried named drop counters since S02 with nothing to produce them; this
//! module is the producer. Three rules hold:
//!
//! - Each eviction advances `buffer_dropped` exactly once, counted inside the
//!   buffer so that an unwinding acquisition thread cannot take the count with
//!   it.
//! - Each write that did not happen advances `sink_dropped` exactly once, per
//!   sink. A [`SinkError::Full`] and a packet withheld from a retired sink are
//!   the same event as far as the counter is concerned, because section 12.4
//!   defines it as "dropped by a sink that could not accept" and a failed sink
//!   is a sink that cannot accept.
//! - The backend's own counters are relayed unaltered and folded into nothing.
//!
//! The property worth asserting is not that a counter can be non-zero but that
//! nothing escaped the accounting. For every sink:
//!
//! ```text
//! received + buffer_dropped + refusals == packets_captured
//! ```
//!
//! That identity holds under every interleaving, which is what makes it usable
//! as a test in a concurrent module. It is checked in every pipeline test
//! below, and a discard path added later with no counter fails there rather
//! than passing quietly.
//!
//! # Fidelity
//!
//! Constitution P-9. Nothing here alters a field, reorders a packet, or
//! withholds one. Drop-oldest is a declared omission counted under P-4, not an
//! exception to P-9. A packet that produced no flow key is written and marked,
//! never discarded for being unparseable.

pub(crate) mod buffer;

use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::error::{SinkError, SourceError};
use crate::interface::{InterfaceId, InterfaceRetirement, RetirementReason};
use crate::packet::{AttributionState, CapturedPacket};
use crate::parse::{HeaderParser, InterfaceAddrs};
use crate::stats::CaptureStats;
use crate::traits::{FlowAttributor, PacketSource, Sink};

use buffer::Item;

/// The section 12.4 default buffer capacity, in packets.
pub const DEFAULT_CAPACITY: usize = 65_536;

/// How long the acquisition loop waits on the source before looking at the stop
/// flag again.
///
/// Inert for a replay source, which ignores the timeout, and therefore inert
/// for every test in slice S08. It matters to S09, and it is the bound on stop
/// latency, which is why it is stated rather than hidden.
pub const DEFAULT_READ_TIMEOUT: Duration = Duration::from_millis(100);

/// What an operator can vary without changing which components are attached.
#[derive(Clone, Debug)]
pub struct PipelineConfig {
    /// Bounded buffer capacity, in packets. Never zero.
    pub capacity: usize,
    /// Passed to [`PacketSource::next_packet`]. Bounds stop latency.
    pub read_timeout: Duration,
    /// The capturing host's addresses, for section 12.6 direction
    /// determination. An empty set is legal and means every packet is rejected
    /// with `NoLocalEndpoint`, which is a configuration a test uses
    /// deliberately.
    pub addrs: InterfaceAddrs,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        PipelineConfig {
            capacity: DEFAULT_CAPACITY,
            read_timeout: DEFAULT_READ_TIMEOUT,
            addrs: InterfaceAddrs::default(),
        }
    }
}

/// A configuration a pipeline cannot be built from.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConfigError {
    /// No packet source was supplied. A run over no interfaces would exit
    /// having captured nothing, which is the silent-empty-capture failure the
    /// interface module exists to prevent, so it is refused at construction.
    NoSources,
    /// A buffer that drops everything can only be a mistake, so it is refused
    /// rather than honored.
    ZeroCapacity,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::NoSources => f.write_str("a pipeline needs at least one packet source"),
            ConfigError::ZeroCapacity => {
                f.write_str("a buffer capacity of zero would discard every packet")
            }
        }
    }
}

impl Error for ConfigError {}

/// A cooperative request to end a run.
///
/// Observed by the acquisition loop between packets, so a source already inside
/// [`PacketSource::next_packet`] finishes that call first. Stop latency is
/// therefore bounded by [`PipelineConfig::read_timeout`], which is the caller's
/// own choice.
#[derive(Clone, Debug, Default)]
pub struct StopHandle(Arc<AtomicBool>);

impl StopHandle {
    /// Request the end. Idempotent.
    pub fn stop(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    /// Whether a stop has been requested.
    pub fn is_stopped(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// Why a run stopped.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EndReason {
    /// The source reported [`SourceError::Closed`]. The ordinary ending.
    SourceClosed,
    /// A stop was requested and observed.
    Stopped,
    /// A source error that was not recoverable and was not `Closed`.
    SourceFailed(SourceError),
    /// Every attached sink returned a non-countable error and was retired.
    /// Unreachable with no sinks attached.
    AllSinksRetired,
}

impl EndReason {
    /// The same ending, said in the vocabulary a per-interface report uses.
    ///
    /// `Stopped` becomes `SourceClosed` rather than acquiring a variant of its
    /// own: from the interface's point of view a stop it observed is simply
    /// where its own stream ended, and inventing a third reason would imply
    /// fragcap knew something about the interface that it did not.
    pub fn as_retirement(&self) -> RetirementReason {
        match self {
            EndReason::SourceFailed(SourceError::DeviceLost { detail }) => {
                RetirementReason::DeviceLost {
                    detail: detail.clone(),
                }
            }
            EndReason::SourceFailed(e) => RetirementReason::Backend {
                detail: e.to_string(),
            },
            _ => RetirementReason::SourceClosed,
        }
    }
}

impl fmt::Display for EndReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EndReason::SourceClosed => f.write_str("the source was exhausted"),
            EndReason::Stopped => f.write_str("a stop was requested"),
            EndReason::SourceFailed(e) => write!(f, "the source failed: {e}"),
            EndReason::AllSinksRetired => f.write_str("every sink was retired after failing"),
        }
    }
}

/// A sink that failed in a way the pipeline recorded rather than counted.
///
/// Two distinct events produce one of these, and a caller reading
/// `sink_failures` should not assume which:
///
/// - A `write` returning an error for which [`SinkError::is_countable`] is
///   false. The sink is retired, and every subsequent packet advances
///   `sink_dropped` for it.
/// - A `flush` or `finish` returning any error. Those run once, after the last
///   write, so there is nothing left to retire and nothing further to count.
///   The output is likely incomplete, and this record is the only place that
///   says so.
///
/// A [`SinkError::Full`] from `write` produces no record at all: it is counted
/// in `sink_dropped` and the sink stays in service.
///
/// One sink can therefore appear more than once, at most once for retirement
/// and once each for a failing flush and finish.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SinkFailure {
    /// Position in the sink list, in the order the sinks were added.
    pub index: usize,
    /// The error that retired it, or that it produced while being flushed or
    /// finished.
    pub error: SinkError,
}

/// What a run produced.
///
/// Carries the statistics unconditionally, so there is no path on which a
/// caller learns the outcome without also being handed the accounting. A bare
/// `Result` would discard the counters on the error path, which is the one path
/// where constitution P-4 matters most.
#[derive(Clone, Debug)]
#[must_use = "a pipeline report carries the run's loss accounting"]
pub struct PipelineReport {
    /// The run's own final counters, and the same value every sink was finished
    /// with.
    pub stats: CaptureStats,
    /// Why the run ended.
    pub ended: EndReason,
    /// Every sink that failed, in the order the failures were observed. Empty
    /// on a run where no sink failed.
    pub sink_failures: Vec<SinkFailure>,
    /// Why each capture thread ended, one entry per interface.
    ///
    /// Never empty on a completed run: every source retires eventually, and an
    /// interface that ended for an unremarkable reason is still reported,
    /// because "it was watched and produced nothing" and "it stopped being
    /// watched" are different facts and an operator needs to tell them apart.
    ///
    /// A retirement advances no drop counter. See
    /// [`crate::interface::InterfaceRetirement`] for why that is deliberate.
    pub retirements: Vec<InterfaceRetirement>,
}

impl PipelineReport {
    /// Whether the run ended the ordinary way with no sink failure.
    ///
    /// Says nothing about drops. A clean ending can still have lost packets,
    /// and conflating the two is the mistake [`CaptureStats::lost_anything`]
    /// exists to prevent.
    pub fn is_clean(&self) -> bool {
        self.ended == EndReason::SourceClosed && self.sink_failures.is_empty()
    }

    /// The ordinary `Result` shape, for callers that want failure to propagate.
    pub fn into_result(self) -> Result<CaptureStats, PipelineError> {
        if self.is_clean() {
            Ok(self.stats)
        } else {
            Err(PipelineError {
                report: Box::new(self),
            })
        }
    }
}

/// A run that did not end cleanly, carrying the whole report so the accounting
/// survives the failure path.
///
/// The report is boxed so that the error variant of every `Result` carrying
/// this stays one pointer wide. `CaptureStats` is a large value by design, one
/// named counter per discard cause, and an unboxed error would put all of it on
/// every caller's success path too.
#[derive(Clone, Debug)]
pub struct PipelineError {
    /// The report, including the statistics.
    pub report: Box<PipelineReport>,
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the capture run ended abnormally: {}", self.report.ended)?;
        for failure in &self.report.sink_failures {
            write!(f, "; sink {} failed: {}", failure.index, failure.error)?;
        }
        Ok(())
    }
}

impl Error for PipelineError {}

/// Everything one run owns.
///
/// Built from trait objects throughout, so a new source, attributor, or sink
/// composes without this type changing. Constitution P-3: the source and the
/// attributor are held side by side and neither appears in the other's
/// signatures.
pub struct Pipeline {
    sources: Vec<SourceBinding>,
    attributor: Box<dyn FlowAttributor>,
    sinks: Vec<Box<dyn Sink>>,
    config: PipelineConfig,
    stop: StopHandle,
}

/// A packet source together with the interface identity its packets carry.
///
/// The pair rather than the source alone, because [`PacketSource`] answers what
/// it produces and not where it produces it from, and the pipeline is what
/// attaches the identity at the lift from `RawPacket` to `CapturedPacket`.
///
/// The link type is deliberately not carried here. [`PacketSource::link_type`]
/// already answers it per source, and a second copy would be a second answer
/// that could disagree with the first.
pub struct SourceBinding {
    pub id: InterfaceId,
    pub source: Box<dyn PacketSource>,
}

impl SourceBinding {
    pub fn new(id: InterfaceId, source: Box<dyn PacketSource>) -> Self {
        SourceBinding { id, source }
    }
}

impl fmt::Debug for SourceBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceBinding")
            .field("id", &self.id)
            .finish()
    }
}

impl fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pipeline")
            .field("sources", &self.sources.len())
            .field("sinks", &self.sinks.len())
            .field("config", &self.config)
            .field("stopped", &self.stop.is_stopped())
            .finish()
    }
}

impl Pipeline {
    /// Build a pipeline. Starts no thread, opens no file, and reads no packet.
    ///
    /// # The control thread seam
    ///
    /// Specification section 8.6 has a control thread owning the process
    /// watcher, the process tree, and the filter manager, and publishing an
    /// attribution snapshot that the acquisition thread reads without blocking.
    /// None of those exist before slices S11 and S13, and building the
    /// publication mechanism now would fix the snapshot's shape before S10
    /// knows what a socket table snapshot costs to publish.
    ///
    /// So the attributor is owned outright by the acquisition side for now.
    /// [`FlowAttributor`] is `Send`, which is exactly the bound that permits
    /// it, and no `Sync` bound is implied or required. When the control thread
    /// arrives, what changes is where the attributor lives, not what this
    /// pipeline does with the answers.
    /// # Several sources
    ///
    /// Specification section 12.1 captures each interface on its own handle and
    /// its own thread, all feeding one bounded buffer. So this takes a
    /// collection and [`Pipeline::run`] spawns a thread per entry.
    ///
    /// It deliberately does not accept a multiplexing source that fans several
    /// sources into one. That arrangement leaves this constructor untouched and
    /// is wrong twice: it needs its own fan-in buffer where section 12.4
    /// specifies exactly one, and `next_packet` yields a `RawPacket` carrying no
    /// interface identity, so the multiplexer would have to invent a side
    /// channel for the very thing this pipeline attaches one line later.
    pub fn new(
        sources: Vec<SourceBinding>,
        attributor: Box<dyn FlowAttributor>,
        config: PipelineConfig,
    ) -> Result<Self, ConfigError> {
        if config.capacity == 0 {
            return Err(ConfigError::ZeroCapacity);
        }
        if sources.is_empty() {
            return Err(ConfigError::NoSources);
        }
        Ok(Pipeline {
            sources,
            attributor,
            sinks: Vec::new(),
            config,
            stop: StopHandle::default(),
        })
    }

    /// Attach a sink. Its index in the report is the order it was added.
    pub fn add_sink(&mut self, sink: Box<dyn Sink>) {
        self.sinks.push(sink);
    }

    /// A handle that ends the run. Valid for the whole run, and obtainable
    /// before [`Pipeline::run`] consumes the pipeline.
    pub fn stop_handle(&self) -> StopHandle {
        self.stop.clone()
    }

    /// How many sinks are attached.
    pub fn sink_count(&self) -> usize {
        self.sinks.len()
    }

    /// Run to an ending, on this thread.
    ///
    /// Acquires on the calling thread and spawns the output thread; see the
    /// module documentation for why that way round. Blocks until the run ends.
    ///
    /// # Panics
    ///
    /// Does not catch a panic from a source, an attributor, or a sink. If the
    /// acquisition side panics, the buffer's producer is dropped during
    /// unwinding, the output thread observes the ending, drains, flushes, and
    /// finishes every sink, and a join guard waits for it before the panic
    /// escapes. A panic is never converted into an [`EndReason`]: it is a
    /// defect, and filing it under an accounting category would describe a
    /// program that was not running correctly as though it were.
    pub fn run(self) -> PipelineReport {
        let Pipeline {
            sources,
            attributor,
            sinks,
            config,
            stop,
        } = self;

        let (tx, rx) = buffer::channel(config.capacity);
        let output_stop = stop.clone();
        let handle = std::thread::spawn(move || {
            // Ends the acquisition loop however this thread terminates,
            // including an unwinding panic from a sink. Without it a panicking
            // sink would leave the calling thread in `acquire` until the source
            // closed on its own, which a live capture never does, so `run`
            // would never reach the join that re-raises the panic. On the
            // ordinary path this is a no-op: the output thread only returns
            // after acquisition has already ended and sent its terminal item.
            let _end_acquisition = StopOnDrop(output_stop.clone());
            output_loop(rx, sinks, output_stop)
        });
        // The guard owns the producer as well as the join handle, so that
        // closing the buffer always precedes joining the thread that is
        // draining it. Holding them separately would deadlock on the panic
        // path: locals drop in reverse declaration order, so the guard would
        // join a thread still waiting on a buffer the producer had not yet
        // closed. This is the arrangement that makes "the sinks are finished
        // before the panic reaches the caller" true with no panic-specific
        // code path.
        let mut guard = OutputThread {
            tx: Some(Arc::new(tx)),
            handle: Some(handle),
        };

        // Shared because every capture thread asks the same attributor, and
        // `FlowAttributor` is `Send` without being `Sync`, so a shared reference
        // is not enough on its own.
        //
        // A mutex on the per-packet path is not the destination. Specification
        // section 8.6 has a control thread owning the attributor and publishing
        // a snapshot the capture threads read without blocking, which is the
        // arrangement that removes this lock. That thread arrives with S11 and
        // S13. Taking it now would fix the snapshot's shape before S10 knows
        // what a socket table snapshot costs to publish, and adding `Sync` to
        // the trait instead would be a fourth deviation against section 8.5 to
        // buy something the control thread makes moot.
        let attributor = Arc::new(Mutex::new(attributor));

        let mut threads = Vec::with_capacity(sources.len());
        for SourceBinding { id, mut source } in sources {
            let attributor = Arc::clone(&attributor);
            let tx = Arc::clone(guard.producer());
            let stop = stop.clone();
            let addrs = config.addrs.clone();
            let read_timeout = config.read_timeout;
            threads.push(std::thread::spawn(move || {
                let mut parser = HeaderParser::new(addrs);
                let mut stats = CaptureStats::default();
                let link = source.link_type();
                let ended = acquire(
                    source.as_mut(),
                    &attributor,
                    &mut parser,
                    &tx,
                    &stop,
                    &mut stats,
                    id,
                    link,
                    read_timeout,
                );
                stats.parse = *parser.stats();
                stats.set_source(id, source.stats());
                AcquisitionOutcome { id, stats, ended }
            }));
        }

        let mut merged = CaptureStats::default();
        let mut retirements = Vec::new();
        let mut endings = Vec::new();
        for thread in threads {
            // A capture thread that panicked is a defect, and resuming here
            // would abandon the other threads and the output side mid-drain. It
            // is carried after every thread has been joined, below.
            match thread.join() {
                Ok(outcome) => {
                    retirements.push(InterfaceRetirement {
                        interface: outcome.id,
                        reason: outcome.ended.as_retirement(),
                    });
                    endings.push(outcome.ended);
                    merged.absorb(outcome.stats);
                }
                Err(payload) => std::panic::resume_unwind(payload),
            }
        }

        let ended = combine_endings(endings);
        let output = guard.finish(merged);
        PipelineReport {
            stats: output.stats,
            // Retirement outranks the stop it requested, because the stop was a
            // consequence rather than a cause. It does not outrank an ending
            // acquisition reached on its own: a source that closed, or one that
            // failed with a `DeviceLost` worth diagnosing, ended the run before
            // the output side finished draining and any later retirement is a
            // second fact rather than the reason. Those failures are still
            // named in `sink_failures` either way.
            ended: if output.all_retired && ended == EndReason::Stopped {
                EndReason::AllSinksRetired
            } else {
                ended
            },
            sink_failures: output.failures,
            retirements,
        }
    }
}

/// What one capture thread produced.
struct AcquisitionOutcome {
    id: InterfaceId,
    stats: CaptureStats,
    ended: EndReason,
}

/// The run's ending, from every capture thread's.
///
/// A failure outranks an ordinary close, because a run in which one interface
/// disappeared did not end cleanly even though the others did. A close outranks
/// a stop, because a source that ran out said more about why the run ended than
/// the stop that followed. With one source this reduces to that source's own
/// ending, which is what every pre-S09 test asserts.
fn combine_endings(endings: Vec<EndReason>) -> EndReason {
    if let Some(failed) = endings
        .iter()
        .find(|e| matches!(e, EndReason::SourceFailed(_)))
    {
        return failed.clone();
    }
    if endings.iter().all(|e| *e == EndReason::SourceClosed) && !endings.is_empty() {
        return EndReason::SourceClosed;
    }
    if endings.contains(&EndReason::SourceClosed)
        && endings
            .iter()
            .all(|e| matches!(e, EndReason::SourceClosed | EndReason::Stopped))
    {
        return EndReason::SourceClosed;
    }
    EndReason::Stopped
}

/// Requests a stop when dropped, however the holder terminated.
struct StopOnDrop(StopHandle);

impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.stop();
    }
}

/// The acquisition loop. Returns why it stopped.
///
/// Deliberately a free function taking each piece it needs, so the borrow of
/// the source does not extend over the whole of [`Pipeline::run`].
#[allow(clippy::too_many_arguments)]
fn acquire(
    source: &mut dyn PacketSource,
    attributor: &Mutex<Box<dyn FlowAttributor>>,
    parser: &mut HeaderParser,
    tx: &buffer::Producer,
    stop: &StopHandle,
    stats: &mut CaptureStats,
    interface: InterfaceId,
    link: crate::link::LinkType,
    timeout: Duration,
) -> EndReason {
    loop {
        if stop.is_stopped() {
            return EndReason::Stopped;
        }
        let raw = match source.next_packet(timeout) {
            Ok(Some(raw)) => raw,
            // A timeout the backend chose to report as success. Nothing
            // arrived, nothing was lost, and nothing is counted.
            Ok(None) => continue,
            Err(SourceError::Closed) => return EndReason::SourceClosed,
            Err(e) if e.is_recoverable() => continue,
            Err(e) => return EndReason::SourceFailed(e),
        };

        let mut packet = CapturedPacket::from_raw(raw, interface);
        stats.packets_captured = stats.packets_captured.saturating_add(1);
        parser.apply(link, &mut packet);
        if let Some(key) = packet.flow.as_ref() {
            // The packet's own instant, not the present one. Specification
            // section 11.4: capture and socket table observation are not
            // synchronized, so the question is who owned this flow then.
            let attributor = attributor
                .lock()
                .expect("the attributor mutex is never poisoned");
            packet.attribution = attributor.resolve(key, packet.ts);
        }
        match packet.attribution_state() {
            AttributionState::Resolved => {
                stats.packets_attributed = stats.packets_attributed.saturating_add(1);
            }
            AttributionState::Unresolved => {
                stats.packets_unattributed = stats.packets_unattributed.saturating_add(1);
            }
            // No flow key, so attribution was never attempted, which is not the
            // same as attempted and failed. Neither counter moves.
            AttributionState::NotAttempted => {}
        }

        // The only interaction with the output side. Never waits for a sink to
        // make progress; see the buffer's module documentation for the exact
        // claim and why it is stated that way.
        tx.push(Item::Packet(Box::new(packet)));
    }
}

/// What the output thread produced.
struct OutputOutcome {
    stats: CaptureStats,
    failures: Vec<SinkFailure>,
    all_retired: bool,
}

/// Owns the producer and the output thread together, so that on every path out
/// of [`Pipeline::run`] the buffer is closed before the thread draining it is
/// joined. Holding the two separately deadlocks while unwinding.
struct OutputThread {
    tx: Option<Arc<buffer::Producer>>,
    handle: Option<JoinHandle<OutputOutcome>>,
}

impl OutputThread {
    /// The shared producer. Each capture thread holds a clone, so the buffer
    /// closes only once the last of them and this guard have let go, which is
    /// the same "closed when the producer is gone" rule the buffer has always
    /// had, now counted across several holders.
    fn producer(&self) -> &Arc<buffer::Producer> {
        self.tx
            .as_ref()
            .expect("the producer is taken only when the run ends")
    }

    /// Send the terminal item, close the buffer, and join.
    fn finish(&mut self, stats: CaptureStats) -> OutputOutcome {
        if let Some(tx) = self.tx.take() {
            tx.push(Item::End(Box::new(stats)));
            // Explicit, because the ordering is the whole point: the consumer
            // must see the terminal item and then the close, in that order.
            drop(tx);
        }
        match self.handle.take() {
            Some(handle) => match handle.join() {
                Ok(outcome) => outcome,
                // The output thread panicked, which means a sink did. Carry the
                // panic to the caller rather than reporting it: a panic is a
                // defect, and an EndReason would file it as accounting.
                Err(payload) => std::panic::resume_unwind(payload),
            },
            None => unreachable!("the output thread is joined exactly once"),
        }
    }
}

impl Drop for OutputThread {
    fn drop(&mut self) {
        // Close first. Dropping the producer sets the buffer closed and wakes
        // the consumer, which is what lets an unwinding acquisition thread be
        // observed as an ending rather than as an absent terminal item.
        drop(self.tx.take());
        if let Some(handle) = self.handle.take() {
            // Already unwinding, or the caller dropped the guard without
            // finishing. Either way, wait for the sinks to be flushed and
            // finished before the panic escapes. A panic from the output thread
            // is discarded here, because resuming one during an unwind aborts.
            let _ = handle.join();
        }
    }
}

/// The output loop: drain, fan out, then flush and finish.
fn output_loop(
    rx: buffer::Consumer,
    mut sinks: Vec<Box<dyn Sink>>,
    stop: StopHandle,
) -> OutputOutcome {
    let mut failures: Vec<SinkFailure> = Vec::new();
    let mut retired = vec![false; sinks.len()];
    let mut sink_dropped: u64 = 0;
    let mut all_retired = false;
    // Default rather than an option: an acquisition thread that panicked sends
    // no terminal item, and the output side still has to report what it
    // counted.
    let mut stats = CaptureStats::default();

    while let Some(item) = rx.next() {
        let packet = match item {
            Item::Packet(packet) => packet,
            Item::End(final_stats) => {
                stats = *final_stats;
                break;
            }
        };
        for (index, sink) in sinks.iter_mut().enumerate() {
            if retired[index] {
                // A retired sink cannot accept, which is exactly what section
                // 12.4 defines this counter as. Counting it here is what makes
                // retirement conserve.
                sink_dropped = sink_dropped.saturating_add(1);
                continue;
            }
            match sink.write(&packet) {
                Ok(()) => {}
                Err(e) if e.is_countable() => {
                    sink_dropped = sink_dropped.saturating_add(1);
                }
                Err(e) => {
                    failures.push(SinkFailure { index, error: e });
                    retired[index] = true;
                    sink_dropped = sink_dropped.saturating_add(1);
                }
            }
        }
        if !sinks.is_empty() && retired.iter().all(|r| *r) && !all_retired {
            all_retired = true;
            // Wind the acquisition side down, then keep draining and counting
            // so that nothing leaves the buffer uncounted.
            stop.stop();
        }
    }

    stats.buffer_dropped = rx.evicted();
    stats.sink_dropped = sink_dropped;

    for (index, sink) in sinks.iter_mut().enumerate() {
        if let Err(error) = sink.flush() {
            failures.push(SinkFailure { index, error });
        }
    }
    for (index, sink) in sinks.into_iter().enumerate() {
        if let Err(error) = sink.finish(&stats) {
            failures.push(SinkFailure { index, error });
        }
    }

    OutputOutcome {
        stats,
        failures,
        all_retired,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribution::{Attribution, Fidelity};
    use crate::filter::FilterProgram;
    use crate::flow::{Endpoint, FlowKey, Proto};
    use crate::link::LinkType;
    use crate::packet::{Payload, RawPacket, Timestamp};
    use crate::stats::SourceStats;
    use std::collections::HashMap;
    use std::net::IpAddr;
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::sync::Mutex;

    // ------------------------------------------------------------------
    // Frames. Real enough to parse, small enough to read.
    // ------------------------------------------------------------------

    /// An Ethernet frame carrying IPv4 and UDP, from 192.0.2.10 to
    /// 198.51.100.5, with `marker` as its single payload byte so a packet is
    /// identifiable after crossing the buffer.
    fn frame(marker: u8, src_port: u16) -> Vec<u8> {
        use crate::parse::testframe as build;
        let udp = build::cat(&[&build::udp(src_port, 5055, 9), &[marker]]);
        let ip = build::ipv4(
            build::V4 {
                proto: 17,
                ..build::V4::default()
            },
            &udp,
        );
        build::ethernet(0x0800, &ip)
    }

    /// A frame that parses to nothing: an Ethernet type neither IPv4 nor IPv6.
    fn unparseable(marker: u8) -> Vec<u8> {
        use crate::parse::testframe as build;
        build::ethernet(0x88CC, &[marker])
    }

    fn raw(bytes: Vec<u8>) -> RawPacket {
        let len = bytes.len() as u32;
        RawPacket::new(
            Timestamp::from_nanos(1),
            Payload::copy_from_slice(&bytes),
            len,
        )
    }

    fn local_addrs() -> InterfaceAddrs {
        InterfaceAddrs::new([("192.0.2.10").parse::<IpAddr>().expect("parses")])
    }

    /// The marker byte a frame was built with, read back out of a written
    /// packet. Both frame builders put it last.
    fn marker_of(packet: &CapturedPacket) -> u8 {
        *packet.data.last().expect("every test frame has a payload")
    }

    // ------------------------------------------------------------------
    // Stubs. None of these is a product type.
    // ------------------------------------------------------------------

    /// How a scripted source ends once its packets run out.
    #[derive(Clone, Debug)]
    enum Ending {
        Closed,
        Failed(SourceError),
        /// This many recoverable timeouts, then `Closed`.
        TimeoutsThenClosed(usize),
    }

    /// T015. A source yielding a scripted sequence and then a chosen ending.
    struct StubSource {
        queued: std::collections::VecDeque<RawPacket>,
        ending: Ending,
        remaining_timeouts: usize,
        delivered: u64,
        stats: SourceStats,
        /// Fires once the source has nothing left. Used to release a gated
        /// sink at a moment the test can name: "acquisition is done".
        exhausted: Option<Sender<()>>,
    }

    impl StubSource {
        fn new(frames: Vec<Vec<u8>>) -> Self {
            StubSource {
                queued: frames.into_iter().map(raw).collect(),
                ending: Ending::Closed,
                remaining_timeouts: 0,
                delivered: 0,
                stats: SourceStats::default(),
                exhausted: None,
            }
        }

        fn ending(mut self, ending: Ending) -> Self {
            if let Ending::TimeoutsThenClosed(n) = ending {
                self.remaining_timeouts = n;
            }
            self.ending = ending;
            self
        }

        fn backend_stats(mut self, stats: SourceStats) -> Self {
            self.stats = stats;
            self
        }

        /// Signal on this channel the moment the source runs dry.
        ///
        /// This is what makes the eviction tests deterministic without a sleep.
        /// The gated sink waits on the other end, so "the sink is still held"
        /// and "acquisition finished" are ordered by construction rather than
        /// by scheduling luck.
        fn signal_on_exhaustion(mut self, tx: Sender<()>) -> Self {
            self.exhausted = Some(tx);
            self
        }
    }

    impl PacketSource for StubSource {
        fn next_packet(&mut self, _timeout: Duration) -> Result<Option<RawPacket>, SourceError> {
            if let Some(packet) = self.queued.pop_front() {
                self.delivered = self.delivered.saturating_add(1);
                return Ok(Some(packet));
            }
            if let Some(tx) = self.exhausted.take() {
                let _ = tx.send(());
            }
            match &self.ending {
                Ending::Closed => Err(SourceError::Closed),
                Ending::Failed(e) => Err(e.clone()),
                Ending::TimeoutsThenClosed(_) => {
                    if self.remaining_timeouts > 0 {
                        self.remaining_timeouts -= 1;
                        Err(SourceError::Timeout)
                    } else {
                        Err(SourceError::Closed)
                    }
                }
            }
        }

        fn set_filter(&mut self, _filter: &FilterProgram) -> Result<(), SourceError> {
            Ok(())
        }

        fn stats(&self) -> SourceStats {
            SourceStats {
                received: self.delivered,
                ..self.stats
            }
        }

        fn link_type(&self) -> LinkType {
            LinkType::ETHERNET
        }
    }

    /// T021. Panics at a chosen index, so the panic path is reachable.
    struct PanickingSource {
        queued: std::collections::VecDeque<RawPacket>,
        panic_at: usize,
        served: usize,
    }

    impl PacketSource for PanickingSource {
        fn next_packet(&mut self, _timeout: Duration) -> Result<Option<RawPacket>, SourceError> {
            if self.served == self.panic_at {
                panic!("the acquisition side failed");
            }
            self.served += 1;
            match self.queued.pop_front() {
                Some(packet) => Ok(Some(packet)),
                None => Err(SourceError::Closed),
            }
        }
        fn set_filter(&mut self, _filter: &FilterProgram) -> Result<(), SourceError> {
            Ok(())
        }
        fn stats(&self) -> SourceStats {
            SourceStats::default()
        }
        fn link_type(&self) -> LinkType {
            LinkType::ETHERNET
        }
    }

    /// T016. Answers from a map keyed by flow, so one run can produce resolved,
    /// unresolved, and never-attempted packets.
    struct StubAttributor {
        answers: HashMap<FlowKey, Attribution>,
    }

    impl StubAttributor {
        fn empty() -> Self {
            StubAttributor {
                answers: HashMap::new(),
            }
        }

        /// Resolves every flow the test frames produce.
        fn resolving() -> Self {
            let mut answers = HashMap::new();
            for port in 40000u16..40016 {
                answers.insert(
                    flow_key(port),
                    Attribution::new(4242, "game.exe", Fidelity::Live),
                );
            }
            StubAttributor { answers }
        }
    }

    fn flow_key(src_port: u16) -> FlowKey {
        FlowKey::new(
            Proto::Udp,
            format!("192.0.2.10:{src_port}").parse().expect("parses"),
            "198.51.100.5:5055".parse().expect("parses"),
        )
    }

    impl FlowAttributor for StubAttributor {
        fn resolve(&self, key: &FlowKey, _at: Timestamp) -> Option<Attribution> {
            self.answers.get(key).cloned()
        }
        fn refresh(&mut self) -> Result<(), crate::error::AttrError> {
            Ok(())
        }
        fn active_endpoints(&self) -> Vec<Endpoint> {
            Vec::new()
        }
    }

    /// What a sink recorded, readable after the sink was consumed by `finish`.
    #[derive(Debug, Default)]
    struct SinkLog {
        written: Vec<CapturedPacket>,
        flushes: usize,
        finishes: usize,
        finished_with: Option<CaptureStats>,
        flushed_before_finish: bool,
    }

    type Log = Arc<Mutex<SinkLog>>;

    fn log() -> Log {
        Arc::new(Mutex::new(SinkLog::default()))
    }

    /// How a stub sink behaves on a given write.
    #[derive(Clone, Debug, Default)]
    struct SinkScript {
        /// Packet indices, zero based, refused with `Full`.
        refuse: Vec<usize>,
        /// Packet index at which a non-countable error is returned.
        fail_at: Option<usize>,
        /// Fail in `flush`.
        fail_flush: bool,
        /// Fail in `finish`.
        fail_finish: bool,
    }

    /// T017 through T020, in one type. A single configurable sink beats four
    /// nearly identical ones, and every test states which behavior it wants.
    struct StubSink {
        log: Log,
        script: SinkScript,
        seen: usize,
        /// T020. Blocks in `write` until the test sends on this channel.
        gate: Option<Receiver<()>>,
    }

    impl StubSink {
        fn recording(log: &Log) -> Box<dyn Sink> {
            Box::new(StubSink {
                log: Arc::clone(log),
                script: SinkScript::default(),
                seen: 0,
                gate: None,
            })
        }

        fn scripted(log: &Log, script: SinkScript) -> Box<dyn Sink> {
            Box::new(StubSink {
                log: Arc::clone(log),
                script,
                seen: 0,
                gate: None,
            })
        }

        /// A sink that blocks on its first write until the paired sender
        /// fires. Give the sender to [`StubSource::signal_on_exhaustion`] and
        /// the sink cannot move until acquisition has finished.
        fn gated(log: &Log) -> (Box<dyn Sink>, Sender<()>) {
            Self::gated_scripted(log, SinkScript::default())
        }

        /// A gated sink that also follows a script once released.
        ///
        /// The combination is what orders a sink failure strictly after
        /// acquisition has ended. Without the gate the two race, and the test
        /// asserting on the end reason passes or fails depending on which
        /// thread won.
        fn gated_scripted(log: &Log, script: SinkScript) -> (Box<dyn Sink>, Sender<()>) {
            let (tx, rx) = mpsc::channel();
            (
                Box::new(StubSink {
                    log: Arc::clone(log),
                    script,
                    seen: 0,
                    gate: Some(rx),
                }),
                tx,
            )
        }
    }

    impl Sink for StubSink {
        fn write(&mut self, packet: &CapturedPacket) -> Result<(), SinkError> {
            if let Some(gate) = self.gate.take() {
                // Blocks until the test decides to let the output side move,
                // which is what makes the eviction deterministic without a
                // sleep. Research decision R-7.
                let _ = gate.recv();
            }
            let index = self.seen;
            self.seen += 1;
            if self.script.refuse.contains(&index) {
                return Err(SinkError::Full);
            }
            if self.script.fail_at == Some(index) {
                return Err(SinkError::Write {
                    detail: "scripted failure".into(),
                });
            }
            self.log
                .lock()
                .expect("the log mutex is never poisoned")
                .written
                .push(packet.clone());
            Ok(())
        }

        fn flush(&mut self) -> Result<(), SinkError> {
            let mut log = self.log.lock().expect("the log mutex is never poisoned");
            log.flushes += 1;
            if self.script.fail_flush {
                return Err(SinkError::Write {
                    detail: "scripted flush failure".into(),
                });
            }
            Ok(())
        }

        fn finish(self: Box<Self>, stats: &CaptureStats) -> Result<(), SinkError> {
            let mut log = self.log.lock().expect("the log mutex is never poisoned");
            log.flushed_before_finish = log.flushes > log.finishes;
            log.finishes += 1;
            log.finished_with = Some(stats.clone());
            if self.script.fail_finish {
                return Err(SinkError::Closed);
            }
            Ok(())
        }
    }

    /// A sink that panics on a chosen write.
    struct PanickingSink {
        panic_at: usize,
        seen: usize,
    }

    impl Sink for PanickingSink {
        fn write(&mut self, _packet: &CapturedPacket) -> Result<(), SinkError> {
            if self.seen == self.panic_at {
                panic!("the sink failed");
            }
            self.seen += 1;
            Ok(())
        }
        fn flush(&mut self) -> Result<(), SinkError> {
            Ok(())
        }
        fn finish(self: Box<Self>, _stats: &CaptureStats) -> Result<(), SinkError> {
            Ok(())
        }
    }

    // ------------------------------------------------------------------
    // Harness
    // ------------------------------------------------------------------

    fn pipeline(source: Box<dyn PacketSource>, capacity: usize) -> Pipeline {
        Pipeline::new(
            vec![SourceBinding::new(InterfaceId::default(), source)],
            Box::new(StubAttributor::resolving()),
            PipelineConfig {
                capacity,
                addrs: local_addrs(),
                ..PipelineConfig::default()
            },
        )
        .expect("a non-zero capacity builds")
    }

    /// Sixteen parseable frames, each identifiable and each on its own port so
    /// the attributor map covers them.
    fn frames(n: u8) -> Vec<Vec<u8>> {
        (0..n).map(|i| frame(i, 40000 + i as u16)).collect()
    }

    /// T043. The conservation identity, checked for one sink.
    ///
    /// This is the assertion the slice exists for, and it is what constitution
    /// P-4 actually requires: not that a counter can be non-zero, but that
    /// nothing escaped the accounting. It holds under every interleaving, which
    /// is what makes it usable in a concurrent module. Every test below calls
    /// it. A discard path added later with no counter fails here.
    fn assert_conserved(report: &PipelineReport, log: &Log, refusals: u64) {
        let received = log
            .lock()
            .expect("the log mutex is never poisoned")
            .written
            .len() as u64;
        assert_eq!(
            received + report.stats.buffer_dropped + refusals,
            report.stats.packets_captured,
            "conservation failed: {received} written, {} evicted, {refusals} refused, \
             {} captured",
            report.stats.buffer_dropped,
            report.stats.packets_captured
        );
    }

    fn written(log: &Log) -> Vec<u8> {
        log.lock()
            .expect("the log mutex is never poisoned")
            .written
            .iter()
            .map(marker_of)
            .collect()
    }

    // ------------------------------------------------------------------
    // Phase 4: composition
    // ------------------------------------------------------------------

    // T030. FR-011.
    #[test]
    fn a_zero_capacity_is_refused_at_construction() {
        let err = Pipeline::new(
            vec![SourceBinding::new(
                InterfaceId::default(),
                Box::new(StubSource::new(Vec::new())),
            )],
            Box::new(StubAttributor::empty()),
            PipelineConfig {
                capacity: 0,
                ..PipelineConfig::default()
            },
        )
        .expect_err("a buffer that drops everything cannot be built");
        assert_eq!(err, ConfigError::ZeroCapacity);
        assert!(err.to_string().contains("zero"));
    }

    // T030. Construction is inert.
    #[test]
    fn construction_reads_no_packet_and_starts_no_run() {
        let source = StubSource::new(frames(4));
        let p = pipeline(Box::new(source), 16);
        assert_eq!(p.sink_count(), 0);
        assert!(!p.stop_handle().is_stopped());
        // Dropping without running must not hang or panic.
        drop(p);
    }

    // T031. FR-014, FR-038.
    #[test]
    fn every_packet_reaches_the_sink_in_source_order() {
        let log = log();
        let mut p = pipeline(Box::new(StubSource::new(frames(8))), 64);
        p.add_sink(StubSink::recording(&log));
        let report = p.run();

        assert_eq!(written(&log), vec![0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(report.stats.packets_captured, 8);
        assert_eq!(report.ended, EndReason::SourceClosed);
        assert!(report.is_clean());
        assert_conserved(&report, &log, 0);
    }

    // T032, T034. FR-025, US6.
    #[test]
    fn several_sinks_each_receive_every_packet() {
        let a = log();
        let b = log();
        // Built entirely through the seams: nothing here names a concrete
        // source, attributor, or sink type at the pipeline boundary.
        let source: Box<dyn PacketSource> = Box::new(StubSource::new(frames(5)));
        let attributor: Box<dyn FlowAttributor> = Box::new(StubAttributor::resolving());
        let mut p = Pipeline::new(
            vec![SourceBinding::new(InterfaceId::default(), source)],
            attributor,
            PipelineConfig {
                addrs: local_addrs(),
                ..PipelineConfig::default()
            },
        )
        .expect("the default capacity builds");
        p.add_sink(StubSink::recording(&a));
        p.add_sink(StubSink::recording(&b));
        let report = p.run();

        assert_eq!(written(&a), vec![0, 1, 2, 3, 4]);
        assert_eq!(written(&b), vec![0, 1, 2, 3, 4]);
        assert_conserved(&report, &a, 0);
        assert_conserved(&report, &b, 0);
    }

    // T033. The zero-sink case: the operator's declared scope, not a loss.
    #[test]
    fn a_run_with_no_sinks_completes_and_counts_nothing_as_a_sink_drop() {
        let p = pipeline(Box::new(StubSource::new(frames(6))), 64);
        let report = p.run();
        assert_eq!(report.stats.packets_captured, 6);
        assert_eq!(report.stats.sink_dropped, 0, "no sink refused anything");
        assert_eq!(report.stats.buffer_dropped, 0);
        assert_eq!(report.ended, EndReason::SourceClosed);
        assert!(
            !report.stats.lost_anything(),
            "not writing is scope, not loss"
        );
    }

    // ------------------------------------------------------------------
    // Phase 5: accounting
    // ------------------------------------------------------------------

    // T040, SC-002. Deterministic without a sleep: the sink cannot move until
    // the source runs dry, so eviction is forced by construction.
    #[test]
    fn a_held_sink_and_a_small_buffer_drive_buffer_dropped_above_zero() {
        let log = log();
        let (sink, release) = StubSink::gated(&log);
        let source = StubSource::new(frames(32)).signal_on_exhaustion(release);
        let mut p = pipeline(Box::new(source), 2);
        p.add_sink(sink);
        let report = p.run();

        assert_eq!(report.stats.packets_captured, 32);
        assert!(
            report.stats.buffer_dropped > 0,
            "a two-packet buffer and a held sink must have evicted"
        );
        assert_conserved(&report, &log, 0);
    }

    // T041, SC-003. An exact count, not merely a non-zero one.
    #[test]
    fn a_refusing_sink_drives_sink_dropped_to_an_exact_value() {
        let log = log();
        let mut p = pipeline(Box::new(StubSource::new(frames(10))), 64);
        p.add_sink(StubSink::scripted(
            &log,
            SinkScript {
                refuse: vec![1, 4, 7],
                ..SinkScript::default()
            },
        ));
        let report = p.run();

        assert_eq!(report.stats.sink_dropped, 3);
        assert_eq!(written(&log), vec![0, 2, 3, 5, 6, 8, 9]);
        assert_conserved(&report, &log, 3);
    }

    // T042. FR-017. Per refusal, not per packet.
    #[test]
    fn three_sinks_refusing_one_packet_advance_the_counter_three_times() {
        let a = log();
        let b = log();
        let c = log();
        let script = SinkScript {
            refuse: vec![2],
            ..SinkScript::default()
        };
        let mut p = pipeline(Box::new(StubSource::new(frames(5))), 64);
        p.add_sink(StubSink::scripted(&a, script.clone()));
        p.add_sink(StubSink::scripted(&b, script.clone()));
        p.add_sink(StubSink::scripted(&c, script));
        let report = p.run();

        assert_eq!(
            report.stats.sink_dropped, 3,
            "one packet, three outputs short, three losses"
        );
        for l in [&a, &b, &c] {
            assert_conserved(&report, l, 1);
        }
    }

    // T044. FR-018.
    #[test]
    fn backend_counters_are_relayed_and_folded_into_nothing() {
        let log = log();
        let source = StubSource::new(frames(4)).backend_stats(SourceStats {
            received: 0,
            kernel_dropped: 11,
            interface_dropped: 3,
        });
        let mut p = pipeline(Box::new(source), 64);
        p.add_sink(StubSink::recording(&log));
        let report = p.run();

        assert_eq!(report.stats.source().kernel_dropped, 11);
        assert_eq!(report.stats.source().interface_dropped, 3);
        assert_eq!(
            report.stats.fragcap_dropped(),
            0,
            "the backend's losses are not fragcap's"
        );
        assert_eq!(report.stats.total_dropped(), 14);
        assert_conserved(&report, &log, 0);
    }

    // T045.
    #[test]
    fn a_clean_run_reports_zero_in_every_drop_counter() {
        let log = log();
        let mut p = pipeline(Box::new(StubSource::new(frames(6))), 64);
        p.add_sink(StubSink::recording(&log));
        let report = p.run();

        assert_eq!(report.stats.buffer_dropped, 0);
        assert_eq!(report.stats.sink_dropped, 0);
        assert!(!report.stats.lost_anything());
        assert_conserved(&report, &log, 0);
    }

    // T046. FR-023. The counters the writers had been handed by hand.
    #[test]
    fn every_sink_is_finished_with_the_runs_own_final_statistics() {
        let a = log();
        let b = log();
        let mut p = pipeline(Box::new(StubSource::new(frames(9))), 2);
        p.add_sink(StubSink::scripted(
            &a,
            SinkScript {
                refuse: vec![0],
                ..SinkScript::default()
            },
        ));
        p.add_sink(StubSink::recording(&b));
        let report = p.run();

        for l in [&a, &b] {
            let seen = l
                .lock()
                .expect("the log mutex is never poisoned")
                .finished_with
                .clone()
                .expect("every sink is finished");
            assert_eq!(
                seen, report.stats,
                "the value a sink writes into its trailer must be the run's own"
            );
            // Non-default, which is what makes the assertion mean something:
            // an implementation that passed CaptureStats::default() would have
            // satisfied a weaker check.
            assert_eq!(seen.packets_captured, 9);
            assert_eq!(seen.sink_dropped, 1);
        }
    }

    // T047. FR-020, FR-039. Retained and marked, and counted nowhere else.
    #[test]
    fn a_packet_with_no_flow_key_is_written_and_advances_neither_counter() {
        let log = log();
        let source = StubSource::new(vec![unparseable(7), frame(1, 40001), unparseable(9)]);
        let mut p = pipeline(Box::new(source), 64);
        p.add_sink(StubSink::recording(&log));
        let report = p.run();

        assert_eq!(written(&log), vec![7, 1, 9], "P-4: all three are retained");
        assert_eq!(report.stats.packets_captured, 3);
        assert_eq!(report.stats.packets_attributed, 1);
        assert_eq!(
            report.stats.packets_unattributed, 0,
            "never attempted is not attempted and failed"
        );
        assert!(!report.stats.lost_anything());
        assert_conserved(&report, &log, 0);
    }

    // T036. FR-021. The parser's own counters reach the run.
    #[test]
    fn parse_counters_reach_the_run_and_are_not_loss() {
        let log = log();
        let source = StubSource::new(vec![unparseable(1), unparseable(2), frame(3, 40003)]);
        let mut p = pipeline(Box::new(source), 64);
        p.add_sink(StubSink::recording(&log));
        let report = p.run();

        assert_eq!(report.stats.parse.unsupported_ether_type, 2);
        assert_eq!(report.stats.parse.rejected(), 2);
        assert_eq!(
            report.stats.fragcap_dropped(),
            0,
            "a parse rejection is not a drop"
        );
        assert_conserved(&report, &log, 0);
    }

    // ------------------------------------------------------------------
    // Phase 6: the producer never waits on the sink
    // ------------------------------------------------------------------

    // T049, SC-005. The property section 12.4 exists for, asserted by the only
    // means that proves it: the gate is released by the source running dry, so
    // if the producer waited on the consumer this test would deadlock rather
    // than fail. A hang here is the failure.
    #[test]
    fn acquisition_reaches_exhaustion_while_the_sink_is_still_held() {
        let log = log();
        let (sink, release) = StubSink::gated(&log);
        // Far more packets than the buffer holds.
        let source = StubSource::new(frames(64)).signal_on_exhaustion(release);
        let mut p = pipeline(Box::new(source), 4);
        p.add_sink(sink);
        let report = p.run();

        assert_eq!(
            report.stats.packets_captured, 64,
            "the source was read to exhaustion despite the held sink"
        );
        assert_eq!(report.ended, EndReason::SourceClosed);
        assert!(report.stats.buffer_dropped > 0);
        assert_conserved(&report, &log, 0);
    }

    // T050. FR-014, FR-038. Eviction removes; it never reorders.
    #[test]
    fn survivors_of_eviction_keep_their_relative_order() {
        let log = log();
        let (sink, release) = StubSink::gated(&log);
        let source = StubSource::new(frames(32)).signal_on_exhaustion(release);
        let mut p = pipeline(Box::new(source), 4);
        p.add_sink(sink);
        let report = p.run();

        let seen = written(&log);
        assert!(seen.len() < 32, "this test is only meaningful if some were");
        let mut sorted = seen.clone();
        sorted.sort_unstable();
        assert_eq!(seen, sorted, "what survived arrived in source order");
        assert_conserved(&report, &log, 0);
    }

    // ------------------------------------------------------------------
    // Phase 7: termination
    // ------------------------------------------------------------------

    // T058. FR-030.
    #[test]
    fn source_exhaustion_drains_the_buffer_before_finishing() {
        let log = log();
        let mut p = pipeline(Box::new(StubSource::new(frames(12))), 64);
        p.add_sink(StubSink::recording(&log));
        let report = p.run();

        assert_eq!(report.ended, EndReason::SourceClosed);
        assert_eq!(
            written(&log).len(),
            12,
            "the tail was written, not abandoned"
        );
        let l = log.lock().expect("the log mutex is never poisoned");
        assert_eq!(l.finishes, 1);
        assert!(l.flushed_before_finish);
        drop(l);
        assert_conserved(&report, &log, 0);
    }

    // T059. FR-032.
    #[test]
    fn a_requested_stop_ends_the_run_and_says_so() {
        let log = log();
        // A source that would never close on its own.
        let mut p = pipeline(
            Box::new(StubSource::new(frames(4)).ending(Ending::TimeoutsThenClosed(usize::MAX))),
            64,
        );
        p.add_sink(StubSink::recording(&log));
        let stop = p.stop_handle();
        stop.stop();
        let report = p.run();

        assert_eq!(
            report.ended,
            EndReason::Stopped,
            "a stop must not be reported as exhaustion"
        );
        assert_conserved(&report, &log, 0);
    }

    // T060. FR-032. The buffered tail survives a terminal source failure.
    #[test]
    fn a_terminal_source_failure_is_named_and_the_tail_is_still_written() {
        let log = log();
        let source = StubSource::new(frames(5)).ending(Ending::Failed(SourceError::DeviceLost {
            detail: "interface removed".into(),
        }));
        let mut p = pipeline(Box::new(source), 64);
        p.add_sink(StubSink::recording(&log));
        let report = p.run();

        match &report.ended {
            EndReason::SourceFailed(SourceError::DeviceLost { detail }) => {
                assert!(detail.contains("interface removed"));
            }
            other => panic!("expected a named device loss, found {other:?}"),
        }
        assert_eq!(written(&log).len(), 5, "P-4: nothing already read is lost");
        assert!(!report.is_clean());
        assert_conserved(&report, &log, 0);
    }

    // T061. FR-027, FR-027a. One sink retires, the other keeps working.
    #[test]
    fn one_sink_retiring_leaves_the_other_receiving() {
        let doomed = log();
        let healthy = log();
        let mut p = pipeline(Box::new(StubSource::new(frames(10))), 64);
        p.add_sink(StubSink::scripted(
            &doomed,
            SinkScript {
                fail_at: Some(3),
                ..SinkScript::default()
            },
        ));
        p.add_sink(StubSink::recording(&healthy));
        let report = p.run();

        assert_eq!(
            written(&healthy).len(),
            10,
            "a healthy sink is unaffected by its neighbor"
        );
        assert_eq!(
            written(&doomed).len(),
            3,
            "the retired sink wrote three before failing"
        );
        assert_eq!(report.sink_failures.len(), 1);
        assert_eq!(report.sink_failures[0].index, 0);
        assert_eq!(
            report.ended,
            EndReason::SourceClosed,
            "one sink failing does not end the run"
        );
        // Seven withheld writes, counted exactly as a refusal would be.
        assert_eq!(report.stats.sink_dropped, 7);
        assert_conserved(&report, &doomed, 7);
        assert_conserved(&report, &healthy, 0);
    }

    // T062, T067b. FR-027b, FR-028, SC-006.
    #[test]
    fn every_sink_retiring_ends_the_run_and_names_each_failure() {
        let a = log();
        let b = log();
        let script = SinkScript {
            fail_at: Some(1),
            ..SinkScript::default()
        };
        let mut p = pipeline(
            Box::new(StubSource::new(frames(16)).ending(Ending::TimeoutsThenClosed(usize::MAX))),
            64,
        );
        p.add_sink(StubSink::scripted(&a, script.clone()));
        p.add_sink(StubSink::scripted(&b, script));
        let report = p.run();

        assert_eq!(report.ended, EndReason::AllSinksRetired);
        assert_eq!(report.sink_failures.len(), 2);
        assert_eq!(report.sink_failures[0].index, 0);
        assert_eq!(report.sink_failures[1].index, 1);
        assert!(!report.is_clean());
        assert_conserved(&report, &a, report.stats.packets_captured - 1);
        assert_conserved(&report, &b, report.stats.packets_captured - 1);
    }

    // T063. FR-033.
    #[test]
    fn a_recoverable_source_error_neither_ends_the_run_nor_counts_as_loss() {
        let log = log();
        let source = StubSource::new(frames(4)).ending(Ending::TimeoutsThenClosed(5));
        let mut p = pipeline(Box::new(source), 64);
        p.add_sink(StubSink::recording(&log));
        let report = p.run();

        assert_eq!(report.ended, EndReason::SourceClosed);
        assert_eq!(report.stats.packets_captured, 4);
        assert!(!report.stats.lost_anything(), "a timeout lost nothing");
        assert_conserved(&report, &log, 0);
    }

    // T064. The degenerate startings.
    #[test]
    fn a_source_closed_on_the_first_call_produces_a_well_formed_empty_run() {
        let log = log();
        let mut p = pipeline(Box::new(StubSource::new(Vec::new())), 64);
        p.add_sink(StubSink::recording(&log));
        let report = p.run();

        assert_eq!(report.stats.packets_captured, 0);
        assert_eq!(report.ended, EndReason::SourceClosed);
        assert!(report.is_clean());
        let l = log.lock().expect("the log mutex is never poisoned");
        assert_eq!(l.finishes, 1, "an empty capture is still terminated");
        assert!(l.flushed_before_finish);
    }

    // T064.
    #[test]
    fn a_stop_requested_before_the_run_begins_produces_empty_output() {
        let log = log();
        let mut p = pipeline(Box::new(StubSource::new(frames(8))), 64);
        p.add_sink(StubSink::recording(&log));
        p.stop_handle().stop();
        let report = p.run();

        assert_eq!(report.stats.packets_captured, 0);
        assert_eq!(report.ended, EndReason::Stopped);
        assert_eq!(
            log.lock()
                .expect("the log mutex is never poisoned")
                .finishes,
            1
        );
    }

    // T065, SC-011. FR-033a, FR-033b.
    #[test]
    fn an_acquisition_panic_still_finishes_every_sink_and_still_panics() {
        let log = log();
        let source = PanickingSource {
            queued: frames(8).into_iter().map(raw).collect(),
            panic_at: 4,
            served: 0,
        };
        let mut p = Pipeline::new(
            vec![SourceBinding::new(InterfaceId::default(), Box::new(source))],
            Box::new(StubAttributor::resolving()),
            PipelineConfig {
                addrs: local_addrs(),
                ..PipelineConfig::default()
            },
        )
        .expect("builds");
        p.add_sink(StubSink::recording(&log));

        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.run()));
        assert!(outcome.is_err(), "the panic must reach the caller");

        // And the sinks were still finished, before the panic got here.
        let l = log.lock().expect("the log mutex is never poisoned");
        assert_eq!(l.finishes, 1, "the output was terminated despite the panic");
        assert!(l.flushed_before_finish);
        assert_eq!(l.written.len(), 4, "what was acquired was still written");
    }

    // A sink panic is not caught either, and reaches the caller through the
    // join. The output side's other work is not attempted after it.
    #[test]
    fn a_sink_panic_reaches_the_caller() {
        let mut p = pipeline(Box::new(StubSource::new(frames(8))), 64);
        p.add_sink(Box::new(PanickingSink {
            panic_at: 2,
            seen: 0,
        }));
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.run()));
        assert!(
            outcome.is_err(),
            "a panicking sink is a defect in that sink, not a counted drop"
        );
    }

    // T066. FR-031.
    #[test]
    fn every_sink_is_flushed_once_then_finished_once() {
        let a = log();
        let b = log();
        let mut p = pipeline(Box::new(StubSource::new(frames(3))), 64);
        p.add_sink(StubSink::recording(&a));
        p.add_sink(StubSink::recording(&b));
        let _ = p.run();

        for l in [&a, &b] {
            let l = l.lock().expect("the log mutex is never poisoned");
            assert_eq!(l.flushes, 1);
            assert_eq!(l.finishes, 1);
            assert!(l.flushed_before_finish);
        }
    }

    // T067a. FR-028a.
    #[test]
    fn a_failure_in_flush_or_finish_is_recorded_and_stops_nothing_else() {
        let a = log();
        let b = log();
        let mut p = pipeline(Box::new(StubSource::new(frames(4))), 64);
        p.add_sink(StubSink::scripted(
            &a,
            SinkScript {
                fail_flush: true,
                fail_finish: true,
                ..SinkScript::default()
            },
        ));
        p.add_sink(StubSink::recording(&b));
        let report = p.run();

        assert_eq!(
            report.sink_failures.len(),
            2,
            "the flush failure and the finish failure are both recorded"
        );
        assert!(report.sink_failures.iter().all(|f| f.index == 0));
        let l = b.lock().expect("the log mutex is never poisoned");
        assert_eq!(l.finishes, 1, "sink one is still finished");
        assert_eq!(l.written.len(), 4);
    }

    // T067. FR-035, FR-036.
    #[test]
    fn the_report_carries_the_statistics_on_both_paths() {
        // Clean: into_result yields the statistics.
        let clean_log = log();
        let mut p = pipeline(Box::new(StubSource::new(frames(4))), 64);
        p.add_sink(StubSink::recording(&clean_log));
        let stats = p.run().into_result().expect("a clean run");
        assert_eq!(stats.packets_captured, 4);

        // Abnormal: into_result errs, and the statistics are still there.
        let failing_log = log();
        let source = StubSource::new(frames(4)).ending(Ending::Failed(SourceError::Backend {
            detail: "oops".into(),
        }));
        let mut p = pipeline(Box::new(source), 64);
        p.add_sink(StubSink::recording(&failing_log));
        let err = p.run().into_result().expect_err("a failed run");
        assert_eq!(err.report.stats.packets_captured, 4);
        assert!(err.to_string().contains("abnormally"));
    }

    // FR-036. Ordinary drops are not an abnormal ending, and conflating the two
    // would make every lossy capture look like a failure.
    #[test]
    fn ordinary_drops_do_not_make_a_run_abnormal() {
        let log = log();
        let mut p = pipeline(Box::new(StubSource::new(frames(6))), 64);
        p.add_sink(StubSink::scripted(
            &log,
            SinkScript {
                refuse: vec![0, 1],
                ..SinkScript::default()
            },
        ));
        let report = p.run();
        assert!(report.stats.lost_anything(), "two writes did not happen");
        assert!(report.is_clean(), "but the run itself ended normally");
        // Exactly two, not merely some. Asserting only that loss occurred would
        // accept any wrong positive `sink_dropped`, which is the failure this
        // helper exists to catch.
        assert_eq!(report.stats.sink_dropped, 2);
        assert_conserved(&report, &log, 2);
        assert!(report.into_result().is_ok());
    }

    // A sink panic must end the acquisition loop, not merely reach the caller
    // once the source happens to run out. The source here never closes on its
    // own, so a `run` that did not signal acquisition would hang here rather
    // than fail, which is the shape of the defect: the original panic test used
    // a finite source and could not have detected it.
    #[test]
    fn a_sink_panic_ends_acquisition_even_when_the_source_never_closes() {
        let mut p = pipeline(
            Box::new(StubSource::new(frames(64)).ending(Ending::TimeoutsThenClosed(usize::MAX))),
            4,
        );
        p.add_sink(Box::new(PanickingSink {
            panic_at: 1,
            seen: 0,
        }));
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.run()));
        assert!(
            outcome.is_err(),
            "the panic reached the caller, which means acquisition stopped"
        );
    }

    // An ending acquisition reached on its own outranks a retirement the output
    // side observed afterwards, while draining. Reporting `AllSinksRetired`
    // here would bury the reason an operator most needs.
    //
    // The ordering is established by the gate, not by capacity and hope. The
    // first version of this test used a roomy buffer and assumed acquisition
    // would finish before the output side popped anything, which is a race: it
    // passed locally and failed on the Windows runner, where the output side
    // won, retired the sink, and set the stop that acquisition then reported.
    // The gate is released by the source running dry, so the retirement is
    // strictly after acquisition has chosen its ending.
    #[test]
    fn a_terminal_source_failure_outranks_a_later_retirement() {
        let log = log();
        let (sink, release) = StubSink::gated_scripted(
            &log,
            SinkScript {
                fail_at: Some(0),
                ..SinkScript::default()
            },
        );
        let source = StubSource::new(frames(8))
            .ending(Ending::Failed(SourceError::DeviceLost {
                detail: "interface removed".into(),
            }))
            .signal_on_exhaustion(release);
        let mut p = pipeline(Box::new(source), 8);
        p.add_sink(sink);
        let report = p.run();

        assert!(
            matches!(
                report.ended,
                EndReason::SourceFailed(SourceError::DeviceLost { .. })
            ),
            "the device loss is the reason the run ended, not the retirement \
             the output side found afterwards; got {:?}",
            report.ended
        );
        assert_eq!(
            report.sink_failures.len(),
            1,
            "and the retirement is still reported, just not as the reason"
        );
        assert_conserved(&report, &log, report.stats.sink_dropped);
    }
}
