// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use super::SensitiveRetention;
use crate::targets::CompatibilityFactKey;
use crate::FlowId;

/// Stable identifier for one immutable prepared plan.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PlanId(String);

impl PlanId {
    /// Construct an identifier supplied by the configured identifier source.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Return the identifier text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PlanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// One supported managed-launch shape.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaunchCase {
    SteamProtocolWarm,
    SteamProtocolCold,
    DirectExeWarm,
    DirectExeCold,
    PublisherLauncher,
    PublisherLauncherWarm,
    PublisherLauncherGameStartCleanWarm,
    PublisherLauncherCold,
    /// The committed controlled target harness.
    Controlled,
}

impl LaunchCase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SteamProtocolWarm => "steam-protocol-warm",
            Self::SteamProtocolCold => "steam-protocol-cold",
            Self::DirectExeWarm => "direct-exe-warm",
            Self::DirectExeCold => "direct-exe-cold",
            Self::PublisherLauncher => "publisher-launcher",
            Self::PublisherLauncherWarm => "publisher-launcher-warm",
            Self::PublisherLauncherGameStartCleanWarm => "publisher-launcher-game-start-clean-warm",
            Self::PublisherLauncherCold => "publisher-launcher-cold",
            Self::Controlled => "controlled",
        }
    }
}

/// Ordinary Deep Capture or one explicit compatibility calibration phase.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionMode {
    /// Consume known compatibility evidence.
    Capture,
    /// Establish whether traffic reaches the final client through the proxy.
    ReachabilityCalibration,
    /// Establish inspectability after explicit current-user trust.
    TlsCalibration,
}

/// Explicit compatibility-calibration phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalibrationPhase {
    Reachability,
    Tls,
}

impl CalibrationPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reachability => "reachability",
            Self::Tls => "tls",
        }
    }
}

/// Evidence-based result of one compatibility calibration.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalibrationOutcome {
    Failed,
    Interrupted,
    ReachedClient,
    LauncherOnly,
    EscapedTree,
    ProxyNotReached,
    NoRelevantTraffic,
    Inconclusive,
    LocalCaAccepted,
    CertificatePinned,
    UnknownTrust,
    MetadataOnly,
    UnsupportedProtocol,
}

impl fmt::Display for CalibrationOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Failed => "failed",
            Self::Interrupted => "interrupted",
            Self::ReachedClient => "reached-client",
            Self::LauncherOnly => "launcher-only",
            Self::EscapedTree => "escaped-tree",
            Self::ProxyNotReached => "proxy-not-reached",
            Self::NoRelevantTraffic => "no-relevant-traffic",
            Self::Inconclusive => "inconclusive",
            Self::LocalCaAccepted => "local-ca-accepted",
            Self::CertificatePinned => "certificate-pinned",
            Self::UnknownTrust => "unknown-trust",
            Self::MetadataOnly => "metadata-only",
            Self::UnsupportedProtocol => "unsupported-protocol",
        })
    }
}

/// Caller intent for one target-scoped session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionConfig {
    /// Stored target handle or identifier.
    pub target: String,
    /// Declared launch shape.
    pub launch_case: Option<LaunchCase>,
    /// Requested operation.
    pub mode: SessionMode,
    /// Whether the committed controlled target adapter is selected.
    pub controlled: bool,
    /// Session bundle destination.
    pub bundle: PathBuf,
    /// Whether current-user CA trust is explicitly authorized.
    pub trust_ca: bool,
    /// Whether HAR output is requested.
    pub har: bool,
    /// Whether TLS key logging is requested.
    pub key_log: bool,
    /// Whether an explicit operator-owned upstream client identity is configured.
    pub client_identity: bool,
    /// Lifecycle policy for sensitive bundle artifacts.
    pub sensitive_retention: SensitiveRetention,
    /// Requested lifecycle deadlines.
    pub deadlines: Deadlines,
}

