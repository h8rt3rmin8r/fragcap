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

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fragcap_core::attribution::{Attribution, StageId};
use fragcap_core::error::AttrError;
use fragcap_core::flow::{Endpoint, FlowKey};
use fragcap_core::packet::{CapturedPacket, Timestamp};
use fragcap_core::process::{ProcessEvent, ProcessId, ProcessRecord, ProcessTree};
use fragcap_core::traits::{FlowAttributor, WriteGate};

use fragcap_profile::matching::{stage_for, stage_identity_holds};
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
    /// No target was acquired, or an awaited terminal stage did not bind,
    /// before the acquisition timeout elapsed.
    AcquisitionTimeout,
    /// More than one process satisfied one declared stage identity. No later
    /// candidate is bound or promoted to terminal ownership.
    AmbiguousStageMatch,
    /// The exact owned platform root exited before its terminal client bound.
    PlatformExitedBeforeClient,
    /// The declared client identity appeared outside the exact platform tree.
    EscapedPlatformClient,
    /// The retained title dispatch failed after the platform root was observed.
    PlatformDispatchFailed,
    /// The exact prepared platform root could not be created.
    PlatformStartFailed,
    /// Process lifecycle observation ended before terminal ownership completed.
    ProcessWatcherLost,
}

/// What the session decided for one packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PacketDisposition {
    /// Dropped, because no target was acquired yet. Counted, per P-4.
    Discarded,
    /// Kept, because a target is being captured.
    Retained,
}

/// What a capture writes out, as distinct from what it observes.
///
/// Specification section 12.3 places this decision in userspace, on every
/// packet, independent of whatever kernel filter happens to be installed. Slice
/// S064 is where that was implemented; before it the only gate tested the
/// capture window and the volume bound, so everything on the wire reached the
/// file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CaptureScope {
    /// Only what the operator asked for: a packet whose attribution carries a
    /// bound stage or role, and whose role is inside the `--roles` set.
    ///
    /// The default. fragcap's claim is process attribution, and a user reading
    /// that expects the file to hold their target's traffic. Before slice S064
    /// it held everything on the wire: the first real end-to-end run returned a
    /// file that was 91 percent other processes' traffic, because the narrowed
    /// kernel filter cannot engage until the target opens its first socket and
    /// nothing in the write path made the decision that specification section
    /// 12.3 places in userspace.
    #[default]
    Target,
    /// Everything captured, which is the behavior before slice S064.
    ///
    /// Still a real use: correlating the target against the rest of the machine,
    /// and debugging attribution itself, both of which need the traffic that
    /// resolves to nothing or to something else.
    All,
}

impl CaptureScope {
    /// Whether this scope admits a packet with the given attribution.
    ///
    /// `allowed_roles` is the `--roles` set, or `None` for unscoped. A packet is
    /// "bound" when the session's binding snapshot stamped it, which is the same
    /// condition that puts `role=` and `stage=` into the written packet comment,
    /// so the file's contents and the file's own annotations cannot disagree.
    fn admits(self, attribution: Option<&Attribution>, allowed_roles: Option<&[String]>) -> bool {
        match self {
            CaptureScope::All => true,
            CaptureScope::Target => {
                let Some(attr) = attribution else {
                    return false;
                };
                let bound = attr.stage.is_some() || attr.role.is_some();
                if !bound {
                    return false;
                }
                // Belt and braces, and knowingly so. `CaptureSession::match_and_bind`
                // already refuses to bind a stage whose role is outside the set,
                // so a stamped packet's role is always allowed and this test
                // cannot currently fail. It is kept because the gate should
                // assert its own contract rather than inherit it from an
                // invariant three modules away: if binding ever stamps a stage it
                // does not trigger on, retention stays scoped without anyone
                // having to remember this coupling.
                match (allowed_roles, attr.role.as_deref()) {
                    // Unscoped roles: every bound stage is in scope.
                    (None, _) => true,
                    // Scoped roles, and the packet names one: membership decides.
                    (Some(allowed), Some(role)) => allowed.iter().any(|r| r == role),
                    // Scoped roles, and the packet carries a stage but no role.
                    // Admitted: it is bound to the profile, and refusing it would
                    // drop the target's own traffic over a stage that simply did
                    // not name a role.
                    (Some(_), None) => true,
                }
            }
        }
    }
}

/// The bounds an operator sets on a session, and what it writes out.
///
/// Every bound is optional. The acquisition timeout and the duration are
/// measured from the instant the session was armed (decision D-5).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
    /// What reaches the sinks. Defaults to [`CaptureScope::Target`].
    pub scope: CaptureScope,
    /// The `--roles` set, or `None` when unscoped. Read only under
    /// [`CaptureScope::Target`], where it is what makes the run's `roles ...
    /// (enforced)` line true of packet retention rather than only of which
    /// stages trigger acquisition.
    pub allowed_roles: Option<Vec<String>>,
    /// Apply S111's exact publisher-chain ownership rules: one process per
    /// declared role, and keep the acquisition deadline active until the
    /// terminal stage binds. False for ordinary authored profiles.
    pub exact_stage_ownership: bool,
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
    /// Packets offered while the session was neither Watching nor Capturing:
    /// before it armed, or after a stop condition moved it to Draining. Counted
    /// so that no packet handed to the session escapes the accounting, which is
    /// the whole of P-4; a discard path without a counter is a defect.
    pub discarded_out_of_window: u64,
}

impl SessionStats {
    /// Every packet handed to the session: retained, discarded while watching,
    /// or discarded out of window, and nothing else. This is the session's
    /// conservation identity, and it holds for every call to
    /// [`on_packet`](CaptureSession::on_packet) regardless of state.
    pub fn observed(&self) -> u64 {
        self.watching_discarded
            .saturating_add(self.retained)
            .saturating_add(self.discarded_out_of_window)
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
    role: String,
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
    /// Whether an allowed terminal stage has not yet bound exactly once. An
    /// acquisition timeout remains active while this is true, even after a
    /// launcher or intermediate stage begins capture.
    pending_terminal: bool,
    /// The roles the run was scoped to with `--roles` (specification FR-011b).
    /// `None` means unscoped: every stage triggers and is captured. `Some` means
    /// only a stage whose role is in the set may bind, so a stage outside it
    /// never becomes pending, never stamps, and never influences the stop
    /// conditions, exactly as if it were not in the profile.
    allowed_roles: Option<Vec<String>>,
}

impl CaptureSession {
    /// A new session in Arming, over a validated profile. Unscoped: every stage
    /// the profile declares may trigger and be captured.
    pub fn new(profile: Profile, config: SessionConfig) -> Self {
        Self::new_scoped(profile, config, None)
    }

