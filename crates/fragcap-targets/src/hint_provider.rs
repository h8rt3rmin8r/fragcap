// SPDX-License-Identifier: Apache-2.0

//! The hint-database resolution provider, specification section 15.7 (S037).
//!
//! This is the concrete provider at `Precedence::HintDatabase`, the precedence-2
//! position the resolver reserves and the profile-crate stub used to hold. It
//! lives here, not in `fragcap-profile`, because it reads the targets [`Store`]
//! and `fragcap-profile` may not depend on `fragcap-targets` (the dependency
//! direction). The trait, the precedence enum, and the [`HintTarget`] origin stay
//! in `fragcap-profile`; the store read lives in the crate that owns the store,
//! exactly as the platform walker (S030) lives in `fragcap-steam`.
//!
//! Given a Steam application id carried on the request, it looks up the one row
//! and answers with a heuristic-unverified [`Target`] built from the row's launch
//! executable, carrying the `launcher_mediated` and engine facts. It answers only
//! when the row names exactly one usable client (P-9): a sparse row, an
//! engine-only row, a row it cannot reduce to one executable, and a
//! launcher-mediated row (whose launch executable is the publisher launcher, not
//! the socket-holding client) are all declines, so the cascade continues to the
//! engine rule and below (P-4). Every answer is stamped `hint-db`, the same name
//! the store's export uses, and never claims a source it did not read. It reads
//! the embedded database only: no process handle, no memory, no launch, no network
//! (P-1).

use fragcap_profile::{
    FidelityTier, HintTarget, MatchPredicates, Precedence, Provenance, ProviderError,
    ResolutionNotes, ResolutionRequest, Target, TargetOrigin, TargetProvider,
};

use crate::entry::TargetEntry;
use crate::export::PROVENANCE_SOURCE;
use crate::model::{Game, LaunchEntry};
use crate::store::Store;

/// Resolves a Steam title from the targets hint databases. The precedence-2
/// provider of the resolution cascade.
///
/// Since the two-store split (slice S050) it holds an ordered list of stores and
/// consults them in turn, returning the first usable answer. The CLI orders them
/// `[local.db, catalog.db]`, so the user's learned launch data (from this
/// machine's own Steam install) is preferred over the shipped catalog seed. It
/// remains one provider at the single `Precedence::HintDatabase` slot; the
/// resolver rejects two providers at one precedence, and the fidelity-ordered
/// cascade collapse is a later slice.
pub struct HintDatabaseProvider {
    stores: Vec<Store>,
}

impl HintDatabaseProvider {
    /// Build the provider over a single already-opened store.
    ///
    /// The store is opened once by the caller (the CLI), so a corrupt or
    /// wrong-version database fails at that boundary and is surfaced there, not as
    /// a per-request surprise inside the cascade. Holding the store is sound
    /// because resolution is single-threaded and the store's reads take `&self`.
    pub fn new(store: Store) -> HintDatabaseProvider {
        HintDatabaseProvider {
            stores: vec![store],
        }
    }

    /// Build the provider over several stores, consulted in the given order.
    ///
    /// The first store to yield a usable client wins; a store that is absent of the
    /// title or declines it (launcher-mediated, ambiguous, or no usable executable)
    /// yields to the next. The CLI passes `[local, catalog]`.
    pub fn new_layered(stores: Vec<Store>) -> HintDatabaseProvider {
        HintDatabaseProvider { stores }
    }

