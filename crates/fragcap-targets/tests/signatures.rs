// SPDX-License-Identifier: Apache-2.0

//! The detection signature seed, end to end (slice S053 US2, extended in S065).
//!
//! Seeds the bundled Appendix B set into a store, loads it back, and detects each
//! matchable product from a fixture directory (SC-001, SC-003). Proves a new
//! signature row is honored with no code change (SC-002) and that the inert
//! byte-marker rows are counted rather than dropped (P-4).
//!
//! S065 adds the directed subset invariant between the two engine detectors: every
//! engine the launch-resolution rules can select a client executable for must have
//! an engine-category signature naming the same product. See the slice decisions
//! fragment for why the invariant is directed rather than an equality.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use fragcap_profile::pe;
use fragcap_profile::{
    Engine, Signature, SignatureCategory, SignatureConfidence, SignatureKind, SignatureSet,
};
use fragcap_targets::{parse_seed_document, seed_bundled, Store, BUNDLED_SIGNATURES};

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
        self.write(rel, b"");
    }

    fn write(&self, rel: &str, bytes: &[u8]) {
        let full = self.root.join(rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).expect("parents");
        }
        fs::write(&full, bytes).expect("write");
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
    tree.touch("EACLaunch.exe"); // Easy Anti-Cheat bootstrapper (#170)
    tree.touch("Installers/AntiCheatInstaller.exe"); // Easy Anti-Cheat bootstrapper (#170)
    tree.touch("start_protected_game.exe"); // Easy Anti-Cheat launcher shim (#170)
    tree.mkdir("EasyAntiCheat_EOS"); // Easy Anti-Cheat EOS variant (directory-shape, #170)
    tree.touch("BEService_x64.exe"); // BattlEye
    tree.touch("vgk.sys"); // Vanguard
    tree.touch("mhyprot3.sys"); // mhyprot
    tree.mkdir("GameGuard"); // nProtect GameGuard (directory-shape)
    tree.touch("xhunter1.sys"); // Xigncode3
    tree.touch("renpy/bootstrap.py"); // Ren'Py (directory-shape)
    tree.touch("data.win"); // GameMaker (filename)
                            // DRM (matchable): the wrapper appends a `.bind` PE
                            // section to the launch executable. Shipping
                            // `steam_api64.dll` is the Steamworks SDK and is
                            // deliberately not a DRM signal any more (S065, #169).
    tree.touch("steam_api64.dll");
    tree.write(
        "Game.exe",
        &pe::fixtures::minimal_pe_with_sections(&[".text", ".rdata", ".bind"]),
    );
    tree
}

const MATCHABLE_PRODUCTS: &[&str] = &[
    "Unity",
    "Unreal",
    "Source",
    "Godot",
    "CryEngine",
    "RE Engine",
    "Ren'Py",
    "GameMaker",
    "Easy Anti-Cheat",
    "BattlEye",
    "Vanguard",
    "mhyprot",
    "nProtect GameGuard",
    "Xigncode3",
    "Steam DRM",
];

/// The byte-sequence marker products this build carries but cannot match. Derived
/// from the seed rather than listed twice: a row moving out of this class changes
/// the seed and the assertion together.
fn inert_products(signatures: &[Signature]) -> Vec<String> {
    signatures
        .iter()
        .filter(|s| !s.is_matchable())
        .map(|s| s.product.clone())
        .collect()
}

#[test]
fn seeding_the_bundled_set_populates_the_table_and_counts_inert_rows() {
    let mut store = Store::open_in_memory().expect("store");
    let count = seed_bundled(&mut store).expect("seed");
    assert!(count >= 16, "the bundled set has at least 16 rows");

    let signatures = store.load_signatures().expect("load");
    // SC-001: every Appendix B product is represented.
    let products: Vec<&str> = signatures.iter().map(|s| s.product.as_str()).collect();
    let inert = inert_products(&signatures);
    for p in MATCHABLE_PRODUCTS
        .iter()
        .copied()
        .chain(inert.iter().map(String::as_str))
    {
        assert!(products.contains(&p), "product {p} is seeded");
    }

    // The byte-marker rows are inert, counted rather than dropped (P-4). The count
    // is derived from the seed itself rather than asserted as a literal, so adding
    // or implementing one of them does not need this assertion edited, and cannot
    // silently stop being covered.
    let set = SignatureSet::compile(&signatures);
    assert_eq!(
        set.inert_count(),
        inert.len(),
        "every unmatchable row is inert, none dropped: {inert:?}"
    );
    assert!(
        !inert.is_empty(),
        "the byte-marker rows are still carried and still inert"
    );
    assert_eq!(
        set.applied_count() + set.inert_count() + set.skipped_count(),
        set.total_count(),
        "the load accounting is conserved"
    );
    assert_eq!(
        set.skipped_count(),
        0,
        "the shipped seed has no malformed row"
    );
}

