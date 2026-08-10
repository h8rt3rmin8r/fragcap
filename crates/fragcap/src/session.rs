// SPDX-License-Identifier: Apache-2.0

//! The capture session lifecycle, specification sections 10.4 through 10.6.
//!
//! A session moves through five states. It arms before any target exists, so the
//! capture handle is open and the watcher attached before the launcher
//! authentication exchange that section 5.2 says is the most information-dense
//! traffic of the session. While no stage has matched it discards packets, and
//! it counts every one it discards (constitution P-4). On the first stage match
//! it begins retaining, at no setup cost because the handle is already open. It
//! stops on the first of six conditions, and every one produces the same orderly
//! shutdown.
//!
//! ```text
//! Arming --attach--> Watching --first match--> Capturing --stop--> Draining --finalize--> Complete
//!                    Watching --acquisition timeout--> Complete
//! ```
//!
//! This slice models the lifecycle as a decision over an event and packet
//! stream, so the whole of sections 10.4 through 10.6 is tested against scripted
//! inputs with no capture driver, no elevation, and no game. Wiring the session
//! to a live `PacketSource`, installing filters, and stamping the role and stage
//! onto an [`Attribution`](fragcap_core::attribution::Attribution) are S13 and
//! S14.

use std::time::Duration;

use fragcap_core::attribution::StageId;
use fragcap_core::packet::Timestamp;
use fragcap_core::process::{ProcessEvent, ProcessId, ProcessTree};

use fragcap_profile::matching::stage_for;
use fragcap_profile::schema::{Lifecycle, Profile};

/// The five states of specification section 10.5.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionState {
    /// Opening the capture handle and attaching the watcher, before any target.
    Arming,
    /// Armed, no target matched, discarding packets.
    Watching,
    /// A stage has matched; packets are retained.
    Capturing,
    /// A stop condition was met; the buffer is draining and sinks are finishing.
    Draining,
    /// Shutdown is complete and the capture is valid.
    Complete,
}

/// Why capture ended, per specification section 10.6.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    /// The elapsed duration reached the configured bound.
    DurationReached,
    /// The captured bytes or packets reached the configured bound.
    VolumeReached,
    /// The profile's terminal stage exited.
    TerminalStageExited,
    /// Every matched non-service process exited and no stage remained awaited.
    AllProcessesExited,
    /// An operator interrupt was received.
    Interrupt,
    /// An unrecoverable sink error occurred.
    SinkError,
    /// No target was acquired before the acquisition timeout elapsed. The one
    /// reason reached from Watching rather than Capturing.
    AcquisitionTimeout,
}

/// What the session decided for one packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PacketDisposition {
    /// Dropped, because no target was acquired yet. Counted, per P-4.
    Discarded,
    /// Kept, because a target is being captured.
    Retained,
}

/// The bounds an operator sets on a session.
///
/// Every bound is optional. The acquisition timeout and the duration are
/// measured from the instant the session was armed (decision D-5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SessionConfig {
    /// How long Watching waits for a target before giving up. Unset means the
    /// session waits, ending instead by the duration bound or an interrupt.
    pub acquisition_timeout: Option<Duration>,
    /// The wall-clock bound on the whole session, from arm.
    pub duration: Option<Duration>,
    /// The retained-packet bound.
    pub packet_bound: Option<u64>,
    /// The retained-byte bound.
    pub byte_bound: Option<u64>,
}

/// The session's own accounting.
///
/// Separate from [`CaptureStats`](fragcap_core::stats::CaptureStats) because the
/// Watching discard happens upstream of the pipeline whose conservation identity
/// that structure carries (decision D-4). `WatcherReport` and `SourceStats` set
/// the same precedent: a component's own counts are a value the run assembles
/// alongside the capture's. The discard is nonetheless named and surfaced, which
/// is what P-4 requires.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SessionStats {
    /// Packets discarded while Watching, before a target was acquired.
    pub watching_discarded: u64,
    /// Packets retained while Capturing.
    pub retained: u64,
    /// Bytes retained while Capturing.
    pub retained_bytes: u64,
}