    /// Combine the row for `app_id` across the layered stores into one game, as
    /// the single pre-split store held it.
    ///
    /// Launch entries and the name and engine come from the highest-priority store
    /// that carries them (local first). The launcher-mediated fact is authoritative
    /// wherever it is known, so a title the catalog marks mediated still declines
    /// even when the local store learned its launcher executable with the flag
    /// unset: before the split these were one merged row, and the local
    /// launch-data accumulation inserts an appid-only row with mediation unset, so
    /// reading the stores independently would resolve the launcher as the client
    /// and lose the gameplay traffic (P-4, P-9). Returns `None` when no store has
    /// the row.
    fn combined_game(&self, app_id: u32) -> Result<Option<Game>, ProviderError> {
        let mut combined: Option<Game> = None;
        for store in &self.stores {
            let game = match store.game(app_id) {
                Ok(Some(game)) => game,
                // Absent row: not an error, try the next store.
                Ok(None) => continue,
                // A read that failed after the store opened cleanly (a disk fault)
                // is a real fault, not a decline: abort rather than hide it (P-4).
                Err(e) => return Err(ProviderError::Hint(e.to_string())),
            };
            match combined.as_mut() {
                None => combined = Some(game),
                Some(c) => {
                    // Fill facts the higher-priority store left unset from the lower
                    // one; the local store's learned launch entries win when present.
                    if c.name.is_none() {
                        c.name = game.name;
                    }
                    if c.launch.is_empty() {
                        c.launch = game.launch;
                    }
                    if c.engine.is_none() {
                        c.engine = game.engine;
                    }
                    c.launcher_mediated =
                        merge_mediation(c.launcher_mediated, game.launcher_mediated);
                }
            }
        }
        Ok(combined)
    }

    /// Resolve a combined game to a target, or decline. The single-store rule set
    /// the provider has always applied, now over the combined row.
    fn resolve_game(
        game: &Game,
        app_id: u32,
        notes: &mut ResolutionNotes,
    ) -> Result<Option<Target>, ProviderError> {
        // A launcher-mediated title's launch executable is the publisher launcher,
        // not the socket-holding client (specification section 15.6.1): Steam
        // starts the launcher, which then starts the real game. Resolving the
        // launch entry here would record the launcher as the game, and the
        // synthesized one-stage capture would match a process that exits before the
        // gameplay traffic flows, losing it with nothing counted (P-4, P-9). The
        // database's single launch array cannot name the descendant client, so a
        // mediated row is a decline: the engine rule, the platform walker, and
        // runtime observation are the providers that find the client past the hop.
        if game.launcher_mediated == Some(true) {
            return Ok(None);
        }

        let executables = windows_executables(&game.launch);
        match executables.len() {
            // No usable client executable (a Tier-1-only or engine-only row, or a
            // row whose only launch entries are for other platforms): decline.
            0 => Ok(None),
            // Exactly one client: resolve it.
            1 => {
                let image_name = executables.into_iter().next().expect("length checked");
                // A file name is a valid exe glob; if it somehow is not, the row
                // cannot yield a usable identity, so decline rather than error.
                let Ok(identity) = MatchPredicates::with_exe(&image_name) else {
                    return Ok(None);
                };
                let engine = game.engine.as_ref().and_then(|e| e.name.clone());
                let target =
                    HintTarget::new(app_id, image_name, identity, game.launcher_mediated, engine);
                Ok(Some(Target::new(
                    FidelityTier::HeuristicUnverified,
                    Provenance::new(PROVENANCE_SOURCE.to_string(), None),
                    TargetOrigin::HintDatabase(target),
                )))
            }
            // Several distinct clients the row cannot reduce to one: decline, but
            // record the ambiguity so a not-resolved outcome can explain itself
            // (P-4). Runtime observation disambiguates at runtime.
            n => {
                notes.note_hint_ambiguous(app_id, n);
                Ok(None)
            }
        }
    }

    /// Resolve a locally registered target entry for a Steam anchor, at the
    /// entry's own fidelity, or decline (slice S051).
    ///
    /// The four declines are preserved as fidelity-aware query conditions: an
    /// entry with no usable Windows executable (a sparse or engine-only entry, or
    /// one whose launch entries are all for other platforms) declines, and one
    /// naming more than one distinct Windows executable declines and records the
    /// ambiguity, exactly as the catalog games path does. Only a single-client
    /// entry resolves, so the entry read cannot name a launcher as the game or
    /// guess among clients (P-9). Local stores are consulted first.
    fn resolve_entry(
        &self,
        app_id: u32,
        notes: &mut ResolutionNotes,
    ) -> Result<Option<Target>, ProviderError> {
        let anchor = crate::identifier::steam_anchor(app_id);
        for store in &self.stores {
            let entry = match store.target_by_anchor(&anchor) {
                Ok(Some(entry)) => entry,
                Ok(None) => continue,
                Err(e) => return Err(ProviderError::Hint(e.to_string())),
            };
            let executables = windows_executables(&entry_launch_entries(&entry));
            match executables.len() {
                // Sparse or other-platform-only entry: decline, try the next store.
                0 => continue,
                1 => {
                    let image_name = executables.into_iter().next().expect("length checked");
                    let Ok(identity) = MatchPredicates::with_exe(&image_name) else {
                        continue;
                    };
                    // A user-registered entry names its client directly, so it is
                    // not launcher-mediated; it carries no engine fact.
                    let hint = HintTarget::new(app_id, image_name, identity, Some(false), None);
                    return Ok(Some(Target::new(
                        entry.fidelity,
                        Provenance::new(PROVENANCE_SOURCE.to_string(), None),
                        TargetOrigin::HintDatabase(hint),
                    )));
                }
                n => {
                    // More than one distinct client the entry cannot reduce to one.
                    notes.note_hint_ambiguous(app_id, n);
                    continue;
                }
            }
        }
        Ok(None)
    }
}

