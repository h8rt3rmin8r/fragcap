// SPDX-License-Identifier: Apache-2.0

use std::path::Path;
use std::time::{Duration, SystemTime};

use super::{
    ArtifactResult, BackendDescriptor, Budget, CleanupResult, CompatibilityFact,
    CompatibilityObservation, DeepCaptureEvent, FactWriteStatus, LaunchCase, LoopbackEndpoint,
    PreflightRefusal, PreparedCapture, PreparedTarget, SessionConfig, StageFailure,
    TerminalSnapshot,
};

/// Side-effect-free target and compatibility resolution.
pub trait TargetResolver {
    fn resolve(&mut self, config: &SessionConfig) -> Result<PreparedTarget, PreflightRefusal>;
    fn validate_compatibility(
        &mut self,
        target: &PreparedTarget,
        config: &SessionConfig,
    ) -> Result<(), PreflightRefusal>;
}

/// Side-effect-free loopback endpoint selection.
pub trait EndpointAllocator {
    fn select(&mut self) -> Result<LoopbackEndpoint, PreflightRefusal>;
}

/// Clock used for audit stamps and cooperative deadline enforcement.
pub trait SessionClock {
    fn wall_now(&mut self) -> SystemTime;
    fn monotonic_elapsed(&mut self) -> Duration;
}

/// Deterministic source for session and plan identifiers.
pub trait IdentifierSource {
    fn next_id(&mut self, kind: &'static str) -> Result<String, PreflightRefusal>;
}

/// Running proxy resource owned by one coordinator.
pub trait ProxyLease {
    fn observations(
        &mut self,
        budget: Budget,
    ) -> Result<Vec<CompatibilityObservation>, StageFailure>;
    fn stop(&mut self, budget: Budget) -> CleanupResult;
    fn cleanup(&mut self, budget: Budget) -> Vec<CleanupResult>;
}

/// Replaceable loopback proxy backend.
pub trait ProxyBackend {
    fn descriptor(&self) -> BackendDescriptor;
    fn start(
        &mut self,
        plan: &super::SessionPlan,
        budget: Budget,
    ) -> Result<Box<dyn ProxyLease>, StageFailure>;
}

/// Optional current-user trust resource.
pub trait TrustLease {
    fn cleanup(&mut self, budget: Budget) -> CleanupResult;
}

/// Explicit, reversible trust management.
pub trait TrustManager {
    fn acquire(
        &mut self,
        plan: &super::SessionPlan,
        budget: Budget,
    ) -> Result<Box<dyn TrustLease>, StageFailure>;
}

/// Managed target resource.
pub trait LaunchLease {
    fn cleanup(&mut self, budget: Budget) -> CleanupResult;
}

/// Target-scoped launch preparation and execution.
pub trait LaunchAdapter {
    fn launch(
        &mut self,
        target: &PreparedTarget,
        launch_case: LaunchCase,
        endpoint: LoopbackEndpoint,
        budget: Budget,
    ) -> Result<Box<dyn LaunchLease>, StageFailure>;
}

/// Existing ordinary Capture composition used by Deep Capture.
pub trait CaptureRunner {
    fn prepare(
        &mut self,
        config: &SessionConfig,
        target: &PreparedTarget,
    ) -> Result<PreparedCapture, PreflightRefusal>;

    fn run(
        &mut self,
        prepared: &PreparedCapture,
        endpoint: LoopbackEndpoint,
        budget: Budget,
    ) -> Result<super::CaptureRunResult, StageFailure>;

    fn stop(&mut self, budget: Budget) -> CleanupResult;
}

/// Append-only target-owned compatibility persistence.
pub trait CompatibilityRepository {
    fn append(&mut self, target: &PreparedTarget, fact: &CompatibilityFact) -> FactWriteStatus;
}

/// Bundle destination validation and independent artifact persistence.
pub trait ArtifactSink {
    fn validate_destination(&mut self, path: &Path) -> Result<(), PreflightRefusal>;
    fn finalize(&mut self, bundle: &Path, snapshot: &TerminalSnapshot) -> Vec<ArtifactResult>;
}

/// Ordered typed event delivery. Presentation is outside this trait.
pub trait EventSink {
    fn emit(&mut self, event: &DeepCaptureEvent) -> Result<(), StageFailure>;
}

/// Complete replaceable environment for one coordinator.
pub struct AdapterSet<'a> {
    pub targets: Box<dyn TargetResolver + 'a>,
    pub endpoints: Box<dyn EndpointAllocator + 'a>,
    pub clock: Box<dyn SessionClock + 'a>,
    pub identifiers: Box<dyn IdentifierSource + 'a>,
    pub proxy: Box<dyn ProxyBackend + 'a>,
    pub trust: Box<dyn TrustManager + 'a>,
    pub launch: Box<dyn LaunchAdapter + 'a>,
    pub capture: Box<dyn CaptureRunner + 'a>,
    pub facts: Box<dyn CompatibilityRepository + 'a>,
    pub artifacts: Box<dyn ArtifactSink + 'a>,
    pub events: Box<dyn EventSink + 'a>,
}

impl std::fmt::Debug for AdapterSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdapterSet").finish_non_exhaustive()
    }
}
