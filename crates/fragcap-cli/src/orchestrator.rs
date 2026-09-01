// SPDX-License-Identifier: Apache-2.0

//! The capture engine shared by `run` and `tap`.
//!
//! The pipeline owns the packet threads and the shared attributor; a session
//! driver owns the [`CaptureSession`], and the two connect through a
//! [`StopHandle`] and a published binding snapshot rather than by routing
//! packets through the session (slice S14 research D-c). A [`SessionGate`]
//! attached to the pipeline (slice 017) decides, synchronously on the write
//! path, whether each packet reaches the sinks: it admits only while the session
//! is capturing and within the bound, discards and counts everything else, and
//! forwards each admitted packet's length and instant to the driver. That makes a
//! volume bound produce an exactly-bounded file and lets the live path read and
//! count watch-time frames from arm, reversing S14's D-e observe-only tee for
//! those two cases while keeping the offline unbounded run a pass-through.
//!
//! # The offline shape is deterministic in two phases
//!
//! Acquisition runs first: process events are folded until a non-service stage
//! matches, the bindings are published, and only then does the pipeline start,
//! so every packet the pipeline attributes sees the role and stage already
//! published. That ordering is what makes the stamped output a stable golden
//! rather than a race between the publish and the resolve. The remaining events,
//! chiefly the exits that decide the stop reason, are folded during capture,
//! interleaved with the tee's packets by timestamp. A live capture reaches the
//! same driver with events arriving on a channel instead of pre-collected.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Once};

use fragcap::core::CaptureStats;
// The stamper's active_endpoints (profiled-filtered, slice 015) is read for the
// filter-narrowed event; the trait brings the method into scope.
use fragcap::core::FlowAttributor;
use fragcap::{
    BindingPublisher, CaptureScope, CaptureSession, GateHandle, LiveStats, Pipeline,
    PipelineConfig, PipelineReport, ProcessEvent, Profile, SessionGate, SessionState, StopHandle,
    StopReason, Timestamp,
};

use crate::args::Direction;
use crate::assemble::{self, CaptureComponents, EffectiveConfig, EventStream, ARMED_AT};
use crate::emit::Emitter;
#[cfg(all(feature = "etw", windows))]
use crate::emit::{Format, Verbosity};
use crate::events::Event;
use crate::exit::{CliError, Exit};
#[cfg(all(feature = "etw", windows))]
use crate::live_status;
use crate::output::CompletionSummary;

/// The process-global operator-interrupt flag.
///
/// A console interrupt handler sets it; the driver observes it and drives the
/// session to a clean `StopReason::Interrupt`. Process-global rather than
/// per-run because a console handler can be installed at most once per process,
/// which [`install_interrupt_handler`] enforces with a `Once`.
pub static INTERRUPT: AtomicBool = AtomicBool::new(false);

static INSTALL: Once = Once::new();

/// Install the console-interrupt handler, at most once per process.
///
/// A repeated call is a no-op, so `run` and `tap` may each ask for it without
/// the second failing. In a tier-1 test the interrupt is fired through the
/// hidden `fire_interrupt` path instead, so this handler is never triggered.
pub fn install_interrupt_handler() {
    INSTALL.call_once(|| {
        let _ = ctrlc::set_handler(|| INTERRUPT.store(true, Ordering::Relaxed));
    });
}

/// What a capture run reports back to its caller (slice S059).
///
/// `exit` is the run's exit result, what `capture` returned before this type
/// existed. `observed_holder` is the dominant socket-holding image the run
/// attributed the most packets to, or `None` when nothing was attributed. The
/// `capture` command uses it to promote an unresolved target after the run; the
/// `extcap` path ignores it. It is deliberately not on the golden-pinned
/// completion summary, so a nondeterministic image list never churns a golden.
pub struct CaptureOutcome {
    /// The run's exit result.
    pub exit: Exit,
    /// The dominant observed socket-holder, or `None` if nothing was attributed.
    pub observed_holder: Option<Arc<str>>,
    /// The terminal session reason, retained for composed commands that need to
    /// distinguish a clean operator interruption from ordinary completion.
    pub stop_reason: Option<StopReason>,
}

impl CaptureOutcome {
    /// A run that acquired no target and observed nothing, carrying only its
    /// exit. The no-target-acquired paths return this.
    fn bare(exit: Exit) -> Self {
        CaptureOutcome {
            exit,
            observed_holder: None,
            stop_reason: None,
        }
    }
}

/// Run a capture to completion and report its outcome.
///
/// The single entry both `capture` and `extcap` reach, once each has resolved or
/// synthesized a profile and built the effective configuration. The armed
/// session and the operator-facing preamble are shared; the event source then
/// decides which driver runs. An offline timeline drives the deterministic
/// two-phase path; a live ETW stream drives the streaming merged-channel path.
#[allow(clippy::too_many_arguments)]
pub fn capture(
    profile: Profile,
    config: &EffectiveConfig,
    mut components: CaptureComponents,
    emitter: &mut Emitter,
    interrupt: &AtomicBool,
    fire_interrupt: bool,
    allowed_roles: Option<Vec<String>>,
    sink_failure_is_clean: bool,
) -> Result<CaptureOutcome, CliError> {
    let mut session = CaptureSession::new_scoped(profile, config.session_config(), allowed_roles);
    session.attach(ARMED_AT);

    // Attach-to-running (section 15.7): fold the startup snapshot the watcher took
    // at arm, so a target already running when the session armed acquires now,
    // without a later start event. Empty when no watcher took one, which makes
    // this a no-op and keeps a run with no already-running target byte-identical.
    if !components.startup_snapshot.is_empty() {
        session.apply_snapshot(
            &components.startup_snapshot,
            components.snapshot_at.unwrap_or(ARMED_AT),
        );
    }

    let interface_names = config.interface_names();
    emitter.event(&Event::SessionArmed {
        interfaces: interface_names.clone(),
    });
    emitter.progress(&format!("session armed on {}", interface_names.join(", ")));

    // The effective scope, recorded and surfaced. Directional filtering of
    // output is deferred (specification FR-011b), so a non-default direction is
    // carried and warned about rather than silently ignored.
    if config.direction != Direction::Both {
        emitter.warn(
            "--direction is recorded on the effective configuration but directional output \
             filtering is deferred to a later slice",
        );
    }
    // Since slice S064 the roles claim is true of what the file contains, not
    // only of which stages trigger acquisition: under the target scope the write
    // gate consults the same role set. It said `(enforced)` before that was true
    // of retention, which is the kind of claim P-9 exists to stop (issue #184).
    // The word is now attached to the scope, which is what actually enforces.
    let roles = config
        .roles
        .as_ref()
        .map(|r| r.join(","))
        .unwrap_or_else(|| "all".to_string());
    let scope = match config.scope {
        CaptureScope::Target => "target",
        CaptureScope::All => "all",
    };
    emitter.progress(&format!(
        "scope: writing {} traffic, direction {}, roles {}, loopback {}",
        scope,
        config.direction.as_str(),
        roles,
        config.loopback
    ));

    // The event source decides the driver. Taking it out by replacement leaves
    // the rest of the components (sources, stamper, publisher) in place for the
    // chosen driver to consume.
    let events = std::mem::replace(&mut components.events, EventStream::Prerecorded(Vec::new()));
    match events {
        EventStream::Prerecorded(events) => capture_prerecorded(
            session,
            config,
            components,
            events,
            emitter,
            interrupt,
            fire_interrupt,
            sink_failure_is_clean,
        ),
        #[cfg(all(feature = "etw", windows))]
        EventStream::Live(rx) => capture_live(
            session,
            config,
            components,
            rx,
            emitter,
            interrupt,
            fire_interrupt,
            sink_failure_is_clean,
        ),
    }
}

/// Decide the run's exit from whether a sink failed.
///
/// A run that ended with no sink failure is a success. An unrecoverable sink
/// failure normally ends the run at `Exit::FAILURE` (specification FR-005a). For
/// an extcap capture, though, the single sink is the analyzer's FIFO and the
/// analyst closing it is the defined clean stop, so `sink_failure_is_clean`
/// reinterprets that end as a success while the summary still carries the loss
/// accounting (constitution P-4). The FIFO is opened before capture starts (a bad
/// path fails at assembly), so a mid-capture failure is a consumer disconnect
/// rather than a broken destination.
fn final_exit(
    had_sink_failure: bool,
    sink_failure_is_clean: bool,
    stop_reason: Option<StopReason>,
) -> Exit {
    if matches!(
        stop_reason,
        Some(
            StopReason::AcquisitionTimeout
                | StopReason::AmbiguousStageMatch
                | StopReason::PlatformExitedBeforeClient
                | StopReason::EscapedPlatformClient
                | StopReason::PlatformDispatchFailed
                | StopReason::PlatformStartFailed
                | StopReason::ProcessWatcherLost
        )
    ) {
        return Exit::new(1);
    }
    if !had_sink_failure || sink_failure_is_clean {
        Exit::SUCCESS
    } else {
        Exit::FAILURE
    }
}