impl TargetProvider for HintDatabaseProvider {
    fn precedence(&self) -> Precedence {
        Precedence::HintDatabase
    }

    fn provide(
        &self,
        request: &ResolutionRequest,
        notes: &mut ResolutionNotes,
    ) -> Result<Option<Target>, ProviderError> {
        // No application id on the request: nothing to look up.
        let Some(app_id) = request.steam_app_id() else {
            return Ok(None);
        };

        // Fidelity-ordered store read (slice S051). A locally authored/verified
        // target entry for this anchor resolves at its own fidelity; the catalog
        // games row always answers heuristic-unverified. The higher fidelity wins,
        // so an authored local entry beats a shipped catalog hint for the same
        // title (P-10: fidelity is a column the resolver reads, not a convention).
        let entry_answer = self.resolve_entry(app_id, notes)?;

        // Combine the games row across the stores (the CLI passes [local, catalog])
        // so the mediation fact, wherever it lives, is applied to the learned launch
        // data before resolving; then answer over the combined row. The four
        // declines (sparse, engine-only, launcher-mediated, multi-exe) still hold.
        let catalog_answer = match self.combined_game(app_id)? {
            Some(game) => Self::resolve_game(&game, app_id, notes)?,
            None => None,
        };

        Ok(highest_fidelity(entry_answer, catalog_answer))
    }
}

/// Combine two launcher-mediated facts, letting a known value win over the unknown
/// and a known-mediated fact win over not-mediated, so a title any store marks
/// mediated declines (P-4/P-9).
fn merge_mediation(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), _) | (_, Some(false)) => Some(false),
        _ => None,
    }
}

/// The distinct Windows client executables a target entry's `launch_entries` name.
///
/// Applies the same Windows-only filter, file-name reduction, and case-insensitive
/// dedup [`windows_executables`] applies to a catalog games row, so a stored target
/// reduces to a capturable client exactly as the hint provider reduces a catalog
/// row (P-10). Zero means no usable client; more than one means the entry is
/// ambiguous and a caller must decline rather than guess (P-9). A `capture --target`
/// of a stored target uses this to synthesize a single-client identity, and to
/// refuse an ambiguous or empty one.
pub fn entry_windows_clients(entry: &TargetEntry) -> Vec<String> {
    windows_executables(&entry_launch_entries(entry))
}

/// The distinct exact Windows launch paths a stored row names.
///
/// Unlike [`entry_windows_clients`], this preserves every relative directory
/// component. Process matching needs only the image name; managed launch needs
/// the exact path beneath the stored install root. Both functions share the
/// same parsed launch entries, Windows filter, case folding, and first-seen
/// ordering so they cannot disagree about eligibility.
pub fn entry_windows_launch_paths(entry: &TargetEntry) -> Vec<String> {
    entry_windows_launch_entries(entry)
        .into_iter()
        .map(|launch| launch.executable().to_string())
        .collect()
}

/// The distinct complete Windows launch entries a stored row names.
///
/// Managed launch needs the arguments paired with the exact executable rather
/// than a second lookup after effects begin. Entries that differ by arguments
/// remain distinct and therefore ambiguous; byte-for-byte duplicates collapse.
pub fn entry_windows_launch_entries(entry: &TargetEntry) -> Vec<LaunchEntry> {
    let mut seen_folded = Vec::new();
    let mut out = Vec::new();
    for launch in entry_launch_entries(entry) {
        if !is_windows_entry(&launch) {
            continue;
        }
        let executable = launch.executable();
        if executable.is_empty() {
            continue;
        }
        let folded = (executable.to_lowercase(), launch.arguments.clone());
        if seen_folded.contains(&folded) {
            continue;
        }
        seen_folded.push(folded);
        out.push(launch);
    }
    out
}

