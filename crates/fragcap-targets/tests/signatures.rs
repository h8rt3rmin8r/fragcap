// SPDX-License-Identifier: Apache-2.0

//! The detection signature seed, end to end (slice S053 US2).
//!
//! Seeds the bundled Appendix B set into a store, loads it back, and detects each
//! implemented-kind product from a fixture directory (SC-001, SC-003). Proves a new
//! signature row is honored with no code change (SC-002) and that the inert
//! binary-marker rows are counted rather than dropped (P-4).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use fragcap_profile::{
    Signature, SignatureCategory, SignatureConfidence, SignatureKind, SignatureSet,
};
use fragcap_targets::{seed_bundled, Store};

struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(tag: &str) -> TempTree {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "fragcap-sig-seed-{}-{}-{}",
            std::process::id(),
            tag,
            n
        ));
        fs::create_dir_all(&root).expect("create temp root");
        TempTree { root }
    }

    fn touch(&self, rel: &str) {
        let full = self.root.join(rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).expect("parents");
        }
        fs::write(&full, b"").expect("write");
    }

    fn mkdir(&self, rel: &str) {
        fs::create_dir_all(self.root.join(rel)).expect("dir");
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// A fixture install carrying a marker for every implemented-kind Appendix B
/// product, so a single scan should detect them all.
fn all_markers_tree() -> TempTree {
    let tree = TempTree::new("all-markers");
    // Engines.
    tree.touch("UnityPlayer.dll"); // Unity (filename)
    tree.mkdir("Engine/Binaries/Win64"); // Unreal (directory-shape)
    tree.touch("tier0.dll"); // Source (filename)
    tree.touch("game.pck"); // Godot (filename)
    tree.touch("CrySystem.dll"); // CryEngine (filename)
    tree.touch("re_chunk_000.pak"); // RE Engine (filename)
                                    // Anti-cheat.
    tree.touch("EasyAntiCheat/EasyAntiCheat_x64.dll"); // Easy Anti-Cheat
    tree.touch("BEService_x64.exe"); // BattlEye
    tree.touch("vgk.sys"); // Vanguard
    tree.touch("mhyprot3.sys"); // mhyprot
    tree.mkdir("GameGuard"); // nProtect GameGuard (directory-shape)
    tree.touch("xhunter1.sys"); // Xigncode3
                                // DRM (implemented).
    tree.touch("steam_api64.dll"); // Steam DRM
    tree
}

const IMPLEMENTED_PRODUCTS: &[&str] = &[
    "Unity",
    "Unreal",
    "Source",
    "Godot",
    "CryEngine",
    "RE Engine",
    "Easy Anti-Cheat",
    "BattlEye",
    "Vanguard",
    "mhyprot",
    "nProtect GameGuard",
    "Xigncode3",
    "Steam DRM",
];

#[test]
fn seeding_the_bundled_set_populates_the_table_and_counts_inert_rows() {
    let mut store = Store::open_in_memory().expect("store");
    let count = seed_bundled(&mut store).expect("seed");
    assert!(count >= 16, "the bundled set has at least 16 rows");

    let signatures = store.load_signatures().expect("load");
    // SC-001: every Appendix B product is represented.
    let products: Vec<&str> = signatures.iter().map(|s| s.product.as_str()).collect();
    for p in IMPLEMENTED_PRODUCTS
        .iter()
        .chain(&["Denuvo", "Arxan", "VMProtect"])
    {
        assert!(products.contains(p), "product {p} is seeded");
    }

    // The three content-only DRM products are inert (binary-marker), counted not
    // dropped (P-4).
    let set = SignatureSet::compile(&signatures);
    assert_eq!(set.inert_count(), 3, "Denuvo/Arxan/VMProtect are inert");
    assert_eq!(
        set.applied_count() + set.inert_count() + set.skipped_count(),
        set.total_count(),
        "the load accounting is conserved"
    );
}

#[test]
fn every_implemented_product_is_detected_from_a_fixture() {
    // SC-003: seed the store, load and compile, and detect each implemented product
    // from one fixture directory carrying its marker.
    let mut store = Store::open_in_memory().expect("store");
    seed_bundled(&mut store).expect("seed");
    let set = SignatureSet::compile(&store.load_signatures().expect("load"));

    let tree = all_markers_tree();
    let outcome = set.detect(tree.path()).expect("readable");
    let found: Vec<&str> = outcome
        .findings
        .iter()
        .map(|f| f.product.as_str())
        .collect();
    for product in IMPLEMENTED_PRODUCTS {
        assert!(
            found.contains(product),
            "implemented product {product} detected; found {found:?}"
        );
    }
    // No inert DRM product matches (they never do this slice).
    for inert in ["Denuvo", "Arxan", "VMProtect"] {
        assert!(
            !found.contains(&inert),
            "{inert} is inert and does not match"
        );
    }
}

#[test]
fn a_new_signature_row_is_honored_with_no_code_change() {
    // SC-002: add one filename signature for a fictional product to the table, then
    // detect it with no code change and no rebuild of the matcher.
    let mut store = Store::open_in_memory().expect("store");
    seed_bundled(&mut store).expect("seed");

    let mut signatures = store.load_signatures().expect("load");
    signatures.push(Signature {
        category: SignatureCategory::Engine,
        kind: SignatureKind::Filename,
        pattern: "FictionEngine.dll".to_string(),
        product: "FictionEngine".to_string(),
        confidence: SignatureConfidence::Definitive,
    });
    store
        .seed_signatures(&signatures)
        .expect("re-seed with new row");

    let set = SignatureSet::compile(&store.load_signatures().expect("reload"));
    let tree = TempTree::new("fiction");
    tree.touch("FictionEngine.dll");
    let outcome = set.detect(tree.path()).expect("readable");
    assert!(
        outcome
            .findings
            .iter()
            .any(|f| f.product == "FictionEngine"),
        "the new row is honored with no code change"
    );
}
