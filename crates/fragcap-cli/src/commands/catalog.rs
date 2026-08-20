// SPDX-License-Identifier: Apache-2.0

//! `catalog`: maintain the shipped, disposable catalog store (`catalog.db`).
//!
//! Every subcommand operates on the catalog store the maintainer seeds and any
//! user refreshes: `import` loads a JSON seed document, `export` projects the store
//! to schema-conformant JSON, `seed`/`seed-engine`/`seed-signatures` fill the
//! catalog, engine, and detection-signature tiers, and `update` fetches the current
//! catalog. The store defaults to the per-user location; user-owned target
//! management lives under `targets`.
//!
//! A malformed seed fails without leaving a store behind: the import is
//! transactional, and a store this command freshly created for a failed import is
//! removed rather than left as a stray empty file (P-4).

use std::io::Write;
use std::path::{Path, PathBuf};

use fragcap::targets::{
    export, import, seed_catalog, seed_engine, CorpusGate, FixtureCatalog, FixtureEngineFeed,
    SeedSummary, Store,
};

use crate::cli::{CatalogArgs, CatalogCommand, SeedTierArg, TargetsSeedArgs};
use crate::exit::{CliError, Exit};

/// Run the `catalog` command, writing results to `out`.
pub fn run(args: &CatalogArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    match &args.command {
        CatalogCommand::Import { seed, db } => import_cmd(seed, &resolve_store(db.as_ref())?, out),
        CatalogCommand::Export { db } => export_cmd(&resolve_store(db.as_ref())?, out),
        CatalogCommand::Seed(args) => seed(args, out),
    }
}

/// Resolve the catalog store a `catalog` subcommand should act on.
///
/// The precedence is the one every other command already uses: an explicit
/// `--db`, else `FRAGCAP_CATALOG_DB`, else the per-user default, which is
/// bootstrapped on first use from the template shipped beside the executable.
/// Slice S058 (issue #157) established that a store path is an override rather
/// than a requirement and applied it to `targets`; its FR-005 scoped `catalog`
/// out, and nothing picked it up, so these commands went on demanding a path to
/// a component fragcap installs and manages (issue #179).
///
/// The asymmetry in [`ensure_catalog_store`] is inherited deliberately: a
/// defaulted store is created, with parents, while an operator-named path is
/// opened exactly as given and never conjured on their behalf.
fn resolve_store(flag: Option<&PathBuf>) -> Result<PathBuf, CliError> {
    crate::commands::target_resolve::ensure_catalog_store(flag.map(PathBuf::as_path))
        .map_err(CliError::failure)?
        .ok_or_else(|| {
            CliError::failure(
                "no catalog store could be resolved: pass --db, set FRAGCAP_CATALOG_DB, \
                 or run on a machine with a per-user application data directory",
            )
        })
}

/// Run `catalog import`: load a JSON seed document into the store, creating it if
/// needed. A malformed seed leaves no store behind (the import is transactional).
fn import_cmd(seed: &Path, db: &Path, out: &mut dyn Write) -> Result<Exit, CliError> {
    let text = std::fs::read_to_string(seed)
        .map_err(|e| CliError::failure(format!("cannot read seed {}: {e}", seed.display())))?;

    let existed = db.exists();
    let mut store = Store::open(db).map_err(|e| CliError::failure(e.to_string()))?;

    match import(&mut store, &text) {
        Ok(summary) => {
            let _ = writeln!(
                out,
                "imported {} games into {}",
                summary.imported,
                db.display()
            );
            Ok(Exit::SUCCESS)
        }
        Err(e) => {
            // Do not leave a store we just created for a seed that did not load; a
            // pre-existing store is untouched (the import is atomic).
            drop(store);
            if !existed {
                let _ = std::fs::remove_file(db);
            }
            Err(CliError::failure(e.to_string()))
        }
    }
}