impl SessionStats {
    /// Every packet the session saw while armed. Retained plus discarded, and
    /// nothing else, which is the session's conservation identity.
    pub fn observed(&self) -> u64 {
        self.watching_discarded.saturating_add(self.retained)
    }
}

/// One bound process, tracked for the stop conditions of section 10.6.
///
/// Only the fields the stop conditions read are kept: the operating system
/// identifier the exit event names, whether the stage is terminal, whether it is
/// a service (which section 10.4 never awaits), and whether the process is still
/// live.
struct Binding {
    pid: u32,
    terminal: bool,
    service: bool,
    live: bool,
}

/// A capture session driven by an event and packet stream.
pub struct CaptureSession {
    state: SessionState,
    profile: Profile,
    config: SessionConfig,
    tree: ProcessTree,
    stats: SessionStats,
    armed_at: Option<Timestamp>,
    stop: Option<StopReason>,
    bindings: Vec<Binding>,
    /// Roles of non-service stages that have not yet bound a process. A stage is
    /// awaited only while its role is in this set.
    pending_nonservice: Vec<String>,
    /// Count of bound non-service processes still live.
    live_nonservice: u64,
}

impl CaptureSession {
    /// A new session in Arming, over a validated profile.
    pub fn new(profile: Profile, config: SessionConfig) -> Self {
        let pending_nonservice = profile
            .stages()
            .iter()
            .filter(|s| s.lifecycle() != Lifecycle::Service)
            .map(|s| s.role().to_string())
            .collect();
        CaptureSession {
            state: SessionState::Arming,
            profile,
            config,
            tree: ProcessTree::new(),
            stats: SessionStats::default(),
            armed_at: None,
            stop: None,
            bindings: Vec::new(),
            pending_nonservice,
            live_nonservice: 0,
        }
    }

    /// Arm the session: the capture handle is open and the watcher attached.
    /// Arming to Watching. A no-op if the session is not in Arming.
    pub fn attach(&mut self, at: Timestamp) {
        if self.state == SessionState::Arming {
            self.state = SessionState::Watching;
            self.armed_at = Some(at);
        }
    }

    /// Fold one process event into the session: apply it to the tree, match and
    /// bind a start, handle a bound exit. The event carries its own timestamp;
    /// wall-clock bounds are checked in [`on_tick`](CaptureSession::on_tick).
    pub fn on_process_event(&mut self, event: ProcessEvent) {
        if !self.is_active() {
            return;
        }
        let at = event.at();
        let pid = event.pid();
        let is_start = matches!(event, ProcessEvent::Started { .. });
        let is_exit = matches!(event, ProcessEvent::Exited { .. });
        self.tree.apply(event);

        if is_start {
            self.match_and_bind(pid, at);
        } else if is_exit {
            self.on_bound_exit(pid);
        }
    }

    /// One packet arrived. Discard and count while Watching, retain and count
    /// while Capturing, and drop uncounted in any other state (nothing is
    /// captured before arm or after drain).
    pub fn on_packet(&mut self, len: u32) -> PacketDisposition {
        match self.state {
            SessionState::Watching => {
                self.stats.watching_discarded = self.stats.watching_discarded.saturating_add(1);
                PacketDisposition::Discarded
            }
            SessionState::Capturing => {
                self.stats.retained = self.stats.retained.saturating_add(1);
                self.stats.retained_bytes = self.stats.retained_bytes.saturating_add(len as u64);
                self.check_volume_bounds();
                PacketDisposition::Retained
            }
            _ => PacketDisposition::Discarded,
        }
    }

    /// The clock advanced. Fire the acquisition timeout from Watching, and the
    /// duration bound from any active state, both measured from arm.
    pub fn on_tick(&mut self, now: Timestamp) {
        if self.state == SessionState::Watching {
            if let (Some(timeout), Some(armed)) = (self.config.acquisition_timeout, self.armed_at) {
                if elapsed(armed, now) >= timeout {
                    self.stop = Some(StopReason::AcquisitionTimeout);
                    self.state = SessionState::Complete;
                    return;
                }
            }
        }
        if self.is_active() {
            if let (Some(dur), Some(armed)) = (self.config.duration, self.armed_at) {
                if elapsed(armed, now) >= dur {
                    self.stop(StopReason::DurationReached);
                }
            }
        }
    }

