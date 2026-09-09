// SPDX-License-Identifier: Apache-2.0

//! Pure case discovery and work proposal for guided compatibility calibration.

use crate::targets::{
    entry_windows_launch_entries, CompatibilityAddressFamily, CompatibilityApplicability,
    CompatibilityCase, CompatibilityEvidenceSource, CompatibilityFact, CompatibilityFactKey,
    CompatibilityLaunchCase, CompatibilityProtocol, CompatibilityRoutingStrategy, LaunchEntry,
    TargetEntry,
};

use super::CalibrationPhase;

/// Caller-supplied process inventory used only to decide whether a launch is cold.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CalibrationProcessSnapshot {
    Complete(Vec<String>),
    Unavailable { reason: String },
}

impl CalibrationProcessSnapshot {
    pub fn complete<I, S>(images: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::Complete(images.into_iter().map(Into::into).collect())
    }

    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self::Unavailable {
            reason: reason.into(),
        }
    }
}

/// Input to one deterministic calibration proposal.
#[derive(Clone, Debug, PartialEq)]
pub struct CalibrationProposalRequest {
    target: TargetEntry,
    process_snapshot: CalibrationProcessSnapshot,
    backend_name: String,
    backend_version: String,
    fragcap_version: String,
    target_version: Option<String>,
    routing_strategy: CompatibilityRoutingStrategy,
    address_family: CompatibilityAddressFamily,
    protocol_candidates: Vec<CompatibilityProtocol>,
    facts: Vec<CompatibilityFact>,
}

impl CalibrationProposalRequest {
    pub fn new(
        target: TargetEntry,
        backend_name: impl Into<String>,
        backend_version: impl Into<String>,
        fragcap_version: impl Into<String>,
    ) -> Self {
        Self {
            target,
            process_snapshot: CalibrationProcessSnapshot::unavailable(
                "no process inventory was supplied",
            ),
            backend_name: backend_name.into(),
            backend_version: backend_version.into(),
            fragcap_version: fragcap_version.into(),
            target_version: None,
            routing_strategy: CompatibilityRoutingStrategy::ChildEnvironment,
            address_family: CompatibilityAddressFamily::Ipv4,
            protocol_candidates: Vec::new(),
            facts: Vec::new(),
        }
    }

    pub fn with_process_snapshot(mut self, snapshot: CalibrationProcessSnapshot) -> Self {
        self.process_snapshot = snapshot;
        self
    }

    pub fn with_target_version(mut self, version: Option<String>) -> Self {
        self.target_version = version;
        self
    }

    pub fn with_routing_strategy(mut self, strategy: CompatibilityRoutingStrategy) -> Self {
        self.routing_strategy = strategy;
        self
    }

    pub fn with_address_family(mut self, family: CompatibilityAddressFamily) -> Self {
        self.address_family = family;
        self
    }

    pub fn with_protocol_candidates<I>(mut self, protocols: I) -> Self
    where
        I: IntoIterator<Item = CompatibilityProtocol>,
    {
        self.protocol_candidates = protocols.into_iter().collect();
        self
    }

