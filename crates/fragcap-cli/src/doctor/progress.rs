// SPDX-License-Identifier: Apache-2.0

//! Progress vocabulary for `fragcap doctor`.
//!
//! These lines are diagnostics on standard error, not part of the doctor report
//! contract. The names are deliberately coarser than helper functions so they
//! stay useful to an operator and stable enough for timing comparisons.

use std::time::Duration;

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
    if timings {
        format!(
            "doctor: checked {} in {} ms",
            probe.label(),
            elapsed.as_millis()
        )
    } else {
        format!("doctor: checked {}", probe.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
