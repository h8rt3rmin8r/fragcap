// SPDX-License-Identifier: Apache-2.0

//! `technologies`: detect the technologies present in a game's install
//! directory, from file paths only.
//!
//! Scans the given directory against the vendored SteamDB detection ruleset and
//! prints the technologies it recognizes, grouped by category. Every finding is
//! a heuristic-unverified guess from a file path, which the header states and
//! which is why an anti-cheat listed here is a safety signal, not a claim.
//!
//! This labels technologies; it does not decide which executable is the
//! socket-holding client, and it is independent of capture. An unreadable target
//! directory is an expected failure (exit 1); an unreadable subtree under a
//! readable root is a surfaced warning, and the scan still succeeds.

use std::io::Write;

use fragcap::profile::{Category, CompiledRuleset};

use crate::cli::TechnologiesArgs;
use crate::exit::{CliError, Exit};

/// Run the `technologies` command, writing the report to `out`.
pub fn run(args: &TechnologiesArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    let ruleset = CompiledRuleset::embedded();
    let outcome = ruleset
        .detect(&args.path)
        .map_err(|e| CliError::failure(e.to_string()))?;

    let _ = writeln!(
        out,
        "Technologies detected in {} (heuristic-unverified):",
        args.path.display()
    );

    if outcome.findings.is_empty() {
        let _ = writeln!(out, "  no technologies detected");
    } else {
        for category in Category::ORDER {
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
                let _ = writeln!(out, "    {:<24} {}", finding.name, finding.marker_path);
            }
        }
    }

    // An unreadable subtree reduced coverage: say so rather than let the result
    // read as complete (P-4).
    for path in &outcome.unreadable {
        let _ = writeln!(out, "  warning: could not read {}", path.display());
    }

    // Reduced ruleset coverage from incompatible patterns is surfaced, not
    // implied-absent (P-4).
    if ruleset.skipped_count() > 0 {
        let _ = writeln!(
            out,
            "({} ruleset patterns skipped as incompatible)",
            ruleset.skipped_count()
        );
    }

    Ok(Exit::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::TechnologiesArgs;
    use std::fs;
    use std::path::PathBuf;
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

    fn write(dir: &std::path::Path, rel: &str) {
        let full = dir.join(rel);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(full, b"").unwrap();
    }

    #[test]
    fn it_reports_detected_technologies_grouped_by_category() {
        let dir = temp_dir("report");
        write(&dir, "Content/Maps/Level.uasset");
        write(&dir, "EasyAntiCheat/EasyAntiCheat_x64.dll");
        let args = TechnologiesArgs { path: dir.clone() };
        let mut out: Vec<u8> = Vec::new();
        let exit = run(&args, &mut out).expect("scan succeeds");
        let text = String::from_utf8(out).unwrap();
        assert_eq!(exit, Exit::SUCCESS);
        assert!(text.contains("engine"), "grouped by category: {text}");
        assert!(text.contains("Unreal"));
        assert!(text.contains("anti_cheat"));
        assert!(text.contains("EasyAntiCheat"));
        assert!(text.contains("heuristic-unverified"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_empty_install_reports_no_technologies_not_an_error() {
        let dir = temp_dir("empty");
        write(&dir, "readme.txt");
        let args = TechnologiesArgs { path: dir.clone() };
        let mut out: Vec<u8> = Vec::new();
        let exit = run(&args, &mut out).expect("scan succeeds");
        let text = String::from_utf8(out).unwrap();
        assert_eq!(exit, Exit::SUCCESS);
        assert!(text.contains("no technologies detected"), "{text}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_unreadable_target_directory_is_a_failure() {
        let missing = std::env::temp_dir().join(format!(
            "fragcap-cli-technologies-absent-{}",
            std::process::id()
        ));
        let args = TechnologiesArgs { path: missing };
        let mut out: Vec<u8> = Vec::new();
        assert!(run(&args, &mut out).is_err(), "an absent directory fails");
    }
}