/// The offline, deterministic two-phase driver.
///
/// Acquisition folds the pre-collected timeline until a terminal stage matches,
/// then the pipeline starts and the remaining events are folded during capture,
/// interleaved with the tee's packets by timestamp. This is the path every
/// tier-1 test and every committed golden exercises; its observable behavior is
/// unchanged from before the live path existed.
#[allow(clippy::too_many_arguments)]
fn capture_prerecorded(
    mut session: CaptureSession,
    config: &EffectiveConfig,
    mut components: CaptureComponents,
    events: Vec<ProcessEvent>,
    emitter: &mut Emitter,
    interrupt: &AtomicBool,
    fire_interrupt: bool,
    sink_failure_is_clean: bool,
) -> Result<CaptureOutcome, CliError> {
    // The pid to role map, so a new binding is detected as a match and an exit
    // of a bound pid is detected as a stage exit.
    let mut bound: HashMap<u32, String> = HashMap::new();
    let publisher = components.publisher.clone();

    // Acquisition: fold events until a non-service stage acquires the target.
    let mut cursor = 0usize;
    while cursor < events.len() && session.state() != SessionState::Capturing {
        let event = events[cursor].clone();
        cursor += 1;
        apply_event(event, &mut session, &mut bound, emitter);
        publisher.publish(session.role_bindings());
    }

    let acquired = session.state() == SessionState::Capturing;
    if !acquired {
        // No target was acquired. If an acquisition timeout was set, fire it so
        // the summary names it; either way the run captured nothing and exits 1.
        if let Some(timeout) = config.acquisition_timeout {
            session.on_tick(Timestamp::from_nanos(
                ARMED_AT.as_nanos() + timeout.as_nanos() as i64,
            ));
        }
        let summary = build_summary(false, &session, &CaptureStats::default(), None);
        emitter.summary(&summary);
        return Ok(CaptureOutcome::bare(Exit::FAILURE));
    }

    publisher.publish(session.role_bindings());
    // Drive an initial refresh so the count reflects the first real snapshot.
    // On the live socket-table path the pipeline control thread performs the
    // refreshes during the run, but it has not started yet at this point, so
    // without this the attributor still holds its empty initial publication and
    // the count would always be reported as zero (found in review of pull
    // request 24). Offline the scripted attributor reports it wants no refresh,
    // so this is a no-op and the count is unchanged.
    if let Some(s) = components.stamper.as_ref() {
        if s.wants_refresh() {
            let _ = s.refresh();
        }
    }
    // The count of endpoints actually narrowed to: the stamper reports only
    // endpoints owned by profiled processes (slice 015), not the full
    // socket-table set the inner attributor holds. Offline this is the scripted
    // set unchanged (those endpoints carry no owner and are kept); live it is
    // the profiled subset.
    let mut narration = FilterNarration::new(components.stamper.clone());
    narration.announce_watching(emitter);
    narration.poll(emitter);

    // The events not consumed during acquisition, folded during capture.
    let mut pending: Vec<ProcessEvent> = events[cursor..].to_vec();

    // Build the gate and run the pipeline on its own thread. Offline is
    // two-phase: acquisition has already reached Capturing above, so the gate's
    // window is opened before the pipeline starts and it never sees a watch-time
    // packet. For an unbounded run the gate then admits every packet, a
    // pass-through that keeps the committed goldens byte-identical (slice 017,
    // decision D-6).
    let (tx, rx) = mpsc::channel::<(u32, Timestamp)>();
    let (gate, gate_handle) = SessionGate::new(&config.session_config(), tx);
    // Offline is two-phase: acquisition is already complete, so the window admits
    // every replayed frame from the earliest instant and there is no watch-time
    // frame; the bound alone does the bounding.
    gate_handle.open_from(Timestamp::from_nanos(i64::MIN));
    // A zero volume bound is met before any packet is retained, so no on_packet
    // fires it; stop for it now so the reason is volume-reached (review of PR #26).
    if zero_volume_bound(config) {
        session.on_volume_reached();
    }
    // The offline driver has no long-silence problem to solve (research R-1)
    // and does not read the live counters.
    let (handle, stop, stream_reports, ring_evicted, _live) =
        spawn_pipeline(config, &mut components, gate)?;

    drive(
        &rx,
        &mut session,
        &mut bound,
        &mut pending,
        &publisher,
        emitter,
        interrupt,
        &stop,
        &mut narration,
    );

    let report: PipelineReport = handle.join().expect("the pipeline thread did not panic");
    emit_stream_reports(&stream_reports, emitter);
    emit_ring_report(&ring_evicted, emitter);

    // Fold the events past the last packet (the exits that decide the stop
    // reason a script implies).
    for event in pending {
        apply_event(event, &mut session, &mut bound, emitter);
        publisher.publish(session.role_bindings());
    }

    // An interrupt fired for the whole run is a clean stop once everything has
    // been captured (specification FR-005).
    if (fire_interrupt || interrupt.load(Ordering::Relaxed)) && is_active(&session) {
        session.on_interrupt();
    }
    session.finalize();

    let summary = build_summary(true, &session, &report.stats, Some(&gate_handle));
    emitter.summary(&summary);

    // An unrecoverable sink failure ended the run; the output may be partial
    // (specification FR-005a). Not a usage error. For an extcap capture, the
    // analyzer closing its FIFO is the defined clean stop, so that end is a
    // success (the summary still carries the accounting).
    Ok(CaptureOutcome {
        exit: final_exit(
            !report.sink_failures.is_empty(),
            sink_failure_is_clean,
            summary.stop_reason,
        ),
        observed_holder: report.stats.dominant_holder(),
        stop_reason: summary.stop_reason,
    })
}

/// Build the sinks, attach the write gate, and spawn the pipeline on its own
/// thread. Shared by both drivers so the two construct the output path
/// identically and the offline bytes cannot drift from the live ones.
///
/// The gate is the synchronous authority for what reaches the sinks: the output
/// loop consults it before the fan-out (slice 017). It forwards each admitted
/// packet's length and instant to the driver over the gate's own channel, which
/// is what feeds `CaptureSession::on_packet` so `VolumeReached` and the duration
/// bound still fire in the session.
/// What [`spawn_pipeline`] hands back: the pipeline thread's join handle, its stop
/// handle, the streaming sinks' per-consumer report handles, the ring sink's
/// eviction counter (present only in ring mode), and a live-readable clone of
/// the pipeline's `sink_dropped`/holder-tally/`buffer_dropped` counters (slice
/// S069), safe to read while the pipeline thread is still running.
type SpawnedPipeline = (
    std::thread::JoinHandle<PipelineReport>,
    StopHandle,
    Vec<assemble::StreamReports>,
    Option<Arc<AtomicU64>>,
    LiveStats,
);

fn spawn_pipeline(
    config: &EffectiveConfig,
    components: &mut CaptureComponents,
    gate: SessionGate,
) -> Result<SpawnedPipeline, CliError> {
    let built = assemble::build_sinks(config, &components.interfaces)?;
    let stamper = components
        .stamper
        .take()
        .expect("the stamper is taken exactly once");

    let mut pipeline = Pipeline::new(
        std::mem::take(&mut components.sources),
        Box::new(stamper),
        PipelineConfig::default(),
    )?;
    if let Some(registry) = components.flow_registry.take() {
        pipeline.set_flow_registry(registry);
    }
    pipeline.set_write_gate(Arc::new(gate));
    for sink in built.sinks {
        pipeline.add_sink(sink);
    }
    let stop = pipeline.stop_handle();
    // Taken before the pipeline moves into the spawned thread: `live_stats()`
    // only needs `&self`, and `run(self)` below consumes it by value.
    let live = pipeline.live_stats();
    let handle = std::thread::spawn(move || pipeline.run());
    Ok((handle, stop, built.stream_reports, built.ring_evicted, live))
}

