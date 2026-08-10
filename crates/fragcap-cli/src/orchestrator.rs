// SPDX-License-Identifier: Apache-2.0

//! The capture engine shared by `run` and `tap`.
//!
//! The pipeline owns the packet threads and the shared attributor; a session
//! driver owns the [`CaptureSession`], and the two connect through a
//! [`StopHandle`] and a published binding snapshot rather than by routing
//! packets through the session (slice S14 research D-c). A [`TeeCountingSink`]
//! prepended to the sink list forwards each retained packet's length and
//! instant to the driver, which is what keeps the session the single authority
//! for the volume bound and its counters while it never sees the packet path
//! (D-e).
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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Once;

use fragcap::core::CaptureStats;
// The stamper's active_endpoints (profiled-filtered, slice 015) is read for the
// filter-narrowed event; the trait brings the method into scope.
use fragcap::core::FlowAttributor;
#[cfg(all(feature = "etw", windows))]
use fragcap::StopReason;
use fragcap::{
    BindingPublisher, CaptureSession, CapturedPacket, Pipeline, PipelineConfig, PipelineReport,
    ProcessEvent, Profile, SessionState, Sink, SinkError, StopHandle, Timestamp,
};

use crate::args::Direction;
use crate::assemble::{self, CaptureComponents, EffectiveConfig, EventStream, ARMED_AT};
use crate::emit::Emitter;
use crate::events::Event;
use crate::exit::{CliError, Exit};
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

/// A sink that counts and forwards, writing nothing of its own.
///
/// First in the sink list, so it is inside the pipeline's conservation identity
/// (it receives every packet and refuses none) and its receipts feed the
/// session. It forwards each packet's retained length and instant to the driver
/// over an unbounded channel, so a `write` never blocks the output thread.
struct TeeCountingSink {
    tx: Sender<(u32, Timestamp)>,
}

impl Sink for TeeCountingSink {
    fn write(&mut self, packet: &CapturedPacket) -> Result<(), SinkError> {
        let len = packet.data.as_ref().len() as u32;
        // A send fails only once the driver has dropped its receiver, which
        // happens only as the run is torn down. Nothing the session needed is
        // lost, so the failure is ignored.
        let _ = self.tx.send((len, packet.ts));
        Ok(())
    }

    fn flush(&mut self) -> Result<(), SinkError> {
        Ok(())
    }

    fn finish(self: Box<Self>, _stats: &CaptureStats) -> Result<(), SinkError> {
        Ok(())
    }
}