#[test]
fn the_shipped_seed_reports_no_drm_for_the_steamworks_sdk() {
    // #169: the two `steam_api*.dll` rows reported "Steam DRM" on 28 of 32 real
    // rows, on the basis of a library that ships with essentially every Steam
    // title. They are gone, and no row may reintroduce that pattern.
    let signatures = parse_seed_document(BUNDLED_SIGNATURES).expect("parse");
    for s in &signatures {
        assert!(
            !s.pattern.to_lowercase().contains("steam_api"),
            "the Steamworks SDK library is not a DRM signal: {s:?}"
        );
    }

    // And a tree that ships the library with an unwrapped executable reports none.
    let set = SignatureSet::compile(&signatures);
    let tree = TempTree::new("sdk-only");
    tree.touch("steam_api64.dll");
    tree.touch("steam_api.dll");
    tree.write(
        "Game.exe",
        &pe::fixtures::minimal_pe_with_sections(&[".text", ".rdata", ".reloc"]),
    );
    let outcome = set.detect(tree.path()).expect("readable");
    assert!(
        !outcome
            .findings
            .iter()
            .any(|f| f.category == SignatureCategory::Drm),
        "no DRM from the SDK alone: {:?}",
        outcome.findings
    );
}

#[test]
fn the_measured_division_2_layout_reports_easy_anti_cheat() {
    // #170: measured on a real machine. The old two-row EAC set (`.dll`/`.sys`
    // filenames only) matched none of this.
    let signatures = parse_seed_document(BUNDLED_SIGNATURES).expect("parse");
    let set = SignatureSet::compile(&signatures);

    let tree = TempTree::new("division2-eac");
    tree.touch("EACLaunch.exe");
    tree.touch("EasyAntiCheat/EasyAntiCheat_EOS_Setup.exe");
    tree.touch("EOSSDK-Win64-Shipping.dll");
    let findings = set.detect(tree.path()).expect("readable").findings;
    assert!(
        findings
            .iter()
            .any(|f| f.category == SignatureCategory::AntiCheat && f.product == "Easy Anti-Cheat"),
        "expected Easy Anti-Cheat from the measured Division 2 layout: {findings:?}"
    );
}

#[test]
fn the_measured_arc_raiders_layout_reports_easy_anti_cheat() {
    // #170: measured on a real machine.
    let signatures = parse_seed_document(BUNDLED_SIGNATURES).expect("parse");
    let set = SignatureSet::compile(&signatures);

    let tree = TempTree::new("arc-raiders-eac");
    tree.touch("Installers/AntiCheatInstaller.exe");
    tree.touch("Engine/Binaries/Win64/EOSSDK-Win64-Shipping.dll");
    let findings = set.detect(tree.path()).expect("readable").findings;
    assert!(
        findings
            .iter()
            .any(|f| f.category == SignatureCategory::AntiCheat && f.product == "Easy Anti-Cheat"),
        "expected Easy Anti-Cheat from the measured Arc Raiders layout: {findings:?}"
    );
}

