// SPDX-License-Identifier: Apache-2.0

//! Deep Capture compatibility facts stored with local targets.
//!
//! Compatibility is observed behavior, not a property inferred from a title's
//! platform metadata. The values here are deliberately closed sets where the
//! product needs exact language, and `Unknown` is a real value rather than a
//! prompt to guess.

use std::cmp::Ordering;

use crate::TargetsError;

/// Which Deep Capture behavior a row records.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CompatibilityFactKey {
    ProxyEnvironmentHonored,
    ProxyRouting,
    ProxyPropagation,
    LaunchCase,
    FinalSocketOwnerRole,
    PublisherLauncherPresent,
    RequiresPlatformColdStartForProxy,
    DirectExeSupported,
    SteamProtocolSupported,
    TlsTrustBehavior,
    ProtocolBehavior,
    Inspectability,
    ProxyVariableTested,
}

impl CompatibilityFactKey {
    /// The stored token for this fact key.
    pub fn as_str(self) -> &'static str {
        match self {
            CompatibilityFactKey::ProxyEnvironmentHonored => "proxy-environment-honored",
            CompatibilityFactKey::ProxyRouting => "proxy-routing",
            CompatibilityFactKey::ProxyPropagation => "proxy-propagation",
            CompatibilityFactKey::LaunchCase => "launch-case",
            CompatibilityFactKey::FinalSocketOwnerRole => "final-socket-owner-role",
            CompatibilityFactKey::PublisherLauncherPresent => "publisher-launcher-present",
            CompatibilityFactKey::RequiresPlatformColdStartForProxy => {
                "requires-platform-cold-start-for-proxy"
            }
            CompatibilityFactKey::DirectExeSupported => "direct-exe-supported",
            CompatibilityFactKey::SteamProtocolSupported => "steam-protocol-supported",
            CompatibilityFactKey::TlsTrustBehavior => "tls-trust-behavior",
            CompatibilityFactKey::ProtocolBehavior => "protocol-behavior",
            CompatibilityFactKey::Inspectability => "inspectability",
            CompatibilityFactKey::ProxyVariableTested => "proxy-variable-tested",
        }
    }

    /// Parse a stored token, rejecting an out-of-set key.
    pub fn parse(s: &str) -> Result<Self, TargetsError> {
        match s {
            "proxy-environment-honored" => Ok(CompatibilityFactKey::ProxyEnvironmentHonored),
            "proxy-routing" => Ok(CompatibilityFactKey::ProxyRouting),
            "proxy-propagation" => Ok(CompatibilityFactKey::ProxyPropagation),
            "launch-case" => Ok(CompatibilityFactKey::LaunchCase),
            "final-socket-owner-role" => Ok(CompatibilityFactKey::FinalSocketOwnerRole),
            "publisher-launcher-present" => Ok(CompatibilityFactKey::PublisherLauncherPresent),
            "requires-platform-cold-start-for-proxy" => {
                Ok(CompatibilityFactKey::RequiresPlatformColdStartForProxy)
            }
            "direct-exe-supported" => Ok(CompatibilityFactKey::DirectExeSupported),
            "steam-protocol-supported" => Ok(CompatibilityFactKey::SteamProtocolSupported),
            "tls-trust-behavior" => Ok(CompatibilityFactKey::TlsTrustBehavior),
            "protocol-behavior" => Ok(CompatibilityFactKey::ProtocolBehavior),
            "inspectability" => Ok(CompatibilityFactKey::Inspectability),
            "proxy-variable-tested" => Ok(CompatibilityFactKey::ProxyVariableTested),
            other => Err(TargetsError::Model(format!(
                "unknown compatibility fact key {other:?}"
            ))),
        }
    }
}

/// A measured launch path, stored alongside facts whose meaning depends on how
/// the target was started.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CompatibilityLaunchCase {
    SteamProtocolWarm,
    SteamProtocolCold,
    DirectExeWarm,
    DirectExeCold,
    PublisherLauncher,
    PublisherLauncherWarm,
    PublisherLauncherGameStartCleanWarm,
    PublisherLauncherCold,
}