/// Surface each streaming consumer's per-consumer accounting after the run, so
/// a dropped or disconnected consumer is reported (specification 14.4, P-4).
/// The streaming sink owns these counters, distinct from the capture-wide
/// `sink_dropped`.
fn emit_stream_reports(reports: &[assemble::StreamReports], emitter: &mut Emitter) {
    for stream in reports {
        let consumers = stream.handle.lock().expect("reports mutex not poisoned");
        for consumer in consumers.iter() {
            // A structured event so a `--json` consumer sees the per-consumer
            // loss, and a progress line for the human summary. `progress` is a
            // no-op in JSON mode and `event` a no-op in human mode, so exactly
            // one shape is emitted.
            emitter.event(&Event::StreamConsumer {
                transport: stream.transport.clone(),
                id: consumer.id.clone(),
                written: consumer.written,
                dropped: consumer.dropped,
                reason: consumer.reason.as_str().to_string(),
            });
            emitter.progress(&format!(
                "stream {} consumer {}: {} written, {} dropped, {}",
                stream.transport,
                consumer.id,
                consumer.written,
                consumer.dropped,
                consumer.reason.as_str()
            ));
        }
    }
}

/// Surface the ring sink's eviction count after the run, so a ring capture that
/// rolled its window reports how many packets it dropped from the tail rather than
/// reporting zero loss (specification section 7.2, constitution P-4). The ring
/// sink owns this counter, distinct from the capture-wide `sink_dropped`: an
/// eviction is the operator's declared window scope, not a capture loss, but it is
/// still surfaced so the omission is never silent. `None` outside ring mode.
fn emit_ring_report(evicted: &Option<Arc<AtomicU64>>, emitter: &mut Emitter) {
    if let Some(handle) = evicted {
        let evicted = handle.load(Ordering::Relaxed);
        // A structured event for a `--json` consumer and a progress line for the
        // human summary; exactly one shape is emitted (the other call is a no-op
        // for the active output mode).
        emitter.event(&Event::RingEvicted { evicted });
        emitter.progress(&format!(
            "ring mode: {evicted} packet(s) evicted from the rolling window"
        ));
    }
}

/// The driver loop over the tee's packets, interleaving the pending events by
/// timestamp and honoring the interrupt flag.
#[allow(clippy::too_many_arguments)]
fn drive(
    rx: &Receiver<(u32, Timestamp)>,
    session: &mut CaptureSession,
    bound: &mut HashMap<u32, String>,
    pending: &mut Vec<ProcessEvent>,
    publisher: &BindingPublisher,
    emitter: &mut Emitter,
    interrupt: &AtomicBool,
    stop: &StopHandle,
    narration: &mut FilterNarration,
) {
    let mut folded = 0usize;
    while let Ok((len, ts)) = rx.recv() {
        while folded < pending.len() && pending[folded].at().as_nanos() <= ts.as_nanos() {
            let event = pending[folded].clone();
            folded += 1;
            apply_event(event, session, bound, emitter);
            publisher.publish(session.role_bindings());
        }
        session.on_packet(len);
        session.on_tick(ts);
        // Report the narrowing from the transition, not from a sample taken at
        // acquisition. Rate-limited inside `poll`.
        narration.poll(emitter);
        if interrupt.load(Ordering::Relaxed) && is_active(session) {
            session.on_interrupt();
        }
        if !is_active(session) {
            stop.stop();
        }
    }
    // Leave the unfolded events for the post-run drain.
    pending.drain(0..folded);
}

/// A message on the live driver's merged channel.
///
/// A live capture has no pre-collected timeline to read ahead through, and its
/// packets and its process events arrive on two independent channels. Merging
/// them into one totally ordered stream is what lets the driver stop on a
/// terminal-stage exit even while no further packets arrive: the exit is a
/// `Proc` message that reaches the driver regardless of the packet path.
#[cfg(all(feature = "etw", windows))]
enum DriverMsg {
    /// A packet the pipeline retained, by length and observed instant.
    Packet(u32, Timestamp),
    /// A process start or exit the ETW watcher observed.
    Proc(ProcessEvent),
    /// The ETW watcher ended, so process ownership can no longer be maintained.
    WatcherLost,
}

/// One-shot authority gate for the title dispatch of an owned platform launch.
///
/// The retained title action is authorized only after the watcher has bound the
/// exact process created for the platform root to the `platform` role. Repeated
/// snapshots cannot authorize a second dispatch.
#[derive(Default)]
#[cfg(any(test, all(feature = "etw", windows)))]
struct PlatformDispatchGate {
    root_pid: Option<u32>,
    issued: bool,
}

#[cfg(any(test, all(feature = "etw", windows)))]
type RoleBinding = (u32, Option<Arc<str>>, Option<fragcap::StageId>);

#[cfg(any(test, all(feature = "etw", windows)))]
impl PlatformDispatchGate {
    fn arm(&mut self, root_pid: Option<u32>) {
        self.root_pid = root_pid;
    }

    fn observe(&mut self, bindings: &[RoleBinding]) -> bool {
        if self.issued {
            return false;
        }
        let Some(root_pid) = self.root_pid else {
            return false;
        };
        if bindings.iter().any(|(pid, role, _)| {
            *pid == root_pid && role.as_deref().is_some_and(|role| role == "platform")
        }) {
            self.issued = true;
            return true;
        }
        false
    }
}