/// The distinct client executables a row names for the capture platform.
///
/// Keeps only launch entries applicable to Windows (an `os` filter that is unset
/// or names Windows), reduces each to its file-name component, and returns the set
/// of distinct names, preserving the first-seen casing. Two names are the same when
/// they fold to the same Unicode lowercase, matching the case-insensitive
/// equivalence the `ImagePattern` matcher applies (see `fragcap-profile`'s
/// `glob::folded_eq`), so a name the resulting identity would match is not counted
/// as a second candidate. One executable repeated across arguments, osarch values,
/// or beta branches, or differing only by case (ASCII or non-ASCII), collapses to
/// one entry; a macOS or Linux entry is dropped.
fn windows_executables(launch: &[LaunchEntry]) -> Vec<String> {
    let mut seen_folded: Vec<String> = Vec::new();
    let mut out: Vec<String> = Vec::new();
    for entry in launch {
        if !is_windows_entry(entry) {
            continue;
        }
        let name = file_name(entry.executable());
        if name.is_empty() {
            continue;
        }
        // Unicode-aware lowercasing, not ASCII-only, so `Spel.exe` and its
        // non-ASCII case variants fold together exactly as the matcher folds them.
        let folded = name.to_lowercase();
        if seen_folded.iter().any(|s| s == &folded) {
            continue;
        }
        seen_folded.push(folded);
        out.push(name.to_string());
    }
    out
}

/// Choose the higher-fidelity of two candidate answers, preferring the first on a
/// tie. The store read's ordering key: `Authored > Verified > HeuristicUnverified
/// > Observed` (slice S051). A `None` yields to the other.
fn highest_fidelity(a: Option<Target>, b: Option<Target>) -> Option<Target> {
    match (a, b) {
        (Some(a), Some(b)) => {
            if a.fidelity() >= b.fidelity() {
                Some(a)
            } else {
                Some(b)
            }
        }
        (Some(a), None) => Some(a),
        (None, b) => b,
    }
}

/// The launch entries a target entry carries, parsed from its `launch_entries`
/// JSON (an array of objects with an `executable` and an optional `os`). Reuses
/// the model's [`LaunchEntry`] so the same Windows filter and dedup apply to an
/// entry as to a catalog games row. A malformed or absent value yields none.
fn entry_launch_entries(entry: &TargetEntry) -> Vec<LaunchEntry> {
    let mut out = Vec::new();
    if let Some(serde_json::Value::Array(items)) = &entry.launch_entries {
        for item in items {
            let Some(exe) = item.get("executable").and_then(|v| v.as_str()) else {
                continue;
            };
            let Ok(mut le) = LaunchEntry::new(exe) else {
                continue;
            };
            le.os = item.get("os").and_then(|v| v.as_str()).map(str::to_string);
            le.arguments = item
                .get("arguments")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            out.push(le);
        }
    }
    out
}

/// Whether a launch entry applies to the Windows capture platform: its `os` filter
/// is unset or names Windows (case-insensitively).
fn is_windows_entry(entry: &LaunchEntry) -> bool {
    match &entry.os {
        None => true,
        Some(os) => os.eq_ignore_ascii_case("windows"),
    }
}