    /// An operator interrupt: a normal stop, not an abort.
    pub fn on_interrupt(&mut self) {
        self.stop(StopReason::Interrupt);
    }

    /// A sink failed unrecoverably.
    pub fn on_sink_error(&mut self) {
        self.stop(StopReason::SinkError);
    }

    /// Finish draining: Draining to Complete. A no-op in any other state.
    pub fn finalize(&mut self) {
        if self.state == SessionState::Draining {
            self.state = SessionState::Complete;
        }
    }

    /// The current state.
    pub fn state(&self) -> SessionState {
        self.state
    }

    /// The session's accounting.
    pub fn stats(&self) -> SessionStats {
        self.stats
    }

    /// Why capture stopped, once it has.
    pub fn stop_reason(&self) -> Option<StopReason> {
        self.stop
    }

    /// The process tree, carrying the bindings this session applied.
    pub fn tree(&self) -> &ProcessTree {
        &self.tree
    }

    // -- internals --------------------------------------------------------

    fn is_active(&self) -> bool {
        matches!(
            self.state,
            SessionState::Arming | SessionState::Watching | SessionState::Capturing
        )
    }

    fn match_and_bind(&mut self, pid: u32, at: Timestamp) {
        let Some(id) = self.tree.resolve(ProcessId(pid), at) else {
            return;
        };
        let decision = stage_for(&self.profile, &self.tree, id).map(|s| {
            (
                StageId::new(s.role()),
                s.lifecycle(),
                s.is_terminal(),
                s.role().to_string(),
            )
        });
        let Some((sid, lifecycle, terminal, role)) = decision else {
            return;
        };
        if self.tree.bind_stage(id, sid) {
            let service = lifecycle == Lifecycle::Service;
            if !service {
                self.pending_nonservice.retain(|r| r != &role);
                self.live_nonservice = self.live_nonservice.saturating_add(1);
            }
            self.bindings.push(Binding {
                pid,
                terminal,
                service,
                live: true,
            });
            if self.state == SessionState::Watching {
                self.state = SessionState::Capturing;
            }
        }
    }

    fn on_bound_exit(&mut self, pid: u32) {
        let mut hit: Option<(bool, bool)> = None; // (terminal, service)
        if let Some(b) = self.bindings.iter_mut().find(|b| b.pid == pid && b.live) {
            b.live = false;
            hit = Some((b.terminal, b.service));
        }
        let Some((terminal, service)) = hit else {
            return;
        };
        if !service {
            self.live_nonservice = self.live_nonservice.saturating_sub(1);
        }
        if terminal {
            self.stop(StopReason::TerminalStageExited);
            return;
        }
        // All matched non-service processes have exited and no non-service stage
        // is still awaited. A live service does not gate this (decision D-6).
        if self.state == SessionState::Capturing
            && self.pending_nonservice.is_empty()
            && self.live_nonservice == 0
        {
            self.stop(StopReason::AllProcessesExited);
        }
    }

    fn check_volume_bounds(&mut self) {
        let hit_packets = self
            .config
            .packet_bound
            .is_some_and(|b| self.stats.retained >= b);
        let hit_bytes = self
            .config
            .byte_bound
            .is_some_and(|b| self.stats.retained_bytes >= b);
        if hit_packets || hit_bytes {
            self.stop(StopReason::VolumeReached);
        }
    }

    /// Enter the shutdown common to every stop condition. Sets the reason and
    /// moves to Draining, from which [`finalize`](CaptureSession::finalize)
    /// completes. A no-op once the session is no longer active, so the first
    /// stop condition to occur wins.
    fn stop(&mut self, reason: StopReason) {
        if self.is_active() {
            self.stop = Some(reason);
            self.state = SessionState::Draining;
        }
    }
}

/// Elapsed time from `from` to `to`, clamped at zero (time does not run
/// backward across a session's own clock).
fn elapsed(from: Timestamp, to: Timestamp) -> Duration {
    Duration::from_nanos(to.nanos_since(from).max(0) as u64)
}