impl CompatibilityLaunchCase {
    /// The stored token for this launch case.
    pub fn as_str(self) -> &'static str {
        match self {
            CompatibilityLaunchCase::SteamProtocolWarm => "steam-protocol-warm",
            CompatibilityLaunchCase::SteamProtocolCold => "steam-protocol-cold",
            CompatibilityLaunchCase::DirectExeWarm => "direct-exe-warm",
            CompatibilityLaunchCase::DirectExeCold => "direct-exe-cold",
            CompatibilityLaunchCase::PublisherLauncher => "publisher-launcher",
            CompatibilityLaunchCase::PublisherLauncherWarm => "publisher-launcher-warm",
            CompatibilityLaunchCase::PublisherLauncherGameStartCleanWarm => {
                "publisher-launcher-game-start-clean-warm"
            }
            CompatibilityLaunchCase::PublisherLauncherCold => "publisher-launcher-cold",
        }
    }

    /// Parse a stored launch-case token.
    pub fn parse(s: &str) -> Result<Self, TargetsError> {
        match s {
            "steam-protocol-warm" => Ok(CompatibilityLaunchCase::SteamProtocolWarm),
            "steam-protocol-cold" => Ok(CompatibilityLaunchCase::SteamProtocolCold),
            "direct-exe-warm" => Ok(CompatibilityLaunchCase::DirectExeWarm),
            "direct-exe-cold" => Ok(CompatibilityLaunchCase::DirectExeCold),
            "publisher-launcher" => Ok(CompatibilityLaunchCase::PublisherLauncher),
            "publisher-launcher-warm" => Ok(CompatibilityLaunchCase::PublisherLauncherWarm),
            "publisher-launcher-game-start-clean-warm" => {
                Ok(CompatibilityLaunchCase::PublisherLauncherGameStartCleanWarm)
            }
            "publisher-launcher-cold" => Ok(CompatibilityLaunchCase::PublisherLauncherCold),
            other => Err(TargetsError::Model(format!(
                "unknown compatibility launch case {other:?}"
            ))),
        }
    }
}

/// Target-scoped routing strategy used by one observed Deep Capture case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CompatibilityRoutingStrategy {
    ChildEnvironment,
    CommandArguments,
    TargetConfiguration,
    HttpProxy,
    Socks,
    ProtocolSpecific,
}

impl CompatibilityRoutingStrategy {
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

    pub fn parse(value: &str) -> Result<Self, TargetsError> {
        match value {
            "child-environment" => Ok(Self::ChildEnvironment),
            "command-arguments" => Ok(Self::CommandArguments),
            "target-configuration" => Ok(Self::TargetConfiguration),
            "http-proxy" => Ok(Self::HttpProxy),
            "socks" => Ok(Self::Socks),
            "protocol-specific" => Ok(Self::ProtocolSpecific),
            other => Err(TargetsError::Model(format!(
                "unknown compatibility routing strategy {other:?}"
            ))),
        }
    }
}

/// Loopback address family used by one observed Deep Capture case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CompatibilityAddressFamily {
    Ipv4,
    Ipv6,
}

impl CompatibilityAddressFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ipv4 => "ipv4",
            Self::Ipv6 => "ipv6",
        }
    }

    pub fn parse(value: &str) -> Result<Self, TargetsError> {
        match value {
            "ipv4" => Ok(Self::Ipv4),
            "ipv6" => Ok(Self::Ipv6),
            other => Err(TargetsError::Model(format!(
                "unknown compatibility address family {other:?}"
            ))),
        }
    }
}

/// Protocol dimension attached to one compatibility fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompatibilityProtocol {
    Routing,
    Http1,
    Https,
    Http2,
    WebSocket,
    Sse,
    Grpc,
    GenericTcp,
    NonHttpTls,
    Socks5Tcp,
    Socks5Udp,
    GenericUdp,
    Quic,
    Http3,
    NotApplicable,
}

impl CompatibilityProtocol {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Routing => "routing",
            Self::Http1 => "http1",
            Self::Https => "https",
            Self::Http2 => "http2",
            Self::WebSocket => "websocket",
            Self::Sse => "sse",
            Self::Grpc => "grpc",
            Self::GenericTcp => "generic-tcp",
            Self::NonHttpTls => "non-http-tls",
            Self::Socks5Tcp => "socks5-tcp",
            Self::Socks5Udp => "socks5-udp",
            Self::GenericUdp => "generic-udp",
            Self::Quic => "quic",
            Self::Http3 => "http3",
            Self::NotApplicable => "not-applicable",
        }
    }

    pub fn parse(value: &str) -> Result<Self, TargetsError> {
        match value {
            "routing" => Ok(Self::Routing),
            "http1" => Ok(Self::Http1),
            "https" => Ok(Self::Https),
            "http2" => Ok(Self::Http2),
            "websocket" => Ok(Self::WebSocket),
            "sse" => Ok(Self::Sse),
            "grpc" => Ok(Self::Grpc),
            "generic-tcp" => Ok(Self::GenericTcp),
            "non-http-tls" => Ok(Self::NonHttpTls),
            "socks5-tcp" => Ok(Self::Socks5Tcp),
            "socks5-udp" => Ok(Self::Socks5Udp),
            "generic-udp" => Ok(Self::GenericUdp),
            "quic" => Ok(Self::Quic),
            "http3" => Ok(Self::Http3),
            "not-applicable" => Ok(Self::NotApplicable),
            other => Err(TargetsError::Model(format!(
                "unknown compatibility protocol {other:?}"
            ))),
        }
    }
}

