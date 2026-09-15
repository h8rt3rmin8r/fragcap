// SPDX-License-Identifier: Apache-2.0

//! Progress vocabulary for `fragcap doctor`.
//!
//! These lines are diagnostics on standard error, not part of the doctor report
//! contract. Fixed labels identify coarse phases and late readiness boundaries
//! without exposing local identities or claiming a pending result is failure.

use std::time::{Duration, Instant};

/// Short anomaly threshold and minimum repeated waiting interval.
pub const SLOW_PROBE: Duration = Duration::from_secs(1);

/// A named unit of doctor work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeName {
    /// Version and per-user path facts.
    Identity,
    /// Operating system, subsystem, and privilege facts.
    Platform,
    /// Npcap driver and capture-interface facts.
    CaptureDriverInterfaces,
    /// Event Tracing for Windows availability.
    ProcessEventTracing,
    /// Analyzer extcap registration facts.
    AnalyzerIntegration,
    /// Catalog and local target store facts.
    TargetStores,
    /// Deep Capture readiness and residue facts.
    DeepCaptureReadiness,
    /// Bounded native owner, journal and resource inspection (aggregate).
    NativeResidueInventory,
    /// Bounded manifest and declared artifact filesystem inspection.
    ManifestArtifactScan,
    /// Exact CA identities declared by retained manifests.
    ManifestCaIdentities,
    /// Read-only current-user root certificate enumeration.
    CurrentUserCaStore,
    /// Read-only machine root certificate enumeration.
    MachineCaStore,
    /// Temporary exact IPv4 loopback readiness bind.
    Ipv4Loopback,
    /// Temporary exact IPv6 loopback readiness bind.
    Ipv6Loopback,
    /// Final doctor report rendering.
    ReportRendering,
}

impl ProbeName {
    /// The stable human label used in progress and timing output.
    pub fn label(self) -> &'static str {
        match self {
            ProbeName::Identity => "identity",
            ProbeName::Platform => "platform",
            ProbeName::CaptureDriverInterfaces => "capture driver and interfaces",
            ProbeName::ProcessEventTracing => "process event tracing",
            ProbeName::AnalyzerIntegration => "analyzer integration",
            ProbeName::TargetStores => "target stores",
            ProbeName::DeepCaptureReadiness => "Deep Capture readiness",
            ProbeName::NativeResidueInventory => "Deep Capture native residue inventory",
            ProbeName::ManifestArtifactScan => "Deep Capture manifest and artifact scan",
            ProbeName::ManifestCaIdentities => "Deep Capture manifest CA identities",
            ProbeName::CurrentUserCaStore => "Deep Capture current-user root certificate store",
            ProbeName::MachineCaStore => "Deep Capture machine root certificate store",
            ProbeName::Ipv4Loopback => "Deep Capture IPv4 loopback readiness",
            ProbeName::Ipv6Loopback => "Deep Capture IPv6 loopback readiness",
            ProbeName::ReportRendering => "report rendering",
        }
    }
}

/// Render the line emitted when a probe starts.
pub fn begin_line(probe: ProbeName) -> String {
    format!("doctor: checking {}...", probe.label())
}

/// Render the line emitted when a probe completes.
pub fn complete_line(probe: ProbeName, elapsed: Duration, timings: bool) -> String {
    if timings || elapsed >= SLOW_PROBE {
        format!(
            "doctor: checked {} in {} ms",
            probe.label(),
            elapsed.as_millis()
        )
    } else {
        format!("doctor: checked {}", probe.label())
    }
}

/// Render pending work, never an inferred readiness verdict.
pub fn waiting_line(probe: ProbeName, elapsed: Duration) -> String {
    format!(
        "doctor: still checking {} after {} ms (waiting for this operation to finish)",
        probe.label(),
        elapsed.as_millis()
    )
}

#[derive(Clone, Copy)]
struct ActivePhase {
    name: ProbeName,
    started: Instant,
}

/// One serial coarse phase plus one readiness leaf, with finite storage.
#[derive(Default)]
pub(super) struct PendingPhases {
    active: [Option<ActivePhase>; 2],
    last_wait: Option<Instant>,
}

impl PendingPhases {
    pub(super) fn begin(&mut self, name: ProbeName, started: Instant) {
        let slot = self.active.iter_mut().find(|slot| slot.is_none());
        *slot.expect("Doctor observation nests at most one readiness leaf") =
            Some(ActivePhase { name, started });
    }

