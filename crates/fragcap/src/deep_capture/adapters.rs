// SPDX-License-Identifier: Apache-2.0

use std::path::Path;
use std::time::{Duration, SystemTime};

use zeroize::Zeroizing;

use super::{
    ArtifactResult, BackendDescriptor, Budget, CleanupResult, CompatibilityFact,
    CompatibilityObservation, DeepCaptureEvent, FactWriteStatus, LaunchCase, LoopbackEndpoint,
    PreflightRefusal, PreparedCapture, PreparedTarget, RoutingAdapter, SessionConfig, StageFailure,
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
    /// Child-only route and trust material produced by this exact proxy session.
    fn route(&self) -> Result<ProxyRoute, StageFailure>;

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
        route: &ProxyRoute,
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
        route: &super::AppliedRoute,
        budget: Budget,
    ) -> Result<Box<dyn LaunchLease>, StageFailure>;
}

/// Existing ordinary Capture composition used by Deep Capture.
pub trait CaptureRunner {
    fn prepare(
        &mut self,
        config: &SessionConfig,
        target: &PreparedTarget,
        endpoint: LoopbackEndpoint,
    ) -> Result<PreparedCapture, PreflightRefusal>;

    fn run(
        &mut self,
        prepared: &PreparedCapture,
        route: &super::AppliedRoute,
        budget: Budget,
    ) -> Result<super::CaptureRunResult, StageFailure>;

    fn stop(&mut self, budget: Budget) -> CleanupResult;
}

/// Secret-bearing, session-scoped route passed only to effect adapters.
pub struct ProxyRoute {
    endpoint: LoopbackEndpoint,
    proxy_url: Zeroizing<String>,
    proxy_authorization: Zeroizing<String>,
    ca_der: Vec<u8>,
    ca_sha1_thumbprint: String,
    authority_generation: u64,
    controlled_origins: Option<(std::net::SocketAddr, std::net::SocketAddr)>,
}

impl ProxyRoute {
    pub fn new(
        endpoint: LoopbackEndpoint,
        proxy_url: impl Into<Zeroizing<String>>,
        proxy_authorization: impl Into<Zeroizing<String>>,
        ca_der: Vec<u8>,
        ca_sha1_thumbprint: String,
        authority_generation: u64,
        controlled_origins: Option<(std::net::SocketAddr, std::net::SocketAddr)>,
    ) -> Self {
        Self {
            endpoint,
            proxy_url: proxy_url.into(),
            proxy_authorization: proxy_authorization.into(),
            ca_der,
            ca_sha1_thumbprint,
            authority_generation,
            controlled_origins,
        }
    }

    pub fn endpoint(&self) -> LoopbackEndpoint {
        self.endpoint
    }

    pub fn proxy_url(&self) -> &str {
        self.proxy_url.as_str()
    }

    pub fn proxy_authorization(&self) -> &str {
        self.proxy_authorization.as_str()
    }

    pub fn ca_der(&self) -> &[u8] {
        &self.ca_der
    }

    pub fn ca_sha1_thumbprint(&self) -> &str {
        &self.ca_sha1_thumbprint
    }

    pub fn authority_generation(&self) -> u64 {
        self.authority_generation
    }

    #[doc(hidden)]
    pub fn controlled_origins(&self) -> Option<(std::net::SocketAddr, std::net::SocketAddr)> {
        self.controlled_origins
    }
}

impl std::fmt::Debug for ProxyRoute {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProxyRoute")
            .field("endpoint", &self.endpoint)
            .field("proxy_url", &"[REDACTED]")
            .field("proxy_authorization", &"[REDACTED]")
            .field("ca_der_bytes", &self.ca_der.len())
            .field("ca_sha1_thumbprint", &self.ca_sha1_thumbprint)
            .field("authority_generation", &self.authority_generation)
            .finish()
    }
}

/// Append-only target-owned compatibility persistence.
pub trait CompatibilityRepository {
    fn append(&mut self, target: &PreparedTarget, fact: &CompatibilityFact) -> FactWriteStatus;
}

/// Bundle destination validation and independent artifact persistence.
pub trait ArtifactSink {
    fn validate_destination(&mut self, path: &Path) -> Result<(), PreflightRefusal>;
    /// Apply authorized protection before any proxy or writer may expose bytes.
    fn prepare(&mut self, _plan: &super::SessionPlan) -> Result<(), StageFailure> {
        Ok(())
    }
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
    pub routing: Box<dyn RoutingAdapter + 'a>,
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
