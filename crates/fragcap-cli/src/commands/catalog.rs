// SPDX-License-Identifier: Apache-2.0

//! `catalog`: maintain the shipped, disposable catalog store (`catalog.db`).
//!
//! Every subcommand operates on the catalog store the maintainer seeds and any
//! user refreshes: `import` loads a JSON seed document, `export` projects the store
//! to schema-conformant JSON, `seed`/`seed-engine`/`seed-signatures` fill the
//! catalog, engine, and detection-signature tiers, and `update` fetches the current
//! published catalog. The store is named explicitly with `--db`; user-owned target
//! management lives under `targets`.
//!
//! A malformed seed fails without leaving a store behind: the import is
//! transactional, and a store this command freshly created for a failed import is
//! removed rather than left as a stray empty file (P-4).

use std::io::Write;
use std::path::Path;

use fragcap::targets::{
    export, import, seed_catalog, seed_engine, CorpusGate, FixtureCatalog, FixtureEngineFeed, Store,
};

use crate::cli::{CatalogArgs, CatalogCommand, TargetsSeedArgs, TargetsSeedEngineArgs};
use crate::exit::{CliError, Exit};

/// Run the `catalog` command, writing results to `out`.
pub fn run(args: &CatalogArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    match &args.command {
        CatalogCommand::Import { seed, db } => import_cmd(seed, db, out),
        CatalogCommand::Export { db } => export_cmd(db, out),
        CatalogCommand::Seed(args) => seed(args, out),
        CatalogCommand::SeedEngine(args) => seed_engine_cmd(args, out),
        CatalogCommand::SeedSignatures { db } => seed_signatures_cmd(db, out),
        CatalogCommand::Update { db } => update(db, out),
    }
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

/// Run `catalog seed`: fill the catalog tier from a fixture (offline) or, under the
/// `net` feature with `--steam`, from the live catalog.
fn seed(args: &TargetsSeedArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    let gate = CorpusGate::new(args.min_reviews);
    let mut store = Store::open(&args.db).map_err(|e| CliError::failure(e.to_string()))?;
    let now = now_string();

    let summary = if let Some(from) = &args.from {
        let text = std::fs::read_to_string(from).map_err(|e| {
            CliError::failure(format!("cannot read catalog {}: {e}", from.display()))
        })?;
        let source =
            FixtureCatalog::from_json(&text).map_err(|e| CliError::failure(e.to_string()))?;
        seed_catalog(&mut store, &source, &gate, now)
            .map_err(|e| CliError::failure(e.to_string()))?
    } else {
        #[cfg(feature = "net")]
        {
            if args.steam {
                let source = fragcap::targets::HttpCatalog::new();
                seed_catalog(&mut store, &source, &gate, now)
                    .map_err(|e| CliError::failure(e.to_string()))?
            } else {
                return Err(CliError::usage("specify --from <file> or --steam"));
            }
        }
        #[cfg(not(feature = "net"))]
        {
            return Err(CliError::usage(
                "specify --from <file> (live --steam seeding needs the `net` feature)",
            ));
        }
    };

    let _ = writeln!(
        out,
        "seeded {}: fetched {} written {} excluded {} duplicates {} failed {}",
        args.db.display(),
        summary.fetched,
        summary.written,
        summary.excluded,
        summary.duplicates,
        summary.failed
    );
    Ok(Exit::SUCCESS)
}

/// Run `catalog seed-signatures`: fill the detection signature table from the
/// bundled Appendix B document (slice S053). Offline and idempotent.
fn seed_signatures_cmd(db: &Path, out: &mut dyn Write) -> Result<Exit, CliError> {
    let mut store = Store::open(db).map_err(|e| CliError::failure(e.to_string()))?;
    let count =
        fragcap::targets::seed_bundled(&mut store).map_err(|e| CliError::failure(e.to_string()))?;
    let _ = writeln!(
        out,
        "seeded {} detection signatures into {}",
        count,
        db.display()
    );
    Ok(Exit::SUCCESS)
}

/// Run `catalog seed-engine`: fill the engine tier from a fixture (offline) or,
/// under the `net` feature with `--pcgamingwiki`, from the live PCGamingWiki API.
fn seed_engine_cmd(args: &TargetsSeedEngineArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    let mut store = Store::open(&args.db).map_err(|e| CliError::failure(e.to_string()))?;
    let now = now_string();

    let summary = if let Some(from) = &args.from {
        let text = std::fs::read_to_string(from).map_err(|e| {
            CliError::failure(format!(
                "cannot read engine document {}: {e}",
                from.display()
            ))
        })?;
        let source =
            FixtureEngineFeed::from_json(&text).map_err(|e| CliError::failure(e.to_string()))?;
        seed_engine(&mut store, &source, now).map_err(|e| CliError::failure(e.to_string()))?
    } else {
        #[cfg(feature = "net")]
        {
            if args.pcgamingwiki {
                let source = fragcap::targets::HttpEngineFeed::new();
                seed_engine(&mut store, &source, now)
                    .map_err(|e| CliError::failure(e.to_string()))?
            } else {
                return Err(CliError::usage("specify --from <file> or --pcgamingwiki"));
            }
        }
        #[cfg(not(feature = "net"))]
        {
            return Err(CliError::usage(
                "specify --from <file> (live --pcgamingwiki seeding needs the `net` feature)",
            ));
        }
    };

    let _ = writeln!(
        out,
        "seeded engine {}: fetched {} written {} excluded {} duplicates {} failed {}",
        args.db.display(),
        summary.fetched,
        summary.written,
        summary.excluded,
        summary.duplicates,
        summary.failed
    );
    Ok(Exit::SUCCESS)
}

/// Run `catalog update`: fetch the current published catalog into the store.
///
/// The live fetch reuses the net-gated catalog seeder (slice S035); it is compiled
/// behind the `net` feature and not run in continuous integration. Without that
/// feature the command reports honestly that it is unavailable rather than
/// fabricating a result (P-9).
fn update(db: &Path, out: &mut dyn Write) -> Result<Exit, CliError> {
    #[cfg(feature = "net")]
    {
        let gate = CorpusGate::new(fragcap::targets::DEFAULT_MIN_REVIEWS);
        let mut store = Store::open(db).map_err(|e| CliError::failure(e.to_string()))?;
        let now = now_string();
        let source = fragcap::targets::HttpCatalog::new();
        let summary = seed_catalog(&mut store, &source, &gate, now)
            .map_err(|e| CliError::failure(e.to_string()))?;
        let _ = writeln!(
            out,
            "updated {}: fetched {} written {} excluded {} duplicates {} failed {}",
            db.display(),
            summary.fetched,
            summary.written,
            summary.excluded,
            summary.duplicates,
            summary.failed
        );
        Ok(Exit::SUCCESS)
    }
    #[cfg(not(feature = "net"))]
    {
        let _ = db;
        let _ = out;
        Err(CliError::usage(
            "catalog update fetches the published catalog over the network and needs the `net` \
             feature; this build cannot reach it",
        ))
    }
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
                db: db.to_path_buf(),
            },
        }
    }

    fn export_args(db: &std::path::Path) -> CatalogArgs {
        CatalogArgs {
            command: CatalogCommand::Export {
                db: db.to_path_buf(),
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
                from: Some(from.to_path_buf()),
                #[cfg(feature = "net")]
                steam: false,
                db: db.to_path_buf(),
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
            command: CatalogCommand::SeedEngine(TargetsSeedEngineArgs {
                from: Some(from.to_path_buf()),
                #[cfg(feature = "net")]
                pcgamingwiki: false,
                db: db.to_path_buf(),
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
