// SPDX-License-Identifier: Apache-2.0

//! Immutable target-scoped routing plans and applied route ownership.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
};

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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum BypassHost {
    DnsSuffix(String),
    Ip(IpAddr),
    Cidr { network: IpAddr, prefix: u8 },
}

/// One canonical, deterministic proxy-bypass predicate.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BypassRule {
    host: BypassHost,
    port: Option<u16>,
}

impl BypassRule {
    /// Parse one conventional `NO_PROXY` token under fragcap's closed grammar.
    pub fn parse(value: &str) -> Result<Self, PreflightRefusal> {
        parse_bypass_rule(value).map_err(|detail| {
            PreflightRefusal::new("proxy-bypass-invalid", format!("{value:?}: {detail}"))
        })
    }

    /// Whether this rule selects the canonical requested authority.
    pub fn matches(&self, host: &str, port: u16) -> bool {
        if self.port.is_some_and(|expected| expected != port) {
            return false;
        }
        match &self.host {
            BypassHost::DnsSuffix(expected) => canonical_dns(host).is_ok_and(|host| {
                host == *expected
                    || host
                        .strip_suffix(expected)
                        .is_some_and(|prefix| prefix.ends_with('.'))
            }),
            BypassHost::Ip(expected) => host
                .parse::<IpAddr>()
                .map(canonical_ip)
                .is_ok_and(|host| &host == expected),
            BypassHost::Cidr { network, prefix } => host
                .parse::<IpAddr>()
                .map(canonical_ip)
                .is_ok_and(|host| cidr_contains(*network, *prefix, host)),
        }
    }
}

impl fmt::Display for BypassRule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let host = match &self.host {
            BypassHost::DnsSuffix(value) => format!(".{value}"),
            BypassHost::Ip(IpAddr::V4(value)) => value.to_string(),
            BypassHost::Ip(IpAddr::V6(value)) if self.port.is_some() => format!("[{value}]"),
            BypassHost::Ip(IpAddr::V6(value)) => value.to_string(),
            BypassHost::Cidr { network, prefix } => format!("{network}/{prefix}"),
        };
        match self.port {
            Some(port) => write!(formatter, "{host}:{port}"),
            None => formatter.write_str(&host),
        }
    }
}

/// Stable result of evaluating one requested destination against route scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BypassDecision {
    pub outcome: BypassDecisionOutcome,
    pub authority: &'static str,
    pub reason: &'static str,
    pub matching_rule: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BypassDecisionOutcome {
    Proxied,
    Bypassed,
    Infrastructure,
}

impl BypassDecisionOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Proxied => "proxied",
            Self::Bypassed => "bypassed",
            Self::Infrastructure => "infrastructure",
        }
    }
}

/// Immutable operator policy bound to one exact listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BypassPolicy {
    listener: SocketAddr,
    operator_rules: Vec<BypassRule>,
}

impl BypassPolicy {
    /// Validate endpoint-independent syntax before reserving a listener.
    pub fn validate_inputs(inputs: &[String]) -> Result<(), PreflightRefusal> {
        parse_bypass_inputs(inputs).map(|_| ())
    }

    pub fn parse(listener: SocketAddr, inputs: &[String]) -> Result<Self, PreflightRefusal> {
        let listener = canonical_socket(listener);
        let rules = parse_bypass_inputs(inputs)?;
        for rule in &rules {
            if rule.matches(&listener.ip().to_string(), listener.port()) {
                return Err(PreflightRefusal::new(
                    "proxy-bypass-infrastructure-collision",
                    format!("rule {rule} includes the session proxy listener"),
                ));
            }
        }
        Ok(Self {
            listener,
            operator_rules: rules,
        })
    }

    pub fn operator_rules(&self) -> &[BypassRule] {
        &self.operator_rules
    }

    pub fn infrastructure(&self) -> String {
        self.listener.to_string()
    }