    pub(super) fn complete(&mut self, name: ProbeName) {
        let slot = self.active.iter_mut().rev().find(|slot| slot.is_some());
        let slot = slot.expect("Doctor completion has an active phase");
        assert_eq!(slot.as_ref().unwrap().name, name, "serial phase completion");
        *slot = None;
    }

    pub(super) fn delay(&self, now: Instant) -> Duration {
        self.active
            .iter()
            .rev()
            .flatten()
            .next()
            .map(|phase| {
                (phase.started + SLOW_PROBE)
                    .max(
                        self.last_wait
                            .map(|last| last + SLOW_PROBE)
                            .unwrap_or(phase.started),
                    )
                    .saturating_duration_since(now)
            })
            .unwrap_or(SLOW_PROBE)
    }

    pub(super) fn waiting(&mut self, now: Instant) -> Option<(ProbeName, Duration)> {
        let phase = self.active.iter_mut().rev().flatten().next()?;
        let elapsed = now.saturating_duration_since(phase.started);
        if elapsed < SLOW_PROBE
            || self
                .last_wait
                .is_some_and(|last| now.saturating_duration_since(last) < SLOW_PROBE)
        {
            return None;
        }
        self.last_wait = Some(now);
        Some((phase.name, elapsed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_threshold_cadence_and_nested_parent_resumption_are_exact() {
        let start = Instant::now();
        let mut phases = PendingPhases::default();
        phases.begin(ProbeName::DeepCaptureReadiness, start);
        assert_eq!(phases.delay(start), SLOW_PROBE);
        assert_eq!(phases.waiting(start + Duration::from_millis(999)), None);
        assert_eq!(
            phases.waiting(start + SLOW_PROBE),
            Some((ProbeName::DeepCaptureReadiness, SLOW_PROBE))
        );
        assert_eq!(phases.waiting(start + Duration::from_millis(1999)), None);
        phases.begin(ProbeName::MachineCaStore, start + SLOW_PROBE);
        assert_eq!(phases.waiting(start + Duration::from_millis(1999)), None);
        assert_eq!(
            phases.waiting(start + Duration::from_secs(2)),
            Some((ProbeName::MachineCaStore, SLOW_PROBE))
        );
        assert_eq!(phases.delay(start + Duration::from_secs(2)), SLOW_PROBE);
        phases.complete(ProbeName::MachineCaStore);
        assert_eq!(phases.waiting(start + Duration::from_secs(2)), None);
        assert_eq!(
            phases.waiting(start + Duration::from_secs(3)),
            Some((ProbeName::DeepCaptureReadiness, Duration::from_secs(3)))
        );
        phases.complete(ProbeName::DeepCaptureReadiness);
        assert_eq!(phases.waiting(start + Duration::from_secs(99)), None);
    }

    #[test]
    fn waiting_text_is_pending_and_contains_no_machine_identity() {
        assert_eq!(
            waiting_line(ProbeName::MachineCaStore, Duration::from_millis(1200)),
            "doctor: still checking Deep Capture machine root certificate store after 1200 ms (waiting for this operation to finish)"
        );
    }

    #[test]
    fn slow_completion_is_timed_without_advance_opt_in() {
        assert_eq!(
            complete_line(
                ProbeName::ProcessEventTracing,
                Duration::from_secs(1),
                false
            ),
            "doctor: checked process event tracing in 1000 ms"
        );
    }

    #[test]
    fn begin_line_names_the_probe() {
        assert_eq!(
            begin_line(ProbeName::CaptureDriverInterfaces),
            "doctor: checking capture driver and interfaces..."
        );
    }

    #[test]
    fn complete_line_omits_timings_by_default() {
        assert_eq!(
            complete_line(
                ProbeName::ProcessEventTracing,
                Duration::from_millis(42),
                false
            ),
            "doctor: checked process event tracing"
        );
    }

    #[test]
    fn complete_line_includes_elapsed_milliseconds_when_requested() {
        assert_eq!(
            complete_line(
                ProbeName::DeepCaptureReadiness,
                Duration::from_millis(42),
                true
            ),
            "doctor: checked Deep Capture readiness in 42 ms"
        );
    }
}
