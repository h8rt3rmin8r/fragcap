// SPDX-License-Identifier: Apache-2.0

//! Tier 3, the user-pointed directory and interactive sources (S052 spec US3,
//! FR-011/FR-012). Pure, no filesystem; conservation asserted throughout (P-4).

use fragcap_profile::FidelityTier;
use fragcap_targets::{
    DirectorySource, InteractiveSource, ScriptedConfirm, TargetClassification, TargetSource,
};

#[test]
fn directory_source_yields_one_candidate_at_heuristic_fidelity() {
    let source = DirectorySource::new("D:/Games/Celeste");
    let d = source.discover().unwrap();

    assert_eq!(d.candidates.len(), 1);
    let c = &d.candidates[0];
    assert_eq!(c.display_name, "Celeste");
    assert_eq!(c.fidelity, FidelityTier::HeuristicUnverified);
    assert_eq!(c.classification, TargetClassification::Unknown);
    assert!(d.account.is_conserved());
    assert_eq!(d.account.produced, 1);
}

#[test]
fn an_empty_path_yields_no_candidate() {
    let source = DirectorySource::new("   ");
    let d = source.discover().unwrap();
    assert!(d.candidates.is_empty());
    assert_eq!(d.account.considered, 0);
    assert!(d.account.is_conserved());
}

#[test]
fn interactive_accept_stamps_authored() {
    let yes = ScriptedConfirm::new(true);
    let source = InteractiveSource::new(DirectorySource::new("D:/Games/Hades"), &yes);
    let d = source.discover().unwrap();

    assert_eq!(d.candidates.len(), 1);
    assert_eq!(d.candidates[0].fidelity, FidelityTier::Authored);
    assert_eq!(d.candidates[0].source_name, "interactive");
    assert_eq!(d.account.produced, 1);
    assert!(d.account.is_conserved());
}

#[test]
fn interactive_reject_declines_and_does_not_lose() {
    let no = ScriptedConfirm::new(false);
    let source = InteractiveSource::new(DirectorySource::new("D:/Games/Hades"), &no);
    let d = source.discover().unwrap();

    assert!(d.candidates.is_empty());
    assert_eq!(d.account.declined_by_user, 1);
    assert_eq!(d.account.produced, 0);
    assert!(
        d.account.is_conserved(),
        "a declined candidate is counted, not lost (P-4)"
    );
}