/// Run a capture to completion and return its exit code.
///
/// The single entry both `run` and `tap` reach, once each has resolved or
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
) -> Result<Exit, CliError> {
    let mut session = CaptureSession::new_scoped(profile, config.session_config(), allowed_roles);
    session.attach(ARMED_AT);

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
    // Roles are enforced: the session was scoped above, so a role outside the
    // set never triggers or is captured (specification FR-011b). Direction, by
    // contrast, is still only recorded, which the warning above says.
    let roles = config
        .roles
        .as_ref()
        .map(|r| r.join(","))
        .unwrap_or_else(|| "all".to_string());
    emitter.progress(&format!(
        "scope: direction {}, roles {} (enforced), loopback {}",
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
        ),
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
) -> Result<Exit, CliError> {
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
        let summary = build_summary(false, &session, &CaptureStats::default());
        emitter.summary(&summary);
        return Ok(Exit::FAILURE);
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
    let endpoints = components
        .stamper
        .as_ref()
        .map(|s| s.active_endpoints().len())
        .unwrap_or(0);
    emitter.event(&Event::FilterNarrowed { endpoints });
    emitter.progress(&format!("filter narrowed to {endpoints} endpoint(s)"));

    // The events not consumed during acquisition, folded during capture.
    let mut pending: Vec<ProcessEvent> = events[cursor..].to_vec();

    // Build the sinks, prepend the tee, and run the pipeline on its own thread.
    let (tx, rx) = mpsc::channel::<(u32, Timestamp)>();
    let (handle, stop) = spawn_pipeline(config, &mut components, tx)?;

    drive(
        &rx,
        &mut session,
        &mut bound,
        &mut pending,
        &publisher,
        emitter,
        interrupt,
        &stop,
    );

    let report: PipelineReport = handle.join().expect("the pipeline thread did not panic");

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

    let summary = build_summary(true, &session, &report.stats);
    emitter.summary(&summary);

    if report.sink_failures.is_empty() {
        Ok(Exit::SUCCESS)
    } else {
        // An unrecoverable sink failure ended the run; the output may be partial
        // (specification FR-005a). Not a usage error.
        Ok(Exit::FAILURE)
    }
}

/// Build the sinks, prepend the counting tee, and spawn the pipeline on its own
/// thread. Shared by both drivers so the two construct the output path
/// identically and the offline bytes cannot drift from the live ones.
fn spawn_pipeline(
    config: &EffectiveConfig,
    components: &mut CaptureComponents,
    tee_tx: Sender<(u32, Timestamp)>,
) -> Result<(std::thread::JoinHandle<PipelineReport>, StopHandle), CliError> {
    let sinks = assemble::build_sinks(config, &components.interfaces)?;
    let stamper = components
        .stamper
        .take()
        .expect("the stamper is taken exactly once");

    let mut pipeline = Pipeline::new(
        std::mem::take(&mut components.sources),
        Box::new(stamper),
        PipelineConfig::default(),
    )?;
    pipeline.add_sink(Box::new(TeeCountingSink { tx: tee_tx }));
    for sink in sinks {
        pipeline.add_sink(sink);
    }
    let stop = pipeline.stop_handle();
    let handle = std::thread::spawn(move || pipeline.run());
    Ok((handle, stop))
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
) -> Result<Exit, CliError> {
    use std::time::{Duration, Instant};

    let mut bound: HashMap<u32, String> = HashMap::new();
    let publisher = components.publisher.clone();

    // Elapsed time is real and monotonic, converted onto the session clock from
    // ARMED_AT so the session's tick-driven bounds see wall time pass.
    let started = Instant::now();
    let tick = Duration::from_millis(200);

    // Acquisition: fold live events until a terminal stage acquires the target,
    // or acquisition ends for another reason (the acquisition timeout, a
    // duration bound reached while still watching, an operator interrupt, or the
    // watcher disconnecting).
    loop {
        if session.state() == SessionState::Capturing {
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
                apply_event(event, &mut session, &mut bound, emitter);
                publisher.publish(session.role_bindings());
            }
            // No event arrived; advance the session clock so a tick-based
            // acquisition timeout or duration bound can still fire.
            Err(mpsc::RecvTimeoutError::Timeout) => session.on_tick(elapsed_ts(started)),
            // The watcher is gone; no target can now be acquired.
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    if session.state() != SessionState::Capturing {
        // No target was acquired. Fire the acquisition timeout, if set and the
        // session is still watching, so the summary can name it; an interrupt or
        // a duration bound has already stopped the session and set its reason.
        if is_active(&session) {
            if let Some(timeout) = config.acquisition_timeout {
                session.on_tick(Timestamp::from_nanos(
                    ARMED_AT.as_nanos() + timeout.as_nanos() as i64,
                ));
            }
        }
        session.finalize();
        let summary = build_summary(false, &session, &CaptureStats::default());
        emitter.summary(&summary);
        // A clean operator interrupt exits zero; every other way of capturing
        // nothing (timeout, duration, disconnect) is the target-never-acquired
        // failure that exits one.
        return Ok(if session.stop_reason() == Some(StopReason::Interrupt) {
            Exit::SUCCESS
        } else {
            Exit::FAILURE
        });
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
    let endpoints = components
        .stamper
        .as_ref()
        .map(|s| s.active_endpoints().len())
        .unwrap_or(0);
    emitter.event(&Event::FilterNarrowed { endpoints });
    emitter.progress(&format!("filter narrowed to {endpoints} endpoint(s)"));

    // The same pipeline the offline path builds, feeding a counting tee.
    let (tee_tx, tee_rx) = mpsc::channel::<(u32, Timestamp)>();
    let (handle, stop) = spawn_pipeline(config, &mut components, tee_tx)?;

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
        std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                if merged_tx.send(DriverMsg::Proc(event)).is_err() {
                    break;
                }
            }
        })
    };
    // The two forwarders hold the only remaining senders, so the merged channel
    // disconnects once both have ended.
    drop(merged_tx);

    drive_live(
        &merged_rx,
        &mut session,
        &mut bound,
        &publisher,
        emitter,
        interrupt,
        &stop,
        started,
        tick,
    );

    // The pipeline observed the stop and returns; its tee channel closes and the
    // packet forwarder ends.
    let report: PipelineReport = handle.join().expect("the pipeline thread did not panic");

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

    let summary = build_summary(true, &session, &report.stats);
    emitter.summary(&summary);

    if report.sink_failures.is_empty() {
        Ok(Exit::SUCCESS)
    } else {
        Ok(Exit::FAILURE)
    }
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
    publisher: &BindingPublisher,
    emitter: &mut Emitter,
    interrupt: &AtomicBool,
    stop: &StopHandle,
    started: std::time::Instant,
    tick: std::time::Duration,
) {
    loop {
        match rx.recv_timeout(tick) {
            Ok(DriverMsg::Packet(len, ts)) => {
                session.on_packet(len);
                session.on_tick(ts);
            }
            Ok(DriverMsg::Proc(event)) => {
                apply_event(event, session, bound, emitter);
                publisher.publish(session.role_bindings());
            }
            // Nothing arrived. Advance the clock so a duration bound can fire.
            Err(mpsc::RecvTimeoutError::Timeout) => session.on_tick(elapsed_ts(started)),
            // Both forwarders ended: the pipeline is done and the watcher is
            // gone. Nothing more can arrive.
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if interrupt.load(Ordering::Relaxed) && is_active(session) {
            session.on_interrupt();
        }
        if !is_active(session) {
            stop.stop();
            break;
        }
    }
}

/// Real elapsed time as a session [`Timestamp`], measured from the arm instant.
#[cfg(all(feature = "etw", windows))]
fn elapsed_ts(started: std::time::Instant) -> Timestamp {
    Timestamp::from_nanos(ARMED_AT.as_nanos() + started.elapsed().as_nanos() as i64)
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

fn build_summary(
    acquired: bool,
    session: &CaptureSession,
    stats: &CaptureStats,
) -> CompletionSummary {
    let s = session.stats();
    CompletionSummary {
        acquired,
        stop_reason: session.stop_reason(),
        packets_captured: stats.packets_captured,
        packets_attributed: stats.packets_attributed,
        packets_unattributed: stats.packets_unattributed,
        watching_discarded: s.watching_discarded,
        discarded_out_of_window: s.discarded_out_of_window,
        buffer_dropped: stats.buffer_dropped,
        sink_dropped: stats.sink_dropped,
    }
}
