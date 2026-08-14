// SPDX-License-Identifier: Apache-2.0

//! `targets`: manage the targets hint database (issue #78).
//!
//! Two subcommands, both offline and operating only on local paths: `import`
//! loads a JSON seed document into a store (creating it if needed), and `export`
//! projects a store to schema-conformant JSON on standard output. There is no
//! network access; the seeders that fill the store from the Steam Web API, PICS,
//! and PCGamingWiki are later slices.
//!
//! A malformed seed fails without leaving a store behind: the import is
//! transactional, and a store this command freshly created for a failed import
//! is removed rather than left as a stray empty file (P-4).

use std::io::Write;

use fragcap::targets::{export, import, seed_catalog, CorpusGate, FixtureCatalog, Store};

use crate::cli::{TargetsArgs, TargetsCommand, TargetsSeedArgs};
use crate::exit::{CliError, Exit};

/// Run the `targets` command, writing results to `out`.
pub fn run(args: &TargetsArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    match &args.command {
        TargetsCommand::Import { seed, db } => {
            let text = std::fs::read_to_string(seed).map_err(|e| {
                CliError::failure(format!("cannot read seed {}: {e}", seed.display()))
            })?;

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
                    // Do not leave a store we just created for a seed that did
                    // not load; a pre-existing store is untouched (the import is
                    // atomic).
                    drop(store);
                    if !existed {
                        let _ = std::fs::remove_file(db);
                    }
                    Err(CliError::failure(e.to_string()))
                }
            }
        }
        TargetsCommand::Export { db } => {
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
        TargetsCommand::Seed(args) => seed(args, out),
    }
}

/// A Unix-epoch-seconds stamp for the seed state's last-run field. Informational;
/// avoids a date-formatting dependency.
fn now_string() -> Option<String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs().to_string())
}

/// Run `targets seed`: fill the catalog tier from a fixture (offline) or, under
/// the `net` feature with `--steam`, from the live catalog.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{TargetsArgs, TargetsCommand};

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

    fn import_args(seed: &std::path::Path, db: &std::path::Path) -> TargetsArgs {
        TargetsArgs {
            command: TargetsCommand::Import {
                seed: seed.to_path_buf(),
                db: db.to_path_buf(),
            },
        }
    }

    fn export_args(db: &std::path::Path) -> TargetsArgs {
        TargetsArgs {
            command: TargetsCommand::Export {
                db: db.to_path_buf(),
            },
        }
    }

    const CATALOG: &str = r#"[
      { "appid": 570, "name": "Dota 2", "classification": "game", "review_count": 2000000 },
      { "appid": 440, "name": "Below Threshold", "classification": "game", "review_count": 5 },
      { "appid": 700, "name": "A Tool", "classification": "other", "review_count": 9999 }
    ]"#;

    fn seed_args(from: &std::path::Path, db: &std::path::Path) -> TargetsArgs {
        TargetsArgs {
            command: TargetsCommand::Seed(TargetsSeedArgs {
                from: Some(from.to_path_buf()),
                #[cfg(feature = "net")]
                steam: false,
                db: db.to_path_buf(),
                min_reviews: 100,
            }),
        }
    }

    #[test]
    fn seed_from_a_fixture_then_export_round_trips_to_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        let catalog = dir.path().join("catalog.json");
        let db = dir.path().join("hint.db");
        std::fs::write(&catalog, CATALOG).unwrap();

        let mut out: Vec<u8> = Vec::new();
        let exit = run(&seed_args(&catalog, &db), &mut out).expect("seed succeeds");
        assert_eq!(exit, Exit::SUCCESS);
        let report = String::from_utf8(out).unwrap();
        // Only Dota 2 clears the game + 100-review gate.
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
        let db = dir.path().join("hint.db");
        let mut out: Vec<u8> = Vec::new();
        assert!(run(&seed_args(&missing, &db), &mut out).is_err());
    }

    #[test]
    fn import_then_export_round_trips_to_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        let seed = dir.path().join("seed.json");
        let db = dir.path().join("hint.db");
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
        let db = dir.path().join("hint.db");
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
        let db = dir.path().join("hint.db");
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