    /// A new session scoped to a set of roles, per specification FR-011b.
    ///
    /// `allowed_roles` of `None` is the unscoped session [`new`](Self::new)
    /// builds. `Some` restricts the run to those roles: a stage whose role is
    /// not in the set is treated as if it were absent from the profile. It never
    /// enters the pending set, so it never gates acquisition or
    /// `AllProcessesExited`; it never binds in
    /// [`match_and_bind`](Self::match_and_bind), so it never stamps a role or
    /// stage and a terminal one outside the set never stops the run. This keeps
    /// existing callers working: `new` delegates here with no restriction.
    pub fn new_scoped(
        profile: Profile,
        config: SessionConfig,
        allowed_roles: Option<Vec<String>>,
    ) -> Self {
        let allowed = |role: &str| role_in(&allowed_roles, role);
        let pending_nonservice = profile
            .stages()
            .iter()
            .filter(|s| s.lifecycle() != Lifecycle::Service)
            .filter(|s| allowed(s.role()))
            .map(|s| s.role().to_string())
            .collect();
        let pending_terminal = config.exact_stage_ownership
            && profile
                .stages()
                .iter()
                .any(|s| s.is_terminal() && allowed(s.role()));
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
            pending_terminal,
            allowed_roles,
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

    /// Fold a startup snapshot of already-running processes into the session and
    /// match them, so a process already running when the session armed can acquire
    /// the target without a later start event (attach-to-running, section 15.7).
    ///
    /// The process watcher takes a query-only toolhelp snapshot at arm; the
    /// capture path folds it here. Each snapshot process is matched exactly as a
    /// start event is, in creation order so `descends_from` resolves against an
    /// ancestor bound first, and a non-service match transitions Watching to
    /// Capturing just as [`on_process_event`](Self::on_process_event) does. It
    /// opens no process handle and reads only the image name and path the snapshot
    /// already carries (constitution P-1).
    ///
    /// A no-op unless the session is active. Idempotent: a node already bound is
    /// left alone, because [`ProcessTree::bind_stage`] binds at most once, so
    /// applying the same snapshot twice acquires nothing new.
    pub fn apply_snapshot(&mut self, records: &[ProcessRecord], at: Timestamp) {
        if !self.is_active() {
            return;
        }
        // Fold parent-first. The tree resolves a snapshot record's ancestry link
        // against nodes already folded, with no retroactive linking, and toolhelp
        // enumeration gives no creation-order guarantee, so a child can precede
        // its parent in the raw snapshot. Folding in that order would leave the
        // child's ancestry unresolved and a `descends_from` stage would never bind
        // (review of PR #84). Ordering parent-first makes the ancestry link and
        // the node-identifier match order below both correct.
        let ordered = parent_first(records);
        self.tree.apply_snapshot_at(&ordered, at);
        // Match in creation (node identifier) order, which now visits a parent
        // before its children, so an ancestor a `descends_from` predicate names is
        // bound before its descendant is evaluated, exactly as `bind_stages` and
        // the offline acquisition loop do.
        let mut nodes: Vec<(u32, u32)> = self
            .tree
            .nodes()
            .map(|n| (n.id().get(), n.pid().0))
            .collect();
        nodes.sort_by_key(|(node_id, _)| *node_id);
        for (_, pid) in nodes {
            self.match_and_bind(pid, at);
        }
    }

    /// One packet arrived. Discard and count while Watching, retain and count
    /// while Capturing, and in any other state discard into the out-of-window
    /// counter (a packet still draining in after a stop, or one handed over
    /// before the session armed): counted, never silently dropped, per P-4.
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
            SessionState::Arming | SessionState::Draining | SessionState::Complete => {
                self.stats.discarded_out_of_window =
                    self.stats.discarded_out_of_window.saturating_add(1);
                PacketDisposition::Discarded
            }
        }
    }