/// Optional sensitive artifacts explicitly authorized for one session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArtifactRequests {
    /// Whether HAR output is requested.
    pub har: bool,
    /// Whether TLS key logging is requested.
    pub key_log: bool,
    /// Authorized post-session handling policy.
    pub sensitive_retention: SensitiveRetention,
}

/// Finite lifecycle deadlines.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Deadlines {
    /// Launch deadline.
    pub launch: Duration,
    /// Observation deadline.
    pub observation: Duration,
    /// Proxy stop deadline.
    pub shutdown: Duration,
    /// Total cleanup deadline.
    pub cleanup: Duration,
}

impl Default for Deadlines {
    fn default() -> Self {
        Self {
            launch: Duration::from_secs(30),
            observation: Duration::from_secs(60),
            shutdown: Duration::from_secs(10),
            cleanup: Duration::from_secs(15),
        }
    }
}

/// Remaining cooperative budget for one blocking adapter call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Budget {
    remaining: Duration,
}

impl Budget {
    /// Construct a finite budget.
    pub fn new(remaining: Duration) -> Self {
        Self { remaining }
    }

    /// Return the remaining time.
    pub fn remaining(self) -> Duration {
        self.remaining
    }

    /// Whether the budget was exhausted before the call began.
    pub fn is_exhausted(self) -> bool {
        self.remaining.is_zero()
    }
}

/// Resolved target identity owned by one target-store entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedTarget {
    /// Durable store identifier.
    pub id: i64,
    /// Human-readable handle.
    pub handle: String,
    /// Launch case observed during preflight.
    pub launch_case: LaunchCase,
}

/// Opaque ordinary Capture preparation produced without starting capture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedCapture {
    /// Adapter-owned token returned to the same Capture runner.
    pub token: String,
}

/// Result returned by the ordinary Capture adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureRunResult {
    /// Observations available directly from the Capture integration.
    pub observations: Vec<CompatibilityObservation>,
    /// Whether an operator interrupt ended the run.
    pub interrupted: bool,
}

/// Immutable action plan reviewed by the caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionPlan {
    /// Stable plan identifier.
    pub id: PlanId,
    /// Session identifier used by events and artifacts.
    pub session_id: String,
    /// Resolved target.
    pub target: PreparedTarget,
    /// Requested mode.
    pub mode: SessionMode,
    /// Whether the committed controlled target adapter is selected.
    pub controlled: bool,
    /// Selected proxy backend descriptor.
    pub proxy_backend: BackendDescriptor,
    /// Loopback endpoint reserved for the session.
    pub endpoint: LoopbackEndpoint,
    /// Bundle destination.
    pub bundle: PathBuf,
    /// Immutable target-scoped routing effects authorized with this plan.
    pub routing: super::RoutingPlan,
    /// Whether trust will be acquired.
    pub trust_ca: bool,
    /// Whether the plan is bound to an explicit upstream client identity.
    pub client_identity: bool,
    /// Optional sensitive artifacts authorized with this exact plan.
    pub artifacts: ArtifactRequests,
    /// Effective bounded deadlines.
    pub deadlines: Deadlines,
}

/// Caller decision bound to one prepared plan.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Authorization {
    /// Execute the exact named plan.
    Approved { plan_id: PlanId },
    /// Apply no effects.
    Declined,
}

impl Authorization {
    /// Approve one immutable plan.
    pub fn approved(plan_id: PlanId) -> Self {
        Self::Approved { plan_id }
    }
}

/// A loopback-only proxy endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoopbackEndpoint {
    address: SocketAddr,
}

impl LoopbackEndpoint {
    /// Construct one exact loopback endpoint.
    pub fn new(address: SocketAddr) -> Result<Self, PreflightRefusal> {
        let mapped = matches!(
            address.ip(),
            std::net::IpAddr::V6(ip) if ip.to_ipv4_mapped().is_some()
        );
        if mapped || !address.ip().is_loopback() {
            return Err(PreflightRefusal::new(
                "non-loopback-endpoint",
                "Deep Capture endpoints must use an exact loopback address",
            ));
        }
        Ok(Self { address })
    }

