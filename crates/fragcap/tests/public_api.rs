// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use std::path::PathBuf;

use fragcap::deep_capture::api::{
    AdapterSetBuilder, CancellationToken, ClassificationReason, CompatibilityAddressFamily,
    CompatibilityApplicability, CompatibilityCase, CompatibilityEvidenceSource,
    CompatibilityFreshness, CompatibilityLaunchCase, CompatibilityRoutingStrategy, LaunchCase,
    ResourceKind, SessionConfigBuilder, SessionMode, StoredCompatibilityFact, TargetsError,
    TrustError, DEEP_CAPTURE_API_VERSION, STABLE_API_EXPORTS,
};

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn version_one_inventory_is_curated_and_sorted() {
    assert_eq!(DEEP_CAPTURE_API_VERSION, 1);
    assert_eq!(STABLE_API_EXPORTS.len(), 125);
    assert!(STABLE_API_EXPORTS.windows(2).all(|pair| pair[0] < pair[1]));
    for required in [
        "AdapterSetBuilder",
        "Authorization",
        "CancellationToken",
        "ClassificationReason",
        "CompatibilityCase",
        "DEEP_CAPTURE_API_VERSION",
        "DeepCapture",
        "NativeProxyAdapter",
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