    pub fn environment_value(&self) -> String {
        self.operator_rules
            .iter()
            .map(ToString::to_string)
            .chain(std::iter::once(self.infrastructure()))
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn decide(&self, host: &str, port: u16) -> BypassDecision {
        let listener_host_matches = host
            .parse::<IpAddr>()
            .map(canonical_ip)
            .is_ok_and(|host| host == self.listener.ip());
        if listener_host_matches && port == self.listener.port() {
            return BypassDecision {
                outcome: BypassDecisionOutcome::Infrastructure,
                authority: "session-infrastructure",
                reason: "session-proxy-listener",
                matching_rule: Some(self.infrastructure()),
            };
        }
        if let Some(rule) = self
            .operator_rules
            .iter()
            .find(|rule| rule.matches(host, port))
        {
            return BypassDecision {
                outcome: BypassDecisionOutcome::Bypassed,
                authority: "operator-policy",
                reason: "explicit-proxy-bypass-rule",
                matching_rule: Some(rule.to_string()),
            };
        }
        BypassDecision {
            outcome: BypassDecisionOutcome::Proxied,
            authority: "target-routing-plan",
            reason: "no-proxy-bypass-rule-matched",
            matching_rule: None,
        }
    }
}

fn parse_bypass_inputs(inputs: &[String]) -> Result<Vec<BypassRule>, PreflightRefusal> {
    let mut rules = BTreeSet::new();
    for input in inputs {
        for token in input.split(',') {
            if token.trim().is_empty() {
                return Err(PreflightRefusal::new(
                    "proxy-bypass-invalid",
                    "proxy bypass contains an empty rule",
                ));
            }
            rules.insert(BypassRule::parse(token.trim())?);
        }
    }
    Ok(rules.into_iter().collect())
}

fn parse_bypass_rule(value: &str) -> Result<BypassRule, String> {
    if value == "*" {
        return Err("the complete-bypass wildcard is forbidden".to_string());
    }
    if value.is_empty() || value.contains("//") || value.contains(['?', '#', '@']) {
        return Err("schemes, paths, queries, fragments, and user-info are forbidden".to_string());
    }
    if let Some((address, prefix)) = value.rsplit_once('/') {
        let address = IpAddr::from_str(address)
            .map(canonical_ip)
            .map_err(|_| "CIDR requires an IP literal".to_string())?;
        let prefix = prefix
            .parse::<u8>()
            .map_err(|_| "CIDR prefix is invalid".to_string())?;
        let max = if address.is_ipv4() { 32 } else { 128 };
        if prefix > max || canonical_network(address, prefix) != address {
            return Err("CIDR prefix is out of range or the address has host bits".to_string());
        }
        return Ok(BypassRule {
            host: BypassHost::Cidr {
                network: address,
                prefix,
            },
            port: None,
        });
    }

    let (raw_host, port) = split_host_port(value)?;
    if let Ok(address) = raw_host.parse::<IpAddr>() {
        return Ok(BypassRule {
            host: BypassHost::Ip(canonical_ip(address)),
            port,
        });
    }
    let dns = canonical_dns(raw_host.strip_prefix('.').unwrap_or(raw_host))?;
    Ok(BypassRule {
        host: BypassHost::DnsSuffix(dns),
        port,
    })
}

fn split_host_port(value: &str) -> Result<(&str, Option<u16>), String> {
    if let Some(rest) = value.strip_prefix('[') {
        let (host, port) = rest
            .split_once("]:")
            .ok_or_else(|| "bracketed IPv6 requires a port".to_string())?;
        return Ok((host, Some(parse_port(port)?)));
    }
    if value.matches(':').count() > 1 {
        return Ok((value, None));
    }
    if let Some((host, port)) = value.rsplit_once(':') {
        if host.is_empty() {
            return Err("host is empty".to_string());
        }
        return Ok((host, Some(parse_port(port)?)));
    }
    Ok((value, None))
}

fn parse_port(value: &str) -> Result<u16, String> {
    value
        .parse::<u16>()
        .ok()
        .filter(|port| *port != 0)
        .ok_or_else(|| "port must be in 1..=65535".to_string())
}

fn canonical_dns(value: &str) -> Result<String, String> {
    let value = value
        .strip_suffix('.')
        .unwrap_or(value)
        .to_ascii_lowercase();
    if value.is_empty()
        || value.len() > 253
        || !value.is_ascii()
        || value.split('.').any(|label| {
            label.is_empty()
                || label.len() > 63
                || label.starts_with('-')
                || label.ends_with('-')
                || !label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
    {
        return Err("DNS name is not canonical ASCII label syntax".to_string());
    }
    Ok(value)
}

fn canonical_ip(address: IpAddr) -> IpAddr {
    match address {
        IpAddr::V6(address) => address
            .to_ipv4_mapped()
            .map(IpAddr::V4)
            .unwrap_or(IpAddr::V6(address)),
        address => address,
    }
}

fn canonical_socket(address: SocketAddr) -> SocketAddr {
    SocketAddr::new(canonical_ip(address.ip()), address.port())
}

fn canonical_network(address: IpAddr, prefix: u8) -> IpAddr {
    match address {
        IpAddr::V4(address) => {
            let mask = if prefix == 0 {
                0
            } else {
                u32::MAX << (32 - prefix)
            };
            IpAddr::V4(Ipv4Addr::from(u32::from(address) & mask))
        }
        IpAddr::V6(address) => {
            let mask = if prefix == 0 {
                0
            } else {
                u128::MAX << (128 - prefix)
            };
            IpAddr::V6(Ipv6Addr::from(u128::from(address) & mask))
        }
    }
}

fn cidr_contains(network: IpAddr, prefix: u8, address: IpAddr) -> bool {
    network.is_ipv4() == address.is_ipv4() && canonical_network(address, prefix) == network
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
    pub bypass: Option<BypassPolicy>,
}

impl RoutingPlan {
    pub fn child_environment(
        listener: super::LoopbackEndpoint,
        bypass_inputs: &[String],
    ) -> Result<Self, PreflightRefusal> {
        let bypass = BypassPolicy::parse(listener.address(), bypass_inputs)?;
        let no_proxy = bypass.environment_value();
        Ok(Self {
            strategy: RoutingStrategyKind::ChildEnvironment,
            availability: RoutingAvailability::Implemented,
            effects: ["HTTP_PROXY", "HTTPS_PROXY", "http_proxy", "https_proxy"]
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
                    destination: "all_proxy".to_string(),
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
                    value: RouteValueSource::Literal(no_proxy.clone()),
                    scope: "managed-child-only".to_string(),
                }))
                .chain(std::iter::once(RouteEffect {
                    destination: "no_proxy".to_string(),
                    value: RouteValueSource::Literal(no_proxy),
                    scope: "managed-child-only".to_string(),
                }))
                .collect(),
            verification: "packet-flow-and-socket-owner-evidence".to_string(),
            cleanup: vec!["child-environment-ends-with-process".to_string()],
            bypass: Some(bypass),
        })
    }