/// Exact current context against which stored evidence is evaluated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompatibilityCase {
    pub launch_case: CompatibilityLaunchCase,
    pub proxy_backend: String,
    pub proxy_backend_version: String,
    pub routing_strategy: CompatibilityRoutingStrategy,
    pub address_family: CompatibilityAddressFamily,
    pub protocol: CompatibilityProtocol,
    pub fragcap_version: String,
    pub target_version: Option<String>,
}

/// Why one stored fact can or cannot apply to an exact current case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompatibilityApplicability {
    Applicable,
    Stale,
    LegacyIncomplete,
    Mismatch(&'static str),
}

impl CompatibilityApplicability {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Applicable => "applicable",
            Self::Stale => "stale",
            Self::LegacyIncomplete => "legacy-incomplete",
            Self::Mismatch(_) => "mismatch",
        }
    }

    pub fn dimension(self) -> Option<&'static str> {
        match self {
            Self::Mismatch(dimension) => Some(dimension),
            _ => None,
        }
    }
}

/// Where a compatibility fact came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CompatibilityEvidenceSource {
    ObservedRun,
    UserConfirmed,
    ImportedCatalog,
    StaleObservation,
}

impl CompatibilityEvidenceSource {
    /// The stored token for this source.
    pub fn as_str(self) -> &'static str {
        match self {
            CompatibilityEvidenceSource::ObservedRun => "observed-run",
            CompatibilityEvidenceSource::UserConfirmed => "user-confirmed",
            CompatibilityEvidenceSource::ImportedCatalog => "imported-catalog",
            CompatibilityEvidenceSource::StaleObservation => "stale-observation",
        }
    }

    /// Parse a stored source token.
    pub fn parse(s: &str) -> Result<Self, TargetsError> {
        match s {
            "observed-run" => Ok(CompatibilityEvidenceSource::ObservedRun),
            "user-confirmed" => Ok(CompatibilityEvidenceSource::UserConfirmed),
            "imported-catalog" => Ok(CompatibilityEvidenceSource::ImportedCatalog),
            "stale-observation" => Ok(CompatibilityEvidenceSource::StaleObservation),
            other => Err(TargetsError::Model(format!(
                "unknown compatibility evidence source {other:?}"
            ))),
        }
    }
}

/// One local Deep Capture compatibility fact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompatibilityFact {
    /// The autoincrement row key, `None` until inserted.
    pub id: Option<i64>,
    /// The target row this fact belongs to.
    pub target_id: i64,
    /// Which behavior this row records.
    pub key: CompatibilityFactKey,
    /// The closed-set value token for the key.
    pub value: String,
    /// Optional launch path this fact was observed under.
    pub launch_case: Option<CompatibilityLaunchCase>,
    /// Where the fact came from.
    pub evidence_source: CompatibilityEvidenceSource,
    /// Observation time, if the source recorded one.
    pub observed_at: Option<String>,
    /// fragcap version or commit that produced the observation.
    pub fragcap_version: Option<String>,
    /// Target version, build id, or equivalent local version clue.
    pub target_version: Option<String>,
    /// Proxy backend used to collect the observation.
    pub proxy_backend: Option<String>,
    /// Version of the proxy backend, if known.
    pub proxy_backend_version: Option<String>,
    /// Proxy mode or configuration family used for the observation.
    pub proxy_mode: Option<String>,
    /// Target-scoped routing strategy used for this observation.
    pub routing_strategy: Option<CompatibilityRoutingStrategy>,
    /// Exact loopback family used for this observation.
    pub address_family: Option<CompatibilityAddressFamily>,
    /// Exact protocol family, or explicit inapplicability for route facts.
    pub protocol: Option<CompatibilityProtocol>,
    /// Executable image observed holding the final sockets, if known.
    pub final_owner_executable: Option<String>,
    /// Whether the final socket owner differed from the initially launched
    /// executable for this observed launch.
    pub final_owner_handoff: bool,
    /// Whether the fact is retained as stale context rather than current advice.
    pub stale: bool,
    /// Optional operator-facing note. It must already be scrubbed before export.
    pub note: Option<String>,
}