    /// Return the exact authorized socket address.
    pub fn address(self) -> SocketAddr {
        self.address
    }

    /// Return the authorized port.
    pub fn port(self) -> u16 {
        self.address.port()
    }
}

#[cfg(test)]
mod loopback_endpoint_tests {
    use super::LoopbackEndpoint;

    #[test]
    fn exact_ipv4_and_ipv6_loopback_are_the_only_valid_families() {
        for valid in ["127.0.0.1:8080", "[::1]:8080"] {
            let address = valid.parse().unwrap();
            assert_eq!(LoopbackEndpoint::new(address).unwrap().address(), address);
        }
        for invalid in [
            "0.0.0.0:8080",
            "[::]:8080",
            "192.0.2.1:8080",
            "[::ffff:127.0.0.1]:8080",
        ] {
            let refusal = LoopbackEndpoint::new(invalid.parse().unwrap()).unwrap_err();
            assert_eq!(refusal.code, "non-loopback-endpoint");
        }
    }
}

/// Selected proxy implementation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendDescriptor {
    /// Stable backend name.
    pub name: String,
    /// Reported implementation version.
    pub version: String,
}

/// Coarse checked lifecycle state.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleState {
    /// Prepared and effect-free.
    Prepared,
    /// Proxy, optional trust, and launch are owned.
    Running,
    /// Observations have been collected.
    Observed,
    /// Proxy and Capture have received bounded stop attempts.
    Stopped,
    /// Facts and cleanup are being finalized.
    Finalizing,
    /// No further operation is valid.
    Terminal,
}

/// One lifecycle operation used in typed transition errors.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    Start,
    Observe,
    Stop,
    Finalize,
    Cleanup,
}

/// Stable lifecycle stage for failures and events.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    Preflight,
    Authorization,
    ProxyStart,
    Trust,
    Routing,
    Launch,
    Capture,
    Observe,
    ProxyStop,
    Facts,
    Cleanup,
    Bundle,
    EventDelivery,
}

/// Machine-readable adapter or policy failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageFailure {
    pub stage: Stage,
    pub code: String,
    pub detail: String,
}

impl StageFailure {
    pub fn new(stage: Stage, code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            stage,
            code: code.into(),
            detail: detail.into(),
        }
    }
}

/// Failure before an executable session exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreflightRefusal {
    pub code: String,
    pub detail: String,
}

impl PreflightRefusal {
    pub fn new(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            detail: detail.into(),
        }
    }
}

/// Invalid lifecycle use. No adapter is called for this error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidTransition {
    pub operation: Operation,
    pub actual: LifecycleState,
    pub allowed: &'static [LifecycleState],
}

/// Application-level inspectability observed by the proxy.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Inspectability {
    Full,
    MetadataOnly,
    Unsupported,
    Inconclusive,
}

impl Inspectability {
    pub fn from_label(value: &str) -> Self {
        match value {
            "full" => Self::Full,
            "metadata-only" => Self::MetadataOnly,
            "unsupported" => Self::Unsupported,
            _ => Self::Inconclusive,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::MetadataOnly => "metadata-only",
            Self::Unsupported => "unsupported",
            Self::Inconclusive => "inconclusive",
        }
    }
}

/// One proxy-side observation with optional packet/process correlation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorrelationState {
    Matched,
    FlowOnly,
    Ambiguous,
    Unavailable,
}

impl CorrelationState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Matched => "matched",
            Self::FlowOnly => "flow-only",
            Self::Ambiguous => "ambiguous",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompatibilityObservation {
    pub flow_id: Option<FlowId>,
    pub proxy_connection_id: String,
    pub client_peer: Option<SocketAddr>,
    pub proxy_local: Option<SocketAddr>,
    pub observed_at: String,
    pub process_id: Option<u32>,
    pub process_image: Option<String>,
    pub role: Option<String>,
    pub attribution: Option<String>,
    pub packet_observations: u64,
    pub packet_observations_unretained: u64,
    pub correlation_state: CorrelationState,
    pub correlation_reason: String,
    pub protocol: String,
    pub inspectability: Inspectability,
    pub method: Option<String>,
    pub url: Option<String>,
    pub status: Option<u16>,
    pub reason: Option<String>,
}

