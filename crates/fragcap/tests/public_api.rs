// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use std::path::PathBuf;

use fragcap::deep_capture::api::{
    propose_calibration, AdapterSetBuilder, CalibrationProcessSnapshot, CalibrationProposal,
    CalibrationProposalRequest, CalibrationTopologyKind, CancellationToken, ClassificationReason,
    CompatibilityAddressFamily, CompatibilityApplicability, CompatibilityCase,
    CompatibilityEvidenceSource, CompatibilityFreshness, CompatibilityLaunchCase,
    CompatibilityProtocol, CompatibilityRoutingStrategy, LaunchCase, ResourceKind,
    SessionConfigBuilder, SessionMode, StoredCompatibilityFact, TargetsError, TrustError,
    DEEP_CAPTURE_API_VERSION, STABLE_API_EXPORTS,
};
use fragcap::profile::FidelityTier;
use fragcap::targets::{ClassificationSource, TargetClassification, TargetEntry};
use serde_json::json;

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn version_one_inventory_is_curated_and_sorted() {
    assert_eq!(DEEP_CAPTURE_API_VERSION, 1);
    assert_eq!(STABLE_API_EXPORTS.len(), 137);
    assert!(STABLE_API_EXPORTS.windows(2).all(|pair| pair[0] < pair[1]));
    for required in [
        "AdapterSetBuilder",
        "Authorization",
        "CancellationToken",
        "CalibrationProposal",
        "ClassificationReason",
        "CompatibilityCase",
        "DEEP_CAPTURE_API_VERSION",
        "DeepCapture",
        "NativeProxyAdapter",
        "PreparedNativeAuthority",
        "PreparedSession",
        "ProtocolClassification",
        "RecoveryPlan",
        "RoutingPlan",
        "StoredCompatibilityFact",
        "STABLE_API_EXPORTS",
        "SessionConfigBuilder",
        "TerminalReport",
        "TrafficFamily",
        "TrustError",
        "propose_calibration",
    ] {
        assert!(
            STABLE_API_EXPORTS.binary_search(&required).is_ok(),
            "missing stable export {required}"
        );
    }
    for implementation_detail in [
        "ApplicationArtifactLease",
        "LifecycleWriter",
        "NativeProxyLease",
        "ResourceJournal",
    ] {
        assert!(
            STABLE_API_EXPORTS
                .binary_search(&implementation_detail)
                .is_err(),
            "implementation detail leaked into stable inventory: {implementation_detail}"
        );
    }
}

#[test]
fn calibration_proposal_is_consumable_through_the_stable_module() {
    let target = TargetEntry {
        id: Some(7),
        stable_id: 70,
        handle: "game".into(),
        name: "Game".into(),
        classification: TargetClassification::Game,
        classification_source: ClassificationSource::User,
        fidelity: FidelityTier::Authored,
        provenance: None,
        anchor: None,
        launch_entries: Some(json!([{ "executable": "Game.exe", "role": "client" }])),
        install_root: Some("C:\\Games\\Game".into()),
        evidence: None,
        detection_scan: None,
        folder_name: None,
        executable_hint: None,
    };
    let request = CalibrationProposalRequest::new(target, "native", "0.9.0", "0.9.0")
        .with_process_snapshot(CalibrationProcessSnapshot::complete(Vec::<String>::new()))
        .with_protocol_candidates([CompatibilityProtocol::Https]);
    let proposal: CalibrationProposal = propose_calibration(&request);

    assert_eq!(proposal.topology, Some(CalibrationTopologyKind::Direct));
    assert_eq!(
        proposal.routing_strategy,
        CompatibilityRoutingStrategy::ChildEnvironment
    );
    assert_eq!(proposal.address_family, CompatibilityAddressFamily::Ipv4);
    assert_eq!(proposal.steps.len(), 1);
    assert_eq!(
        proposal.steps[0].case.protocol,
        CompatibilityProtocol::Routing
    );
}

#[test]
fn transitive_signature_types_are_available_from_the_stable_module() {
    let _ = std::mem::size_of::<ClassificationReason>();
    let _ = std::mem::size_of::<CompatibilityAddressFamily>();
    let _ = std::mem::size_of::<CompatibilityApplicability>();
    let _ = std::mem::size_of::<CompatibilityCase>();
    let _ = std::mem::size_of::<CompatibilityEvidenceSource>();
    let _ = std::mem::size_of::<CompatibilityFreshness>();
    let _ = std::mem::size_of::<CompatibilityLaunchCase>();
    let _ = std::mem::size_of::<CompatibilityRoutingStrategy>();
    let _ = std::mem::size_of::<ResourceKind>();
    let _ = std::mem::size_of::<StoredCompatibilityFact>();
    let _ = std::mem::size_of::<TargetsError>();
    let _ = std::mem::size_of::<TrustError>();
}

#[test]
fn session_configuration_uses_an_extensible_builder() {
    let config = SessionConfigBuilder::new("controlled-target", PathBuf::from("bundle"))
        .mode(SessionMode::TlsCalibration)
        .launch_case(LaunchCase::Controlled)
        .controlled(true)
        .trust_ca(true)
        .build()
        .expect("valid public configuration");

    assert_eq!(config.target, "controlled-target");
    assert_eq!(config.launch_case, Some(LaunchCase::Controlled));
    assert!(config.controlled);
    assert!(config.trust_ca);
}

#[test]
fn empty_required_configuration_is_refused_before_preflight() {
    let error = SessionConfigBuilder::new("", PathBuf::new())
        .build()
        .expect_err("empty configuration must fail");
    assert_eq!(error.code(), "session-config-target-empty");
}

#[test]
fn cancellation_handle_is_shared_and_thread_safe() {
    assert_send_sync::<CancellationToken>();
    let first = CancellationToken::new();
    let second = first.clone();
    assert!(!second.is_requested());
    first.request();
    assert!(second.is_requested());
    second.request();
    assert!(first.is_requested());
}

#[test]
fn adapter_builder_reports_the_first_missing_capability() {
    let error = AdapterSetBuilder::new()
        .build()
        .expect_err("empty adapter set must fail");
    assert_eq!(error.capability(), "targets");
}

#[test]
fn cli_product_contract_does_not_bypass_the_stable_module() {
    let source = include_str!("../../fragcap-cli/src/commands/deep_capture.rs");
    for stable_name in STABLE_API_EXPORTS {
        let bypass = format!("fragcap::deep_capture::{stable_name}");
        assert!(
            !source.contains(&bypass),
            "CLI bypasses the stable API for {stable_name}"
        );
    }
}