impl CompatibilityFact {
    /// Build a compatibility fact and validate the value against the key.
    pub fn new(
        target_id: i64,
        key: CompatibilityFactKey,
        value: impl Into<String>,
        evidence_source: CompatibilityEvidenceSource,
    ) -> Result<Self, TargetsError> {
        let value = value.into();
        validate_fact_value(key, &value)?;
        Ok(Self {
            id: None,
            target_id,
            key,
            value,
            launch_case: None,
            evidence_source,
            observed_at: None,
            fragcap_version: None,
            target_version: None,
            proxy_backend: None,
            proxy_backend_version: None,
            proxy_mode: None,
            routing_strategy: None,
            address_family: None,
            protocol: None,
            final_owner_executable: None,
            final_owner_handoff: false,
            stale: false,
            note: None,
        })
    }

    /// Compare this fact with one exact current case without mutating history.
    pub fn applicability(&self, current: &CompatibilityCase) -> CompatibilityApplicability {
        if self.stale || self.evidence_source == CompatibilityEvidenceSource::StaleObservation {
            return CompatibilityApplicability::Stale;
        }
        let Some(launch_case) = self.launch_case else {
            return CompatibilityApplicability::LegacyIncomplete;
        };
        let (
            Some(backend),
            Some(backend_version),
            Some(routing),
            Some(family),
            Some(protocol),
            Some(fragcap_version),
        ) = (
            self.proxy_backend.as_deref(),
            self.proxy_backend_version.as_deref(),
            self.routing_strategy,
            self.address_family,
            self.protocol,
            self.fragcap_version.as_deref(),
        )
        else {
            return CompatibilityApplicability::LegacyIncomplete;
        };
        for (matches, dimension) in [
            (launch_case == current.launch_case, "launch-case"),
            (backend == current.proxy_backend, "proxy-backend"),
            (
                backend_version == current.proxy_backend_version,
                "proxy-backend-version",
            ),
            (routing == current.routing_strategy, "routing-strategy"),
            (family == current.address_family, "address-family"),
            (
                fragcap_version == current.fragcap_version,
                "fragcap-version",
            ),
        ] {
            if !matches {
                return CompatibilityApplicability::Mismatch(dimension);
            }
        }
        match (&self.target_version, &current.target_version) {
            (None, None) => {}
            (Some(left), Some(right)) if left == right => {}
            (None, Some(_)) => return CompatibilityApplicability::LegacyIncomplete,
            _ => return CompatibilityApplicability::Mismatch("target-version"),
        }
        let expected_protocol = if matches!(
            self.key,
            CompatibilityFactKey::TlsTrustBehavior
                | CompatibilityFactKey::ProtocolBehavior
                | CompatibilityFactKey::Inspectability
        ) {
            current.protocol
        } else {
            CompatibilityProtocol::NotApplicable
        };
        if protocol != expected_protocol {
            return CompatibilityApplicability::Mismatch("protocol-family");
        }
        CompatibilityApplicability::Applicable
    }
}

/// Return the latest current fact for one key and exact case.
pub fn latest_applicable_fact<'a>(
    facts: &'a [CompatibilityFact],
    key: CompatibilityFactKey,
    current: &CompatibilityCase,
) -> Option<&'a CompatibilityFact> {
    facts.iter().rev().find(|fact| {
        fact.key == key && fact.applicability(current) == CompatibilityApplicability::Applicable
    })
}

/// The presentation freshness of compatibility evidence.
///
/// `Unknown` is the state of an empty matrix. A stored row is either current or
/// stale; a fact whose value token is `unknown` is still current evidence when
/// neither stale signal is present.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompatibilityFreshness {
    Current,
    Stale,
    Unknown,
}

impl CompatibilityFreshness {
    /// The stable human-facing token for this freshness state.
    pub fn as_str(self) -> &'static str {
        match self {
            CompatibilityFreshness::Current => "current",
            CompatibilityFreshness::Stale => "stale",
            CompatibilityFreshness::Unknown => "unknown",
        }
    }
}

/// One display-safe row in a target's compatibility matrix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompatibilityMatrixRow {
    /// Durable chronology when this row came from the local store.
    pub id: Option<i64>,
    /// Which behavior this row records.
    pub key: CompatibilityFactKey,
    /// The key-specific value token.
    pub value: String,
    /// The launch path under which this fact was observed, when recorded.
    pub launch_case: Option<CompatibilityLaunchCase>,
    /// Where the evidence came from.
    pub evidence_source: CompatibilityEvidenceSource,
    pub routing_strategy: Option<CompatibilityRoutingStrategy>,
    pub address_family: Option<CompatibilityAddressFamily>,
    pub protocol: Option<CompatibilityProtocol>,
    /// Whether the row is current or retained as stale context.
    pub freshness: CompatibilityFreshness,
}

