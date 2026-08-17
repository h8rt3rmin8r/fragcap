// SPDX-License-Identifier: Apache-2.0

//! The seam accepts a new source with no driver change (S052 spec SC-006).
//!
//! Proves P-10's forward-looking property: a `FixtureSource` added to a discovery
//! run yields its candidates and a conserved account, and `discover_all` (the
//! driver) is unchanged by the addition.

use fragcap_profile::FidelityTier;
use fragcap_targets::{
    discover_all, CandidateIdentity, CandidateTarget, Discovery, DiscoveryAccount, FixtureSource,
    TargetClassification, TargetSource,
};

fn one_candidate(name: &str, source: &str, fidelity: FidelityTier) -> Discovery {
    let mut account = DiscoveryAccount::default();
    account.produce();
    Discovery {
        candidates: vec![CandidateTarget {
            identity: CandidateIdentity::Path(format!("D:/games/{name}")),
            display_name: name.to_string(),
            fidelity,
            classification: TargetClassification::Unknown,
            evidence: Vec::new(),
            source_name: source.to_string(),
        }],
        account,
        ..Discovery::default()
    }
}

#[test]
fn a_new_source_needs_no_driver_change() {
    let steam_like = FixtureSource::new(
        "steam-like",
        FidelityTier::HeuristicUnverified,
        one_candidate("half-life", "steam-like", FidelityTier::HeuristicUnverified),
    );
    // A brand new source is added to the same driver with no other edit.
    let epic_like = FixtureSource::new(
        "epic-like",
        FidelityTier::HeuristicUnverified,
        one_candidate("fortnite", "epic-like", FidelityTier::HeuristicUnverified),
    );

    let sources: Vec<&dyn TargetSource> = vec![&steam_like, &epic_like];
    let discovery = discover_all(&sources).unwrap();

    assert_eq!(discovery.candidates.len(), 2);
    assert_eq!(discovery.account.considered, 2);
    assert_eq!(discovery.account.produced, 2);
    assert!(discovery.account.is_conserved());

    // The source names propagate onto the candidates, so a listing can attribute
    // each find to its origin.
    let names: Vec<&str> = discovery
        .candidates
        .iter()
        .map(|c| c.source_name.as_str())
        .collect();
    assert!(names.contains(&"steam-like"));
    assert!(names.contains(&"epic-like"));
}

#[test]
fn default_fidelity_is_the_sources_stamp() {
    let authored = FixtureSource::new(
        "authored-src",
        FidelityTier::Authored,
        one_candidate("thing", "authored-src", FidelityTier::Authored),
    );
    assert_eq!(authored.default_fidelity(), FidelityTier::Authored);
    assert_eq!(authored.name(), "authored-src");
}