    pub fn with_facts<I>(mut self, facts: I) -> Self
    where
        I: IntoIterator<Item = CompatibilityFact>,
    {
        self.facts = facts.into_iter().collect();
        self
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalibrationTopologyKind {
    Steam,
    Direct,
    Publisher,
}

impl CalibrationTopologyKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Steam => "steam",
            Self::Direct => "direct",
            Self::Publisher => "publisher",
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalibrationProposalLimitationKind {
    InvalidContext,
    InvalidSteamAnchor,
    MissingLaunchDeclaration,
    AmbiguousLaunchDeclaration,
    InvalidPublisherChain,
    ProcessInventoryUnavailable,
    UnsupportedRoutingStrategy,
    UnsupportedProtocolCandidate,
}

impl CalibrationProposalLimitationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidContext => "invalid-context",
            Self::InvalidSteamAnchor => "invalid-steam-anchor",
            Self::MissingLaunchDeclaration => "missing-launch-declaration",
            Self::AmbiguousLaunchDeclaration => "ambiguous-launch-declaration",
            Self::InvalidPublisherChain => "invalid-publisher-chain",
            Self::ProcessInventoryUnavailable => "process-inventory-unavailable",
            Self::UnsupportedRoutingStrategy => "unsupported-routing-strategy",
            Self::UnsupportedProtocolCandidate => "unsupported-protocol-candidate",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalibrationProposalLimitation {
    pub kind: CalibrationProposalLimitationKind,
    pub detail: String,
    pub candidates: Vec<String>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CalibrationLaunchReadiness {
    Ready {
        launch_case: CompatibilityLaunchCase,
        images: Vec<String>,
    },
    OperatorAction {
        observed_case: CompatibilityLaunchCase,
        cold_case: CompatibilityLaunchCase,
        images: Vec<String>,
    },
    Unavailable,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalibrationProposalReason {
    Missing,
    Stale,
    LegacyIncomplete,
    ContextMismatch,
    Negative,
    Conflict,
    ReachabilityRequired,
}

impl CalibrationProposalReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Stale => "stale",
            Self::LegacyIncomplete => "legacy-incomplete",
            Self::ContextMismatch => "context-mismatch",
            Self::Negative => "negative",
            Self::Conflict => "conflict",
            Self::ReachabilityRequired => "reachability-required",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalibrationProposalStep {
    pub phase: CalibrationPhase,
    pub case: CompatibilityCase,
    pub reason: CalibrationProposalReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeferredCalibrationProtocol {
    pub protocol: CompatibilityProtocol,
    pub reason: CalibrationProposalReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalibrationProposal {
    pub topology: Option<CalibrationTopologyKind>,
    pub readiness: CalibrationLaunchReadiness,
    pub routing_strategy: CompatibilityRoutingStrategy,
    pub address_family: CompatibilityAddressFamily,
    pub steps: Vec<CalibrationProposalStep>,
    pub deferred_protocols: Vec<DeferredCalibrationProtocol>,
    pub limitations: Vec<CalibrationProposalLimitation>,
}

/// Derive the next useful exact calibration work without applying any effect.
pub fn propose_calibration(request: &CalibrationProposalRequest) -> CalibrationProposal {
    let mut proposal = CalibrationProposal {
        topology: None,
        readiness: CalibrationLaunchReadiness::Unavailable,
        routing_strategy: request.routing_strategy,
        address_family: request.address_family,
        steps: Vec::new(),
        deferred_protocols: Vec::new(),
        limitations: Vec::new(),
    };

    validate_request(request, &mut proposal.limitations);
    let topology = match derive_topology(&request.target) {
        Ok(topology) => topology,
        Err(limitation) => {
            proposal.limitations.push(limitation);
            return proposal;
        }
    };
    proposal.topology = Some(topology.kind);
    proposal.readiness = readiness(
        &topology,
        &request.process_snapshot,
        &mut proposal.limitations,
    );

    if !proposal.limitations.is_empty()
        || !matches!(proposal.readiness, CalibrationLaunchReadiness::Ready { .. })
    {
        return proposal;
    }

    let launch_case = match proposal.readiness {
        CalibrationLaunchReadiness::Ready { launch_case, .. } => launch_case,
        _ => unreachable!("non-ready proposals returned above"),
    };
    let routing_case = exact_case(request, launch_case, CompatibilityProtocol::Routing);
    match assess_facts(
        &request.facts,
        CompatibilityFactKey::ProxyRouting,
        &routing_case,
        "reached-client",
    ) {
        EvidenceAssessment::Positive => add_protocol_steps(request, launch_case, &mut proposal),
        EvidenceAssessment::Needs(reason) => {
            proposal.steps.push(CalibrationProposalStep {
                phase: CalibrationPhase::Reachability,
                case: routing_case,
                reason,
            });
            proposal.deferred_protocols = valid_protocols(&request.protocol_candidates)
                .into_iter()
                .map(|protocol| DeferredCalibrationProtocol {
                    protocol,
                    reason: CalibrationProposalReason::ReachabilityRequired,
                })
                .collect();
        }
    }
    proposal
}

#[derive(Clone, Debug)]
struct DerivedTopology {
    kind: CalibrationTopologyKind,
    images: Vec<String>,
    cold_case: CompatibilityLaunchCase,
}

fn validate_request(
    request: &CalibrationProposalRequest,
    limitations: &mut Vec<CalibrationProposalLimitation>,
) {
    if request.backend_name.trim().is_empty()
        || request.backend_version.trim().is_empty()
        || request.fragcap_version.trim().is_empty()
        || request
            .target_version
            .as_deref()
            .is_some_and(|version| version.trim().is_empty())
    {
        limitations.push(limitation(
            CalibrationProposalLimitationKind::InvalidContext,
            "backend, backend version, fragcap version, and any target version must be non-empty",
            Vec::new(),
        ));
    }
    if request.routing_strategy != CompatibilityRoutingStrategy::ChildEnvironment {
        limitations.push(limitation(
            CalibrationProposalLimitationKind::UnsupportedRoutingStrategy,
            format!(
                "routing strategy {} is not implemented for guided calibration",
                request.routing_strategy.as_str()
            ),
            vec![request.routing_strategy.as_str().to_string()],
        ));
    }
    let mut invalid: Vec<_> = request
        .protocol_candidates
        .iter()
        .copied()
        .filter(|protocol| {
            matches!(
                protocol,
                CompatibilityProtocol::Routing | CompatibilityProtocol::NotApplicable
            )
        })
        .collect();
    invalid.sort_by_key(|protocol| protocol.as_str());
    invalid.dedup();
    for protocol in invalid {
        limitations.push(limitation(
            CalibrationProposalLimitationKind::UnsupportedProtocolCandidate,
            format!(
                "protocol {} is not a concrete observed protocol candidate",
                protocol.as_str()
            ),
            vec![protocol.as_str().to_string()],
        ));
    }
}

fn derive_topology(target: &TargetEntry) -> Result<DerivedTopology, CalibrationProposalLimitation> {
    validate_raw_launch_entries(target)?;
    let launches = entry_windows_launch_entries(target);
    if launches.is_empty() {
        return Err(limitation(
            CalibrationProposalLimitationKind::MissingLaunchDeclaration,
            "the target has no usable Windows launch declaration",
            Vec::new(),
        ));
    }
    let candidates = launches
        .iter()
        .map(|launch| launch.executable().to_string())
        .collect::<Vec<_>>();
    let images = unique_images(
        launches
            .iter()
            .map(|launch| windows_image_name(launch.executable())),
    );

    if let Some(anchor) = target
        .anchor
        .as_deref()
        .filter(|anchor| anchor.starts_with("steam:"))
    {
        let valid_anchor = anchor
            .strip_prefix("steam:")
            .and_then(|value| value.parse::<u32>().ok())
            .is_some_and(|value| value > 0);
        if !valid_anchor {
            return Err(limitation(
                CalibrationProposalLimitationKind::InvalidSteamAnchor,
                format!("Steam anchor {anchor:?} does not contain a positive application id"),
                candidates,
            ));
        }
        if launches.len() != 1 || launches[0].role().is_some_and(|role| role != "client") {
            return Err(limitation(
                CalibrationProposalLimitationKind::AmbiguousLaunchDeclaration,
                "a Steam calibration proposal requires one exact client declaration",
                candidates,
            ));
        }
        return Ok(DerivedTopology {
            kind: CalibrationTopologyKind::Steam,
            images: unique_images(std::iter::once("steam.exe".to_string()).chain(images)),
            cold_case: CompatibilityLaunchCase::SteamProtocolCold,
        });
    }

    let direct_client =
        launches.len() == 1 && launches[0].role().is_none_or(|role| role == "client");
    if !direct_client && launches.iter().any(|launch| launch.role().is_some()) {
        validate_publisher_chain(&launches)?;
        return Ok(DerivedTopology {
            kind: CalibrationTopologyKind::Publisher,
            images,
            cold_case: CompatibilityLaunchCase::PublisherLauncherCold,
        });
    }
    if !direct_client {
        return Err(limitation(
            CalibrationProposalLimitationKind::AmbiguousLaunchDeclaration,
            "a direct calibration proposal requires one exact client declaration",
            candidates,
        ));
    }
    Ok(DerivedTopology {
        kind: CalibrationTopologyKind::Direct,
        images,
        cold_case: CompatibilityLaunchCase::DirectExeCold,
    })
}

fn validate_raw_launch_entries(target: &TargetEntry) -> Result<(), CalibrationProposalLimitation> {
    let Some(serde_json::Value::Array(entries)) = target.launch_entries.as_ref() else {
        return Err(limitation(
            CalibrationProposalLimitationKind::MissingLaunchDeclaration,
            "the target launch declaration is absent or unresolved",
            Vec::new(),
        ));
    };
    for entry in entries {
        let windows = entry
            .get("os")
            .and_then(serde_json::Value::as_str)
            .is_none_or(|os| os.eq_ignore_ascii_case("windows"));
        if !windows {
            continue;
        }
        let executable = entry.get("executable").and_then(serde_json::Value::as_str);
        let role_valid = entry
            .get("role")
            .is_none_or(|role| role.as_str().is_some_and(|value| !value.trim().is_empty()));
        if executable
            .is_none_or(|value| value.trim().is_empty() || windows_image_name(value).is_empty())
            || !role_valid
        {
            return Err(limitation(
                CalibrationProposalLimitationKind::MissingLaunchDeclaration,
                "a Windows launch declaration is malformed",
                Vec::new(),
            ));
        }
    }
    Ok(())
}

fn validate_publisher_chain(launches: &[LaunchEntry]) -> Result<(), CalibrationProposalLimitation> {
    let candidates = launches
        .iter()
        .map(|launch| launch.executable().to_string())
        .collect::<Vec<_>>();
    let invalid_boundary = launches.len() < 2
        || launches.first().and_then(LaunchEntry::role) != Some("launcher")
        || launches.last().and_then(LaunchEntry::role) != Some("client")
        || launches.iter().any(|launch| launch.role().is_none());
    let mut roles = Vec::<String>::new();
    let mut image_roles = Vec::<(String, String)>::new();
    let mut duplicate_role = false;
    let mut conflicting_image = false;
    for launch in launches {
        let Some(role) = launch.role() else {
            continue;
        };
        if roles.iter().any(|known| known == role) {
            duplicate_role = true;
        } else {
            roles.push(role.to_string());
        }
        if image_roles.iter().any(|(image, known_role)| {
            image.eq_ignore_ascii_case(launch.executable()) && known_role != role
        }) {
            conflicting_image = true;
        } else {
            image_roles.push((launch.executable().to_string(), role.to_string()));
        }
    }
    if invalid_boundary || duplicate_role || conflicting_image {
        return Err(limitation(
            CalibrationProposalLimitationKind::InvalidPublisherChain,
            "publisher stages require unique roles, launcher first, client last, and no conflicting image roles",
            candidates,
        ));
    }
    Ok(())
}

fn readiness(
    topology: &DerivedTopology,
    snapshot: &CalibrationProcessSnapshot,
    limitations: &mut Vec<CalibrationProposalLimitation>,
) -> CalibrationLaunchReadiness {
    let present_images = match snapshot {
        CalibrationProcessSnapshot::Complete(images) => unique_images(images.iter().cloned()),
        CalibrationProcessSnapshot::Unavailable { reason } => {
            limitations.push(limitation(
                CalibrationProposalLimitationKind::ProcessInventoryUnavailable,
                reason.clone(),
                topology.images.clone(),
            ));
            return CalibrationLaunchReadiness::Unavailable;
        }
    };
    let present: Vec<bool> = topology
        .images
        .iter()
        .map(|declared| {
            present_images
                .iter()
                .any(|observed| observed.eq_ignore_ascii_case(declared))
        })
        .collect();
    if present.iter().all(|value| !value) {
        return CalibrationLaunchReadiness::Ready {
            launch_case: topology.cold_case,
            images: topology.images.clone(),
        };
    }
    let observed_case = match topology.kind {
        CalibrationTopologyKind::Steam => CompatibilityLaunchCase::SteamProtocolWarm,
        CalibrationTopologyKind::Direct => CompatibilityLaunchCase::DirectExeWarm,
        CalibrationTopologyKind::Publisher => {
            if present[0] && !present[present.len() - 1] {
                CompatibilityLaunchCase::PublisherLauncherGameStartCleanWarm
            } else {
                CompatibilityLaunchCase::PublisherLauncherWarm
            }
        }
    };
    CalibrationLaunchReadiness::OperatorAction {
        observed_case,
        cold_case: topology.cold_case,
        images: topology.images.clone(),
    }
}

fn exact_case(
    request: &CalibrationProposalRequest,
    launch_case: CompatibilityLaunchCase,
    protocol: CompatibilityProtocol,
) -> CompatibilityCase {
    CompatibilityCase {
        launch_case,
        proxy_backend: request.backend_name.clone(),
        proxy_backend_version: request.backend_version.clone(),
        routing_strategy: request.routing_strategy,
        address_family: request.address_family,
        protocol,
        fragcap_version: request.fragcap_version.clone(),
        target_version: request.target_version.clone(),
    }
}

fn add_protocol_steps(
    request: &CalibrationProposalRequest,
    launch_case: CompatibilityLaunchCase,
    proposal: &mut CalibrationProposal,
) {
    for protocol in valid_protocols(&request.protocol_candidates) {
        let case = exact_case(request, launch_case, protocol);
        if let EvidenceAssessment::Needs(reason) = assess_facts(
            &request.facts,
            CompatibilityFactKey::Inspectability,
            &case,
            "full",
        ) {
            proposal.steps.push(CalibrationProposalStep {
                phase: CalibrationPhase::Tls,
                case,
                reason,
            });
        }
    }
}

fn valid_protocols(protocols: &[CompatibilityProtocol]) -> Vec<CompatibilityProtocol> {
    let mut protocols: Vec<_> = protocols
        .iter()
        .copied()
        .filter(|protocol| {
            !matches!(
                protocol,
                CompatibilityProtocol::Routing | CompatibilityProtocol::NotApplicable
            )
        })
        .collect();
    protocols.sort_by_key(|protocol| protocol.as_str());
    protocols.dedup();
    protocols
}

enum EvidenceAssessment {
    Positive,
    Needs(CalibrationProposalReason),
}

fn assess_facts(
    facts: &[CompatibilityFact],
    key: CompatibilityFactKey,
    case: &CompatibilityCase,
    positive_value: &str,
) -> EvidenceAssessment {
    let relevant: Vec<_> = facts.iter().filter(|fact| fact.key == key).collect();
    let mut current_values: Vec<&str> = relevant
        .iter()
        .filter(|fact| fact.applicability(case) == CompatibilityApplicability::Applicable)
        .map(|fact| fact.value.as_str())
        .collect();
    current_values.sort_unstable();
    current_values.dedup();
    if current_values.len() > 1 {
        return EvidenceAssessment::Needs(CalibrationProposalReason::Conflict);
    }
    if let Some(value) = current_values.first() {
        return if *value == positive_value {
            EvidenceAssessment::Positive
        } else {
            EvidenceAssessment::Needs(CalibrationProposalReason::Negative)
        };
    }

    let stale_exact = relevant.iter().any(|fact| {
        let mut candidate = (*fact).clone();
        candidate.stale = false;
        if candidate.evidence_source == CompatibilityEvidenceSource::StaleObservation {
            candidate.evidence_source = CompatibilityEvidenceSource::ObservedRun;
        }
        fact.applicability(case) == CompatibilityApplicability::Stale
            && candidate.applicability(case) == CompatibilityApplicability::Applicable
    });
    if stale_exact {
        return EvidenceAssessment::Needs(CalibrationProposalReason::Stale);
    }
    if relevant
        .iter()
        .any(|fact| fact.applicability(case) == CompatibilityApplicability::LegacyIncomplete)
    {
        return EvidenceAssessment::Needs(CalibrationProposalReason::LegacyIncomplete);
    }
    if !relevant.is_empty() {
        return EvidenceAssessment::Needs(CalibrationProposalReason::ContextMismatch);
    }
    EvidenceAssessment::Needs(CalibrationProposalReason::Missing)
}

fn unique_images<I>(images: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    let mut unique = Vec::<String>::new();
    for image in images {
        if !unique
            .iter()
            .any(|known| known.eq_ignore_ascii_case(&image))
        {
            unique.push(image);
        }
    }
    unique
}

fn windows_image_name(executable: &str) -> String {
    executable
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(executable)
        .to_string()
}

fn limitation(
    kind: CalibrationProposalLimitationKind,
    detail: impl Into<String>,
    candidates: Vec<String>,
) -> CalibrationProposalLimitation {
    CalibrationProposalLimitation {
        kind,
        detail: detail.into(),
        candidates,
    }
}

#[cfg(test)]
mod tests {
    use fragcap_profile::FidelityTier;
    use serde_json::json;

    use crate::targets::{
        ClassificationSource, CompatibilityApplicability, CompatibilityEvidenceSource,
        CompatibilityFactKey, TargetClassification,
    };

    use super::*;

    fn target(anchor: Option<&str>, launches: serde_json::Value) -> TargetEntry {
        TargetEntry {
            id: Some(7),
            stable_id: 70,
            handle: "game".into(),
            name: "Game".into(),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::User,
            fidelity: FidelityTier::Authored,
            provenance: None,
            anchor: anchor.map(str::to_string),
            launch_entries: Some(launches),
            install_root: Some("C:\\Games\\Game".into()),
            evidence: None,
            detection_scan: None,
            folder_name: None,
            executable_hint: None,
        }
    }

    fn request(target: TargetEntry) -> CalibrationProposalRequest {
        CalibrationProposalRequest::new(target, "native", "0.9.0", "0.9.0")
            .with_process_snapshot(CalibrationProcessSnapshot::complete(Vec::<String>::new()))
    }

    fn fact(
        key: CompatibilityFactKey,
        value: &str,
        protocol: CompatibilityProtocol,
    ) -> CompatibilityFact {
        let mut fact =
            CompatibilityFact::new(7, key, value, CompatibilityEvidenceSource::ObservedRun)
                .unwrap();
        fact.id = Some(1);
        fact.launch_case = Some(CompatibilityLaunchCase::DirectExeCold);
        fact.proxy_backend = Some("native".into());
        fact.proxy_backend_version = Some("0.9.0".into());
        fact.routing_strategy = Some(CompatibilityRoutingStrategy::ChildEnvironment);
        fact.address_family = Some(CompatibilityAddressFamily::Ipv4);
        fact.protocol = Some(protocol);
        fact.fragcap_version = Some("0.9.0".into());
        fact
    }

    fn direct_target() -> TargetEntry {
        target(
            None,
            json!([{ "executable": "Game.exe", "role": "client" }]),
        )
    }

    #[test]
    fn direct_and_steam_topologies_select_cold_or_operator_action() {
        let direct = propose_calibration(&request(direct_target()));
        assert_eq!(direct.topology, Some(CalibrationTopologyKind::Direct));
        assert!(matches!(
            direct.readiness,
            CalibrationLaunchReadiness::Ready {
                launch_case: CompatibilityLaunchCase::DirectExeCold,
                ..
            }
        ));

        let steam = request(target(
            Some("steam:123"),
            json!([{ "executable": "Game.exe", "role": "client" }]),
        ))
        .with_process_snapshot(CalibrationProcessSnapshot::complete(["GAME.EXE"]));
        let steam = propose_calibration(&steam);
        assert_eq!(steam.topology, Some(CalibrationTopologyKind::Steam));
        assert!(matches!(
            steam.readiness,
            CalibrationLaunchReadiness::OperatorAction {
                observed_case: CompatibilityLaunchCase::SteamProtocolWarm,
                cold_case: CompatibilityLaunchCase::SteamProtocolCold,
                ..
            }
        ));
        assert!(steam.steps.is_empty());
    }

    #[test]
    fn process_readiness_compares_portable_image_names_not_stored_paths() {
        let direct = target(
            None,
            json!([{ "executable": "C:\\Games\\Game\\Game.exe", "role": "client" }]),
        );
        let request = request(direct)
            .with_process_snapshot(CalibrationProcessSnapshot::complete(["game.exe"]));
        let proposal = propose_calibration(&request);
        assert_eq!(
            proposal.readiness,
            CalibrationLaunchReadiness::OperatorAction {
                observed_case: CompatibilityLaunchCase::DirectExeWarm,
                cold_case: CompatibilityLaunchCase::DirectExeCold,
                images: vec!["Game.exe".into()],
            }
        );
        assert!(proposal.steps.is_empty());
    }

    #[test]
    fn ambiguous_direct_and_invalid_steam_targets_never_gain_steps() {
        let ambiguous = target(
            None,
            json!([
                { "executable": "A.exe" },
                { "executable": "B.exe" }
            ]),
        );
        let proposal = propose_calibration(&request(ambiguous));
        assert_eq!(proposal.readiness, CalibrationLaunchReadiness::Unavailable);
        assert_eq!(
            proposal.limitations[0].kind,
            CalibrationProposalLimitationKind::AmbiguousLaunchDeclaration
        );
        assert_eq!(proposal.limitations[0].candidates, ["A.exe", "B.exe"]);
        assert!(proposal.steps.is_empty());

        let invalid = target(
            Some("steam:not-a-number"),
            json!([{ "executable": "Game.exe" }]),
        );
        let proposal = propose_calibration(&request(invalid));
        assert_eq!(
            proposal.limitations[0].kind,
            CalibrationProposalLimitationKind::InvalidSteamAnchor
        );
        assert!(proposal.steps.is_empty());
    }

    #[test]
    fn publisher_roles_and_warm_states_preserve_declared_order() {
        let publisher = target(
            None,
            json!([
                { "executable": "Launcher.exe", "role": "launcher" },
                { "executable": "Bootstrap.exe", "role": "bootstrap" },
                { "executable": "Game.exe", "role": "client" }
            ]),
        );
        let warm = request(publisher.clone()).with_process_snapshot(
            CalibrationProcessSnapshot::complete(["launcher.exe", "BOOTSTRAP.EXE"]),
        );
        let warm = propose_calibration(&warm);
        assert_eq!(warm.topology, Some(CalibrationTopologyKind::Publisher));
        assert_eq!(
            warm.readiness,
            CalibrationLaunchReadiness::OperatorAction {
                observed_case: CompatibilityLaunchCase::PublisherLauncherGameStartCleanWarm,
                cold_case: CompatibilityLaunchCase::PublisherLauncherCold,
                images: vec![
                    "Launcher.exe".into(),
                    "Bootstrap.exe".into(),
                    "Game.exe".into()
                ],
            }
        );

        let cold = propose_calibration(&request(publisher));
        assert!(matches!(
            cold.readiness,
            CalibrationLaunchReadiness::Ready {
                launch_case: CompatibilityLaunchCase::PublisherLauncherCold,
                ..
            }
        ));
    }

    #[test]
    fn invalid_publisher_roles_and_conflicting_images_are_refused() {
        for launches in [
            json!([
                { "executable": "Launcher.exe", "role": "client" },
                { "executable": "Game.exe", "role": "launcher" }
            ]),
            json!([
                { "executable": "Same.exe", "role": "launcher" },
                { "executable": "SAME.EXE", "role": "client" }
            ]),
        ] {
            let proposal = propose_calibration(&request(target(None, launches)));
            assert_eq!(proposal.readiness, CalibrationLaunchReadiness::Unavailable);
            assert_eq!(
                proposal.limitations[0].kind,
                CalibrationProposalLimitationKind::InvalidPublisherChain
            );
            assert!(proposal.steps.is_empty());
        }
    }

    #[test]
    fn missing_and_malformed_launch_declarations_are_typed_limitations() {
        for launches in [
            serde_json::Value::Null,
            json!([{ "os": "windows", "executable": "" }]),
            json!([{ "os": "linux", "executable": "game" }]),
        ] {
            let mut target = direct_target();
            target.launch_entries = if launches.is_null() {
                None
            } else {
                Some(launches)
            };
            let proposal = propose_calibration(&request(target));
            assert_eq!(proposal.readiness, CalibrationLaunchReadiness::Unavailable);
            assert_eq!(
                proposal.limitations[0].kind,
                CalibrationProposalLimitationKind::MissingLaunchDeclaration
            );
            assert!(proposal.steps.is_empty());
        }
    }

    #[test]
    fn publisher_launcher_and_terminal_presence_selects_general_warm_case() {
        let publisher = target(
            None,
            json!([
                { "executable": "Launcher.exe", "role": "launcher" },
                { "executable": "Game.exe", "role": "client" }
            ]),
        );
        let request =
            request(publisher).with_process_snapshot(CalibrationProcessSnapshot::complete([
                "launcher.exe",
                "game.exe",
                "GAME.EXE",
            ]));
        let proposal = propose_calibration(&request);
        assert!(matches!(
            proposal.readiness,
            CalibrationLaunchReadiness::OperatorAction {
                observed_case: CompatibilityLaunchCase::PublisherLauncherWarm,
                ..
            }
        ));
        assert!(proposal.steps.is_empty());
    }

    #[test]
    fn unavailable_inventory_cannot_claim_cold_state() {
        let request = CalibrationProposalRequest::new(direct_target(), "native", "0.9.0", "0.9.0")
            .with_process_snapshot(CalibrationProcessSnapshot::unavailable("snapshot failed"));
        let proposal = propose_calibration(&request);
        assert_eq!(proposal.readiness, CalibrationLaunchReadiness::Unavailable);
        assert_eq!(
            proposal.limitations[0].kind,
            CalibrationProposalLimitationKind::ProcessInventoryUnavailable
        );
        assert!(proposal.steps.is_empty());
    }

    #[test]
    fn missing_routing_proposes_only_reachability_and_defers_protocols() {
        let request = request(direct_target()).with_protocol_candidates([
            CompatibilityProtocol::WebSocket,
            CompatibilityProtocol::Https,
            CompatibilityProtocol::Https,
        ]);
        let proposal = propose_calibration(&request);
        assert_eq!(proposal.steps.len(), 1);
        assert_eq!(proposal.steps[0].phase, CalibrationPhase::Reachability);
        assert_eq!(
            proposal.steps[0].case.protocol,
            CompatibilityProtocol::Routing
        );
        assert_eq!(proposal.steps[0].reason, CalibrationProposalReason::Missing);
        assert_eq!(
            proposal.deferred_protocols,
            [
                DeferredCalibrationProtocol {
                    protocol: CompatibilityProtocol::Https,
                    reason: CalibrationProposalReason::ReachabilityRequired,
                },
                DeferredCalibrationProtocol {
                    protocol: CompatibilityProtocol::WebSocket,
                    reason: CalibrationProposalReason::ReachabilityRequired,
                }
            ]
        );
    }

    #[test]
    fn current_routing_opens_only_missing_protocol_work() {
        let route = fact(
            CompatibilityFactKey::ProxyRouting,
            "reached-client",
            CompatibilityProtocol::NotApplicable,
        );
        let inspected = fact(
            CompatibilityFactKey::Inspectability,
            "full",
            CompatibilityProtocol::Https,
        );
        let request = request(direct_target())
            .with_facts([route, inspected])
            .with_protocol_candidates([
                CompatibilityProtocol::WebSocket,
                CompatibilityProtocol::Https,
            ]);
        let proposal = propose_calibration(&request);
        assert_eq!(proposal.steps.len(), 1);
        assert_eq!(proposal.steps[0].phase, CalibrationPhase::Tls);
        assert_eq!(
            proposal.steps[0].case.protocol,
            CompatibilityProtocol::WebSocket
        );
        assert_eq!(
            proposal.steps[0].reason,
            CalibrationProposalReason::ContextMismatch
        );
        assert!(proposal.deferred_protocols.is_empty());
    }

    #[test]
    fn stale_legacy_mismatch_negative_and_conflict_have_distinct_reasons() {
        let base = fact(
            CompatibilityFactKey::ProxyRouting,
            "reached-client",
            CompatibilityProtocol::NotApplicable,
        );
        let mut stale = base.clone();
        stale.stale = true;
        assert_eq!(
            propose_calibration(&request(direct_target()).with_facts([stale])).steps[0].reason,
            CalibrationProposalReason::Stale
        );

        let mut legacy = base.clone();
        legacy.address_family = None;
        assert_eq!(
            propose_calibration(&request(direct_target()).with_facts([legacy])).steps[0].reason,
            CalibrationProposalReason::LegacyIncomplete
        );

        let mut mismatch = base.clone();
        mismatch.address_family = Some(CompatibilityAddressFamily::Ipv6);
        assert_eq!(
            propose_calibration(&request(direct_target()).with_facts([mismatch])).steps[0].reason,
            CalibrationProposalReason::ContextMismatch
        );

        let negative = fact(
            CompatibilityFactKey::ProxyRouting,
            "no-proxy-traffic",
            CompatibilityProtocol::NotApplicable,
        );
        assert_eq!(
            propose_calibration(&request(direct_target()).with_facts([negative])).steps[0].reason,
            CalibrationProposalReason::Negative
        );

        let mut conflict = base.clone();
        conflict.id = Some(2);
        conflict.value = "no-proxy-traffic".into();
        let proposal = propose_calibration(&request(direct_target()).with_facts([conflict, base]));
        assert_eq!(
            proposal.steps[0].reason,
            CalibrationProposalReason::Conflict
        );
    }

    #[test]
    fn defaults_and_ipv6_override_flow_into_every_exact_case() {
        let defaults = propose_calibration(&request(direct_target()));
        assert_eq!(
            defaults.routing_strategy,
            CompatibilityRoutingStrategy::ChildEnvironment
        );
        assert_eq!(defaults.address_family, CompatibilityAddressFamily::Ipv4);

        let ipv6 = request(direct_target()).with_address_family(CompatibilityAddressFamily::Ipv6);
        let ipv6 = propose_calibration(&ipv6);
        assert_eq!(ipv6.address_family, CompatibilityAddressFamily::Ipv6);
        assert_eq!(
            ipv6.steps[0].case.address_family,
            CompatibilityAddressFamily::Ipv6
        );
    }

    #[test]
    fn invalid_context_routing_and_protocols_are_zero_step_limitations() {
        let invalid_context =
            CalibrationProposalRequest::new(direct_target(), "", "0.9.0", "0.9.0")
                .with_process_snapshot(CalibrationProcessSnapshot::complete(Vec::<String>::new()));
        let proposal = propose_calibration(&invalid_context);
        assert_eq!(
            proposal.limitations[0].kind,
            CalibrationProposalLimitationKind::InvalidContext
        );
        assert!(proposal.steps.is_empty());

        let unsupported =
            request(direct_target()).with_routing_strategy(CompatibilityRoutingStrategy::Socks);
        let proposal = propose_calibration(&unsupported);
        assert_eq!(
            proposal.limitations[0].kind,
            CalibrationProposalLimitationKind::UnsupportedRoutingStrategy
        );
        assert!(proposal.steps.is_empty());

        let invalid_protocol = request(direct_target())
            .with_protocol_candidates([CompatibilityProtocol::NotApplicable]);
        let proposal = propose_calibration(&invalid_protocol);
        assert_eq!(
            proposal.limitations[0].kind,
            CalibrationProposalLimitationKind::UnsupportedProtocolCandidate
        );
        assert!(proposal.steps.is_empty());
    }

    #[test]
    fn fact_order_and_duplicate_protocols_do_not_change_the_proposal() {
        let positive = fact(
            CompatibilityFactKey::ProxyRouting,
            "reached-client",
            CompatibilityProtocol::NotApplicable,
        );
        let mut negative = positive.clone();
        negative.id = Some(2);
        negative.value = "no-proxy-traffic".into();
        let left = request(direct_target())
            .with_facts([positive.clone(), negative.clone()])
            .with_protocol_candidates([
                CompatibilityProtocol::WebSocket,
                CompatibilityProtocol::Https,
                CompatibilityProtocol::WebSocket,
            ]);
        let right = request(direct_target())
            .with_facts([negative, positive])
            .with_protocol_candidates([
                CompatibilityProtocol::Https,
                CompatibilityProtocol::WebSocket,
            ]);
        assert_eq!(propose_calibration(&left), propose_calibration(&right));
    }

    #[test]
    fn every_exact_case_dimension_can_make_protocol_evidence_mismatched() {
        type FactMutation = fn(&mut CompatibilityFact);
        let mutations: [FactMutation; 8] = [
            |fact| fact.launch_case = Some(CompatibilityLaunchCase::SteamProtocolCold),
            |fact| fact.proxy_backend = Some("other".into()),
            |fact| fact.proxy_backend_version = Some("other".into()),
            |fact| fact.routing_strategy = Some(CompatibilityRoutingStrategy::Socks),
            |fact| fact.address_family = Some(CompatibilityAddressFamily::Ipv6),
            |fact| fact.protocol = Some(CompatibilityProtocol::Http2),
            |fact| fact.fragcap_version = Some("other".into()),
            |fact| fact.target_version = Some("other".into()),
        ];
        let mut route = fact(
            CompatibilityFactKey::ProxyRouting,
            "reached-client",
            CompatibilityProtocol::NotApplicable,
        );
        route.target_version = Some("build-a".into());
        for mutate in mutations {
            let mut inspected = fact(
                CompatibilityFactKey::Inspectability,
                "full",
                CompatibilityProtocol::Https,
            );
            inspected.target_version = Some("build-a".into());
            mutate(&mut inspected);
            let request = request(direct_target())
                .with_target_version(Some("build-a".into()))
                .with_protocol_candidates([CompatibilityProtocol::Https])
                .with_facts([route.clone(), inspected]);
            let proposal = propose_calibration(&request);
            assert_eq!(proposal.steps.len(), 1);
            assert_eq!(
                proposal.steps[0].reason,
                CalibrationProposalReason::ContextMismatch
            );
        }
    }

    #[test]
    fn exact_fact_helper_remains_bound_to_s121_applicability() {
        let fact = fact(
            CompatibilityFactKey::ProxyRouting,
            "reached-client",
            CompatibilityProtocol::NotApplicable,
        );
        let case = CompatibilityCase {
            launch_case: CompatibilityLaunchCase::DirectExeCold,
            proxy_backend: "native".into(),
            proxy_backend_version: "0.9.0".into(),
            routing_strategy: CompatibilityRoutingStrategy::ChildEnvironment,
            address_family: CompatibilityAddressFamily::Ipv4,
            protocol: CompatibilityProtocol::Routing,
            fragcap_version: "0.9.0".into(),
            target_version: None,
        };
        assert_eq!(
            fact.applicability(&case),
            CompatibilityApplicability::Applicable
        );
    }
}