impl CompatibilityMatrixRow {
    fn from_fact(fact: &CompatibilityFact) -> Self {
        let stale =
            fact.stale || fact.evidence_source == CompatibilityEvidenceSource::StaleObservation;
        Self {
            id: fact.id,
            key: fact.key,
            value: fact.value.clone(),
            launch_case: fact.launch_case,
            evidence_source: fact.evidence_source,
            routing_strategy: fact.routing_strategy,
            address_family: fact.address_family,
            protocol: fact.protocol,
            freshness: if stale {
                CompatibilityFreshness::Stale
            } else {
                CompatibilityFreshness::Current
            },
        }
    }

    fn cmp_total(&self, other: &Self) -> Ordering {
        match (self.id, other.id) {
            (Some(left), Some(right)) => left.cmp(&right).then_with(|| self.cmp_fact_fields(other)),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => self.cmp_fact_fields(other),
        }
    }

    fn cmp_fact_fields(&self, other: &Self) -> Ordering {
        self.key
            .as_str()
            .cmp(other.key.as_str())
            .then_with(|| self.value.cmp(&other.value))
            .then_with(|| {
                self.launch_case
                    .map(CompatibilityLaunchCase::as_str)
                    .cmp(&other.launch_case.map(CompatibilityLaunchCase::as_str))
            })
            .then_with(|| {
                self.evidence_source
                    .as_str()
                    .cmp(other.evidence_source.as_str())
            })
            .then_with(|| {
                self.routing_strategy
                    .map(CompatibilityRoutingStrategy::as_str)
                    .cmp(
                        &other
                            .routing_strategy
                            .map(CompatibilityRoutingStrategy::as_str),
                    )
            })
            .then_with(|| {
                self.address_family
                    .map(CompatibilityAddressFamily::as_str)
                    .cmp(&other.address_family.map(CompatibilityAddressFamily::as_str))
            })
            .then_with(|| {
                self.protocol
                    .map(CompatibilityProtocol::as_str)
                    .cmp(&other.protocol.map(CompatibilityProtocol::as_str))
            })
            .then_with(|| self.freshness.as_str().cmp(other.freshness.as_str()))
    }
}

/// A deterministic, non-aggregating projection of one target's stored facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompatibilityMatrix {
    rows: Vec<CompatibilityMatrixRow>,
}

impl CompatibilityMatrix {
    /// Project every fact into a display-safe row without selecting a winner.
    pub fn from_facts(facts: &[CompatibilityFact]) -> Self {
        let mut rows: Vec<_> = facts
            .iter()
            .map(CompatibilityMatrixRow::from_fact)
            .collect();
        rows.sort_by(CompatibilityMatrixRow::cmp_total);
        Self { rows }
    }

    /// Every projected row in deterministic order.
    pub fn rows(&self) -> &[CompatibilityMatrixRow] {
        &self.rows
    }

    /// The matrix-level freshness state.
    pub fn state(&self) -> CompatibilityFreshness {
        if self.rows.is_empty() {
            CompatibilityFreshness::Unknown
        } else if self
            .rows
            .iter()
            .any(|row| row.freshness == CompatibilityFreshness::Current)
        {
            CompatibilityFreshness::Current
        } else {
            CompatibilityFreshness::Stale
        }
    }
}

