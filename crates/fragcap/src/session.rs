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
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fragcap_core::attribution::{Attribution, StageId};
use fragcap_core::error::AttrError;
use fragcap_core::flow::{Endpoint, FlowKey};
use fragcap_core::packet::Timestamp;
use fragcap_core::process::{ProcessEvent, ProcessId, ProcessTree};
use fragcap_core::traits::FlowAttributor;

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
        // A stage outside the scoped role set is treated as if it were not in
        // the profile: no bind, so no stamp, no live count, and no stop
        // condition (specification FR-011b).
        if !role_in(&self.allowed_roles, &role) {
            return;
        }
        if self.tree.bind_stage(id, sid) {
            let service = lifecycle == Lifecycle::Service;
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
                terminal,
                service,
                live: !already_exited,
            });
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
                self.note_bound_exit(terminal);
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