/// One append-only compatibility fact candidate or result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompatibilityFact {
    pub kind: String,
    pub value: String,
    pub evidence: String,
    pub phase: CalibrationPhase,
    pub final_owner_index: Option<usize>,
}

/// One evidence-backed target-store fact selected by library policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompatibilityFactCandidate {
    pub key: CompatibilityFactKey,
    pub value: String,
    pub phase: CalibrationPhase,
    pub final_owner_index: Option<usize>,
}

/// Status of one independent fact append.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FactWriteStatus {
    Appended,
    Skipped { reason: String },
    Failed { code: String, detail: String },
}

/// Result for one proposed fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FactWriteResult {
    pub fact: CompatibilityFact,
    pub status: FactWriteStatus,
}

/// Result of one resource cleanup obligation.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CleanupStatus {
    Released,
    NotNeeded,
    TimedOut,
    Failed,
}

/// One named cleanup result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CleanupResult {
    pub resource: String,
    pub status: CleanupStatus,
    pub reason: String,
}

/// Sensitivity of one bundle artifact.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Sensitivity {
    Metadata,
    Payload,
    Secret,
}

/// Result of one independent artifact write.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactStatus {
    Written,
    Omitted { reason: String },
    Failed { code: String, detail: String },
}

/// One expected or written bundle artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactResult {
    pub role: String,
    pub path: PathBuf,
    pub sensitivity: Sensitivity,
    pub required: bool,
    pub status: ArtifactStatus,
}

/// Operational result before reporting obligations are folded in.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionOutcome {
    Complete,
    Partial,
    Failed,
    Interrupted,
}

/// One failed lifecycle-event delivery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventDeliveryFailure {
    pub sequence: u64,
    pub code: String,
    pub detail: String,
}

/// Immutable post-fact, post-cleanup bundle input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalSnapshot {
    pub session_id: String,
    pub plan_id: PlanId,
    pub target: PreparedTarget,
    pub mode: SessionMode,
    pub controlled: bool,
    pub artifacts: ArtifactRequests,
    pub outcome: SessionOutcome,
    pub observations: Vec<CompatibilityObservation>,
    pub route_verification: Option<super::RouteVerification>,
    pub failures: Vec<StageFailure>,
    pub fact_writes: Vec<FactWriteResult>,
    pub cleanup: Vec<CleanupResult>,
    pub deadlines: Deadlines,
    pub finished_at: SystemTime,
}

/// Authoritative in-memory result of a started or declined session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalReport {
    pub snapshot: TerminalSnapshot,
    pub artifacts: Vec<ArtifactResult>,
    pub event_failures: Vec<EventDeliveryFailure>,
}

impl TerminalReport {
    /// Whether every required operation and reporting obligation completed.
    pub fn is_complete(&self) -> bool {
        self.snapshot.outcome == SessionOutcome::Complete
            && self.event_failures.is_empty()
            && self.artifacts.iter().all(|artifact| {
                !artifact.required || matches!(artifact.status, ArtifactStatus::Written)
            })
    }
}

/// Library-owned lifecycle event; presentation belongs to the consumer.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeepCaptureEvent {
    Plan {
        sequence: u64,
        plan: SessionPlan,
    },
    ProxyStarted {
        sequence: u64,
        session_id: String,
    },
    TrustAcquired {
        sequence: u64,
        session_id: String,
    },
    LaunchStarted {
        sequence: u64,
        session_id: String,
    },
    Started {
        sequence: u64,
        session_id: String,
    },
    Observation {
        sequence: u64,
        session_id: String,
        observation: CompatibilityObservation,
    },
    Cleanup {
        sequence: u64,
        session_id: String,
        result: CleanupResult,
    },
    Terminal {
        sequence: u64,
        report: TerminalSnapshot,
    },
}
