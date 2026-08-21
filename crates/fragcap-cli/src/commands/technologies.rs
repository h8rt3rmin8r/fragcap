// SPDX-License-Identifier: Apache-2.0

//! `technologies`: detect the engine, anti-cheat, and DRM technologies present in a
//! game's install directory (slice S053).
//!
//! Scans the given directory against the catalog's data-driven signature table
//! (`catalog seed --tier signature`) and prints the technologies it recognizes, grouped by
//! category. A locally detected engine is `verified`; every finding is a neutral
//! fact.
//!
//! # Neutral evidence
//!
//! A detected anti-cheat or DRM product is recorded and displayed as a neutral fact
//! (specification section 3.6). Nothing in this output characterizes a title as off
//! limits, risky, or discouraged: detection is a report, never a gate.
//!
//! This labels technologies; it does not decide which executable is the
//! socket-holding client, and it is independent of capture. An unreadable target
//! directory is an expected failure (exit 1); an unreadable subtree under a readable
//! root is a surfaced warning, and the scan still succeeds.

use std::io::Write;

use fragcap::profile::signature::SignatureSet;
use fragcap::profile::SignatureCategory;
use fragcap::targets::Store;

use crate::cli::TechnologiesArgs;
use crate::exit::{CliError, Exit};

/// Run the `technologies` command, writing the report to `out`.
pub fn run(args: &TechnologiesArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    // The catalog store is an override, never a requirement (issue #179).
    let catalog_db =
        crate::commands::target_resolve::ensure_catalog_store(args.catalog_db.as_deref())
            .map_err(CliError::failure)?
            .ok_or_else(|| {
                CliError::failure(concat!(
                    "no catalog store could be resolved: pass --catalog-db, set ",
                    "FRAGCAP_CATALOG_DB, or run on a machine with a per-user ",
                    "application data directory",
                ))
            })?;
    let store = Store::open(&catalog_db).map_err(|e| CliError::failure(e.to_string()))?;
    let signatures = store
        .load_signatures()
        .map_err(|e| CliError::failure(e.to_string()))?;
    let set = SignatureSet::compile(&signatures);

    let outcome = set
        .detect(&args.path)
        .map_err(|e| CliError::failure(e.to_string()))?;

    // Name the store the findings came from. With `--catalog-db` optional since
    // slice S063, the signature table can be an explicit path, an environment
    // override, or the per-user default, and a reader who cannot tell which
    // cannot tell why a detection differs between machines (FR-005, raised in
    // review of PR #190).
    let _ = writeln!(
        out,
        "Technologies detected in {} (signatures from {}):",
        args.path.display(),
        catalog_db.display()
    );

    if outcome.findings.is_empty() {
        let _ = writeln!(out, "  no technologies detected");
    } else {
        for category in SignatureCategory::ORDER {
            let group: Vec<_> = outcome
                .findings
                .iter()
                .filter(|f| f.category == category)
                .collect();
            if group.is_empty() {
                continue;
            }
            let _ = writeln!(out, "  {}", category.as_str());
            for finding in group {
                let _ = writeln!(
                    out,
                    "    {:<20} {:<10} {}",
                    finding.product,
                    finding.fidelity.as_str(),
                    finding.evidence
                );
            }
        }
    }

    // An unreadable subtree reduced coverage: say so rather than let the result read
    // as complete (P-4).
    for path in &outcome.unreadable {
        let _ = writeln!(out, "  warning: could not read {}", path.display());
    }

    // Reduced coverage from an inert (not-yet-matchable) signature kind is surfaced,
    // not implied-absent (P-4).
    if set.inert_count() > 0 {
        let _ = writeln!(
            out,
            "({} signatures carry a match kind not evaluated yet)",
            set.inert_count()
        );
    }

    // A signature of an implemented kind that failed to compile (for example a
    // pattern that exceeds the regex size limit) also reduced coverage; name each so
    // an operator-added row that silently stopped matching is visible (P-4).
    for skip in set.skipped() {
        let _ = writeln!(
            out,
            "  warning: signature for {} was skipped: {}",
            skip.product, skip.error
        );
    }

    Ok(Exit::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::TechnologiesArgs;
    use fragcap::targets::seed_bundled;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU32, Ordering};

    fn temp_dir(tag: &str) -> PathBuf {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "fragcap-cli-technologies-{}-{}-{}",
            std::process::id(),
            tag,
            n
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write(dir: &Path, rel: &str) {
        write_bytes(dir, rel, b"");
    }

    fn write_bytes(dir: &Path, rel: &str, bytes: &[u8]) {
        let full = dir.join(rel);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(full, bytes).unwrap();
    }

    /// A seeded catalog.db in a scratch directory.
    fn seeded_catalog(dir: &Path) -> PathBuf {
        let catalog = dir.join("catalog.db");
        let mut store = Store::open(&catalog).expect("open catalog");
        seed_bundled(&mut store).expect("seed signatures");
        catalog
    }

    #[test]
    fn it_reports_detected_technologies_as_neutral_evidence() {
        let dir = temp_dir("report");
        let catalog = seeded_catalog(&dir);
        write(&dir, "UnityPlayer.dll");
        write(&dir, "EasyAntiCheat/EasyAntiCheat_x64.dll");
        // Since slice S065 the DRM signal is the wrapper's own `.bind` PE section,
        // not the presence of the Steamworks SDK library beside it. Both are written
        // here so the test still covers a DRM finding rendering in the DRM category,
        // and still shows that the library alone would not have produced one.
        write(&dir, "steam_api64.dll");
        write_bytes(
            &dir,
            "Game.exe",
            &fragcap::profile::pe::fixtures::minimal_pe_with_sections(&[".text", ".bind"]),
        );
        let args = TechnologiesArgs {
            path: dir.clone(),
            catalog_db: Some(catalog),
        };
        let mut out: Vec<u8> = Vec::new();
        let exit = run(&args, &mut out).expect("scan succeeds");
        let text = String::from_utf8(out).unwrap();
        assert_eq!(exit, Exit::SUCCESS);
        assert!(text.contains("engine"), "grouped by category: {text}");
        assert!(text.contains("Unity"));
        assert!(text.contains("verified"), "a definitive engine is verified");
        assert!(text.contains("anti-cheat"));
        assert!(text.contains("Easy Anti-Cheat"));
        assert!(text.contains("drm") && text.contains("Steam DRM"));
        // Neutral: no output frames a detected product as a reason not to capture.
        let lowered = text.to_lowercase();
        for banned in [
            "blocked",
            "unsupported",
            "cannot capture",
            "not capturable",
            "risky",
            "warning: anti",
        ] {
            assert!(
                !lowered.contains(banned),
                "no gating wording ({banned}): {text}"
            );
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_empty_install_reports_no_technologies_not_an_error() {
        let dir = temp_dir("empty");
        let catalog = seeded_catalog(&dir);
        write(&dir, "readme.txt");
        let args = TechnologiesArgs {
            path: dir.clone(),
            catalog_db: Some(catalog),
        };
        let mut out: Vec<u8> = Vec::new();
        let exit = run(&args, &mut out).expect("scan succeeds");
        let text = String::from_utf8(out).unwrap();
        assert_eq!(exit, Exit::SUCCESS);
        assert!(text.contains("no technologies detected"), "{text}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_unreadable_target_directory_is_a_failure() {
        let dir = temp_dir("absent-cat");
        let catalog = seeded_catalog(&dir);
        let missing = std::env::temp_dir().join(format!(
            "fragcap-cli-technologies-absent-{}",
            std::process::id()
        ));
        let args = TechnologiesArgs {
            path: missing,
            catalog_db: Some(catalog),
        };
        let mut out: Vec<u8> = Vec::new();
        assert!(run(&args, &mut out).is_err(), "an absent directory fails");
        let _ = fs::remove_dir_all(&dir);
    }
}