/// The live, streaming driver. Distinct from the offline two-phase path because
/// a live stream has no end the driver can read ahead to.
///
/// Acquisition folds live process events until a terminal stage matches or the
/// acquisition timeout elapses in real time. Capture then builds the same
/// pipeline the offline path does, merges the tee's packets and the live events
/// into one channel, and folds them in arrival order, honoring the interrupt and
/// the session's own bounds. When the session leaves an active state, the
/// pipeline is stopped and the run finalizes exactly as offline.
#[cfg(all(feature = "etw", windows))]
#[allow(clippy::too_many_arguments)]
fn capture_live(
    mut session: CaptureSession,
    config: &EffectiveConfig,
    mut components: CaptureComponents,
    rx: Receiver<ProcessEvent>,
    emitter: &mut Emitter,
    interrupt: &AtomicBool,
    fire_interrupt: bool,
    sink_failure_is_clean: bool,
) -> Result<CaptureOutcome, CliError> {
    use std::time::{Duration, Instant};

    let mut bound: HashMap<u32, String> = HashMap::new();
    // Image names by pid, for the live status display's header line (slice
    // S069). `apply_event`'s own `bound` map carries only the role, since
    // that is all its two existing callers (`drive` and `drive_live`) ever
    // needed; this is a second, `capture_live`-local map so `drive`
    // (research R-1: not touched by this slice) keeps its existing type.
    let mut bound_images: HashMap<u32, String> = HashMap::new();

    // Attach-to-running (Codex review of PR #196): `capture()` applied the
    // watcher's startup snapshot to `session` before this function was
    // called, so a target already running when the session armed may
    // already be `SessionState::Capturing` here, before the acquisition
    // loop below ever runs (that loop's first check breaks immediately when
    // the session is already capturing). That loop is the only other place
    // `bound`/`bound_images` are populated, so without this seeding step the
    // live status display would show "waiting for a target" for the whole
    // of an attach-to-running run despite a target actively being captured.
    for (pid, role, _stage) in session.role_bindings() {
        if let Some(role) = role {
            bound.insert(pid, role.to_string());
        }
    }
    for record in &components.startup_snapshot {
        let image = record
            .image
            .rsplit(['\\', '/'])
            .next()
            .unwrap_or(&record.image)
            .to_string();
        if !image.is_empty() {
            bound_images.insert(record.pid, image);
        }
    }

    let publisher = components.publisher.clone();

    // Elapsed time is real and monotonic, converted onto the session clock from
    // ARMED_AT so the session's tick-driven bounds see wall time pass.
    let started = Instant::now();
    let tick = Duration::from_millis(200);

    // Run from arm (slice 017, C2). The live capture handle is open from arm, so
    // the pipeline is spawned before acquisition and its pre-acquisition frames
    // are read. The gate starts in its Watching window, so those frames are
    // discarded and counted rather than never observed. Keep an endpoint reader
    // clone of the stamper, since the stamper itself is moved into the pipeline
    // here and the filter-narrowed count is read after acquisition.
    let (tee_tx, tee_rx) = mpsc::channel::<(u32, Timestamp)>();
    let (gate, gate_handle) = SessionGate::new(&config.session_config(), tee_tx);
    let stamper_reader = components.stamper.clone();
    let (handle, stop, stream_reports, ring_evicted, live) =
        spawn_pipeline(config, &mut components, gate)?;
    let mut platform_dispatch = PlatformDispatchGate::default();

    // Managed launch (S17, specification 16.4): the session is already Watching
    // (attached before this function) and the sinks are open (spawn_pipeline above),
    // so starting the title now means every process in its launch chain produces a
    // start event the acquisition loop below observes, including a launcher whose
    // whole lifetime is shorter than any poll interval. This is the tier-2 path and
    // is never asserted as run in CI.
    if let Some(request) = &config.launch {
        let description = match request {
            fragcap::managed_launch::ManagedLaunch::Steam(request) => {
                format!("{} through Steam", request.url)
            }
            fragcap::managed_launch::ManagedLaunch::Direct(request) => {
                request.executable().display().to_string()
            }
            fragcap::managed_launch::ManagedLaunch::Publisher(request) => format!(
                "{} through publisher chain",
                request.root().executable().display()
            ),
            fragcap::managed_launch::ManagedLaunch::Platform(request) => format!(
                "{} as an owned platform root",
                request.root().executable().display()
            ),
        };
        emitter.progress(&format!("launching {description}"));
        match request.execute() {
            Ok(receipt) => {
                if matches!(request, fragcap::managed_launch::ManagedLaunch::Platform(_)) {
                    platform_dispatch.arm(receipt.process_id());
                }
            }
            Err(e) => {
                // The pipeline is already running from arm. Stop and join it so the
                // sinks finalize and no output file is left unclosed, drop the watcher
                // to end the live receiver, and surface the run's loss accounting,
                // before returning the launch failure (Codex review of PR #31). The
                // window was never opened, so every frame read is a watch-time discard
                // already in the gate's tallies.
                stop.stop();
                let _ = components.watcher.take();
                let report: PipelineReport =
                    handle.join().expect("the pipeline thread did not panic");
                emit_stream_reports(&stream_reports, emitter);
                emit_ring_report(&ring_evicted, emitter);
                if matches!(request, fragcap::managed_launch::ManagedLaunch::Platform(_)) {
                    session.on_platform_start_failure();
                }
                session.finalize();
                let summary = build_summary(false, &session, &report.stats, Some(&gate_handle));
                emitter.summary(&summary);
                return Err(CliError::failure(e.to_string()));
            }
        }
    }

    // Acquisition: fold live events until a terminal stage acquires the target,
    // or acquisition ends for another reason (the acquisition timeout, a
    // duration bound reached while still watching, an operator interrupt, the
    // watcher disconnecting, or the pipeline itself ending). The pipeline is
    // already running; watch-time frames are discarded and counted by the gate as
    // this loop turns. `acquired_at` records the capture instant of the acquiring
    // event, so the gate can classify a buffered pre-acquisition frame by its own
    // instant rather than by the window state at write time (review of PR #26).
    let mut acquired_at: Option<Timestamp> = None;
    loop {
        if session.state() == SessionState::Capturing {
            break;
        }
        // The pipeline was spawned from arm, so it can end before a target is
        // acquired: the sole capture interface may close or fail. Its tee channel
        // disconnects when the output thread ends, which is the signal that no
        // source remains; without observing it here the command would wait forever
        // on a still-running watcher (review of PR #26). No receipt is ever sent
        // while watching, so this only ever observes empty or disconnected.
        if let Err(mpsc::TryRecvError::Disconnected) = tee_rx.try_recv() {
            break;
        }
        // An operator interrupt during acquisition is a clean cancellation, not
        // a failure to acquire: stop the session cleanly and leave the loop.
        if (fire_interrupt || interrupt.load(Ordering::Relaxed)) && is_active(&session) {
            session.on_interrupt();
            break;
        }
        // A tick may have moved the session out of an active state (a --duration
        // bound reached while still watching); nothing more can be acquired.
        if !is_active(&session) {
            break;
        }
        if let Some(timeout) = config.acquisition_timeout {
            if started.elapsed() >= timeout {
                break;
            }
        }
        match rx.recv_timeout(tick) {
            Ok(event) => {
                let event_at = event.at();
                record_bound_image(&event, &mut bound_images);
                apply_event(event, &mut session, &mut bound, emitter);
                let bindings = session.role_bindings();
                publisher.publish(bindings.clone());
                if is_active(&session) && platform_dispatch.observe(&bindings) {
                    let dispatch = config.launch.as_ref().and_then(|launch| match launch {
                        fragcap::managed_launch::ManagedLaunch::Platform(platform) => {
                            Some(platform.dispatch_title())
                        }
                        _ => None,
                    });
                    if let Some(Err(error)) = dispatch {
                        emitter.progress(&format!("platform title dispatch failed: {error}"));
                        session.on_platform_dispatch_failure();
                    }
                }
                if session.state() == SessionState::Capturing {
                    acquired_at = Some(event_at);
                }
            }
            // No event arrived; advance the session clock so a tick-based
            // acquisition timeout or duration bound can still fire.
            Err(mpsc::RecvTimeoutError::Timeout) => session.on_tick(elapsed_ts(started)),
            // The watcher is gone; no target can now be acquired.
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                session.on_process_watcher_lost();
                break;
            }
        }
    }

    if session.state() != SessionState::Capturing {
        // No target was acquired. The pipeline has been running from arm, so it
        // is stopped and joined before reporting; its watch-time frames are in
        // the gate's tallies and surface in the summary. Dropping the watcher
        // ends the live receiver.
        stop.stop();
        let _ = components.watcher.take();
        let report: PipelineReport = handle.join().expect("the pipeline thread did not panic");
        emit_stream_reports(&stream_reports, emitter);
        emit_ring_report(&ring_evicted, emitter);
        // Fire the acquisition timeout, if set and the session is still watching,
        // so the summary can name it; an interrupt or a duration bound has
        // already stopped the session and set its reason.
        if is_active(&session) {
            if let Some(timeout) = config.acquisition_timeout {
                session.on_tick(Timestamp::from_nanos(
                    ARMED_AT.as_nanos() + timeout.as_nanos() as i64,
                ));
            }
        }
        // The window was never opened (admit_from stayed at its watching sentinel),
        // so every frame the pipeline read is already a watch-time discard in the
        // gate's tallies; there is nothing to close.
        session.finalize();
        let summary = build_summary(false, &session, &report.stats, Some(&gate_handle));
        emitter.summary(&summary);
        // A clean operator interrupt exits zero; every other way of capturing
        // nothing (timeout, duration, disconnect) is the target-never-acquired
        // failure that exits one. Nothing was captured, so nothing was observed.
        return Ok(CaptureOutcome::bare(
            if session.stop_reason() == Some(StopReason::Interrupt) {
                Exit::SUCCESS
            } else {
                Exit::FAILURE
            },
        ));
    }

    // Acquired: open the window from the acquiring event's capture instant, so a
    // frame captured before it (a buffered pre-acquisition frame) stays a
    // watch-time discard even though the window is now open (review of PR #26).
    // The acquiring event set `acquired_at`; fall back to the arm instant if it is
    // somehow unset (it never is once the session is Capturing).
    gate_handle.open_from(acquired_at.unwrap_or(ARMED_AT));
    // A zero volume bound is met before any packet is retained; stop for it now so
    // the reason is volume-reached (review of PR #26). drive_live observes the
    // inactive session and stops the pipeline within a tick.
    if zero_volume_bound(config) {
        session.on_volume_reached();
    }

    publisher.publish(session.role_bindings());
    // Drive an initial refresh so the count reflects the first real snapshot.
    // The pipeline's control thread also drives refreshes, but reading the count
    // through the retained stamper reader here keeps the filter-narrowed event
    // meaningful at the moment of acquisition. An attributor with nothing to
    // re-read reports it wants no refresh, so this is a no-op there.
    if let Some(s) = stamper_reader.as_ref() {
        if s.wants_refresh() {
            let _ = s.refresh();
        }
    }
    // The count of endpoints actually narrowed to: the stamper reports only
    // endpoints owned by profiled processes (slice 015), not the full
    // socket-table set the inner attributor holds. Read through the retained
    // reader, since the stamper itself is now inside the pipeline.
    let mut narration = FilterNarration::new(stamper_reader.clone());
    narration.announce_watching(emitter);
    narration.poll(emitter);

    // The merged channel. Two forwarders fold the two source channels into it so
    // the driver reads one totally ordered stream.
    let (merged_tx, merged_rx) = mpsc::channel::<DriverMsg>();
    let packet_forward = {
        let merged_tx = merged_tx.clone();
        std::thread::spawn(move || {
            while let Ok((len, ts)) = tee_rx.recv() {
                if merged_tx.send(DriverMsg::Packet(len, ts)).is_err() {
                    break;
                }
            }
        })
    };
    let event_forward = {
        let merged_tx = merged_tx.clone();
        std::thread::spawn(move || forward_process_events(rx, merged_tx))
    };
    // The two forwarders hold the only remaining senders, so the merged channel
    // disconnects once both have ended.
    drop(merged_tx);

    // extcap drives a real live capture through this same function (Codex
    // review of PR #196: `assemble::components` selects `EventStream::Live`
    // whenever the run is not an offline `--offline` replay, which includes
    // a genuine Wireshark-driven extcap session). `sink_failure_is_clean` is
    // `true` only for `extcap` (`crates/fragcap-cli/src/commands/extcap.rs`
    // passes the literal `true`; the ordinary `capture` command passes
    // `false`), so it doubles as the one marker available here for "this
    // capture must stay byte-identical to before this slice" (FR-008): the
    // live status display, including the non-terminal heartbeat, is
    // suppressed entirely for it.
    let mut display = LiveStatusDisplay::new(std::time::Instant::now(), sink_failure_is_clean);
    drive_live(
        &merged_rx,
        &mut session,
        &mut bound,
        &mut bound_images,
        &publisher,
        &gate_handle,
        &live,
        stamper_reader.as_ref(),
        config.max_bytes,
        config.max_packets,
        emitter,
        interrupt,
        &stop,
        started,
        tick,
        &mut narration,
        &mut display,
    );

    // The pipeline observed the stop and returns; its tee channel closes and the
    // packet forwarder ends. A terminal-stage exit closed the window at its own
    // capture instant inside drive_live, so a post-stop frame still draining is out
    // of window; an interrupt or duration stop left the window open, keeping what
    // was captured before it (specification FR-005).
    let report: PipelineReport = handle.join().expect("the pipeline thread did not panic");
    emit_stream_reports(&stream_reports, emitter);
    emit_ring_report(&ring_evicted, emitter);

    // Slice 015: the socket-table refresh is driven by the pipeline's own
    // section 8.6 control thread and ends with the pipeline, so there is no
    // separate refresh thread to stop here.

    // Dropping the watcher stops its ETW session, which disconnects the live
    // receiver and ends the event forwarder. Done before joining so the join
    // cannot block on a watcher that is still alive.
    let _ = components.watcher.take();
    let _ = packet_forward.join();
    let _ = event_forward.join();

    if (fire_interrupt || interrupt.load(Ordering::Relaxed)) && is_active(&session) {
        session.on_interrupt();
    }
    session.finalize();

    let summary = build_summary(true, &session, &report.stats, Some(&gate_handle));
    emitter.summary(&summary);

    Ok(CaptureOutcome {
        exit: final_exit(
            !report.sink_failures.is_empty(),
            sink_failure_is_clean,
            summary.stop_reason,
        ),
        observed_holder: report.stats.dominant_holder(),
        stop_reason: summary.stop_reason,
    })
}

