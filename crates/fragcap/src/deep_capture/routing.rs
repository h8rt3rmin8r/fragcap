// SPDX-License-Identifier: Apache-2.0

//! Immutable target-scoped routing plans and applied route ownership.

use std::collections::BTreeMap;

use zeroize::Zeroizing;

use super::{
    Budget, CleanupResult, CleanupStatus, CompatibilityObservation, PreflightRefusal,
    PreparedTarget, ProxyRoute, SessionPlan, Stage, StageFailure,
};

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoutingStrategyKind {
    ChildEnvironment,
    CommandArguments,
    TargetConfiguration,
    HttpProxy,
    Socks,
    ProtocolSpecific,
}

impl RoutingStrategyKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ChildEnvironment => "child-environment",
            Self::CommandArguments => "command-arguments",
            Self::TargetConfiguration => "target-configuration",
            Self::HttpProxy => "http-proxy",
            Self::Socks => "socks",
            Self::ProtocolSpecific => "protocol-specific",
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoutingAvailability {
    Implemented,
    Planned,
    Refused,
}

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteValueSource {
    SessionProxyUrl,
    SessionSocks5hUrl,
    SessionProxyAuthorization,
    Literal(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteEffect {
    pub destination: String,
    pub value: RouteValueSource,
    pub scope: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingPlan {
    pub strategy: RoutingStrategyKind,
    pub availability: RoutingAvailability,
    pub effects: Vec<RouteEffect>,
    pub verification: String,
    pub cleanup: Vec<String>,
}

impl RoutingPlan {
    pub fn child_environment() -> Self {
        Self {
            strategy: RoutingStrategyKind::ChildEnvironment,
            availability: RoutingAvailability::Implemented,
            effects: ["HTTP_PROXY", "HTTPS_PROXY"]
                .into_iter()
                .map(|name| RouteEffect {
                    destination: name.to_string(),
                    value: RouteValueSource::SessionProxyUrl,
                    scope: "managed-child-only".to_string(),
                })
                .chain(std::iter::once(RouteEffect {
                    destination: "ALL_PROXY".to_string(),
                    value: RouteValueSource::SessionSocks5hUrl,
                    scope: "managed-child-only".to_string(),
                }))
                .chain(std::iter::once(RouteEffect {
                    destination: "FRAGCAP_PROXY_AUTHORIZATION".to_string(),
                    value: RouteValueSource::SessionProxyAuthorization,
                    scope: "managed-child-only".to_string(),
                }))
                .chain(std::iter::once(RouteEffect {
                    destination: "NO_PROXY".to_string(),
                    value: RouteValueSource::Literal(String::new()),
                    scope: "managed-child-only".to_string(),
                }))
                .collect(),
            verification: "packet-flow-and-socket-owner-evidence".to_string(),
            cleanup: vec!["child-environment-ends-with-process".to_string()],
        }
    }

    pub fn planned(strategy: RoutingStrategyKind) -> Self {
        Self {
            strategy,
            availability: RoutingAvailability::Planned,
            effects: Vec::new(),
            verification: "not-implemented".to_string(),
            cleanup: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), PreflightRefusal> {
        if self.availability != RoutingAvailability::Implemented {
            return Err(PreflightRefusal::new(
                "routing-strategy-unavailable",
                format!(
                    "routing strategy {} is not implemented",
                    self.strategy.as_str()
                ),
            ));
        }
        if self.effects.is_empty()
            || self.effects.iter().any(|effect| {
                effect.scope != "managed-child-only" || effect.destination.trim().is_empty()
            })
        {
            return Err(PreflightRefusal::new(
                "routing-plan-invalid",
                "the routing plan is empty or contains a non-child-scoped effect",
            ));
        }
        Ok(())
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouteVerificationState {
    ReachedSocketOwner,
    NotReached,
    EscapedTree,
    Ambiguous,
    Unavailable,
    NotAttempted,
}

impl RouteVerificationState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReachedSocketOwner => "reached-socket-owner",
            Self::NotReached => "not-reached",
            Self::EscapedTree => "escaped-tree",
            Self::Ambiguous => "ambiguous",
            Self::Unavailable => "unavailable",
            Self::NotAttempted => "not-attempted",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteVerification {
    pub state: RouteVerificationState,
    pub reason: String,
}

/// Secret-bearing child environment resolved only after the exact proxy exists.
pub struct AppliedRoute {
    proxy: ProxyRoute,
    environment: BTreeMap<String, Zeroizing<String>>,
}

impl AppliedRoute {
    fn child_environment(plan: &RoutingPlan, proxy: ProxyRoute) -> Result<Self, StageFailure> {
        plan.validate()
            .map_err(|error| StageFailure::new(Stage::Routing, error.code, error.detail))?;
        let mut environment = BTreeMap::new();
        for effect in &plan.effects {
            let value = match &effect.value {
                RouteValueSource::SessionProxyUrl => proxy.proxy_url().to_string(),
                RouteValueSource::SessionSocks5hUrl => proxy.socks5h_url().to_string(),
                RouteValueSource::SessionProxyAuthorization => {
                    proxy.proxy_authorization().to_string()
                }
                RouteValueSource::Literal(value) => value.clone(),
            };
            environment.insert(effect.destination.clone(), Zeroizing::new(value));
        }
        Ok(Self { proxy, environment })
    }

    pub fn proxy(&self) -> &ProxyRoute {
        &self.proxy
    }

    pub fn environment(&self) -> impl Iterator<Item = (&str, &str)> {
        self.environment
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }
}

impl std::fmt::Debug for AppliedRoute {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AppliedRoute")
            .field("proxy", &self.proxy)
            .field("environment_names", &self.environment.keys())
            .finish()
    }
}

pub trait RoutingLease {
    fn applied(&self) -> &AppliedRoute;

    fn verify(&self, observations: &[CompatibilityObservation]) -> RouteVerification {
        if observations.iter().any(|observation| {
            observation.process_id.is_some()
                && observation.flow_id.is_some()
                && observation.role.as_deref() == Some("client")
        }) {
            RouteVerification {
                state: RouteVerificationState::ReachedSocketOwner,
                reason: "packet flow and final client ownership were observed".to_string(),
            }
        } else if observations
            .iter()
            .any(|observation| observation.correlation_state == super::CorrelationState::Ambiguous)
        {
            RouteVerification {
                state: RouteVerificationState::Ambiguous,
                reason: "competing packet or process owners remained".to_string(),
            }
        } else if observations.is_empty() {
            RouteVerification {
                state: RouteVerificationState::Unavailable,
                reason: "no relevant proxy observation was retained".to_string(),
            }
        } else {
            RouteVerification {
                state: RouteVerificationState::NotReached,
                reason: "proxy evidence did not resolve to the final socket owner".to_string(),
            }
        }
    }

    fn cleanup(&mut self, _budget: Budget) -> CleanupResult {
        CleanupResult {
            resource: "target-route".to_string(),
            status: CleanupStatus::Released,
            reason: "child-only route ended with the managed process".to_string(),
        }
    }
}

pub trait RoutingAdapter {
    fn prepare(
        &mut self,
        target: &PreparedTarget,
        plan: &RoutingPlan,
    ) -> Result<(), PreflightRefusal>;

    fn apply(
        &mut self,
        session: &SessionPlan,
        proxy: ProxyRoute,
        budget: Budget,
    ) -> Result<Box<dyn RoutingLease>, StageFailure>;
}

#[derive(Default)]
pub struct ChildEnvironmentRouting;

impl RoutingAdapter for ChildEnvironmentRouting {
    fn prepare(
        &mut self,
        _target: &PreparedTarget,
        plan: &RoutingPlan,
    ) -> Result<(), PreflightRefusal> {
        plan.validate()
    }

    fn apply(
        &mut self,
        session: &SessionPlan,
        proxy: ProxyRoute,
        _budget: Budget,
    ) -> Result<Box<dyn RoutingLease>, StageFailure> {
        if session.routing.strategy != RoutingStrategyKind::ChildEnvironment {
            return Err(StageFailure::new(
                Stage::Routing,
                "routing-strategy-unavailable",
                "only child-environment routing is implemented",
            ));
        }
        Ok(Box::new(ChildEnvironmentLease {
            applied: AppliedRoute::child_environment(&session.routing, proxy)?,
        }))
    }
}

struct ChildEnvironmentLease {
    applied: AppliedRoute,
}

impl RoutingLease for ChildEnvironmentLease {
    fn applied(&self) -> &AppliedRoute {
        &self.applied
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep_capture::{
        ArtifactRequests, BackendDescriptor, Deadlines, LaunchCase, LoopbackEndpoint, PlanId,
        PreparedTarget, SensitiveRetention, SessionMode,
    };
    use std::path::PathBuf;

    fn session(routing: RoutingPlan) -> SessionPlan {
        SessionPlan {
            id: PlanId::new("plan"),
            session_id: "session".to_string(),
            target: PreparedTarget {
                id: 1,
                handle: "target".to_string(),
                launch_case: LaunchCase::DirectExeCold,
            },
            mode: SessionMode::Capture,
            calibration_protocol: None,
            controlled: false,
            proxy_backend: BackendDescriptor {
                name: "test".to_string(),
                version: "1".to_string(),
            },
            endpoint: LoopbackEndpoint::new("127.0.0.1:8080".parse().unwrap()).unwrap(),
            bundle: PathBuf::from("bundle"),
            routing,
            trust_ca: true,
            client_identity: false,
            artifacts: ArtifactRequests {
                har: false,
                key_log: false,
                sensitive_retention: SensitiveRetention::Retain,
            },
            deadlines: Deadlines::default(),
        }
    }

    #[test]
    fn future_strategy_refuses_before_application() {
        let mut adapter = ChildEnvironmentRouting;
        let routing = RoutingPlan::planned(RoutingStrategyKind::Socks);
        assert_eq!(
            adapter
                .prepare(&session(routing.clone()).target, &routing)
                .unwrap_err()
                .code,
            "routing-strategy-unavailable"
        );
    }
}