    pub fn planned(strategy: RoutingStrategyKind) -> Self {
        Self {
            strategy,
            availability: RoutingAvailability::Planned,
            effects: Vec::new(),
            verification: "not-implemented".to_string(),
            cleanup: Vec::new(),
            bypass: None,
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
            || self.bypass.is_none()
            || self.effects.iter().any(|effect| {
                effect.scope != "managed-child-only" || effect.destination.trim().is_empty()
            })
        {
            return Err(PreflightRefusal::new(
                "routing-plan-invalid",
                "the routing plan lacks bypass policy or contains an invalid effect",
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

    fn endpoint(value: &str) -> LoopbackEndpoint {
        LoopbackEndpoint::new(value.parse().unwrap()).unwrap()
    }

    #[test]
    fn bypass_rules_are_canonical_deduplicated_and_order_independent() {
        let left = BypassPolicy::parse(
            endpoint("127.0.0.1:8080").address(),
            &[
                "Example.COM.,.example.net,192.0.2.0/24".to_string(),
                "example.com,[2001:db8::1]:443".to_string(),
            ],
        )
        .unwrap();
        let right = BypassPolicy::parse(
            endpoint("127.0.0.1:8080").address(),
            &[
                "[2001:db8::1]:443,192.0.2.0/24".to_string(),
                ".EXAMPLE.NET,example.com".to_string(),
            ],
        )
        .unwrap();
        assert_eq!(left, right);
        assert_eq!(
            left.operator_rules()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec![
                ".example.com",
                ".example.net",
                "[2001:db8::1]:443",
                "192.0.2.0/24"
            ]
        );
    }

    #[test]
    fn bypass_matching_respects_dns_boundaries_ports_and_cidrs() {
        let policy = BypassPolicy::parse(
            endpoint("127.0.0.1:8080").address(),
            &[".example.com,api.test:443,192.0.2.0/24,2001:db8::/32".to_string()],
        )
        .unwrap();
        for (host, port) in [
            ("example.com", 80),
            ("A.Example.com.", 443),
            ("api.test", 443),
            ("sub.api.test", 443),
            ("192.0.2.99", 53),
            ("2001:db8::99", 53),
        ] {
            assert_eq!(
                policy.decide(host, port).outcome,
                BypassDecisionOutcome::Bypassed
            );
        }
        for (host, port) in [
            ("notexample.com", 80),
            ("api.test", 80),
            ("192.0.3.1", 53),
            ("2001:db9::1", 53),
        ] {
            assert_eq!(
                policy.decide(host, port).outcome,
                BypassDecisionOutcome::Proxied
            );
        }
    }

    #[test]
    fn unsafe_or_ambiguous_rules_refuse() {
        for value in [
            "*",
            "",
            "https://example.com",
            "user@example.com",
            "example..com",
            "example.com:0",
            "..example.com",
            "192.0.2.1/24",
            "[2001:db8::1]",
        ] {
            assert!(BypassRule::parse(value).is_err(), "accepted {value:?}");
        }
    }

    #[test]
    fn listener_is_infrastructure_and_cannot_be_operator_bypass() {
        let listener = endpoint("[::1]:8080");
        let policy = BypassPolicy::parse(listener.address(), &[]).unwrap();
        assert_eq!(
            policy.decide("::1", 8080).outcome,
            BypassDecisionOutcome::Infrastructure
        );
        assert_eq!(
            BypassPolicy::parse(listener.address(), &["[::1]:8080".to_string()])
                .unwrap_err()
                .code,
            "proxy-bypass-infrastructure-collision"
        );
    }

    #[test]
    fn mapped_listener_and_explicit_localhost_aliases_have_distinct_authority() {
        let listener = endpoint("127.0.0.1:8080");
        let policy = BypassPolicy::parse(
            listener.address(),
            &["localhost:9000,127.0.0.2:9000".to_string()],
        )
        .unwrap();
        assert_eq!(
            policy.decide("::ffff:127.0.0.1", 8080).outcome,
            BypassDecisionOutcome::Infrastructure
        );
        for host in ["localhost", "127.0.0.2"] {
            let decision = policy.decide(host, 9000);
            assert_eq!(decision.outcome, BypassDecisionOutcome::Bypassed);
            assert_eq!(decision.authority, "operator-policy");
        }
        assert_eq!(
            policy.decide("127.0.0.2", 9001).outcome,
            BypassDecisionOutcome::Proxied
        );
    }

    #[test]
    fn applied_route_owns_uppercase_and_lowercase_proxy_environment() {
        let endpoint = endpoint("127.0.0.1:8080");
        let plan = RoutingPlan::child_environment(endpoint, &["example.com".to_string()]).unwrap();
        let proxy = ProxyRoute::new(
            endpoint,
            (
                Zeroizing::new("http://fragcap:secret@127.0.0.1:8080".to_string()),
                Zeroizing::new("socks5h://fragcap:secret@127.0.0.1:8080".to_string()),
            ),
            Zeroizing::new("Basic secret".to_string()),
            Vec::new(),
            "thumbprint".to_string(),
            1,
            None,
        );
        let applied = AppliedRoute::child_environment(&plan, proxy).unwrap();
        let environment = applied.environment().collect::<BTreeMap<_, _>>();
        assert_eq!(environment["HTTP_PROXY"], environment["http_proxy"]);
        assert_eq!(environment["HTTPS_PROXY"], environment["https_proxy"]);
        assert_eq!(environment["ALL_PROXY"], environment["all_proxy"]);
        assert_eq!(environment["NO_PROXY"], environment["no_proxy"]);
        assert_eq!(environment["NO_PROXY"], ".example.com,127.0.0.1:8080");
    }

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