/// Run `catalog export`: project the store to schema-conformant JSON on stdout.
fn export_cmd(db: &Path, out: &mut dyn Write) -> Result<Exit, CliError> {
    if !db.exists() {
        return Err(CliError::failure(format!(
            "store not found: {}",
            db.display()
        )));
    }
    let store = Store::open(db).map_err(|e| CliError::failure(e.to_string()))?;
    let text = export(&store).map_err(|e| CliError::failure(e.to_string()))?;
    let _ = write!(out, "{text}");
    Ok(Exit::SUCCESS)
}

/// A Unix-epoch-seconds stamp for the seed state's last-run field. Informational;
/// avoids a date-formatting dependency.
fn now_string() -> Option<String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs().to_string())
}

/// Run `catalog seed`: fill one tier, or every tier that has a source.
///
/// One verb replacing three (issue #180). The split was accretion: `seed`,
/// `seed-engine`, and `seed-signatures` each arrived with the slice that needed
/// it, no recorded decision defended three over one, and the scheme's own next
/// step was a fourth top-level verb for the launch tier that `SeedTier::Launch`
/// already names.
///
/// Every skipped tier is reported by name with its reason. A merged verb that
/// quietly filled fewer tiers than the operator expected would be the
/// configuration-side form of an uncounted discard (P-4).
fn seed(args: &TargetsSeedArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    let requested = requested_tiers(args)?;
    let db = resolve_store(args.db.as_ref())?;
    let mut store = Store::open(&db).map_err(|e| CliError::failure(e.to_string()))?;

    let mut filled = 0usize;
    for tier in requested {
        match tier {
            SeedTierArg::Signature => {
                let count = fragcap::targets::seed_bundled(&mut store)
                    .map_err(|e| CliError::failure(e.to_string()))?;
                let _ = writeln!(out, "signature tier: seeded {count} detection signature(s)");
                filled += 1;
            }
            SeedTierArg::Catalog => match catalog_summary(args, &mut store)? {
                Some(summary) => {
                    report(out, "title tier", &summary);
                    filled += 1;
                }
                None => skipped(out, "title tier", "no source; pass --from or --steam"),
            },
            SeedTierArg::Engine => match engine_summary(args, &mut store)? {
                Some(summary) => {
                    report(out, "engine tier", &summary);
                    filled += 1;
                }
                None => skipped(
                    out,
                    "engine tier",
                    "no source; pass --from or --pcgamingwiki",
                ),
            },
            SeedTierArg::Launch => skipped(
                out,
                "launch tier",
                "no seeder exists; launch data accumulates from captures",
            ),
        }
    }

    let _ = writeln!(out, "seeded {} ({filled} tier(s) filled)", db.display());
    Ok(Exit::SUCCESS)
}

/// The tiers this invocation should attempt.
///
/// An explicit `--tier` is taken as given. With none, every tier is attempted
/// and the ones without a source report themselves skipped, which is what makes
/// a bare `catalog seed` useful (it fills the signature table) without making it
/// silently partial.
///
/// `--from` requires exactly one tier. Both offline documents are bare JSON
/// arrays with no discriminator, so a merged command cannot tell them apart, and
/// a guess that picks the wrong tier writes the wrong columns with no error
/// (P-9). Refusing is the only honest answer.
fn requested_tiers(args: &TargetsSeedArgs) -> Result<Vec<SeedTierArg>, CliError> {
    if args.from.is_some() && args.tier.len() != 1 {
        return Err(CliError::usage(
            "--from fills one tier and the document does not say which; pass exactly one \
             --tier (catalog, launch, engine, or signature)",
        ));
    }
    if args.tier.is_empty() {
        return Ok(vec![
            SeedTierArg::Signature,
            SeedTierArg::Catalog,
            SeedTierArg::Engine,
            SeedTierArg::Launch,
        ]);
    }
    Ok(args.tier.clone())
}