#[test]
fn eossdk_alone_is_never_anti_cheat_evidence() {
    // #170's explicit false-positive warning: EOSSDK-Win64-Shipping.dll ships in
    // Carnal Instinct, Oblivion Remastered, Palworld, Satisfactory, and ESO, none
    // of which are EAC titles. Fixtured as a standing regression (SC-002).
    let signatures = parse_seed_document(BUNDLED_SIGNATURES).expect("parse");
    let set = SignatureSet::compile(&signatures);

    let tree = TempTree::new("eossdk-only");
    tree.touch("EOSSDK-Win64-Shipping.dll");
    tree.touch("Game.exe");
    let findings = set.detect(tree.path()).expect("readable").findings;
    assert!(
        !findings
            .iter()
            .any(|f| f.category == SignatureCategory::AntiCheat),
        "EOSSDK alone must never report anti-cheat: {findings:?}"
    );
}

#[test]
fn every_launch_resolution_engine_is_nameable_by_the_signature_set() {
    // The directed subset invariant (S065, #168 option b). An engine the
    // launch-resolution rules can select a client executable for, that the
    // signature set cannot name, produces a run where the resolver silently used an
    // engine rule while the listing reports no engine at all.
    //
    // It is directed on purpose: the signature set legitimately names engines
    // nobody has written a client-selection rule for (Source, CryEngine, RE Engine,
    // GameMaker), and requiring equality would force either a fabricated selection
    // rule or the removal of a true detection.
    //
    // It iterates `Engine::ALL` rather than a list maintained beside it, so adding
    // a variant without adding a signature fails here, and it asserts no count.
    let signatures = parse_seed_document(BUNDLED_SIGNATURES).expect("parse");
    for engine in Engine::ALL {
        let product = engine.product_name();
        assert!(
            signatures
                .iter()
                .any(|s| s.category == SignatureCategory::Engine && s.product == product),
            "engine {product:?} can be resolved to a client but has no detection \
             signature: add an engine-category row for it to \
             crates/fragcap-targets/assets/signatures.json"
        );
    }
}

#[test]
fn the_measured_renpy_and_gamemaker_trees_each_name_one_engine() {
    // The two trees measured on the operator machine (#168), fixtured so this is
    // covered with no Steam install.
    let signatures = parse_seed_document(BUNDLED_SIGNATURES).expect("parse");
    let set = SignatureSet::compile(&signatures);

    let renpy = TempTree::new("renpy-tree");
    renpy.touch("TrappedWithIvyAndPiper-EA.exe");
    renpy.touch("TrappedWithIvyAndPiper-EA.py");
    renpy.touch("game/archive.rpa");
    renpy.touch("renpy/bootstrap.py");
    renpy.touch("lib/py3-windows-x86_64/librenpython.dll");
    renpy.touch("lib/py3-windows-x86_64/steam_api64.dll");
    let findings = set.detect(renpy.path()).expect("readable").findings;
    let engines: Vec<&str> = findings
        .iter()
        .filter(|f| f.category == SignatureCategory::Engine)
        .map(|f| f.product.as_str())
        .collect();
    assert_eq!(
        engines,
        vec!["Ren'Py"],
        "one engine after dedup: {findings:?}"
    );
    assert!(
        !findings
            .iter()
            .any(|f| f.category == SignatureCategory::Drm),
        "no wrapper, so no DRM: {findings:?}"
    );

    let gamemaker = TempTree::new("gamemaker-tree");
    gamemaker.touch("Shale Hill Secrets.exe");
    gamemaker.touch("data.win");
    gamemaker.touch("options.ini");
    gamemaker.touch("Steamworks_x64.dll");
    gamemaker.touch("steam_api64.dll");
    let findings = set.detect(gamemaker.path()).expect("readable").findings;
    let engines: Vec<&str> = findings
        .iter()
        .filter(|f| f.category == SignatureCategory::Engine)
        .map(|f| f.product.as_str())
        .collect();
    assert_eq!(
        engines,
        vec!["GameMaker"],
        "one engine after dedup: {findings:?}"
    );
    assert!(
        !findings
            .iter()
            .any(|f| f.category == SignatureCategory::Drm),
        "no wrapper, so no DRM: {findings:?}"
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
    for product in MATCHABLE_PRODUCTS {
        assert!(
            found.contains(product),
            "implemented product {product} detected; found {found:?}"
        );
    }
    // No inert product matches, by construction.
    for inert in inert_products(&store.load_signatures().expect("reload")) {
        assert!(
            !found.contains(&inert.as_str()),
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