/// The live driver loop over the merged channel.
///
/// Folds process events, counts packets and advances the session clock, honors
/// the interrupt, and stops the pipeline the moment the session leaves an active
/// state, which is what makes a terminal-stage exit end the run even when no
/// further packets arrive.
#[cfg(all(feature = "etw", windows))]
#[allow(clippy::too_many_arguments)]
fn drive_live(
    rx: &Receiver<DriverMsg>,
    session: &mut CaptureSession,
    bound: &mut HashMap<u32, String>,
    bound_images: &mut HashMap<u32, String>,
    publisher: &BindingPublisher,
    gate_handle: &GateHandle,
    live: &LiveStats,
    stamper: Option<&fragcap::RoleStampingAttributor>,
    byte_bound: Option<u64>,
    packet_bound: Option<u64>,
    emitter: &mut Emitter,
    interrupt: &AtomicBool,
    stop: &StopHandle,
    started: std::time::Instant,
    tick: std::time::Duration,
    narration: &mut FilterNarration,
    display: &mut LiveStatusDisplay,
) {
    loop {
        let now = std::time::Instant::now();
        let progress_before = emitter.progress_written();
        // Every wakeup, including the idle timeout. The idle case is the whole
        // point: on a `--launch` run the target is silent for tens of seconds
        // before it opens its first socket, and a narrator driven only by packet
        // arrivals would miss the transition it exists to report.
        narration.poll(emitter);
        match rx.recv_timeout(tick) {
            Ok(DriverMsg::Packet(len, ts)) => {
                session.on_packet(len);
                session.on_tick(ts);
            }
            Ok(DriverMsg::Proc(event)) => {
                // A process event carries its own capture instant. If applying it
                // stops the session (a terminal-stage or all-processes exit), close
                // the window at that instant, so a frame captured at or after the
                // exit is out of window even while the pipeline drains (review of
                // PR #26). An interrupt or duration stop has no such instant and
                // does not close the window, keeping what was captured before it.
                let event_at = event.at();
                let was_active = is_active(session);
                record_bound_image(&event, bound_images);
                apply_event(event, session, bound, emitter);
                publisher.publish(session.role_bindings());
                if was_active && !is_active(session) {
                    gate_handle.close_at(event_at);
                }
            }
            Ok(DriverMsg::WatcherLost) => session.on_process_watcher_lost(),
            // Nothing arrived. Advance the clock so a duration bound can fire.
            Err(mpsc::RecvTimeoutError::Timeout) => session.on_tick(elapsed_ts(started)),
            // Both forwarders ended: the pipeline is done and the watcher is
            // gone. Nothing more can arrive.
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        let progress_happened = emitter.progress_written() != progress_before;
        if progress_happened {
            // A real progress line (a stage match/exit, a filter-narrowing
            // announcement) just fired; the non-terminal heartbeat's whole
            // purpose is to substitute for exactly this, so it resets rather
            // than firing on top of real output (S069 Clarifications
            // session, 2026-08-22). The terminal redraw's tracked frame is
            // also forgotten, not erased, here: the progress line just
            // written sits below it now, and erasing against the old line
            // count would land the cursor in the wrong place and corrupt
            // both (Codex review of PR #196).
            display.note_progress(now);
        }
        if interrupt.load(Ordering::Relaxed) && is_active(session) {
            session.on_interrupt();
        }
        if !is_active(session) {
            stop.stop();
            break;
        }
        // Rate-limited independent of how often a message arrives (Codex
        // review of PR #196): `rx.recv_timeout(tick)` returns immediately,
        // not on the `tick` cadence, whenever a packet is already queued, so
        // a busy capture would otherwise redraw or emit a JSON event once
        // per packet rather than at the intended cadence, flooding stderr
        // and contending with the merged channel. A progress line having
        // just fired forces an out-of-cycle redraw too, so the frame
        // reappears right below it immediately rather than leaving a gap
        // until the next due tick.
        if progress_happened || display.tick_due(now) {
            let snapshot = build_live_snapshot(
                started.elapsed(),
                bound,
                bound_images,
                gate_handle,
                live,
                stamper,
                byte_bound,
                packet_bound,
            );
            display.tick(emitter, &snapshot, now);
        }
    }
    display.resolve(emitter);
}

#[cfg(all(feature = "etw", windows))]
fn forward_process_events(rx: Receiver<ProcessEvent>, merged_tx: mpsc::Sender<DriverMsg>) {
    while let Ok(event) = rx.recv() {
        if merged_tx.send(DriverMsg::Proc(event)).is_err() {
            return;
        }
    }
    let _ = merged_tx.send(DriverMsg::WatcherLost);
}

/// Everything the live status display needs across a run: which terminal
/// state was decided once at the start (redraw vs. heartbeat vs. neither),
/// and the redraw/heartbeat bookkeeping each needs. Bundled into one type so
/// `drive_live`'s per-tick call is a single line and the three behaviors
/// (redraw, heartbeat, the optional JSON event) stay mutually exclusive by
/// construction (FR-005, FR-006).
#[cfg(all(feature = "etw", windows))]
struct LiveStatusDisplay {
    redraw: live_status::redraw::RedrawState,
    heartbeat: live_status::heartbeat::Heartbeat,
    is_terminal: bool,
    /// `true` for an `extcap` capture (FR-008), which suppresses the whole
    /// display, redraw and heartbeat alike, regardless of format or
    /// verbosity (Codex review of PR #196).
    suppressed: bool,
    /// The next instant a redraw or JSON tick is due. Independent of
    /// `drive_live`'s own `tick` (the 200ms session-clock wakeup): that one
    /// fires on every message, including every packet on a busy capture,
    /// while this one paces the display itself (Codex review of PR #196).
    next_tick_at: std::time::Instant,
}

/// How often the redraw or JSON tick fires on its own cadence, independent
/// of message arrival. Within the 4-10 Hz range the issue itself suggests
/// and comfortably inside FR-001's "at least once per second."
#[cfg(all(feature = "etw", windows))]
const DISPLAY_TICK_INTERVAL: std::time::Duration = std::time::Duration::from_millis(250);

#[cfg(all(feature = "etw", windows))]
impl LiveStatusDisplay {
    fn new(now: std::time::Instant, suppressed: bool) -> Self {
        LiveStatusDisplay {
            redraw: live_status::redraw::RedrawState::new(),
            heartbeat: live_status::heartbeat::Heartbeat::new(now),
            is_terminal: live_status::is_terminal(),
            suppressed,
            next_tick_at: now + DISPLAY_TICK_INTERVAL,
        }
    }

    fn note_progress(&mut self, now: std::time::Instant) {
        self.heartbeat.note_progress(now);
        // The frame just drawn (if any) now sits above an ordinary progress
        // line rather than at the cursor; see `RedrawState::forget`'s own
        // documentation for why erasing it from here would corrupt both.
        self.redraw.forget();
    }

    /// Whether the display's own cadence, independent of message arrival,
    /// is due to fire.
    fn tick_due(&self, now: std::time::Instant) -> bool {
        now >= self.next_tick_at
    }

    /// The one call per tick: exactly one of a terminal redraw, a
    /// non-terminal heartbeat, or a JSON `capture.progress` event happens,
    /// gated on `emitter`'s own format and verbosity (FR-005, FR-006,
    /// FR-009), unless this is an extcap capture, which suppresses all
    /// three (FR-008).
    fn tick(
        &mut self,
        emitter: &mut Emitter,
        snapshot: &live_status::LiveStatusSnapshot,
        now: std::time::Instant,
    ) {
        self.next_tick_at = now + DISPLAY_TICK_INTERVAL;
        if self.suppressed {
            return;
        }
        match emitter.format() {
            Format::Json => {
                emitter.event(&capture_progress_event(snapshot));
            }
            Format::Human => {
                if emitter.verbosity() != Verbosity::Normal {
                    return;
                }
                if self.is_terminal {
                    let width = live_status::terminal_width();
                    let (text, lines) = live_status::render_status(
                        snapshot,
                        live_status::use_status_color(),
                        width,
                    );
                    let frame = self.redraw.frame(&text, lines);
                    emitter.live_write(&frame);
                } else if self.heartbeat.due(now) {
                    emitter.progress(&live_status::heartbeat::render_heartbeat(
                        snapshot.elapsed,
                        snapshot.written_packets,
                    ));
                    self.heartbeat.note_progress(now);
                }
            }
        }
    }

    /// Clear any outstanding redrawn frame before the completion summary
    /// prints, so the two never interleave (FR-012). A no-op on the
    /// non-terminal path, where nothing was ever drawn in place.
    fn resolve(&mut self, emitter: &mut Emitter) {
        if self.is_terminal {
            let clear = self.redraw.clear();
            if !clear.is_empty() {
                emitter.live_write(&clear);
            }
        }
    }
}

/// Assemble one tick's [`live_status::LiveStatusSnapshot`] from the handles
/// `drive_live` already holds. The one call site this crate has that reads a
/// live pipeline's counters mid-run; everything it reads is already
/// documented as safe to read without blocking the capture or output
/// threads (research R-2).
#[cfg(all(feature = "etw", windows))]
#[allow(clippy::too_many_arguments)]
fn build_live_snapshot(
    elapsed: std::time::Duration,
    bound: &HashMap<u32, String>,
    bound_images: &HashMap<u32, String>,
    gate_handle: &GateHandle,
    live: &LiveStats,
    stamper: Option<&fragcap::RoleStampingAttributor>,
    byte_bound: Option<u64>,
    packet_bound: Option<u64>,
) -> live_status::LiveStatusSnapshot {
    // Prefer the `target` role's binding when one exists, so a run with more
    // than one bound stage (a launcher alongside the target, for example)
    // shows the process an operator actually cares about rather than
    // whichever pid happens to sort lowest (Copilot review of PR #196). The
    // smallest-pid fallback still applies, and stays deterministic, for the
    // rarer case of no `target`-role binding at all.
    let process = bound
        .iter()
        .filter(|(_, role)| role.as_str() == "target")
        .min_by_key(|(pid, _)| **pid)
        .or_else(|| bound.iter().min_by_key(|(pid, _)| **pid))
        .map(|(pid, role)| live_status::BoundProcess {
            name: bound_images.get(pid).cloned(),
            pid: *pid,
            role: role.clone(),
            stage: None,
        });
    let active_endpoints = stamper.map(|s| s.active_endpoints().len()).unwrap_or(0);

    live_status::LiveStatusSnapshot {
        elapsed,
        process,
        written_packets: gate_handle.admitted(),
        written_bytes: gate_handle.admitted_bytes(),
        byte_bound,
        packet_bound,
        active_endpoints,
        narrowed: active_endpoints > 0,
        watch_discarded: gate_handle.watch_discarded(),
        out_of_window_discarded: gate_handle.out_of_window_discarded(),
        scope_discarded: gate_handle.scope_discarded(),
        scope_unresolved_discarded: gate_handle.scope_unresolved_discarded(),
        buffer_dropped: live.buffer_dropped(),
        sink_dropped: live.sink_dropped(),
        holder_tally: live.holder_tally_snapshot(),
    }
}

/// The optional `capture.progress` JSON event (FR-009), carrying the same
/// scalar counters as the human status block but no holder-tally breakdown
/// (`contracts/capture-progress-event.md`).
#[cfg(all(feature = "etw", windows))]
fn capture_progress_event(snapshot: &live_status::LiveStatusSnapshot) -> Event {
    Event::CaptureProgress {
        elapsed_secs: snapshot.elapsed.as_secs(),
        packets: snapshot.written_packets,
        bytes: snapshot.written_bytes,
        active_endpoints: snapshot.active_endpoints,
        watching_discarded: snapshot.watch_discarded,
        discarded_out_of_window: snapshot.out_of_window_discarded,
        buffer_dropped: snapshot.buffer_dropped,
        sink_dropped: snapshot.sink_dropped,
        scope_discarded: snapshot.scope_discarded,
        scope_unresolved_discarded: snapshot.scope_unresolved_discarded,
    }
}

/// Real elapsed time as a session [`Timestamp`], measured from the arm instant.
#[cfg(all(feature = "etw", windows))]
fn elapsed_ts(started: std::time::Instant) -> Timestamp {
    Timestamp::from_nanos(ARMED_AT.as_nanos() + started.elapsed().as_nanos() as i64)
}

/// Record a started process's image name by pid, for the live status
/// display's header line (slice S069). A no-op for any other event kind
/// (`image_name` returns an empty string for one, which is never recorded).
#[cfg(all(feature = "etw", windows))]
fn record_bound_image(event: &ProcessEvent, bound_images: &mut HashMap<u32, String>) {
    if let ProcessEvent::Started { pid, .. } = event {
        let image = assemble::image_name(event);
        if !image.is_empty() {
            bound_images.insert(*pid, image);
        }
    }
}

/// Fold one process event and emit the match or exit it produced.
fn apply_event(
    event: ProcessEvent,
    session: &mut CaptureSession,
    bound: &mut HashMap<u32, String>,
    emitter: &mut Emitter,
) {
    let exited_pid = match &event {
        ProcessEvent::Exited { pid, .. } => Some(*pid),
        _ => None,
    };
    let image = assemble::image_name(&event);

    session.on_process_event(event);

    let current: HashMap<u32, String> = session
        .role_bindings()
        .into_iter()
        .map(|(pid, role, _)| (pid, role.map(|r| r.to_string()).unwrap_or_default()))
        .collect();

    for (pid, role) in &current {
        if !bound.contains_key(pid) {
            emitter.event(&Event::StageMatched {
                role: role.clone(),
                pid: *pid,
                process: image.clone(),
            });
            emitter.progress(&format!("stage matched: {role} pid {pid} {image}"));
        }
    }

    if let Some(pid) = exited_pid {
        if let Some(role) = bound.get(&pid) {
            emitter.event(&Event::StageExited {
                role: role.clone(),
                pid,
            });
            emitter.progress(&format!("stage exited: {role} pid {pid}"));
        }
    }

    *bound = current;
}

fn is_active(session: &CaptureSession) -> bool {
    matches!(
        session.state(),
        SessionState::Arming | SessionState::Watching | SessionState::Capturing
    )
}

/// Whether a zero volume bound is configured (`--max-packets 0` or
/// `--max-bytes 0`). Such a bound is met before any packet is retained, so the
/// session's per-packet volume check never fires it; the driver stops for it
/// explicitly after acquisition (review of PR #26).
fn zero_volume_bound(config: &EffectiveConfig) -> bool {
    config.max_packets == Some(0) || config.max_bytes == Some(0)
}

/// Reports the kernel filter narrowing, from the transition rather than from a
/// sample.
///
/// Both capture paths used to read `active_endpoints().len()` once, at
/// acquisition, and print `filter narrowed to N endpoint(s)`. Three things were
/// wrong with that and issue #185 records all three. The wording inverts the
/// meaning: zero endpoints means *no* narrowing has happened and fragcap is
/// still capturing everything, which reads as though the run gave up. The sample
/// is taken at the one instant where zero is close to guaranteed, because on a
/// `--launch` run acquisition happens when the process starts, many seconds
/// before the title touches the network. And it is never updated, so the
/// transition that actually matters, capture ceasing to be machine-wide, is
/// invisible; on the run that prompted the issue the filter narrowed at t+22.5s
/// and the terminal's last line described a moment sixteen minutes stale.
/// How often the narrator samples the endpoint set. Slow enough to cost nothing
/// on the packet path, fast enough that the transition reads as immediate.
const SAMPLE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

struct FilterNarration {
    stamper: Option<fragcap::RoleStampingAttributor>,
    /// The last count reported, so only transitions are announced.
    last: usize,
    /// Whether the first narrowing has been announced in human output. Later
    /// changes go to the structured stream only, so a busy target does not
    /// produce a line per socket.
    announced: bool,
    /// When the endpoint set was last sampled.
    ///
    /// [`FilterNarration::poll`] is called from the driver loops, which run once
    /// per packet, so the sample is rate-limited here rather than at every call
    /// site. A human-visible transition does not need finer resolution than this
    /// and the packet path must not pay for one.
    last_sampled: Option<std::time::Instant>,
    /// The image the run is capturing, when the caller knows it.
    ///
    /// Left `None` today on both paths. The image is printed on the `stage
    /// matched: <role> pid <n> <image>` line immediately above these, so the
    /// operator has it in view; carrying it here as well would mean threading
    /// the narrator through `apply_event` and its three callers for a second
    /// copy of a name already on screen. The field exists so a caller that does
    /// have it can say it.
    target: Option<String>,
}

impl FilterNarration {
    fn new(stamper: Option<fragcap::RoleStampingAttributor>) -> Self {
        FilterNarration {
            stamper,
            last: 0,
            announced: false,
            target: None,
            last_sampled: None,
        }
    }

    /// Announce that capture is machine-wide until the target opens a socket.
    ///
    /// Said in words that carry their own meaning: an operator should not have
    /// to know what an endpoint is, or that zero of them means the opposite of
    /// what it sounds like.
    fn announce_watching(&self, emitter: &mut Emitter) {
        let what = self.target.as_deref().unwrap_or("the target");
        emitter.progress(&format!(
            "capturing all traffic while {what} opens its first connection"
        ));
    }

    /// Check for a transition and report one if it happened.
    ///
    /// Called from the driver loops on every packet and, on the live path, on
    /// every idle tick. The idle case is the one that matters: on a `--launch`
    /// run the target is silent for tens of seconds before it opens its first
    /// socket, so a narrator driven only by packet arrivals would not notice the
    /// transition until traffic it was waiting for started flowing.
    fn poll(&mut self, emitter: &mut Emitter) {
        let Some(stamper) = self.stamper.as_ref() else {
            return;
        };
        // Rate-limit the sample, not the transition. Reading the endpoint set is
        // cheap but not free, and this runs on the packet path.
        let now = std::time::Instant::now();
        if let Some(last) = self.last_sampled {
            if now.duration_since(last) < SAMPLE_INTERVAL {
                return;
            }
        }
        self.last_sampled = Some(now);
        let now = stamper.active_endpoints().len();
        if now == self.last {
            return;
        }
        self.last = now;
        // The structured stream gets every transition: a machine consumer wants
        // the series, not one sample of zero (issue #185).
        emitter.event(&Event::FilterNarrowed { endpoints: now });
        if now > 0 && !self.announced {
            self.announced = true;
            let what = self.target.as_deref().unwrap_or("the target");
            emitter.progress(&format!(
                "now capturing {what} only ({now} connection(s) matched)"
            ));
        }
    }
}

fn build_summary(
    acquired: bool,
    session: &CaptureSession,
    stats: &CaptureStats,
    gate: Option<&GateHandle>,
) -> CompletionSummary {
    // The gate is the authority for what was written and what was discarded by
    // cause (slice 017): its admitted count is the packets on disk, and its
    // watch-time and out-of-window tallies are the discards the summary reports.
    // Sourcing these from the gate rather than the session is what makes the
    // summary match the produced file. On the target-never-acquired path there is
    // no gate (the pipeline never started), and the session's own counters, which
    // are zero there, are used instead.
    let (retained, watching, out_of_window, out_of_scope, scope_unresolved) = match gate {
        Some(g) => (
            g.admitted(),
            g.watch_discarded(),
            g.out_of_window_discarded(),
            g.scope_discarded(),
            g.scope_unresolved_discarded(),
        ),
        None => {
            let s = session.stats();
            (
                s.retained,
                s.watching_discarded,
                s.discarded_out_of_window,
                0,
                0,
            )
        }
    };
    // What reached the file, per image, largest first, from the tally the output
    // loop already accumulates over gate-admitted packets (slice S059). Ties break
    // to the earlier image name so two runs over identical traffic report
    // identically (P-9: no coin flip).
    let mut written_by_image: Vec<(String, u64)> = stats
        .holder_tally
        .iter()
        .map(|(image, count)| (image.to_string(), *count))
        .collect();
    written_by_image.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    CompletionSummary {
        acquired,
        stop_reason: session.stop_reason(),
        packets_captured: stats.packets_captured,
        retained,
        packets_attributed: stats.packets_attributed,
        packets_unattributed: stats.packets_unattributed,
        watching_discarded: watching,
        discarded_out_of_window: out_of_window,
        buffer_dropped: stats.buffer_dropped,
        sink_dropped: stats.sink_dropped,
        scope_discarded: out_of_scope,
        scope_unresolved_discarded: scope_unresolved,
        written_by_image,
    }
}

#[cfg(test)]
mod tests {
    use super::{final_exit, PlatformDispatchGate};
    #[cfg(all(feature = "etw", windows))]
    use super::{forward_process_events, DriverMsg};
    use fragcap::{StageId, StopReason};
    use std::sync::Arc;

    #[cfg(all(feature = "etw", windows))]
    #[test]
    fn process_forwarder_reports_watcher_loss_after_acquisition() {
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let (merged_tx, merged_rx) = std::sync::mpsc::channel();
        drop(event_tx);

        forward_process_events(event_rx, merged_tx);

        assert!(matches!(merged_rx.recv(), Ok(DriverMsg::WatcherLost)));
    }

    #[test]
    fn platform_dispatch_waits_for_the_exact_root_and_is_issued_once() {
        let mut gate = PlatformDispatchGate::default();
        gate.arm(Some(42));
        let wrong_pid = vec![(
            41,
            Some(Arc::from("platform")),
            Some(StageId::new("platform")),
        )];
        let wrong_role = vec![(42, Some(Arc::from("client")), Some(StageId::new("client")))];
        let exact = vec![(
            42,
            Some(Arc::from("platform")),
            Some(StageId::new("platform")),
        )];
        assert!(!gate.observe(&wrong_pid));
        assert!(!gate.observe(&wrong_role));
        assert!(gate.observe(&exact));
        assert!(!gate.observe(&exact), "the retained dispatch is one-shot");
    }

    // A run that ended with no sink failure is a success on any surface.
    #[test]
    fn no_sink_failure_is_always_a_success() {
        assert_eq!(final_exit(false, false, None).code(), 0);
        assert_eq!(final_exit(false, true, None).code(), 0);
    }

    // A sink failure is an unrecoverable end for `run` and `tap` (exit 1), but the
    // defined clean stop for an extcap capture, whose single sink is the
    // analyzer's FIFO: the analyst closing it ends the run at exit 0, with the
    // loss accounting still surfaced in the summary (Codex review of PR #34).
    #[test]
    fn a_sink_failure_ends_cleanly_only_for_an_extcap_capture() {
        assert_eq!(
            final_exit(true, false, None).code(),
            1,
            "run/tap: a sink failure is a failure"
        );
        assert_eq!(
            final_exit(true, true, None).code(),
            0,
            "extcap: a FIFO disconnect is a clean stop"
        );
    }

    #[test]
    fn an_incomplete_or_ambiguous_chain_is_a_failure() {
        assert_eq!(
            final_exit(false, false, Some(StopReason::AcquisitionTimeout)).code(),
            1
        );
        assert_eq!(
            final_exit(false, false, Some(StopReason::AmbiguousStageMatch)).code(),
            1
        );
    }
}

// S069 T030, T031, T032. `LiveStatusDisplay::tick` is the one call site that
// decides among a terminal redraw, a non-terminal heartbeat, and the JSON
// `capture.progress` event; these tests exercise the decision directly,
// independent of a real ETW/live capture, per research R-5.
#[cfg(all(feature = "etw", windows))]
#[cfg(test)]
mod live_status_display_tests {
    use super::*;
    use crate::emit::{Format, Verbosity};
    use crate::live_status::LiveStatusSnapshot;
    use std::time::Duration;

    fn snapshot() -> LiveStatusSnapshot {
        LiveStatusSnapshot {
            elapsed: Duration::from_secs(1),
            process: None,
            written_packets: 1,
            written_bytes: 2,
            byte_bound: None,
            packet_bound: None,
            active_endpoints: 0,
            narrowed: false,
            watch_discarded: 0,
            out_of_window_discarded: 0,
            scope_discarded: 0,
            scope_unresolved_discarded: 0,
            buffer_dropped: 0,
            sink_dropped: 0,
            holder_tally: Vec::new(),
        }
    }

    fn run(format: Format, verbosity: Verbosity, is_terminal: bool) -> (String, bool) {
        let mut buf: Vec<u8> = Vec::new();
        let mut emitter = Emitter::new(&mut buf, format, verbosity);
        let now = std::time::Instant::now();
        let mut display = LiveStatusDisplay {
            redraw: live_status::redraw::RedrawState::new(),
            heartbeat: live_status::heartbeat::Heartbeat::new(now),
            is_terminal,
            suppressed: false,
            next_tick_at: now,
        };
        display.tick(&mut emitter, &snapshot(), now);
        let text = String::from_utf8(buf).unwrap();
        let has_escape = text.contains('\x1b');
        (text, has_escape)
    }

    #[test]
    fn json_format_emits_capture_progress_and_never_a_redraw_or_heartbeat() {
        let (text, has_escape) = run(Format::Json, Verbosity::Normal, true);
        assert!(text.contains("\"event\":\"capture.progress\""));
        assert!(!has_escape, "no redraw in json mode");
        assert!(
            !text.contains("still capturing"),
            "no heartbeat in json mode"
        );
    }

    #[test]
    fn human_terminal_normal_draws_a_frame_with_no_prior_erase() {
        let (text, has_escape) = run(Format::Human, Verbosity::Normal, true);
        assert!(!has_escape, "the first frame has nothing to erase yet");
        assert!(text.contains("fragcap"));
    }

    #[test]
    fn human_non_terminal_never_draws_a_redraw_frame() {
        // A fresh Heartbeat's interval starts at construction, so it is not
        // due on this immediate tick; this test asserts the terminal branch
        // is correctly skipped, not heartbeat timing (already covered in
        // `live_status::heartbeat`'s own tests).
        let (text, has_escape) = run(Format::Human, Verbosity::Normal, false);
        assert!(
            !has_escape,
            "the non-terminal path never writes an escape byte"
        );
        assert!(
            !text.contains("fragcap"),
            "no redraw block on a non-terminal stream"
        );
    }

    #[test]
    fn quiet_and_silent_suppress_the_human_display_entirely() {
        for verbosity in [Verbosity::Quiet, Verbosity::Silent] {
            let (terminal_text, _) = run(Format::Human, verbosity, true);
            assert!(
                terminal_text.is_empty(),
                "quiet/silent must suppress the terminal redraw"
            );
            let (non_terminal_text, _) = run(Format::Human, verbosity, false);
            assert!(
                non_terminal_text.is_empty(),
                "quiet/silent must suppress the heartbeat too"
            );
        }
    }

    // S069 T038, SC-002. The standing regression test SC-002 names by name:
    // a non-terminal run, driven across several ticks mixing ordinary
    // progress lines and at least one due heartbeat, must never write an
    // escape byte anywhere in the whole captured stream. T024 and T032 each
    // cover one piece of this (the heartbeat line alone, stdout isolation);
    // this test is the one that scans the *whole* accumulated non-terminal
    // stream, which is the specific claim SC-002 makes.
    #[test]
    fn a_non_terminal_run_across_several_ticks_never_writes_an_escape_byte() {
        let mut buf: Vec<u8> = Vec::new();
        let mut emitter = Emitter::new(&mut buf, Format::Human, Verbosity::Normal);
        let start = std::time::Instant::now();
        let mut display = LiveStatusDisplay {
            redraw: live_status::redraw::RedrawState::new(),
            heartbeat: live_status::heartbeat::Heartbeat::new(start),
            is_terminal: false,
            suppressed: false,
            next_tick_at: start,
        };

        // Ordinary progress lines interleave with ticks, exactly as
        // `drive_live`'s loop calls `emitter.progress` (via `apply_event` or
        // `narration.poll`) before each `display.tick` call.
        emitter.progress("stage matched: target pid 44460 AngelLegion.exe");
        display.tick(&mut emitter, &snapshot(), start);

        // No further progress line for a long stretch: the heartbeat must
        // fire without ever emitting a redraw or a color code.
        let later = start + Duration::from_secs(31);
        display.tick(&mut emitter, &snapshot(), later);

        let even_later = later + Duration::from_secs(31);
        display.tick(&mut emitter, &snapshot(), even_later);

        let text = String::from_utf8(buf).unwrap();
        assert!(
            !text.contains('\x1b'),
            "a non-terminal run must never write an escape byte, found in: {text:?}"
        );
        assert!(
            text.contains("stage matched"),
            "the ordinary progress line must still appear"
        );
        assert!(
            text.matches("still capturing").count() >= 1,
            "at least one heartbeat line must have fired across two 31-second gaps"
        );
    }

    // S069, Codex P1 review of PR #196. `tick_due` is `drive_live`'s own
    // gate against redrawing or emitting a JSON event once per packet on a
    // busy capture; this exercises the gate directly, independent of
    // `tick()`'s own content (which the tests above already cover).
    #[test]
    fn tick_due_paces_independently_of_how_often_it_is_polled() {
        let start = std::time::Instant::now();
        let mut display = LiveStatusDisplay {
            redraw: live_status::redraw::RedrawState::new(),
            heartbeat: live_status::heartbeat::Heartbeat::new(start),
            is_terminal: true,
            suppressed: false,
            next_tick_at: start + DISPLAY_TICK_INTERVAL,
        };
        // Polling many times within the interval, as a busy capture's
        // per-packet loop iterations would, must not be due even once.
        for millis in 0..DISPLAY_TICK_INTERVAL.as_millis() as u64 {
            assert!(
                !display.tick_due(start + std::time::Duration::from_millis(millis)),
                "must not be due before the interval elapses"
            );
        }
        assert!(display.tick_due(start + DISPLAY_TICK_INTERVAL));

        let mut buf: Vec<u8> = Vec::new();
        let mut emitter = Emitter::new(&mut buf, Format::Human, Verbosity::Normal);
        display.tick(&mut emitter, &snapshot(), start + DISPLAY_TICK_INTERVAL);
        // A call to `tick` reschedules the next deadline forward, so a
        // caller checking `tick_due` again immediately after is not due
        // again until the next full interval.
        assert!(!display.tick_due(start + DISPLAY_TICK_INTERVAL));
    }

    // S069, Codex P1 review of PR #196. `sink_failure_is_clean` is `true`
    // only for `extcap` (`commands/extcap.rs` passes the literal `true`);
    // `LiveStatusDisplay::new` takes it directly as `suppressed`, so this
    // asserts the whole display, redraw and heartbeat alike, goes silent
    // for that case regardless of format or verbosity (FR-008).
    #[test]
    fn an_extcap_capture_suppresses_the_whole_display() {
        let now = std::time::Instant::now();
        for is_terminal in [true, false] {
            let mut buf: Vec<u8> = Vec::new();
            let mut emitter = Emitter::new(&mut buf, Format::Human, Verbosity::Normal);
            let mut display = LiveStatusDisplay::new(now, true);
            display.is_terminal = is_terminal;
            display.tick(&mut emitter, &snapshot(), now + Duration::from_secs(60));
            let text = String::from_utf8(buf).unwrap();
            assert!(
                text.is_empty(),
                "an extcap capture (is_terminal={is_terminal}) must emit nothing from the \
                 live display, found: {text:?}"
            );
        }

        let mut buf: Vec<u8> = Vec::new();
        let mut emitter = Emitter::new(&mut buf, Format::Json, Verbosity::Normal);
        let mut display = LiveStatusDisplay::new(now, true);
        display.tick(&mut emitter, &snapshot(), now);
        assert!(
            String::from_utf8(buf).unwrap().is_empty(),
            "an extcap capture must not emit capture.progress either"
        );
    }
}