    /// The clock advanced. Fire the acquisition timeout while no stage has
    /// acquired or while an allowed terminal stage is still pending, and the
    /// duration bound from any active state. Both are measured from arm.
    pub fn on_tick(&mut self, now: Timestamp) {
        if self.is_active() && (self.state == SessionState::Watching || self.pending_terminal) {
            if let (Some(timeout), Some(armed)) = (self.config.acquisition_timeout, self.armed_at) {
                if elapsed(armed, now) >= timeout {
                    if self.state == SessionState::Watching {
                        self.stop = Some(StopReason::AcquisitionTimeout);
                        self.state = SessionState::Complete;
                    } else {
                        self.stop(StopReason::AcquisitionTimeout);
                    }
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

    /// Stop for a volume bound that was already met.
    ///
    /// The volume bound is normally reached through [`on_packet`](Self::on_packet)
    /// as retained packets accumulate. A zero bound (`--max-packets 0` or
    /// `--max-bytes 0`) is met before any packet is retained, so no `on_packet`
    /// ever runs to fire it; the write gate rejects every packet and forwards no
    /// receipt. The driver calls this once, immediately after acquisition, when a
    /// zero bound is configured, so the stop reason is the promised
    /// [`VolumeReached`](StopReason::VolumeReached) rather than a later
    /// source-exhausted reason. It cannot be folded into the acquisition
    /// transition, because the offline driver detects acquisition by the session
    /// resting in [`Capturing`](SessionState::Capturing), which an immediate stop
    /// would skip past. A no-op once the session is no longer active.
    pub fn on_volume_reached(&mut self) {
        self.stop(StopReason::VolumeReached);
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

    /// The stage bindings the session has applied, as `(pid, role, stage)`.
    ///
    /// The snapshot source for [`RoleStampingAttributor`]. A node bound to a
    /// stage carries the stage's role name, and the stage identifier is that
    /// same name (a stage binds as `StageId::new(role)`), so both are reported:
    /// the role for [`Attribution::with_role`] and the stage for
    /// [`Attribution::with_stage`]. A node with no bound stage is omitted, so
    /// the orchestrator republishes only what has matched.
    pub fn role_bindings(&self) -> Vec<(u32, Option<Arc<str>>, Option<StageId>)> {
        self.tree
            .nodes()
            .filter_map(|n| {
                n.stage()
                    .map(|s| (n.pid().get(), Some(Arc::from(s.as_str())), Some(s.clone())))
            })
            .collect()
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
        if self.tree.node(id).and_then(|node| node.stage()).is_some() {
            return;
        }
        let decision = stage_for(&self.profile, &self.tree, id).map(|s| {
            (
                StageId::new(s.role()),
                s.lifecycle(),
                s.is_terminal(),
                s.role().to_string(),
            )
        });
        let Some((sid, lifecycle, terminal, role)) = decision else {
            if self.owns_platform_client()
                && self.profile.stages().iter().any(|stage| {
                    stage.is_terminal()
                        && role_in(&self.allowed_roles, stage.role())
                        && stage.predicates().descends_from() == Some("platform")
                        && stage_identity_holds(stage, &self.tree, id)
                })
            {
                self.stop(StopReason::EscapedPlatformClient);
            }
            return;
        };
        // A stage outside the scoped role set is treated as if it were not in
        // the profile: no bind, so no stamp, no live count, and no stop
        // condition (specification FR-011b).
        if !role_in(&self.allowed_roles, &role) {
            return;
        }
        if self.config.exact_stage_ownership
            && self.bindings.iter().any(|binding| binding.role == role)
        {
            self.stop(StopReason::AmbiguousStageMatch);
            return;
        }
        if self.tree.bind_stage(id, sid) {
            let service = lifecycle == Lifecycle::Service;
            let platform = role == "platform";
            // A process whose exit was delivered before its start is bound
            // already exited: the tree joins a held exit on the start event, so
            // the node is not live even though this is its first appearance to
            // the session. Such a binding must not add to the live count.
            let already_exited = self.tree.node(id).map(|n| !n.is_live()).unwrap_or(false);
            if !service {
                self.pending_nonservice.retain(|r| r != &role);
                if !already_exited {
                    self.live_nonservice = self.live_nonservice.saturating_add(1);
                }
            }
            self.bindings.push(Binding {
                pid,
                role,
                terminal,
                service,
                live: !already_exited,
            });
            if terminal {
                self.pending_terminal = false;
            }
            // Only a non-service match acquires the target. Section 10.4: a
            // service is never awaited during acquisition, so a persistent
            // service that appears while Watching binds for attribution but does
            // not begin capturing, which would otherwise disable the acquisition
            // timeout and retain service noise before any target exists.
            if !service && self.state == SessionState::Watching {
                self.state = SessionState::Capturing;
            }
            // A target already gone by the time we see it start is the exit it
            // is: a terminal that has exited still stops capture, and a stale
            // live count cannot otherwise block AllProcessesExited.
            if already_exited {
                if platform && self.owns_platform_client() && self.pending_terminal {
                    self.stop(StopReason::PlatformExitedBeforeClient);
                } else {
                    self.note_bound_exit(terminal);
                }
            }
        }
    }

    fn on_bound_exit(&mut self, pid: u32) {
        let mut hit: Option<(bool, bool, bool)> = None; // (terminal, service, platform)
        if let Some(b) = self.bindings.iter_mut().find(|b| b.pid == pid && b.live) {
            b.live = false;
            hit = Some((b.terminal, b.service, b.role == "platform"));
        }
        let Some((terminal, service, platform)) = hit else {
            return;
        };
        if platform && self.owns_platform_client() && self.pending_terminal {
            self.stop(StopReason::PlatformExitedBeforeClient);
            return;
        }
        if !service {
            self.live_nonservice = self.live_nonservice.saturating_sub(1);
        }
        self.note_bound_exit(terminal);
    }

    /// Apply the stop conditions an exit can trigger. The caller has already
    /// adjusted the live count; this decides whether the exit ends capture.
    fn note_bound_exit(&mut self, terminal: bool) {
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

    /// Record failure of the separately authorized title dispatch.
    pub fn on_platform_dispatch_failure(&mut self) {
        self.stop(StopReason::PlatformDispatchFailed);
    }

    /// Record failure to create the exact prepared platform root.
    pub fn on_platform_start_failure(&mut self) {
        self.stop(StopReason::PlatformStartFailed);
    }

    /// Record loss of lifecycle observation while acquisition is incomplete.
    pub fn on_process_watcher_lost(&mut self) {
        self.stop(StopReason::ProcessWatcherLost);
    }

    fn owns_platform_client(&self) -> bool {
        self.config.exact_stage_ownership
            && self.profile.game().platform().is_some()
            && self
                .profile
                .stages()
                .iter()
                .any(|stage| stage.role() == "platform" && stage.lifecycle() == Lifecycle::Service)
    }
}

/// Elapsed time from `from` to `to`, clamped at zero (time does not run
/// backward across a session's own clock).
fn elapsed(from: Timestamp, to: Timestamp) -> Duration {
    Duration::from_nanos(to.nanos_since(from).max(0) as u64)
}

/// Whether a role is in scope: always so when the run is unscoped (`None`),
/// otherwise only when the set names it.
fn role_in(allowed: &Option<Vec<String>>, role: &str) -> bool {
    allowed
        .as_ref()
        .is_none_or(|set| set.iter().any(|r| r == role))
}

/// Order snapshot records so a parent precedes its children.
///
/// The tree resolves a snapshot record's ancestry against nodes already folded,
/// with no retroactive linking, and toolhelp enumeration gives no creation-order
/// guarantee, so folding a child before its parent leaves the child's ancestry
/// unresolved. A stable Kahn-style pass emits a record once its parent is
/// external to the snapshot (a root) or already emitted. Records left in a cycle,
/// which real process ancestry does not form, are appended in input order so none
/// is dropped. The record count is a process count, so the quadratic worst case
/// is not a concern.
fn parent_first(records: &[ProcessRecord]) -> Vec<ProcessRecord> {
    use std::collections::HashSet;
    let present: HashSet<u32> = records.iter().map(|r| r.pid).collect();
    let mut emitted: HashSet<u32> = HashSet::new();
    let mut order: Vec<ProcessRecord> = Vec::with_capacity(records.len());
    loop {
        let mut progressed = false;
        for r in records {
            if emitted.contains(&r.pid) {
                continue;
            }
            let parent_ready = !present.contains(&r.parent) || emitted.contains(&r.parent);
            if parent_ready {
                order.push(r.clone());
                emitted.insert(r.pid);
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    for r in records {
        if !emitted.contains(&r.pid) {
            order.push(r.clone());
        }
    }
    order
}

/// A published map from process identifier to its role and stage.
type BindingMap = HashMap<u32, (Option<Arc<str>>, Option<StageId>)>;

/// A handle that publishes new role and stage bindings to a live
/// [`RoleStampingAttributor`].
///
/// The orchestrator keeps a clone of this after the attributor is handed to the
/// pipeline, so that when the session binds a new stage it republishes the
/// updated snapshot. The write is rare (a process start or exit) and swaps an
/// `Arc` under a short-held lock; the per-packet read on the capture threads
/// clones the current `Arc` without contending with a writer for longer than
/// the swap, which is what keeps the acquisition path off a lock it holds.
#[derive(Clone, Default)]
pub struct BindingPublisher {
    cell: Arc<Mutex<Arc<BindingMap>>>,
}

impl BindingPublisher {
    /// Replace the published snapshot with these bindings.
    ///
    /// Takes the whole set rather than a delta: the session already knows every
    /// binding it has applied ([`CaptureSession::role_bindings`]), and
    /// republishing the whole set is simpler than reconciling a delta and
    /// cannot leave a stale entry behind.
    pub fn publish(&self, bindings: Vec<(u32, Option<Arc<str>>, Option<StageId>)>) {
        let map: BindingMap = bindings
            .into_iter()
            .map(|(pid, role, stage)| (pid, (role, stage)))
            .collect();
        *self.cell.lock().expect("binding publisher lock") = Arc::new(map);
    }

    /// The current snapshot, cloned by pointer.
    fn snapshot(&self) -> Arc<BindingMap> {
        Arc::clone(&self.cell.lock().expect("binding publisher lock"))
    }
}

/// A [`FlowAttributor`] decorator that stamps the profile role and stage onto
/// an attribution the inner attributor resolved.
///
/// The seam that joins the capture session's profile knowledge to the packet
/// path without either the pipeline or the attribution crate learning about
/// profiles (constitution P-3). It wraps the real attributor as a shared
/// `Arc<dyn FlowAttributor>`, resolves through it, and then, if the resolved
/// process identifier is in the published binding map, applies
/// [`Attribution::with_role`] and [`Attribution::with_stage`]. Those fields
/// already exist on [`Attribution`], so this changes no type; it only populates
/// what the writers already emit.
///
/// It performs no packet acquisition and adds no drop path, so P-3 and P-4 both
/// hold. It lives in the facade `session` module because that is the one place
/// already above both `fragcap-capture` and `fragcap-attr`.
///
/// `Clone` shares the inner attributor and the binding publisher by pointer, so
/// a clone reads the same published snapshot. The orchestrator keeps a clone as
/// an endpoint reader when the stamper itself is moved into the pipeline on the
/// live run-from-arm path (slice 017).
#[derive(Clone)]
pub struct RoleStampingAttributor {
    inner: Arc<dyn FlowAttributor>,
    publisher: BindingPublisher,
}

impl RoleStampingAttributor {
    /// Wrap the real attributor. Bindings are empty until the first
    /// [`BindingPublisher::publish`].
    pub fn new(inner: Arc<dyn FlowAttributor>) -> Self {
        RoleStampingAttributor {
            inner,
            publisher: BindingPublisher::default(),
        }
    }

    /// A handle the orchestrator keeps to publish bindings after this
    /// attributor has been handed to the pipeline.
    pub fn publisher(&self) -> BindingPublisher {
        self.publisher.clone()
    }
}

impl FlowAttributor for RoleStampingAttributor {
    fn resolve(&self, key: &FlowKey, at: Timestamp) -> Option<Attribution> {
        let mut attribution = self.inner.resolve(key, at)?;
        let snapshot = self.publisher.snapshot();
        if let Some((role, stage)) = snapshot.get(&attribution.pid) {
            if let Some(role) = role {
                attribution = attribution.with_role(role);
            }
            if let Some(stage) = stage {
                attribution = attribution.with_stage(stage.clone());
            }
        }
        Some(attribution)
    }

    /// Forwards to the inner attributor. Slice 015: the pipeline control thread
    /// drives the refresh through the shared `Arc<dyn FlowAttributor>`, which is
    /// this decorator, so the refresh must reach the real attributor it wraps
    /// rather than stopping here. Before 015 `refresh` was `&mut self` and could
    /// not be forwarded, which is why the CLI kept a separate refresh thread.
    fn refresh(&self) -> Result<(), AttrError> {
        self.inner.refresh()
    }

    /// Forwards to the inner attributor, so the control thread learns the
    /// section 11.2 cadence of whatever real attributor this wraps.
    fn wants_refresh(&self) -> bool {
        self.inner.wants_refresh()
    }

    /// The active endpoints restricted to those belonging to profiled processes.
    ///
    /// Specification section 12.2 narrows the kernel filter to endpoints owned by
    /// profiled processes. This decorator holds the session's binding snapshot,
    /// whose keys are exactly the stage-bound (profiled) process identifiers, so
    /// it is the one place that can perform the join. It asks the inner attributor
    /// for owner-carrying endpoints and keeps an endpoint when its owner is a
    /// profiled process.
    ///
    /// An endpoint whose owner is not known is kept rather than dropped: on the
    /// live socket-table backend every endpoint carries an owning identifier, so
    /// this reduces to admitting only profiled endpoints; on the offline scripted
    /// substrate no endpoint carries one, and dropping them would narrow a
    /// declared capture to nothing. "Restrict to profiled processes" is therefore
    /// read as "exclude endpoints known to belong to a non-profiled process,"
    /// which is exactly the live-backend behavior and a pass-through offline.
    fn active_endpoints(&self) -> Vec<Endpoint> {
        let snapshot = self.publisher.snapshot();
        let mut endpoints: Vec<Endpoint> = self
            .inner
            .active_endpoints_owned()
            .into_iter()
            .filter(|owned| match owned.owner {
                Some(pid) => snapshot.contains_key(&pid),
                None => true,
            })
            .map(|owned| owned.endpoint)
            .collect();
        // One endpoint can arrive as several owner candidates (a profiled owner
        // and, after port reuse, an unprofiled one), and more than one profiled
        // candidate is possible; deduplicate the endpoints that survive the
        // filter so a reused port is admitted once rather than compiled twice.
        endpoints.sort();
        endpoints.dedup();
        endpoints
    }
}

/// The capture window a [`SessionGate`] reads to decide whether a packet is
/// admitted, expressed as the half-open interval of capture instants
/// `[admit_from, admit_until)`.
///
/// A packet is classified by its own capture timestamp, not by the window state
/// at the moment the output thread happens to process it. That distinction is
/// load-bearing: the bounded buffer sits between capture and the gate, so a frame
/// captured while watching can be processed after the target is acquired, and a
/// frame captured after a stop can still be draining. Comparing the packet's own
/// instant against the acquisition and stop instants classifies each frame by
/// when it was captured, so a buffered pre-acquisition frame stays a watch-time
/// discard and a post-stop frame stays out of window regardless of how the drain
/// races the transition. On the live path both the packet instant (the pcap
/// header) and the event instant that opens or closes the window (the ETW event
/// header) are Unix wall-clock, so they are directly comparable. Offline the
/// interval opens at `i64::MIN` before the pipeline starts, so every replayed
/// frame is in window and the classification reduces to the bound alone.
///
/// The two bounds are single-writer atomics (only the driver writes them, through
/// the handle), read lock-free by the output thread (specification section 11.6's
/// discipline, applied to the write decision).
const NANOS_WATCHING: i64 = i64::MAX;
const NANOS_NO_STOP: i64 = i64::MAX;

/// The atomics a [`SessionGate`] and its [`GateHandle`] share.
///
/// Every field is either immutable after construction (the bounds) or an atomic
/// with a single writer: the driver writes `admit_from` and `admit_until` (through
/// the handle), and the output thread (the only caller of [`WriteGate::admit`])
/// writes the tallies and `bound_hit`. The driver reads the tallies after the
/// pipeline has joined, which is a happens-before point, so `Relaxed` ordering
/// suffices throughout.
///
/// The tee sender is deliberately NOT here: it lives on the [`SessionGate`] alone,
/// so it is dropped when the pipeline finishes and the driver's read loop, which
/// waits on the receiver, ends. Sharing it would keep the channel open for as long
/// as the driver held its handle, which is the whole run.
struct GateShared {
    /// The first capture instant admitted (the acquisition instant, in nanos).
    /// A packet captured before it is a watch-time frame. `NANOS_WATCHING`
    /// (`i64::MAX`) before acquisition, so every frame is watch-time; the driver
    /// sets it to the acquisition instant (or `i64::MIN` offline, admitting all).
    admit_from: AtomicI64,
    /// The first capture instant no longer admitted (the stop instant, in nanos).
    /// A packet captured at or after it is out of window. `NANOS_NO_STOP`
    /// (`i64::MAX`) until a stop with a known capture instant closes the window
    /// (a terminal-stage exit); an interrupt or duration stop leaves it open, so
    /// what was captured before the stop is kept (specification FR-005).
    admit_until: AtomicI64,
    /// The retained-packet bound, if any.
    packet_bound: Option<u64>,
    /// The retained-byte bound, if any.
    byte_bound: Option<u64>,
    /// Packets admitted to the sinks.
    admitted: AtomicU64,
    /// Bytes admitted to the sinks (captured length, matching what the sinks
    /// write and what the S14 tee forwarded).
    admitted_bytes: AtomicU64,
    /// Packets discarded because they were captured before the acquisition
    /// instant (watch-time frames).
    watch_discarded: AtomicU64,
    /// Packets discarded because they were captured at or after the stop instant,
    /// or beyond the bound.
    out_of_window_discarded: AtomicU64,
    /// What reaches the sinks (slice S064).
    scope: CaptureScope,
    /// The `--roles` set, read only under [`CaptureScope::Target`].
    allowed_roles: Option<Vec<String>>,
    /// Packets discarded on scope grounds whose attribution named a process this
    /// capture does not cover. Confidently not the capture's.
    ///
    /// In practice that means a process no profile stage binds, which is the
    /// common case. It also covers a packet whose bound role falls outside the
    /// `--roles` set, which cannot happen while binding is itself role-gated but
    /// is counted here if it ever does, so the counter's name stays true to what
    /// it counts rather than to today's binding semantics.
    scope_discarded: AtomicU64,
    /// Packets discarded on scope grounds carrying no attribution at all.
    ///
    /// Kept apart from `scope_discarded` deliberately, and this is the split
    /// most likely to be "simplified" by a later reader. These packets *might*
    /// have been the target's, dropped because the socket table had not yet
    /// published the socket that would have named them. Folding the two would
    /// hide a possible real loss inside an intended exclusion, which is exactly
    /// the P-4 failure a scope gate risks introducing. A non-zero value here on
    /// a real capture is a signal to investigate, not an expected outcome.
    scope_unresolved_discarded: AtomicU64,
    /// Set the moment a bound is reached. Informational: beyond-bound packets are
    /// already rejected by the admitted-count comparison.
    bound_hit: AtomicBool,
}

impl GateShared {
    fn new(config: &SessionConfig) -> Self {
        GateShared {
            admit_from: AtomicI64::new(NANOS_WATCHING),
            admit_until: AtomicI64::new(NANOS_NO_STOP),
            packet_bound: config.packet_bound,
            byte_bound: config.byte_bound,
            admitted: AtomicU64::new(0),
            admitted_bytes: AtomicU64::new(0),
            watch_discarded: AtomicU64::new(0),
            out_of_window_discarded: AtomicU64::new(0),
            scope: config.scope,
            allowed_roles: config.allowed_roles.clone(),
            scope_discarded: AtomicU64::new(0),
            scope_unresolved_discarded: AtomicU64::new(0),
            bound_hit: AtomicBool::new(false),
        }
    }
}

/// A [`WriteGate`] driven by a capture session's decision.
///
/// The synchronous authority for what reaches the sinks. It admits a packet only
/// while its published window is open (the session is capturing) and the
/// configured bound has not been reached, and discards and counts every other
/// packet by cause. Because the admit-or-discard decision is made on the write
/// path, a volume bound produces an exactly-bounded file rather than a soft one,
/// and a live capture's pre-acquisition frames are read, discarded, and counted
/// (constitution P-4) rather than never observed.
///
/// The gate lives in the facade `session` module, beside [`CaptureSession`] and
/// [`RoleStampingAttributor`], because that is the one crate above both
/// `fragcap-capture` and `fragcap-attr` and the one that already bridges the
/// session to the packet path. `fragcap-core` sees only the generic
/// [`WriteGate`] seam it is handed as an `Arc<dyn WriteGate>` (constitution P-3).
///
/// [`SessionGate::new`] returns the gate together with a [`GateHandle`]. The gate
/// is moved into the pipeline; the handle stays with the driver to publish the
/// window and read the tallies. The gate forwards each admitted packet to the
/// driver over the tee channel, so the session's `on_packet` and `on_tick` still
/// fire `VolumeReached` and the duration bound in the session, keeping the six
/// stop conditions (specification section 10.6) in one place.
pub struct SessionGate {
    shared: Arc<GateShared>,
    /// The channel to the driver, carrying an admitted packet's captured length
    /// and instant. Owned by the gate alone so it drops when the pipeline
    /// finishes and the driver's read loop ends.
    tee: Sender<(u32, Timestamp)>,
}

/// The driver's handle to a [`SessionGate`]: publish the window, read the tallies.
///
/// Holds no sender, so keeping it for the whole run does not keep the tee channel
/// open. Cloneable, sharing the same atomics.
#[derive(Clone)]
pub struct GateHandle {
    shared: Arc<GateShared>,
}

impl SessionGate {
    /// A new gate over a session's bounds, forwarding admitted packets on `tee`,
    /// together with the driver's handle onto the same state.
    ///
    /// The window starts empty (`admit_from` at `i64::MAX`), so on the live path,
    /// where the pipeline runs from arm, every frame captured before acquisition is
    /// a watch-time discard until the driver opens the window from the acquisition
    /// instant. The offline driver opens the window from `i64::MIN` before it starts
    /// the pipeline, so every replayed frame is in window and the classification
    /// reduces to the bound alone.
    pub fn new(config: &SessionConfig, tee: Sender<(u32, Timestamp)>) -> (Self, GateHandle) {
        let shared = Arc::new(GateShared::new(config));
        let gate = SessionGate {
            shared: Arc::clone(&shared),
            tee,
        };
        (gate, GateHandle { shared })
    }
}

impl GateHandle {
    /// Open the window from `from`: the session has acquired a target, and a
    /// packet captured at or after `from` is admitted (subject to the bound). A
    /// packet captured before it stays a watch-time discard even if the output
    /// thread processes it after this call, which is what keeps a buffered
    /// pre-acquisition frame off disk. Offline the driver passes `i64::MIN`, so
    /// every replayed frame is in window.
    pub fn open_from(&self, from: Timestamp) {
        self.shared
            .admit_from
            .store(from.as_nanos(), Ordering::Relaxed);
    }

    /// Close the window at `until`: a packet captured at or after `until` is out
    /// of window. Used for a stop whose capture instant is known (a terminal-stage
    /// exit); a packet captured before `until` and still draining is kept. An
    /// interrupt or duration stop does not call this, so what was captured before
    /// the stop is retained (specification FR-005).
    pub fn close_at(&self, until: Timestamp) {
        self.shared
            .admit_until
            .store(until.as_nanos(), Ordering::Relaxed);
    }

    /// Packets admitted to the sinks. Equal to the packet records on disk.
    pub fn admitted(&self) -> u64 {
        self.shared.admitted.load(Ordering::Relaxed)
    }

    /// Bytes admitted to the sinks.
    pub fn admitted_bytes(&self) -> u64 {
        self.shared.admitted_bytes.load(Ordering::Relaxed)
    }

    /// Packets discarded while watching, before a target was acquired.
    pub fn watch_discarded(&self) -> u64 {
        self.shared.watch_discarded.load(Ordering::Relaxed)
    }

    /// Packets discarded out of the capture window (arming, draining, or beyond
    /// the bound).
    pub fn out_of_window_discarded(&self) -> u64 {
        self.shared.out_of_window_discarded.load(Ordering::Relaxed)
    }

    /// Packets discarded because they belong to a process the capture does not
    /// cover.
    pub fn scope_discarded(&self) -> u64 {
        self.shared.scope_discarded.load(Ordering::Relaxed)
    }

    /// Packets discarded on scope grounds that carried no attribution at all,
    /// so it is not known whether they were the capture's.
    pub fn scope_unresolved_discarded(&self) -> u64 {
        self.shared
            .scope_unresolved_discarded
            .load(Ordering::Relaxed)
    }

    /// Whether a configured bound has been reached.
    pub fn bound_hit(&self) -> bool {
        self.shared.bound_hit.load(Ordering::Relaxed)
    }
}

impl WriteGate for SessionGate {
    fn admit(&self, packet: &CapturedPacket) -> bool {
        let len = packet.data.as_ref().len() as u64;
        let ts = packet.ts.as_nanos();
        let shared = &self.shared;
        // Classify by the packet's own capture instant, not the window state at
        // this moment: a frame captured before acquisition is a watch-time discard
        // even if it was buffered and is only now being processed.
        if ts < shared.admit_from.load(Ordering::Relaxed) {
            shared.watch_discarded.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        // A frame captured at or after the stop instant is out of window even if
        // it is still draining.
        if ts >= shared.admit_until.load(Ordering::Relaxed) {
            shared
                .out_of_window_discarded
                .fetch_add(1, Ordering::Relaxed);
            return false;
        }
        // Out of scope: the packet was observed, resolved, and belongs to
        // something this capture does not cover. Specification section 12.3
        // places this decision here, in userspace, on every packet and
        // independent of whatever kernel filter is installed, precisely because
        // the filter cannot narrow until the target opens its first socket.
        //
        // Before the bound, deliberately: a packet that is not ours must not
        // consume the operator's `--max-bytes` budget.
        //
        // The two counters are not one. A packet with no attribution might have
        // been the target's, dropped because the socket table had not published
        // yet; a packet attributed to an unbound process certainly was not. See
        // `GateShared::scope_unresolved_discarded`.
        if !shared
            .scope
            .admits(packet.attribution.as_ref(), shared.allowed_roles.as_deref())
        {
            if packet.attribution.is_some() {
                shared.scope_discarded.fetch_add(1, Ordering::Relaxed);
            } else {
                shared
                    .scope_unresolved_discarded
                    .fetch_add(1, Ordering::Relaxed);
            }
            return false;
        }
        // Beyond the bound: the session's `check_volume_bounds` fires
        // `VolumeReached` when `retained >= packet_bound` or `retained_bytes >=
        // byte_bound`, so the gate stops admitting the moment either is met and
        // the file and the stop reason agree.
        let at_packet_bound = shared
            .packet_bound
            .is_some_and(|b| shared.admitted.load(Ordering::Relaxed) >= b);
        let at_byte_bound = shared
            .byte_bound
            .is_some_and(|b| shared.admitted_bytes.load(Ordering::Relaxed) >= b);
        if at_packet_bound || at_byte_bound {
            shared
                .out_of_window_discarded
                .fetch_add(1, Ordering::Relaxed);
            return false;
        }
        let n = shared.admitted.fetch_add(1, Ordering::Relaxed) + 1;
        let bytes = shared.admitted_bytes.fetch_add(len, Ordering::Relaxed) + len;
        // A byte bound admits the crossing packet (retained-inclusive), then
        // closes on the next packet via the comparison above.
        if shared.packet_bound.is_some_and(|b| n >= b)
            || shared.byte_bound.is_some_and(|b| bytes >= b)
        {
            shared.bound_hit.store(true, Ordering::Relaxed);
        }
        // A send fails only once the driver has dropped its receiver, which
        // happens only as the run tears down; nothing the session needs is lost,
        // so the failure is ignored.
        let _ = self.tee.send((len as u32, packet.ts));
        true
    }
}

#[cfg(test)]
mod gate_tests {
    use super::*;
    use fragcap_core::attribution::{Attribution, Fidelity};
    use fragcap_core::interface::InterfaceId;
    use fragcap_core::packet::{Payload, RawPacket};
    use std::sync::mpsc;

    /// A captured packet whose captured length is `len` bytes, at instant `ts`.
    fn packet(len: usize, ts: i64) -> CapturedPacket {
        CapturedPacket::from_raw(
            RawPacket::new(
                Timestamp::from_nanos(ts),
                Payload::copy_from_slice(&vec![0u8; len]),
                len as u32,
            ),
            InterfaceId::default(),
        )
    }

    /// A configuration whose scope admits everything.
    ///
    /// The default scope is `Target` since slice S064, so a synthetic packet
    /// with no attribution is out of scope and never reaches the window and
    /// bound logic. Every test below that is about windows or bounds says so by
    /// using this, which keeps each test about one thing; the scope tests build
    /// their own configurations and their own attributed packets.
    fn unscoped() -> SessionConfig {
        SessionConfig {
            scope: CaptureScope::All,
            ..SessionConfig::default()
        }
    }

    /// A packet carrying an attribution bound to a profile stage, as the session
    /// stamps one. This is what the scope predicate admits.
    fn bound_packet(len: usize, ts: i64, role: &str) -> CapturedPacket {
        let mut p = packet(len, ts);
        let mut attr = Attribution::new(4242, "game.exe", Fidelity::Live);
        attr = attr.with_role(role);
        attr = attr.with_stage(StageId::new("target"));
        p.attribution = Some(attr);
        p
    }

    /// A packet attributed to a process no profile stage binds: observed,
    /// resolved, and certainly not the capture's.
    fn unbound_packet(len: usize, ts: i64) -> CapturedPacket {
        let mut p = packet(len, ts);
        p.attribution = Some(Attribution::new(99, "docker.exe", Fidelity::Live));
        p
    }

    fn gate(config: SessionConfig) -> (SessionGate, GateHandle, mpsc::Receiver<(u32, Timestamp)>) {
        let (tx, rx) = mpsc::channel();
        let (gate, handle) = SessionGate::new(&config, tx);
        (gate, handle, rx)
    }

    // SC-003. A watch-time frame is read, discarded, and counted. The gate starts
    // in the watching window, so nothing is admitted and the watch count advances.
    // This is the counting the live run-from-arm path relies on, tested with no
    // capture driver.
    #[test]
    fn the_gate_counts_a_watch_time_discard() {
        let (gate, handle, rx) = gate(unscoped());
        for i in 0..5 {
            assert!(!gate.admit(&packet(64, i)), "watching admits nothing");
        }
        assert_eq!(handle.watch_discarded(), 5);
        assert_eq!(handle.admitted(), 0);
        assert_eq!(handle.out_of_window_discarded(), 0);
        assert!(
            rx.try_recv().is_err(),
            "no receipt is forwarded while watching"
        );
    }

    // FR-006 at the unit level. An open window admits exactly `packet_bound`
    // packets and rejects the rest into the out-of-window counter, so the file is
    // exactly the bound.
    #[test]
    fn a_packet_bound_admits_exactly_the_bound() {
        let (gate, handle, rx) = gate(SessionConfig {
            packet_bound: Some(3),
            ..unscoped()
        });
        handle.open_from(Timestamp::from_nanos(0));
        let admitted: Vec<bool> = (0..6).map(|i| gate.admit(&packet(10, i))).collect();
        assert_eq!(admitted, vec![true, true, true, false, false, false]);
        assert_eq!(handle.admitted(), 3);
        assert_eq!(handle.out_of_window_discarded(), 3);
        assert!(handle.bound_hit());
        // Exactly three receipts reached the driver, so retained equals the file.
        let forwarded: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
        assert_eq!(forwarded.len(), 3);
    }

    // FR-006, D-4. A byte bound admits the packet that first reaches or crosses
    // the bound (retained-inclusive), then closes.
    #[test]
    fn a_byte_bound_admits_the_crossing_packet_then_closes() {
        let (gate, handle, _rx) = gate(SessionConfig {
            byte_bound: Some(100),
            ..unscoped()
        });
        handle.open_from(Timestamp::from_nanos(0));
        // 40 + 40 = 80 (under), + 40 = 120 (crosses, admitted), then closed.
        assert!(gate.admit(&packet(40, 0)));
        assert!(gate.admit(&packet(40, 1)));
        assert!(
            gate.admit(&packet(40, 2)),
            "the crossing packet is admitted"
        );
        assert!(!gate.admit(&packet(40, 3)), "the next packet is rejected");
        assert_eq!(handle.admitted(), 3);
        assert_eq!(handle.admitted_bytes(), 120);
        assert!(handle.bound_hit());
    }

    // The reconciliation invariant: the gate's own discard tallies sum to what the
    // pipeline counts as gate_dropped, so nothing is double counted and the summary
    // matches the file.
    #[test]
    fn the_discard_tallies_reconcile() {
        let (gate, handle, _rx) = gate(SessionConfig {
            packet_bound: Some(2),
            ..unscoped()
        });
        // Two watch-time discards before the window opens.
        gate.admit(&packet(10, 0));
        gate.admit(&packet(10, 1));
        handle.open_from(Timestamp::from_nanos(0));
        // Two admitted, then two beyond the bound.
        for i in 2..6 {
            gate.admit(&packet(10, i));
        }
        assert_eq!(handle.admitted(), 2);
        let gate_dropped_equivalent = handle.watch_discarded() + handle.out_of_window_discarded();
        assert_eq!(handle.watch_discarded(), 2);
        assert_eq!(handle.out_of_window_discarded(), 2);
        assert_eq!(
            gate_dropped_equivalent, 4,
            "every non-admitted packet is counted once"
        );
    }

    // FR-001, FR-003, FR-007, FR-008. The scope decision, and which counter each
    // rejection lands in. Issue #184: the first real end-to-end run wrote a file
    // that was 91 percent other processes' traffic, because no gate consulted the
    // attribution that was already stamped on every packet.
    #[test]
    fn the_default_scope_admits_the_capture_and_counts_everything_else() {
        let (gate, handle, _rx) = gate(SessionConfig::default());
        handle.open_from(Timestamp::from_nanos(0));

        assert!(
            gate.admit(&bound_packet(10, 1, "target")),
            "a packet bound to a profile stage is the capture's"
        );
        assert!(
            !gate.admit(&unbound_packet(10, 2)),
            "a packet attributed to an unbound process is not"
        );
        assert!(
            !gate.admit(&packet(10, 3)),
            "a packet with no attribution cannot be shown to be the capture's"
        );

        assert_eq!(handle.admitted(), 1);
        assert_eq!(handle.scope_discarded(), 1, "the unbound one");
        assert_eq!(
            handle.scope_unresolved_discarded(),
            1,
            "the unattributed one, counted apart because it might have been ours"
        );
    }

    // FR-005. `--scope all` reproduces the behavior before slice S064 exactly, so
    // correlating a target against the rest of the machine, and debugging
    // attribution itself, both stay possible.
    #[test]
    fn the_all_scope_admits_what_the_gate_admitted_before() {
        let (gate, handle, _rx) = gate(unscoped());
        handle.open_from(Timestamp::from_nanos(0));
        assert!(gate.admit(&bound_packet(10, 1, "target")));
        assert!(gate.admit(&unbound_packet(10, 2)));
        assert!(gate.admit(&packet(10, 3)));
        assert_eq!(handle.admitted(), 3);
        assert_eq!(handle.scope_discarded(), 0);
        assert_eq!(handle.scope_unresolved_discarded(), 0);
    }

    // FR-003. `--roles` decides retention under the target scope, which is what
    // makes the run's roles line true of the file rather than only of which
    // stages trigger acquisition (issue #184).
    //
    // This asserts the gate's own contract, not the whole system's behavior, and
    // the difference matters. `CaptureSession::match_and_bind` already refuses to
    // bind a stage outside the role set, so in a real run a launcher packet never
    // carries a stamp and is rejected by the `bound` test rather than by this
    // one. The packet below is stamped directly to reach the role test at all.
    //
    // That coupling is why `--scope profile` was removed in review of PR #191:
    // it promised to retain "anything the profile binds regardless of --roles",
    // and since a stamped packet's role is always inside the set, it could never
    // admit anything `target` did not. A flag value that cannot differ from the
    // default is a distinction the interface claims and the system cannot make.
    #[test]
    fn a_narrowed_role_set_scopes_retention_under_target() {
        let (target_gate, target_handle, _rx) = gate(SessionConfig {
            allowed_roles: Some(vec!["target".to_string()]),
            ..SessionConfig::default()
        });
        target_handle.open_from(Timestamp::from_nanos(0));
        assert!(target_gate.admit(&bound_packet(10, 1, "target")));
        assert!(
            !target_gate.admit(&bound_packet(10, 2, "launcher")),
            "a bound role outside --roles is out of scope"
        );
        assert_eq!(target_handle.scope_discarded(), 1);
    }

    // FR-001, placement. An out-of-scope packet must not consume the operator's
    // `--max-bytes` budget: the file is bounded by what it contains, not by what
    // the machine happened to be doing while it was written.
    #[test]
    fn an_out_of_scope_packet_does_not_consume_the_volume_bound() {
        let (gate, handle, _rx) = gate(SessionConfig {
            packet_bound: Some(2),
            ..SessionConfig::default()
        });
        handle.open_from(Timestamp::from_nanos(0));
        for i in 0..50 {
            gate.admit(&unbound_packet(10, i));
        }
        assert!(!handle.bound_hit(), "noise did not spend the bound");
        assert!(gate.admit(&bound_packet(10, 100, "target")));
        assert!(gate.admit(&bound_packet(10, 101, "target")));
        assert_eq!(handle.admitted(), 2, "the bound counts the capture's own");
    }

    // FR-009. Every refusal lands in exactly one counter, and the four reasons sum
    // to the refusal count. That refusal count is what the pipeline increments as
    // `gate_dropped` at its single call site, so this gate-local invariant is the
    // testable form of the conservation identity's gate term. A discard path added
    // later with no counter fails here.
    #[test]
    fn the_four_discard_reasons_account_for_every_refusal() {
        let (gate, handle, _rx) = gate(SessionConfig {
            packet_bound: Some(1),
            ..SessionConfig::default()
        });
        let mut refused = 0u64;

        // Watch-time: the window is not open yet.
        if !gate.admit(&bound_packet(10, 0, "target")) {
            refused += 1;
        }
        handle.open_from(Timestamp::from_nanos(1));
        // Out of scope, both kinds.
        if !gate.admit(&unbound_packet(10, 2)) {
            refused += 1;
        }
        if !gate.admit(&packet(10, 3)) {
            refused += 1;
        }
        // Admitted, meeting the bound.
        assert!(gate.admit(&bound_packet(10, 4, "target")));
        // Beyond the bound.
        if !gate.admit(&bound_packet(10, 5, "target")) {
            refused += 1;
        }

        let accounted = handle.watch_discarded()
            + handle.out_of_window_discarded()
            + handle.scope_discarded()
            + handle.scope_unresolved_discarded();
        assert_eq!(
            accounted, refused,
            "every refusal is counted exactly once: watch {}, window {}, scope {},              unresolved {}, refused {refused}",
            handle.watch_discarded(),
            handle.out_of_window_discarded(),
            handle.scope_discarded(),
            handle.scope_unresolved_discarded()
        );
        assert_eq!(refused, 4, "the fixture exercises all four reasons");
    }

    // FR-011 at the unit level. An open, unbounded window admits everything: the
    // pass-through that keeps the offline goldens byte-identical.
    #[test]
    fn an_unbounded_open_window_is_a_pass_through() {
        let (gate, handle, rx) = gate(unscoped());
        handle.open_from(Timestamp::from_nanos(0));
        for i in 0..10 {
            assert!(
                gate.admit(&packet(50, i)),
                "an unbounded open window admits all"
            );
        }
        assert_eq!(handle.admitted(), 10);
        assert_eq!(handle.out_of_window_discarded(), 0);
        assert!(!handle.bound_hit());
        let forwarded: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
        assert_eq!(forwarded.len(), 10);
    }

    // Review of PR #26 (Codex C2). A frame captured before acquisition but still
    // buffered when the window opens must stay a watch-time discard: the gate
    // classifies by the packet's own capture instant, not the window state at the
    // moment the output thread processes it. Without this a whole pre-acquisition
    // buffer would land on disk after acquisition and be omitted from
    // watching_discarded.
    #[test]
    fn a_buffered_pre_acquisition_frame_stays_a_watch_discard() {
        let (gate, handle, rx) = gate(unscoped());
        // Acquisition happened at instant 100.
        handle.open_from(Timestamp::from_nanos(100));
        // A frame captured at 50 (before acquisition) that was buffered and is only
        // now processed, after the window opened.
        assert!(
            !gate.admit(&packet(64, 50)),
            "a pre-acquisition frame is not admitted even with the window open"
        );
        // A frame captured at 150 (after acquisition) is admitted.
        assert!(
            gate.admit(&packet(64, 150)),
            "a post-acquisition frame is admitted"
        );
        assert_eq!(
            handle.watch_discarded(),
            1,
            "the buffered frame is a watch discard"
        );
        assert_eq!(handle.admitted(), 1);
        let forwarded: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
        assert_eq!(
            forwarded.len(),
            1,
            "only the in-window frame reaches the driver"
        );
    }

    // Review of PR #26 (Codex C1). A frame captured at or after a stop with a known
    // capture instant (a terminal-stage exit) must stay out of window even if it is
    // still draining when the pipeline is torn down, so post-stop traffic is not
    // written and miscounted as retained. A frame captured before the stop is kept.
    #[test]
    fn a_frame_captured_after_the_stop_is_out_of_window() {
        let (gate, handle, _rx) = gate(unscoped());
        handle.open_from(Timestamp::from_nanos(0));
        // A terminal-stage exit closed the window at instant 200.
        handle.close_at(Timestamp::from_nanos(200));
        assert!(gate.admit(&packet(64, 150)), "a pre-stop frame is admitted");
        assert!(
            !gate.admit(&packet(64, 250)),
            "a frame captured after the stop is out of window even while draining"
        );
        assert_eq!(handle.admitted(), 1);
        assert_eq!(handle.out_of_window_discarded(), 1);
    }
}

#[cfg(test)]
mod stamping_tests {
    use super::*;
    use fragcap_core::attribution::Fidelity;
    use fragcap_core::flow::Proto;
    use std::net::SocketAddr;

    fn addr(s: &str) -> SocketAddr {
        s.parse().expect("test address parses")
    }

    fn key() -> FlowKey {
        FlowKey::new(
            Proto::Tcp,
            addr("192.0.2.10:51000"),
            addr("198.51.100.5:443"),
        )
    }

    /// An attributor that always resolves the given pid, with no role or stage.
    struct Fixed(u32);

    impl FlowAttributor for Fixed {
        fn resolve(&self, _key: &FlowKey, _at: Timestamp) -> Option<Attribution> {
            Some(Attribution::new(self.0, "game.exe", Fidelity::Live))
        }
        fn refresh(&self) -> Result<(), AttrError> {
            Ok(())
        }
        fn active_endpoints(&self) -> Vec<Endpoint> {
            Vec::new()
        }
    }

    fn at() -> Timestamp {
        Timestamp::from_nanos(0)
    }

    #[test]
    fn an_unstamped_resolve_passes_through_unchanged() {
        let stamper = RoleStampingAttributor::new(Arc::new(Fixed(42)));
        let a = stamper.resolve(&key(), at()).expect("resolves");
        assert_eq!(a.pid, 42);
        assert!(a.role.is_none());
        assert!(a.stage.is_none());
    }

    #[test]
    fn a_published_binding_stamps_role_and_stage() {
        let stamper = RoleStampingAttributor::new(Arc::new(Fixed(42)));
        stamper.publisher().publish(vec![(
            42,
            Some(Arc::from("client")),
            Some(StageId::new("client")),
        )]);
        let a = stamper.resolve(&key(), at()).expect("resolves");
        assert_eq!(a.role.as_deref(), Some("client"));
        assert_eq!(a.stage.as_ref().map(StageId::as_str), Some("client"));
    }

    #[test]
    fn a_pid_with_no_binding_is_unchanged() {
        let stamper = RoleStampingAttributor::new(Arc::new(Fixed(7)));
        stamper.publisher().publish(vec![(
            42,
            Some(Arc::from("client")),
            Some(StageId::new("client")),
        )]);
        let a = stamper.resolve(&key(), at()).expect("resolves");
        assert_eq!(a.pid, 7);
        assert!(a.role.is_none(), "only the bound pid is stamped");
    }

    // --- Slice 015: narrowing restricted to profiled processes -----------

    use fragcap_core::flow::OwnedEndpoint;

    /// An inner attributor reporting a fixed set of owner-carrying endpoints.
    struct OwnedInner(Vec<OwnedEndpoint>);

    impl FlowAttributor for OwnedInner {
        fn resolve(&self, _key: &FlowKey, _at: Timestamp) -> Option<Attribution> {
            None
        }
        fn refresh(&self) -> Result<(), AttrError> {
            Ok(())
        }
        fn active_endpoints(&self) -> Vec<Endpoint> {
            self.0.iter().map(|o| o.endpoint).collect()
        }
        fn active_endpoints_owned(&self) -> Vec<OwnedEndpoint> {
            self.0.clone()
        }
    }

    fn owned(addr_s: &str, proto: Proto, owner: Option<u32>) -> OwnedEndpoint {
        OwnedEndpoint::new(Endpoint::new(addr(addr_s), proto), owner)
    }

    // Slice 015, SC-002. The narrowed endpoint set admits only endpoints owned
    // by a profiled process, across IPv4, IPv6, and a wildcard UDP bind, and
    // excludes an unprofiled process's endpoints sharing the same source.
    #[test]
    fn active_endpoints_admits_only_profiled_process_endpoints() {
        let profiled_v4 = owned("192.0.2.10:30000", Proto::Udp, Some(7));
        let profiled_v6 = owned("[2001:db8::10]:30000", Proto::Udp, Some(7));
        let profiled_wild = owned("0.0.0.0:40000", Proto::Udp, Some(7));
        let unprofiled = owned("192.0.2.10:50000", Proto::Tcp, Some(9));
        let inner = OwnedInner(vec![profiled_v4, profiled_v6, profiled_wild, unprofiled]);
        let stamper = RoleStampingAttributor::new(Arc::new(inner));
        // Only pid 7 is stage-bound (profiled).
        stamper.publisher().publish(vec![(
            7,
            Some(Arc::from("client")),
            Some(StageId::new("client")),
        )]);

        let got = stamper.active_endpoints();
        assert!(
            got.contains(&profiled_v4.endpoint),
            "IPv4 profiled admitted"
        );
        assert!(
            got.contains(&profiled_v6.endpoint),
            "IPv6 profiled admitted"
        );
        assert!(
            got.contains(&profiled_wild.endpoint),
            "wildcard UDP bind admitted by its owning module"
        );
        assert!(
            !got.contains(&unprofiled.endpoint),
            "an unprofiled process's endpoint is excluded"
        );
        assert_eq!(got.len(), 3);
    }

    // An endpoint whose owner is not known is kept (the offline scripted
    // substrate reports none), so narrowing does not empty a declared capture.
    #[test]
    fn an_endpoint_with_no_known_owner_is_kept() {
        let unknown = owned("192.0.2.10:60000", Proto::Udp, None);
        let unprofiled = owned("192.0.2.10:50000", Proto::Tcp, Some(9));
        let stamper = RoleStampingAttributor::new(Arc::new(OwnedInner(vec![unknown, unprofiled])));
        // No bindings at all: nothing is profiled.
        let got = stamper.active_endpoints();
        assert_eq!(
            got,
            vec![unknown.endpoint],
            "the unknown-owner endpoint survives, the known unprofiled one does not"
        );
    }

    // Review of pull request 24. One endpoint arriving as two owner candidates,
    // a live unprofiled reuse and a retained profiled owner, is admitted once
    // via the profiled owner rather than dropped because the unprofiled owner
    // was listed first.
    #[test]
    fn a_profiled_owner_admits_a_reused_endpoint_once() {
        let live_unprofiled = owned("192.0.2.10:30000", Proto::Udp, Some(9));
        let retained_profiled = owned("192.0.2.10:30000", Proto::Udp, Some(7));
        let stamper = RoleStampingAttributor::new(Arc::new(OwnedInner(vec![
            live_unprofiled,
            retained_profiled,
        ])));
        stamper.publisher().publish(vec![(
            7,
            Some(Arc::from("client")),
            Some(StageId::new("client")),
        )]);
        let got = stamper.active_endpoints();
        assert_eq!(
            got,
            vec![Endpoint::new(addr("192.0.2.10:30000"), Proto::Udp)],
            "the endpoint is admitted once, via its profiled owner"
        );
    }

    #[test]
    fn republishing_swaps_the_snapshot() {
        let stamper = RoleStampingAttributor::new(Arc::new(Fixed(42)));
        let publisher = stamper.publisher();
        publisher.publish(vec![(
            42,
            Some(Arc::from("launcher")),
            Some(StageId::new("launcher")),
        )]);
        assert_eq!(
            stamper.resolve(&key(), at()).unwrap().role.as_deref(),
            Some("launcher")
        );
        publisher.publish(vec![(
            42,
            Some(Arc::from("client")),
            Some(StageId::new("client")),
        )]);
        assert_eq!(
            stamper.resolve(&key(), at()).unwrap().role.as_deref(),
            Some("client"),
            "the latest publication wins"
        );
    }
}