/// The file-name component of an executable string, splitting on both `/` and `\`
/// so a Steam launch path resolves the same on any host the tests run on.
fn file_name(executable: &str) -> &str {
    let after_slash = executable.rsplit(['/', '\\']).next().unwrap_or(executable);
    after_slash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{ClassificationSource, TargetClassification};
    use crate::model::{Engine, EngineConfidence, EngineSource, Game};

    fn store_with(game: Game) -> Store {
        let mut store = Store::open_in_memory().expect("in-memory store");
        store.upsert_game(&game).expect("upsert");
        store
    }

    fn launch(executable: &str, os: Option<&str>) -> LaunchEntry {
        let mut entry = LaunchEntry::new(executable).expect("non-empty");
        entry.os = os.map(|s| s.to_string());
        entry
    }

    /// Resolve an app id against a store through the real provider. Consumes the
    /// store (the provider owns it, as in production).
    fn resolve(store: Store, app_id: u32) -> Option<Target> {
        let provider = HintDatabaseProvider::new(store);
        let search = fragcap_profile::SearchPath::new();
        let bundled = fragcap_profile::BundledSet::empty();
        let req =
            ResolutionRequest::for_reference("unused", &search, &bundled).with_steam_app_id(app_id);
        provider
            .provide(&req, &mut ResolutionNotes::default())
            .expect("no hard error")
    }

    #[test]
    fn a_row_with_one_executable_resolves_at_heuristic_unverified() {
        let mut game = Game::new(730);
        game.name = Some("CS2".to_string());
        // Not launcher-mediated: the launch executable is the client itself, so it
        // resolves. The carried fact is still surfaced on the answer.
        game.launcher_mediated = Some(false);
        game.engine = Some(Engine {
            name: Some("Source 2".to_string()),
            source: EngineSource::Pcgamingwiki,
            confidence: EngineConfidence::High,
        });
        game.launch = vec![launch("cs2.exe", Some("windows"))];

        let target = resolve(store_with(game), 730).expect("a hint answer");
        assert_eq!(target.fidelity(), FidelityTier::HeuristicUnverified);
        assert_eq!(target.provenance().source(), "hint-db");
        assert!(target.profile().is_none());
        match target.origin() {
            TargetOrigin::HintDatabase(t) => {
                assert_eq!(t.app_id(), 730);
                assert_eq!(t.image_name(), "cs2.exe");
                assert!(t.identity().exe().is_some());
                assert_eq!(t.launcher_mediated(), Some(false));
                assert_eq!(t.engine(), Some("Source 2"));
            }
            other => panic!("expected a hint-database origin, got {other:?}"),
        }
    }

    #[test]
    fn one_executable_repeated_across_configs_is_one_candidate() {
        let mut game = Game::new(1);
        // Same executable, different os/osarch and a case and path variant: one
        // candidate, not several.
        game.launch = vec![
            launch("Game.exe", None),
            launch("game.exe", Some("windows")),
            {
                let mut e = launch("bin/Game.exe", Some("Windows"));
                e.osarch = Some("64".to_string());
                e
            },
        ];

        let target = resolve(store_with(game), 1).expect("one candidate resolves");
        match target.origin() {
            TargetOrigin::HintDatabase(t) => assert_eq!(t.image_name(), "Game.exe"),
            other => panic!("expected a hint origin, got {other:?}"),
        }
    }

    #[test]
    fn managed_launch_keeps_arguments_paired_with_the_exact_path() {
        let mut first = launch("bin/game.exe", Some("windows"));
        first.arguments = Some("--mode one".to_string());
        let mut duplicate = first.clone();
        duplicate.os = Some("Windows".to_string());
        let mut distinct = launch("BIN/GAME.EXE", Some("windows"));
        distinct.arguments = Some("--mode two".to_string());
        let entry = TargetEntry {
            id: None,
            stable_id: 1,
            handle: "managed".to_string(),
            name: "Managed".to_string(),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::User,
            fidelity: FidelityTier::Authored,
            provenance: None,
            anchor: None,
            launch_entries: Some(serde_json::json!([
                {
                    "executable": first.executable(),
                    "os": first.os,
                    "arguments": first.arguments,
                },
                {
                    "executable": duplicate.executable(),
                    "os": duplicate.os,
                    "arguments": duplicate.arguments,
                },
                {
                    "executable": distinct.executable(),
                    "os": distinct.os,
                    "arguments": distinct.arguments,
                }
            ])),
            install_root: None,
            evidence: None,
            detection_scan: None,
            folder_name: None,
            executable_hint: None,
        };

        let launches = entry_windows_launch_entries(&entry);
        assert_eq!(launches.len(), 2);
        assert_eq!(launches[0].arguments.as_deref(), Some("--mode one"));
        assert_eq!(launches[1].arguments.as_deref(), Some("--mode two"));
    }

    #[test]
    fn a_launcher_mediated_row_declines() {
        // A launcher-mediated title's launch executable is the publisher launcher,
        // not the socket-holding client. Resolving it would record the launcher as
        // the game and lose the gameplay traffic, so the provider declines and lets
        // the engine rule, walker, and runtime observation find the real client
        // past the launcher hop (P-4, P-9).
        let mut game = Game::new(306130);
        game.name = Some("The Elder Scrolls Online".to_string());
        game.launcher_mediated = Some(true);
        game.launch = vec![launch("Launcher.exe", Some("windows"))];
        assert!(
            resolve(store_with(game), 306130).is_none(),
            "a launcher-mediated row names the launcher, not the client"
        );
    }

    #[test]
    fn a_row_with_unknown_mediation_resolves() {
        // Mediation unknown (None) is not the same as known-mediated: the launch
        // executable is the best available identity, resolved heuristic-unverified
        // and overridable by observation.
        let mut game = Game::new(400);
        game.launch = vec![launch("game.exe", Some("windows"))];
        assert!(
            resolve(store_with(game), 400).is_some(),
            "an unknown-mediation row still resolves its single client"
        );
    }

    #[test]
    fn a_case_variant_non_ascii_executable_is_one_candidate() {
        // Two entries spelling the same non-ASCII name with different casing fold
        // to one candidate under Unicode lowercasing, matching the matcher's
        // case-insensitive equivalence, so they are not a false ambiguity. ASCII-
        // only folding (the pre-fix behavior) would leave the accented capital
        // distinct and wrongly report an ambiguity.
        let mut game = Game::new(2);
        game.launch = vec![
            launch("Sp\u{00EB}l.exe", Some("windows")),
            launch("SP\u{00CB}L.EXE", Some("windows")),
        ];
        let target = resolve(store_with(game), 2).expect("one folded candidate resolves");
        match target.origin() {
            TargetOrigin::HintDatabase(t) => assert_eq!(t.image_name(), "Sp\u{00EB}l.exe"),
            other => panic!("expected a hint origin, got {other:?}"),
        }
    }

    #[test]
    fn a_tier_one_only_row_declines() {
        let mut game = Game::new(42);
        game.name = Some("Sparse".to_string());
        assert!(
            resolve(store_with(game), 42).is_none(),
            "a catalog-only row is no answer"
        );
    }

    #[test]
    fn an_engine_only_row_with_no_launch_declines() {
        let mut game = Game::new(43);
        game.engine = Some(Engine {
            name: Some("Unreal".to_string()),
            source: EngineSource::Pcgamingwiki,
            confidence: EngineConfidence::Medium,
        });
        assert!(
            resolve(store_with(game), 43).is_none(),
            "an engine-only row is no answer"
        );
    }

    #[test]
    fn a_missing_row_declines() {
        let store = Store::open_in_memory().expect("in-memory store");
        assert!(resolve(store, 999).is_none(), "an absent row is no answer");
    }

    #[test]
    fn layers_stores_local_first() {
        // Two stores in [local, catalog] order (slice S050): the user's learned
        // data is consulted before the shipped catalog seed.
        let mut catalog = Store::open_in_memory().expect("catalog");
        let mut c100 = Game::new(100);
        c100.launch = vec![launch("catalogclient.exe", Some("windows"))];
        catalog.upsert_game(&c100).expect("upsert catalog 100");
        let mut c300 = Game::new(300);
        c300.launch = vec![launch("catalogonly.exe", Some("windows"))];
        catalog.upsert_game(&c300).expect("upsert catalog 300");

        let mut local = Store::open_in_memory().expect("local");
        let mut l100 = Game::new(100);
        l100.launch = vec![launch("localclient.exe", Some("windows"))];
        local.upsert_game(&l100).expect("upsert local 100");
        let mut l200 = Game::new(200);
        l200.launch = vec![launch("localonly.exe", Some("windows"))];
        local.upsert_game(&l200).expect("upsert local 200");

        let provider = HintDatabaseProvider::new_layered(vec![local, catalog]);
        let search = fragcap_profile::SearchPath::new();
        let bundled = fragcap_profile::BundledSet::empty();
        let image = |app_id: u32| -> Option<String> {
            let req =
                ResolutionRequest::for_reference("x", &search, &bundled).with_steam_app_id(app_id);
            match provider.provide(&req, &mut ResolutionNotes::default()) {
                Ok(Some(t)) => match t.origin() {
                    TargetOrigin::HintDatabase(h) => Some(h.image_name().to_string()),
                    _ => None,
                },
                _ => None,
            }
        };

        // Present in both: local wins.
        assert_eq!(image(100).as_deref(), Some("localclient.exe"));
        // Present only in local.
        assert_eq!(image(200).as_deref(), Some("localonly.exe"));
        // Present only in catalog: falls through to the second store.
        assert_eq!(image(300).as_deref(), Some("catalogonly.exe"));
        // Absent everywhere.
        assert_eq!(image(999), None);
    }

    #[test]
    fn a_mediated_catalog_row_declines_even_with_learned_local_launch() {
        // The regression the split could introduce: the local launch-data
        // accumulation inserts an appid-only row with the learned launcher
        // executable and mediation unset, while the catalog carries
        // launcher_mediated = true. Reading the stores independently (local first)
        // would resolve the launcher as the client and lose gameplay traffic. The
        // combined row must keep the catalog's mediation fact and decline (P-4/P-9).
        let mut catalog = Store::open_in_memory().expect("catalog");
        let mut c = Game::new(306130);
        c.name = Some("The Elder Scrolls Online".to_string());
        c.launcher_mediated = Some(true);
        c.launch = vec![launch("Launcher.exe", Some("windows"))];
        catalog.upsert_game(&c).expect("upsert catalog");

        let mut local = Store::open_in_memory().expect("local");
        let mut l = Game::new(306130);
        // Mediation unset (as merge_launch leaves it), the launcher exe learned
        // from appinfo (which is what a mediated title's launch config names).
        l.launch = vec![launch("Launcher.exe", Some("windows"))];
        local.upsert_game(&l).expect("upsert local");

        let provider = HintDatabaseProvider::new_layered(vec![local, catalog]);
        let search = fragcap_profile::SearchPath::new();
        let bundled = fragcap_profile::BundledSet::empty();
        let req =
            ResolutionRequest::for_reference("x", &search, &bundled).with_steam_app_id(306130);
        let result = provider
            .provide(&req, &mut ResolutionNotes::default())
            .expect("no error");
        assert!(
            result.is_none(),
            "a launcher-mediated title must decline even with learned local launch data"
        );
    }

    #[test]
    fn a_request_with_no_app_id_declines() {
        let mut game = Game::new(7);
        game.launch = vec![launch("g.exe", Some("windows"))];
        let provider = HintDatabaseProvider::new(store_with(game));
        let search = fragcap_profile::SearchPath::new();
        let bundled = fragcap_profile::BundledSet::empty();
        // A reference request with no app id attached.
        let req = ResolutionRequest::for_reference("g", &search, &bundled);
        assert!(provider
            .provide(&req, &mut ResolutionNotes::default())
            .expect("no hard error")
            .is_none());
    }

    #[test]
    fn a_macos_only_launch_entry_declines() {
        let mut game = Game::new(8);
        game.launch = vec![launch("game.app", Some("macos"))];
        assert!(
            resolve(store_with(game), 8).is_none(),
            "a macOS-only entry is not a Windows client"
        );
    }

    #[test]
    fn several_distinct_executables_decline() {
        let mut game = Game::new(480);
        game.launch = vec![
            launch("client.exe", Some("windows")),
            launch("editor.exe", Some("windows")),
        ];
        // The decline is asserted here; that it records a note surfaced through
        // Unresolved is asserted end to end in tests/hint_cascade.rs.
        assert!(
            resolve(store_with(game), 480).is_none(),
            "an ambiguous row is a decline, not a pick"
        );
    }

    #[test]
    fn the_windows_filter_and_distinct_reduction_are_pure() {
        // Two distinct Windows executables, plus a macOS entry that is ignored.
        let launch = vec![
            super::LaunchEntry::new("a.exe").unwrap(),
            {
                let mut e = super::LaunchEntry::new("b.exe").unwrap();
                e.os = Some("linux".to_string());
                e
            },
            {
                let mut e = super::LaunchEntry::new("A.EXE").unwrap();
                e.os = Some("windows".to_string());
                e
            },
        ];
        // "a.exe" (os unset) and "A.EXE" (windows) are one case-insensitive name;
        // "b.exe" (linux) is dropped. So exactly one distinct Windows executable.
        assert_eq!(windows_executables(&launch), vec!["a.exe".to_string()]);
    }
}