/// Seed the title tier, or `None` when no source names it.
fn catalog_summary(
    args: &TargetsSeedArgs,
    store: &mut Store,
) -> Result<Option<SeedSummary>, CliError> {
    let gate = CorpusGate::new(args.min_reviews);
    let now = now_string();
    if let Some(from) = &args.from {
        let text = std::fs::read_to_string(from).map_err(|e| {
            CliError::failure(format!("cannot read catalog {}: {e}", from.display()))
        })?;
        let source =
            FixtureCatalog::from_json(&text).map_err(|e| CliError::failure(e.to_string()))?;
        return Ok(Some(
            seed_catalog(store, &source, &gate, now)
                .map_err(|e| CliError::failure(e.to_string()))?,
        ));
    }
    #[cfg(feature = "net")]
    if args.steam {
        let source = fragcap::targets::HttpCatalog::new();
        return Ok(Some(
            seed_catalog(store, &source, &gate, now)
                .map_err(|e| CliError::failure(e.to_string()))?,
        ));
    }
    Ok(None)
}

/// Seed the engine tier, or `None` when no source names it.
fn engine_summary(
    args: &TargetsSeedArgs,
    store: &mut Store,
) -> Result<Option<SeedSummary>, CliError> {
    let now = now_string();
    if let Some(from) = &args.from {
        let text = std::fs::read_to_string(from).map_err(|e| {
            CliError::failure(format!(
                "cannot read engine document {}: {e}",
                from.display()
            ))
        })?;
        let source =
            FixtureEngineFeed::from_json(&text).map_err(|e| CliError::failure(e.to_string()))?;
        return Ok(Some(
            seed_engine(store, &source, now).map_err(|e| CliError::failure(e.to_string()))?,
        ));
    }
    #[cfg(feature = "net")]
    if args.pcgamingwiki {
        let source = fragcap::targets::HttpEngineFeed::new();
        return Ok(Some(
            seed_engine(store, &source, now).map_err(|e| CliError::failure(e.to_string()))?,
        ));
    }
    Ok(None)
}

/// One tier's counters, in the vocabulary the seed summary has always used.
/// Unchanged by the merge (FR-016): the counts still reconcile as fetched equals
/// written plus excluded plus duplicates plus failed.
fn report(out: &mut dyn Write, tier: &str, summary: &SeedSummary) {
    let _ = writeln!(
        out,
        "{tier}: fetched {} written {} excluded {} duplicates {} failed {}",
        summary.fetched, summary.written, summary.excluded, summary.duplicates, summary.failed
    );
}

/// A tier that was not filled, named with the reason. Never silent (P-4).
fn skipped(out: &mut dyn Write, tier: &str, why: &str) {
    let _ = writeln!(out, "{tier}: skipped, {why}");
}

