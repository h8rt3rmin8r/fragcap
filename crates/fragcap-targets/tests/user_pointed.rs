// SPDX-License-Identifier: Apache-2.0

//! Tier 3, the user-pointed directory and interactive sources (S052 spec US3,
//! FR-011/FR-012). Conservation asserted throughout (P-4).
//!
//! Slice S065 adds the coverage state this source records and the named warning it
//! emits when a scan bound truncates the candidate set: a counted loss that is never
//! named is only half of what P-4 asks for.

use fragcap_profile::{
    FidelityTier, Signature, SignatureCategory, SignatureConfidence, SignatureKind, SignatureSet,
};
use fragcap_targets::{
    DetectionScan, DirectorySource, InteractiveSource, ScriptedConfirm, TargetClassification,
    TargetSource,
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

/// A scratch directory holding `count` executables, one more than the scan cap plus
/// whatever margin the caller asks for, removed on drop.
struct ExeTree {
    root: std::path::PathBuf,
}

impl ExeTree {
    fn new(count: usize) -> ExeTree {
        let root = std::env::temp_dir().join(format!(
            "fragcap-user-pointed-exes-{}-{count}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create root");
        let bytes = fragcap_profile::pe::fixtures::minimal_pe_with_sections(&[".text"]);
        for i in 0..count {
            std::fs::write(root.join(format!("game-{i:04}.exe")), &bytes).expect("write exe");
        }
        ExeTree { root }
    }
}

impl Drop for ExeTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn bind_set() -> SignatureSet {
    SignatureSet::compile(&[Signature {
        category: SignatureCategory::Drm,
        kind: SignatureKind::BinaryMarker,
        pattern: "section:.bind".to_string(),
        product: "Steam DRM".to_string(),
        confidence: SignatureConfidence::Definitive,
    }])
}

#[test]
fn a_scan_that_ran_records_its_coverage_state_and_one_that_did_not_records_none() {
    // A source carrying no signature set runs no scan, so it makes no coverage
    // claim. Recording `Complete` there would assert a clean scan that never
    // happened (P-9).
    let none = DirectorySource::new("D:/Games/Celeste");
    let d = none.discover().unwrap();
    assert_eq!(d.candidates[0].detection_scan, None);

    let tree = ExeTree::new(1);
    let scanned =
        DirectorySource::with_signatures(tree.root.to_string_lossy().into_owned(), bind_set());
    let d = scanned.discover().unwrap();
    assert_eq!(
        d.candidates[0].detection_scan,
        Some(DetectionScan::Complete),
        "a scan that read everything it set out to is complete"
    );
    assert!(d.account.is_conserved());
}

#[test]
fn a_truncated_candidate_set_names_the_scan_root_in_the_warning() {
    // P-4 asks for a loss to be counted *and* surfaced. The count lives on the scan
    // outcome; this is the surfacing, and without it an operator would see a row
    // marked `incomplete` with nothing saying why.
    let over = fragcap_profile::signature::MARKER_SCAN_MAX_CANDIDATES + 3;
    let tree = ExeTree::new(over);
    let source =
        DirectorySource::with_signatures(tree.root.to_string_lossy().into_owned(), bind_set());
    let d = source.discover().unwrap();

    assert_eq!(
        d.candidates[0].detection_scan,
        Some(DetectionScan::Incomplete),
        "a truncated scan is not reported as a clean one"
    );
    let named = d
        .warnings
        .iter()
        .find(|w| w.contains("binary marker"))
        .unwrap_or_else(|| panic!("the truncation is named: {:?}", d.warnings));
    assert!(
        named.contains("3 more were not examined"),
        "the warning says how many were dropped: {named}"
    );
    assert!(
        named.contains(&tree.root.display().to_string()),
        "the warning names the scan root: {named}"
    );
    assert!(
        named.contains("technology detection for this root may be incomplete"),
        "the warning says the technology result may be incomplete: {named}"
    );
    assert!(d.account.is_conserved());
}
