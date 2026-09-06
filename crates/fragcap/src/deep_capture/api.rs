// SPDX-License-Identifier: Apache-2.0

//! Versioned public Deep Capture API.
//!
//! This module is the stable library-first surface. Existing top-level
//! `deep_capture` exports remain available for compatibility, but callers that
//! want the explicit stability policy should import from this module.
//!
//! Session coordinators and adapter sets are intentionally thread-confined.
//! Adapters receive finite cooperative budgets and do not acquire implicit
//! `Send` or `Sync` promises. [`CancellationToken`] is the thread-safe handle
//! for requesting cancellation at coordinator stage boundaries.
//!
//! Evolvable enums require a fallback arm:
//!
//! ```compile_fail
//! use fragcap::deep_capture::api::LaunchCase;
//!
//! let case = LaunchCase::Controlled;
//! match case {
//!     LaunchCase::SteamProtocolWarm => {}
//!     LaunchCase::SteamProtocolCold => {}
//!     LaunchCase::DirectExeWarm => {}
//!     LaunchCase::DirectExeCold => {}
//!     LaunchCase::PublisherLauncher => {}
//!     LaunchCase::PublisherLauncherWarm => {}
//!     LaunchCase::PublisherLauncherGameStartCleanWarm => {}
//!     LaunchCase::PublisherLauncherCold => {}
//!     LaunchCase::Controlled => {}
//! }
//! ```

pub use crate::targets::{CompatibilityFactKey, CompatibilityProtocol};

pub use super::{
    calibration_outcome, calibration_outcome_reason, observation_proves_final_client_ca_acceptance,
    recover_resource_journal, run_controlled_native_requests, terminal_calibration_outcome,
    validate_compatibility_prerequisites, AdapterSet, AdapterSetBuildError, AdapterSetBuilder,
    AppliedRoute, ArtifactRequests, ArtifactResult, ArtifactSink, ArtifactStatus, Authorization,
    BackendDescriptor, Budget, BypassDecision, BypassDecisionOutcome, BypassPolicy, BypassRule,
    CalibrationOutcome, CalibrationPhase, CancellationToken, CaptureRunResult, CaptureRunner,
    CertificateStore, ChildEnvironmentRouting, ClassificationSummary, CleanupResult, CleanupStatus,
    ClientIdentity, CompatibilityFact, CompatibilityFactCandidate, CompatibilityObservation,
    CompatibilityRepository, CorrelationState, Deadlines, DeepCapture, DeepCaptureEvent,
    DeepCaptureSession, DetectionState, EndpointAllocator, EventDeliveryFailure, EventSink,
    FactWriteResult, FactWriteStatus, HarProjection, IdentifierSource, Inspectability,
    InspectabilityState, InvalidClassification, InvalidTransition, LaunchAdapter, LaunchCase,
    LaunchLease, LifecycleState, LifecycleTransition, LoopbackEndpoint, ManifestOmissionReason,
    NativeCertificateStore, NativeListenerReservation, NativeObservationContext,
    NativeProxyAdapter, NativeProxyLimits, Operation, PlanId, PreflightRefusal, PreparedCapture,
    PreparedSession, PreparedTarget, ProtocolClassification, ProxyBackend, ProxyLease, ProxyRoute,
    RecoveryAction, RecoveryPlan, RecoveryRefusal, RouteEffect, RouteValueSource,
    RouteVerification, RouteVerificationState, RoutingAdapter, RoutingAvailability, RoutingLease,
    RoutingPlan, RoutingStrategyKind, SensitiveRetention, Sensitivity, SessionClock, SessionConfig,
    SessionConfigBuildError, SessionConfigBuilder, SessionMode, SessionOutcome, SessionPlan, Stage,
    StageFailure, StageTransition, StageTransitionKind, TargetResolver, TerminalReport,
    TerminalSnapshot, TrafficFamily, TrustLease, TrustManager, TrustMutation, TrustState,
};

/// Major version of the curated stability contract in this module.
pub const DEEP_CAPTURE_API_VERSION: u32 = 1;

/// Exact sorted inventory of the version-one contract.
pub const STABLE_API_EXPORTS: &[&str] = &[
    "AdapterSet",
    "AdapterSetBuildError",
    "AdapterSetBuilder",
    "AppliedRoute",
    "ArtifactRequests",
    "ArtifactResult",
    "ArtifactSink",
    "ArtifactStatus",
    "Authorization",
    "BackendDescriptor",
    "Budget",
    "BypassDecision",
    "BypassDecisionOutcome",
    "BypassPolicy",
    "BypassRule",
    "CalibrationOutcome",
    "CalibrationPhase",
    "CancellationToken",
    "CaptureRunResult",
    "CaptureRunner",
    "CertificateStore",
    "ChildEnvironmentRouting",
    "ClassificationSummary",
    "CleanupResult",
    "CleanupStatus",
    "ClientIdentity",
    "CompatibilityFact",
    "CompatibilityFactCandidate",
    "CompatibilityFactKey",
    "CompatibilityObservation",
    "CompatibilityProtocol",
    "CompatibilityRepository",
    "CorrelationState",
    "DEEP_CAPTURE_API_VERSION",
    "Deadlines",
    "DeepCapture",
    "DeepCaptureEvent",
    "DeepCaptureSession",
    "DetectionState",
    "EndpointAllocator",
    "EventDeliveryFailure",
    "EventSink",
    "FactWriteResult",
    "FactWriteStatus",
    "HarProjection",
    "IdentifierSource",
    "Inspectability",
    "InspectabilityState",
    "InvalidClassification",
    "InvalidTransition",
    "LaunchAdapter",
    "LaunchCase",
    "LaunchLease",
    "LifecycleState",
    "LifecycleTransition",
    "LoopbackEndpoint",
    "ManifestOmissionReason",
    "NativeCertificateStore",
    "NativeListenerReservation",
    "NativeObservationContext",
    "NativeProxyAdapter",
    "NativeProxyLimits",
    "Operation",
    "PlanId",
    "PreflightRefusal",
    "PreparedCapture",
    "PreparedSession",
    "PreparedTarget",
    "ProtocolClassification",
    "ProxyBackend",
    "ProxyLease",
    "ProxyRoute",
    "RecoveryAction",
    "RecoveryPlan",
    "RecoveryRefusal",
    "RouteEffect",
    "RouteValueSource",
    "RouteVerification",
    "RouteVerificationState",
    "RoutingAdapter",
    "RoutingAvailability",
    "RoutingLease",
    "RoutingPlan",
    "RoutingStrategyKind",
    "STABLE_API_EXPORTS",
    "SensitiveRetention",
    "Sensitivity",
    "SessionClock",
    "SessionConfig",
    "SessionConfigBuildError",
    "SessionConfigBuilder",
    "SessionMode",
    "SessionOutcome",
    "SessionPlan",
    "Stage",
    "StageFailure",
    "StageTransition",
    "StageTransitionKind",
    "TargetResolver",
    "TerminalReport",
    "TerminalSnapshot",
    "TrafficFamily",
    "TrustLease",
    "TrustManager",
    "TrustMutation",
    "TrustState",
    "calibration_outcome",
    "calibration_outcome_reason",
    "observation_proves_final_client_ca_acceptance",
    "recover_resource_journal",
    "run_controlled_native_requests",
    "terminal_calibration_outcome",
    "validate_compatibility_prerequisites",
];
