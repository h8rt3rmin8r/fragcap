// SPDX-License-Identifier: Apache-2.0

//! Deep Capture compatibility facts stored with local targets.
//!
//! Compatibility is observed behavior, not a property inferred from a title's
//! platform metadata. The values here are deliberately closed sets where the
//! product needs exact language, and `Unknown` is a real value rather than a
//! prompt to guess.

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
            final_owner_executable: None,
            final_owner_handoff: false,
            stale: false,
            note: None,
        })
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
}