/// Create the default catalog store and fill its signature table, for the
/// `doctor --fix` InitializeCatalog action.
///
/// Replaces the network fetch that action used to perform. The check it answers
/// fires when the catalog store is **absent**, not when it is empty
/// (`doctor::checks::catalog_store`), and an absent store is exactly what the
/// first-run bootstrap creates and what the compiled-in signature document
/// fills. So the condition never needed a network at all: the previous action
/// was net-gated, always degraded in a shipped build, and its degraded guidance
/// told the user to rebuild fragcap from source with a Cargo flag, which asks
/// someone holding a binary to obtain a source checkout and a C toolchain
/// (issue #175). This does the same work offline, so it cannot degrade.
pub(crate) fn initialize_default(out: &mut dyn Write) -> Result<Exit, CliError> {
    let db = crate::commands::target_resolve::ensure_catalog_store(None)
        .map_err(CliError::failure)?
        .ok_or_else(|| CliError::failure("the catalog store path could not be determined"))?;
    let mut store = Store::open(&db).map_err(|e| CliError::failure(e.to_string()))?;
    let count =
        fragcap::targets::seed_bundled(&mut store).map_err(|e| CliError::failure(e.to_string()))?;
    let _ = writeln!(
        out,
        "initialized {} with {} detection signature(s)",
        db.display(),
        count
    );
    Ok(Exit::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{CatalogArgs, CatalogCommand};

    const SEED: &str = r#"{
  "schema": 1,
  "kind": "export",
  "fidelity": "heuristic-unverified",
  "provenance": { "source": "hint-db" },
  "records": [
    {
      "fidelity": "heuristic-unverified",
      "provenance": { "source": "hint-db" },
      "game": { "app_id": "306130", "platform": "steam", "name": "The Elder Scrolls Online" },
      "launcher_mediated": true,
      "launch": [ { "executable": "eso64.exe", "os": "windows" } ]
    }
  ]
}"#;

    const BAD_SEED: &str = r#"{
  "schema": 1,
  "kind": "export",
  "fidelity": "heuristic-unverified",
  "provenance": { "source": "hint-db" },
  "records": [
    {
      "fidelity": "heuristic-unverified",
      "provenance": { "source": "hint-db" },
      "game": { "app_id": "306130", "platform": "steam" },
      "launch": [ { "os": "windows" } ]
    }
  ]
}"#;

    fn import_args(seed: &std::path::Path, db: &std::path::Path) -> CatalogArgs {
        CatalogArgs {
            command: CatalogCommand::Import {
                seed: seed.to_path_buf(),
                db: Some(db.to_path_buf()),
            },
        }
    }

    fn export_args(db: &std::path::Path) -> CatalogArgs {
        CatalogArgs {
            command: CatalogCommand::Export {
                db: Some(db.to_path_buf()),
            },
        }
    }

    const CATALOG: &str = r#"[
      { "appid": 570, "name": "Dota 2", "classification": "game", "review_count": 2000000 },
      { "appid": 440, "name": "Below Threshold", "classification": "game", "review_count": 5 },
      { "appid": 700, "name": "A Tool", "classification": "other", "review_count": 9999 }
    ]"#;

    fn seed_args(from: &std::path::Path, db: &std::path::Path) -> CatalogArgs {
        CatalogArgs {
            command: CatalogCommand::Seed(TargetsSeedArgs {
                tier: vec![SeedTierArg::Catalog],
                from: Some(from.to_path_buf()),
                #[cfg(feature = "net")]
                steam: false,
                #[cfg(feature = "net")]
                pcgamingwiki: false,
                db: Some(db.to_path_buf()),
                min_reviews: 100,
            }),
        }
    }

    const ENGINES: &str = r#"[
      { "appid": 570, "engine": "Source 2", "confidence": "confirmed" },
      { "appid": 730, "engine": ["Unity", "Custom"] },
      { "appid": 440, "engine": "" }
    ]"#;

    fn seed_engine_args(from: &std::path::Path, db: &std::path::Path) -> CatalogArgs {
        CatalogArgs {
            command: CatalogCommand::Seed(TargetsSeedArgs {
                tier: vec![SeedTierArg::Engine],
                from: Some(from.to_path_buf()),
                #[cfg(feature = "net")]
                steam: false,
                #[cfg(feature = "net")]
                pcgamingwiki: false,
                db: Some(db.to_path_buf()),
                min_reviews: 100,
            }),
        }
    }

    #[test]
    fn seed_from_a_fixture_then_export_round_trips_to_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        let catalog = dir.path().join("catalog.json");
        let db = dir.path().join("catalog.db");
        std::fs::write(&catalog, CATALOG).unwrap();

        let mut out: Vec<u8> = Vec::new();
        let exit = run(&seed_args(&catalog, &db), &mut out).expect("seed succeeds");
        assert_eq!(exit, Exit::SUCCESS);
        let report = String::from_utf8(out).unwrap();
        assert!(report.contains("written 1"), "{report}");
        assert!(report.contains("excluded 2"), "{report}");

        let mut out: Vec<u8> = Vec::new();
        run(&export_args(&db), &mut out).expect("export succeeds");
        let text = String::from_utf8(out).unwrap();
        assert!(
            fragcap::profile::validate_json(&text).is_valid(),
            "seeded store must export valid JSON: {text}"
        );
        assert!(text.contains("570"));
    }

    #[test]
    fn seed_from_a_missing_file_fails() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("absent.json");
        let db = dir.path().join("catalog.db");
        let mut out: Vec<u8> = Vec::new();
        assert!(run(&seed_args(&missing, &db), &mut out).is_err());
    }

    #[test]
    fn seed_engine_from_a_fixture_then_export_round_trips_to_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        let engines = dir.path().join("engines.json");
        let db = dir.path().join("catalog.db");
        std::fs::write(&engines, ENGINES).unwrap();

        let mut out: Vec<u8> = Vec::new();
        let exit = run(&seed_engine_args(&engines, &db), &mut out).expect("seed-engine succeeds");
        assert_eq!(exit, Exit::SUCCESS);
        let report = String::from_utf8(out).unwrap();
        assert!(report.contains("written 1"), "{report}");
        assert!(report.contains("excluded 2"), "{report}");

        let mut out: Vec<u8> = Vec::new();
        run(&export_args(&db), &mut out).expect("export succeeds");
        let text = String::from_utf8(out).unwrap();
        assert!(
            fragcap::profile::validate_json(&text).is_valid(),
            "seeded store must export valid JSON: {text}"
        );
        assert!(text.contains("\"pcgamingwiki\""), "{text}");
    }

    #[test]
    fn seed_engine_from_a_missing_file_fails() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("absent.json");
        let db = dir.path().join("catalog.db");
        let mut out: Vec<u8> = Vec::new();
        assert!(run(&seed_engine_args(&missing, &db), &mut out).is_err());
    }

    #[test]
    fn import_then_export_round_trips_to_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        let seed = dir.path().join("seed.json");
        let db = dir.path().join("catalog.db");
        std::fs::write(&seed, SEED).unwrap();

        let mut out: Vec<u8> = Vec::new();
        let exit = run(&import_args(&seed, &db), &mut out).expect("import succeeds");
        assert_eq!(exit, Exit::SUCCESS);
        assert!(String::from_utf8(out).unwrap().contains("imported 1 games"));

        let mut out: Vec<u8> = Vec::new();
        let exit = run(&export_args(&db), &mut out).expect("export succeeds");
        assert_eq!(exit, Exit::SUCCESS);
        let text = String::from_utf8(out).unwrap();
        assert!(
            fragcap::profile::validate_json(&text).is_valid(),
            "exported JSON must validate: {text}"
        );
        assert!(text.contains("306130"));
        assert!(text.contains("\"launcher_mediated\": true"));
    }

    #[test]
    fn reimport_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let seed = dir.path().join("seed.json");
        let db = dir.path().join("catalog.db");
        std::fs::write(&seed, SEED).unwrap();

        let mut sink: Vec<u8> = Vec::new();
        run(&import_args(&seed, &db), &mut sink).unwrap();
        run(&import_args(&seed, &db), &mut sink).unwrap();

        let mut out: Vec<u8> = Vec::new();
        run(&export_args(&db), &mut out).unwrap();
        let doc: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(
            doc["records"].as_array().unwrap().len(),
            1,
            "re-import must replace, not duplicate"
        );
    }

    #[test]
    fn a_malformed_seed_fails_and_leaves_no_store() {
        let dir = tempfile::tempdir().unwrap();
        let seed = dir.path().join("bad.json");
        let db = dir.path().join("catalog.db");
        std::fs::write(&seed, BAD_SEED).unwrap();

        let mut out: Vec<u8> = Vec::new();
        assert!(
            run(&import_args(&seed, &db), &mut out).is_err(),
            "a malformed seed must fail the import"
        );
        assert!(!db.exists(), "a failed import must leave no store behind");
    }

    #[test]
    fn export_of_a_missing_store_is_a_failure() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("absent.db");
        let mut out: Vec<u8> = Vec::new();
        assert!(run(&export_args(&db), &mut out).is_err());
    }
}