/// Validate a fact value against the closed set for its key.
pub fn validate_fact_value(key: CompatibilityFactKey, value: &str) -> Result<(), TargetsError> {
    let allowed = match key {
        CompatibilityFactKey::ProxyEnvironmentHonored
        | CompatibilityFactKey::PublisherLauncherPresent
        | CompatibilityFactKey::RequiresPlatformColdStartForProxy
        | CompatibilityFactKey::DirectExeSupported
        | CompatibilityFactKey::SteamProtocolSupported => {
            matches!(value, "yes" | "no" | "unknown")
        }
        CompatibilityFactKey::ProxyRouting => matches!(
            value,
            "reached-client"
                | "launcher-only-routing"
                | "escaped-tree"
                | "no-proxy-traffic"
                | "not-applicable"
                | "inconclusive"
        ),
        CompatibilityFactKey::ProxyPropagation => {
            matches!(value, "confirmed" | "not-confirmed" | "not-tested")
        }
        CompatibilityFactKey::LaunchCase => CompatibilityLaunchCase::parse(value).is_ok(),
        CompatibilityFactKey::FinalSocketOwnerRole => matches!(
            value,
            "client"
                | "launcher"
                | "platform"
                | "platform-service"
                | "helper"
                | "proxy"
                | "wrapper"
                | "unknown"
        ),
        CompatibilityFactKey::TlsTrustBehavior => {
            matches!(value, "accepts-local-ca" | "certificate-pinned" | "unknown")
        }
        CompatibilityFactKey::ProtocolBehavior => matches!(
            value,
            "http"
                | "https"
                | "websocket"
                | "non-http-tls"
                | "quic"
                | "udp"
                | "plaintext"
                | "unknown"
        ),
        CompatibilityFactKey::Inspectability => {
            matches!(value, "full" | "metadata-only" | "unsupported" | "unknown")
        }
        CompatibilityFactKey::ProxyVariableTested => matches!(
            value,
            "HTTP_PROXY"
                | "HTTPS_PROXY"
                | "ALL_PROXY"
                | "NO_PROXY"
                | "http_proxy"
                | "https_proxy"
                | "all_proxy"
                | "no_proxy"
        ),
    };

    if allowed {
        Ok(())
    } else {
        Err(TargetsError::Model(format!(
            "compatibility value {value:?} is invalid for {}",
            key.as_str()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exact_case(protocol: CompatibilityProtocol) -> CompatibilityCase {
        CompatibilityCase {
            launch_case: CompatibilityLaunchCase::SteamProtocolCold,
            proxy_backend: "fragcap-native".to_string(),
            proxy_backend_version: "0.6.0".to_string(),
            routing_strategy: CompatibilityRoutingStrategy::ChildEnvironment,
            address_family: CompatibilityAddressFamily::Ipv4,
            protocol,
            fragcap_version: "0.6.0".to_string(),
            target_version: Some("build-a".to_string()),
        }
    }

    fn exact_fact(key: CompatibilityFactKey, protocol: CompatibilityProtocol) -> CompatibilityFact {
        let value = match key {
            CompatibilityFactKey::ProxyRouting => "reached-client",
            CompatibilityFactKey::TlsTrustBehavior => "accepts-local-ca",
            CompatibilityFactKey::ProtocolBehavior => "https",
            CompatibilityFactKey::Inspectability => "full",
            _ => "yes",
        };
        let mut fact =
            CompatibilityFact::new(1, key, value, CompatibilityEvidenceSource::ObservedRun)
                .unwrap();
        fact.launch_case = Some(CompatibilityLaunchCase::SteamProtocolCold);
        fact.proxy_backend = Some("fragcap-native".to_string());
        fact.proxy_backend_version = Some("0.6.0".to_string());
        fact.routing_strategy = Some(CompatibilityRoutingStrategy::ChildEnvironment);
        fact.address_family = Some(CompatibilityAddressFamily::Ipv4);
        fact.protocol = Some(protocol);
        fact.fragcap_version = Some("0.6.0".to_string());
        fact.target_version = Some("build-a".to_string());
        fact
    }

    #[test]
    fn calibration_case_tokens_round_trip_closed_sets() {
        for strategy in [
            CompatibilityRoutingStrategy::ChildEnvironment,
            CompatibilityRoutingStrategy::CommandArguments,
            CompatibilityRoutingStrategy::TargetConfiguration,
            CompatibilityRoutingStrategy::HttpProxy,
            CompatibilityRoutingStrategy::Socks,
            CompatibilityRoutingStrategy::ProtocolSpecific,
        ] {
            assert_eq!(
                CompatibilityRoutingStrategy::parse(strategy.as_str()).unwrap(),
                strategy
            );
        }
        for family in [
            CompatibilityAddressFamily::Ipv4,
            CompatibilityAddressFamily::Ipv6,
        ] {
            assert_eq!(
                CompatibilityAddressFamily::parse(family.as_str()).unwrap(),
                family
            );
        }
        for protocol in [
            CompatibilityProtocol::Routing,
            CompatibilityProtocol::Http1,
            CompatibilityProtocol::Https,
            CompatibilityProtocol::Http2,
            CompatibilityProtocol::WebSocket,
            CompatibilityProtocol::Sse,
            CompatibilityProtocol::Grpc,
            CompatibilityProtocol::GenericTcp,
            CompatibilityProtocol::NonHttpTls,
            CompatibilityProtocol::Socks5Tcp,
            CompatibilityProtocol::Socks5Udp,
            CompatibilityProtocol::GenericUdp,
            CompatibilityProtocol::Quic,
            CompatibilityProtocol::Http3,
            CompatibilityProtocol::NotApplicable,
        ] {
            assert_eq!(
                CompatibilityProtocol::parse(protocol.as_str()).unwrap(),
                protocol
            );
        }
    }

    #[test]
    fn exact_applicability_refuses_every_mismatched_or_incomplete_dimension() {
        let current = exact_case(CompatibilityProtocol::Https);
        let base = exact_fact(
            CompatibilityFactKey::TlsTrustBehavior,
            CompatibilityProtocol::Https,
        );
        assert_eq!(
            base.applicability(&current),
            CompatibilityApplicability::Applicable
        );

        type FactMutation = (&'static str, fn(&mut CompatibilityFact));
        let mutations: Vec<FactMutation> = vec![
            ("launch-case", |f| {
                f.launch_case = Some(CompatibilityLaunchCase::DirectExeCold)
            }),
            ("proxy-backend", |f| {
                f.proxy_backend = Some("other".to_string())
            }),
            ("proxy-backend-version", |f| {
                f.proxy_backend_version = Some("other".to_string())
            }),
            ("routing-strategy", |f| {
                f.routing_strategy = Some(CompatibilityRoutingStrategy::Socks)
            }),
            ("address-family", |f| {
                f.address_family = Some(CompatibilityAddressFamily::Ipv6)
            }),
            ("fragcap-version", |f| {
                f.fragcap_version = Some("other".to_string())
            }),
            ("target-version", |f| {
                f.target_version = Some("other".to_string())
            }),
            ("protocol-family", |f| {
                f.protocol = Some(CompatibilityProtocol::Http2)
            }),
        ];
        for (dimension, mutate) in mutations {
            let mut fact = base.clone();
            mutate(&mut fact);
            assert_eq!(
                fact.applicability(&current),
                CompatibilityApplicability::Mismatch(dimension)
            );
        }

        let mut legacy = base.clone();
        legacy.address_family = None;
        assert_eq!(
            legacy.applicability(&current),
            CompatibilityApplicability::LegacyIncomplete
        );
        let mut stale = base;
        stale.stale = true;
        assert_eq!(
            stale.applicability(&current),
            CompatibilityApplicability::Stale
        );
    }

    #[test]
    fn route_facts_use_explicit_protocol_inapplicability() {
        let current = exact_case(CompatibilityProtocol::Http3);
        let applicable = exact_fact(
            CompatibilityFactKey::ProxyRouting,
            CompatibilityProtocol::NotApplicable,
        );
        assert_eq!(
            applicable.applicability(&current),
            CompatibilityApplicability::Applicable
        );
        let wrong = exact_fact(
            CompatibilityFactKey::ProxyRouting,
            CompatibilityProtocol::Routing,
        );
        assert_eq!(
            wrong.applicability(&current),
            CompatibilityApplicability::Mismatch("protocol-family")
        );
    }

    #[test]
    fn latest_applicable_fact_keeps_conflicts_and_uses_latest_exact_row() {
        let current = exact_case(CompatibilityProtocol::Https);
        let mut older = exact_fact(
            CompatibilityFactKey::TlsTrustBehavior,
            CompatibilityProtocol::Https,
        );
        older.value = "accepts-local-ca".to_string();
        let mut mismatch = older.clone();
        mismatch.address_family = Some(CompatibilityAddressFamily::Ipv6);
        mismatch.value = "rejects-local-ca".to_string();
        let mut latest = older.clone();
        latest.value = "rejects-local-ca".to_string();
        let facts = vec![older, mismatch, latest];
        assert_eq!(
            latest_applicable_fact(&facts, CompatibilityFactKey::TlsTrustBehavior, &current)
                .unwrap()
                .value,
            "rejects-local-ca"
        );
    }

    fn fact(
        id: Option<i64>,
        key: CompatibilityFactKey,
        value: &str,
        source: CompatibilityEvidenceSource,
    ) -> CompatibilityFact {
        let mut fact = CompatibilityFact::new(1, key, value, source).unwrap();
        fact.id = id;
        fact
    }

    #[test]
    fn compatibility_matrix_is_unknown_only_when_no_facts_exist() {
        let matrix = CompatibilityMatrix::from_facts(&[]);

        assert_eq!(matrix.state(), CompatibilityFreshness::Unknown);
        assert!(matrix.rows().is_empty());

        let matrix = CompatibilityMatrix::from_facts(&[fact(
            Some(1),
            CompatibilityFactKey::Inspectability,
            "unknown",
            CompatibilityEvidenceSource::ObservedRun,
        )]);

        assert_eq!(matrix.state(), CompatibilityFreshness::Current);
        assert_eq!(matrix.rows().len(), 1);
        assert_eq!(matrix.rows()[0].value, "unknown");
    }

    #[test]
    fn compatibility_matrix_honors_both_stale_signals() {
        let mut marked = fact(
            Some(1),
            CompatibilityFactKey::ProxyPropagation,
            "confirmed",
            CompatibilityEvidenceSource::ObservedRun,
        );
        marked.stale = true;
        let sourced = fact(
            Some(2),
            CompatibilityFactKey::Inspectability,
            "metadata-only",
            CompatibilityEvidenceSource::StaleObservation,
        );
        let current = fact(
            Some(3),
            CompatibilityFactKey::TlsTrustBehavior,
            "accepts-local-ca",
            CompatibilityEvidenceSource::UserConfirmed,
        );

        let matrix = CompatibilityMatrix::from_facts(&[current, sourced, marked]);

        assert_eq!(matrix.rows()[0].freshness, CompatibilityFreshness::Stale);
        assert_eq!(matrix.rows()[1].freshness, CompatibilityFreshness::Stale);
        assert_eq!(matrix.rows()[2].freshness, CompatibilityFreshness::Current);
        assert_eq!(matrix.state(), CompatibilityFreshness::Current);
    }

    #[test]
    fn compatibility_matrix_preserves_repeated_and_conflicting_stored_rows() {
        let mut older = fact(
            Some(4),
            CompatibilityFactKey::ProxyRouting,
            "reached-client",
            CompatibilityEvidenceSource::ObservedRun,
        );
        older.launch_case = Some(CompatibilityLaunchCase::SteamProtocolCold);
        let mut newer = fact(
            Some(9),
            CompatibilityFactKey::ProxyRouting,
            "no-proxy-traffic",
            CompatibilityEvidenceSource::ObservedRun,
        );
        newer.launch_case = Some(CompatibilityLaunchCase::SteamProtocolWarm);

        let matrix = CompatibilityMatrix::from_facts(&[newer, older]);

        assert_eq!(matrix.rows().len(), 2);
        assert_eq!(matrix.rows()[0].id, Some(4));
        assert_eq!(matrix.rows()[0].value, "reached-client");
        assert_eq!(matrix.rows()[1].id, Some(9));
        assert_eq!(matrix.rows()[1].value, "no-proxy-traffic");
    }

    #[test]
    fn compatibility_matrix_totally_orders_unsaved_rows() {
        let first = fact(
            None,
            CompatibilityFactKey::Inspectability,
            "full",
            CompatibilityEvidenceSource::ImportedCatalog,
        );
        let second = fact(
            None,
            CompatibilityFactKey::ProtocolBehavior,
            "https",
            CompatibilityEvidenceSource::UserConfirmed,
        );

        let left = CompatibilityMatrix::from_facts(&[second.clone(), first.clone()]);
        let right = CompatibilityMatrix::from_facts(&[first, second]);

        assert_eq!(left, right);
        assert_eq!(left.rows()[0].key, CompatibilityFactKey::Inspectability);
        assert_eq!(left.rows()[1].key, CompatibilityFactKey::ProtocolBehavior);
    }

    #[test]
    fn compatibility_fact_key_round_trips_every_variant() {
        for key in [
            CompatibilityFactKey::ProxyEnvironmentHonored,
            CompatibilityFactKey::ProxyRouting,
            CompatibilityFactKey::ProxyPropagation,
            CompatibilityFactKey::LaunchCase,
            CompatibilityFactKey::FinalSocketOwnerRole,
            CompatibilityFactKey::PublisherLauncherPresent,
            CompatibilityFactKey::RequiresPlatformColdStartForProxy,
            CompatibilityFactKey::DirectExeSupported,
            CompatibilityFactKey::SteamProtocolSupported,
            CompatibilityFactKey::TlsTrustBehavior,
            CompatibilityFactKey::ProtocolBehavior,
            CompatibilityFactKey::Inspectability,
            CompatibilityFactKey::ProxyVariableTested,
        ] {
            assert_eq!(CompatibilityFactKey::parse(key.as_str()).unwrap(), key);
        }
        assert!(CompatibilityFactKey::parse("nonsense").is_err());
    }

    #[test]
    fn invalid_values_are_rejected_before_storage() {
        assert!(CompatibilityFact::new(
            1,
            CompatibilityFactKey::ProxyRouting,
            "confirmed",
            CompatibilityEvidenceSource::ObservedRun,
        )
        .is_err());
        assert!(CompatibilityFact::new(
            1,
            CompatibilityFactKey::ProxyPropagation,
            "reached-client",
            CompatibilityEvidenceSource::ObservedRun,
        )
        .is_err());
    }

    #[test]
    fn platform_ownership_reuses_separate_routing_and_propagation_sets() {
        for routing in [
            "reached-client",
            "launcher-only-routing",
            "escaped-tree",
            "no-proxy-traffic",
            "inconclusive",
        ] {
            assert!(CompatibilityFact::new(
                1,
                CompatibilityFactKey::ProxyRouting,
                routing,
                CompatibilityEvidenceSource::ObservedRun,
            )
            .is_ok());
        }
        for propagation in ["confirmed", "not-confirmed", "not-tested"] {
            assert!(CompatibilityFact::new(
                1,
                CompatibilityFactKey::ProxyPropagation,
                propagation,
                CompatibilityEvidenceSource::ObservedRun,
            )
            .is_ok());
        }
        assert!(CompatibilityFact::new(
            1,
            CompatibilityFactKey::ProxyPropagation,
            "owned-platform-reached-client",
            CompatibilityEvidenceSource::ObservedRun,
        )
        .is_err());
    }
}
