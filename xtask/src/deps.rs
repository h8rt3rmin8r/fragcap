// SPDX-License-Identifier: Apache-2.0

//! Dependency direction check.
//!
//! Specification section 8.3 fixes the direction dependencies may flow. That
//! is the kind of constraint which survives exactly as long as everyone
//! remembers it, so it is encoded here and asserted mechanically.
//!
//! The expected graph is written down in one place, which also makes the
//! architecture legible: a reader who wants to know the shape reads
//! `EXPECTED` rather than nine manifests.
//!
//! Manifests are parsed directly rather than through `cargo metadata`, which
//! keeps this crate free of external dependencies. The project controls the
//! manifest format, and only workspace-internal edges matter here.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// The complete permitted edge set. Anything observed but absent here is a
/// violation, and anything here but not observed is equally a violation: a
/// missing edge means a crate that should have been wired up was not, and
/// that would otherwise surface only when a later slice tried to use it.
pub const EXPECTED: &[(&str, &str)] = &[
    ("fragcap-cli", "fragcap"),
    ("fragcap", "fragcap-core"),
    ("fragcap", "fragcap-profile"),
    ("fragcap", "fragcap-capture"),
    ("fragcap", "fragcap-attr"),
    ("fragcap", "fragcap-sink"),
    ("fragcap", "fragcap-steam"),
    ("fragcap", "fragcap-targets"),
    ("fragcap-capture", "fragcap-core"),
    ("fragcap-attr", "fragcap-core"),
    ("fragcap-sink", "fragcap-core"),
    ("fragcap-profile", "fragcap-core"),
    ("fragcap-steam", "fragcap-profile"),
    ("fragcap-targets", "fragcap-profile"),
];

/// Crates at the same level below the facade. None may depend on another.
const SIBLINGS: &[&str] = &[
    "fragcap-capture",
    "fragcap-attr",
    "fragcap-sink",
    "fragcap-steam",
    "fragcap-targets",
];

/// Extract every dependency name from one manifest's text, workspace-internal
/// or not.
///
/// Recognizes the two forms this repository uses: `name.workspace = true`
/// and `name = { ... }`. Considers every dependency table, including
/// target-conditional ones, because a platform-specific dependency hidden
/// behind `[target.'cfg(windows)'.dependencies]` is exactly the case
/// constitution P-2 is about.
pub fn parse_all_deps(manifest: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut in_deps = false;

    for raw in manifest.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            // Matches [dependencies] and [target.'cfg(...)'.dependencies],
            // but not [dev-dependencies] or [build-dependencies].
            in_deps = line == "[dependencies]"
                || (line.starts_with("[target.") && line.ends_with(".dependencies]"));
            continue;
        }
        if !in_deps || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let name = line
            .split(['=', '.'])
            .next()
            .unwrap_or("")
            .trim()
            .trim_matches('"');
        if !name.is_empty() {
            out.insert(name.to_string());
        }
    }
    out
}

/// Workspace-internal dependencies only, for the edge-set comparison.
pub fn parse_deps(manifest: &str) -> BTreeSet<String> {
    parse_all_deps(manifest)
        .into_iter()
        .filter(|d| d.starts_with("fragcap"))
        .collect()
}

/// Crates `fragcap-core` is permitted to depend on.
///
/// An allowlist rather than an empty set, and short on purpose. Every entry has
/// been read against constitution P-2: no platform-specific surface, no I/O, no
/// capture library. Adding to this list is a deliberate edit that a reviewer
/// sees, which is the property that matters. Growing it casually is how
/// platform leakage gets in.
///
/// - `bytes`: reference-counted byte buffers. Pure Rust, no platform surface,
///   no I/O. Admitted by slice S02 decision D-1 because a payload is cloned
///   into a bounded ring and fanned out to several sinks, and copying it per
///   sink is the hot path of the program.
const CORE_ALLOWED_DEPS: &[&str] = &["bytes"];

/// Assert that `fragcap-core` depends on nothing outside `CORE_ALLOWED_DEPS`.
///
/// This exists because `cargo xtask neutral` does not cover it, and the gap is
/// not obvious. A platform-specific crate can be added to core and core will
/// still build for a non-Windows target, because such crates are themselves
/// internally cfg-gated and compile to nothing off-platform. The build
/// therefore succeeds while P-2 has been violated. Only a manifest check
/// catches that.
///
/// Until slice S02 this was "no dependencies at all", which was simpler and
/// stricter than the principle it enforces. P-2 forbids a platform-specific
/// dependency, an I/O crate, and a capture library; it does not forbid every
/// dependency. The empty-set rule would have blocked a pure-Rust buffer crate
/// on a reading the constitution does not support, so S02 replaced it with a
/// named allowlist. The check still fails closed: anything not listed is a
/// problem.
pub fn check_core_dependencies_are_allowed(manifest: &str) -> Vec<String> {
    parse_all_deps(manifest)
        .into_iter()
        .filter(|d| !CORE_ALLOWED_DEPS.contains(&d.as_str()))
        .map(|d| {
            format!(
                "fragcap-core may depend only on {CORE_ALLOWED_DEPS:?} (constitution P-2),                  found {d}"
            )
        })
        .collect()
}

/// Compare an observed edge set against `EXPECTED`, returning the problems.
pub fn compare(observed: &BTreeSet<(String, String)>) -> Vec<String> {
    let expected: BTreeSet<(String, String)> = EXPECTED
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();

    let mut problems = Vec::new();

    for (from, to) in observed.difference(&expected) {
        problems.push(format!("unexpected edge: {from} -> {to}"));
    }
    for (from, to) in expected.difference(observed) {
        problems.push(format!("missing edge: {from} -> {to}"));
    }
    for (from, to) in observed {
        if to == "fragcap-cli" {
            problems.push(format!("{from} depends on the binary crate fragcap-cli"));
        }
        if SIBLINGS.contains(&from.as_str()) && SIBLINGS.contains(&to.as_str()) {
            problems.push(format!("sibling dependency: {from} -> {to}"));
        }
        if from == "fragcap-core" {
            problems.push(format!(
                "fragcap-core must have no dependencies, found {to}"
            ));
        }
    }

    problems.sort();
    problems.dedup();
    problems
}

/// Read the workspace's manifests and check the graph. Returns problem count.
pub fn run(root: &Path) -> std::io::Result<usize> {
    let mut observed: BTreeSet<(String, String)> = BTreeSet::new();
    let mut extra: Vec<String> = Vec::new();

    for entry in fs::read_dir(root.join("crates"))? {
        let dir = entry?.path();
        if !dir.is_dir() {
            continue;
        }
        let name = match dir.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let manifest = fs::read_to_string(dir.join("Cargo.toml"))?;
        for dep in parse_deps(&manifest) {
            observed.insert((name.clone(), dep));
        }
        if name == "fragcap-core" {
            extra.extend(check_core_dependencies_are_allowed(&manifest));
        }
    }

    let mut problems = compare(&observed);
    problems.extend(extra);
    problems.sort();
    problems.dedup();
    for p in &problems {
        println!("{p}");
    }
    Ok(problems.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edges(pairs: &[(&str, &str)]) -> BTreeSet<(String, String)> {
        pairs
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    }

    fn expected_set() -> BTreeSet<(String, String)> {
        edges(EXPECTED)
    }

    #[test]
    fn exact_match_has_no_problems() {
        assert_eq!(compare(&expected_set()), Vec::<String>::new());
    }

    #[test]
    fn detects_an_unexpected_edge() {
        let mut o = expected_set();
        o.insert(("fragcap-core".into(), "fragcap-sink".into()));
        let p = compare(&o);
        assert!(p
            .iter()
            .any(|s| s.contains("unexpected edge: fragcap-core -> fragcap-sink")));
    }

    #[test]
    fn detects_a_missing_edge() {
        let mut o = expected_set();
        o.remove(&("fragcap-steam".to_string(), "fragcap-profile".to_string()));
        let p = compare(&o);
        assert!(p
            .iter()
            .any(|s| s.contains("missing edge: fragcap-steam -> fragcap-profile")));
    }

    #[test]
    fn detects_dependency_on_the_binary_crate() {
        let mut o = expected_set();
        o.insert(("fragcap-sink".into(), "fragcap-cli".into()));
        let p = compare(&o);
        assert!(p.iter().any(|s| s.contains("depends on the binary crate")));
    }

    #[test]
    fn detects_a_sibling_dependency() {
        let mut o = expected_set();
        o.insert(("fragcap-capture".into(), "fragcap-attr".into()));
        let p = compare(&o);
        assert!(p.iter().any(|s| s.contains("sibling dependency")));
    }

    #[test]
    fn detects_a_dependency_on_core() {
        let mut o = expected_set();
        o.insert(("fragcap-core".into(), "fragcap-profile".into()));
        let p = compare(&o);
        assert!(p
            .iter()
            .any(|s| s.contains("fragcap-core must have no dependencies")));
    }

    #[test]
    fn parses_workspace_shorthand_dependencies() {
        let m = "[package]\nname = \"x\"\n\n[dependencies]\nfragcap-core.workspace = true\n";
        assert_eq!(
            parse_deps(m),
            edges(&[("fragcap-core", "")])
                .iter()
                .map(|(a, _)| a.clone())
                .collect()
        );
    }

    #[test]
    fn ignores_dependencies_outside_the_dependencies_table() {
        let m = "[dev-dependencies]\nfragcap-core.workspace = true\n";
        assert!(parse_deps(m).is_empty());
    }

    #[test]
    fn core_with_no_dependencies_is_accepted() {
        let m = "[package]
name = \"fragcap-core\"
";
        assert_eq!(check_core_dependencies_are_allowed(m), Vec::<String>::new());
    }

    #[test]
    fn core_may_take_an_allowlisted_dependency() {
        let m = "[dependencies]
bytes.workspace = true
";
        assert_eq!(check_core_dependencies_are_allowed(m), Vec::<String>::new());
    }

    #[test]
    fn core_may_not_take_an_unlisted_dependency_however_harmless() {
        // The check fails closed. A crate being pure Rust is not enough; it has
        // to have been read against P-2 and added deliberately.
        let m = "[dependencies]
itoa = \"1\"
";
        let p = check_core_dependencies_are_allowed(m);
        assert!(p.iter().any(|s| s.contains("itoa")), "got {p:?}");
    }

    #[test]
    fn detects_an_external_dependency_in_core() {
        let m = "[dependencies]
windows-sys = { version = \"0.59\" }
";
        let p = check_core_dependencies_are_allowed(m);
        assert!(p.iter().any(|s| s.contains("windows-sys")), "got {p:?}");
    }

    #[test]
    fn detects_a_target_conditional_dependency_in_core() {
        // The case cargo xtask neutral cannot see: core still builds for a
        // non-Windows target because windows-sys compiles to nothing there,
        // so only a manifest check catches the P-2 violation.
        let m = "[target.'cfg(windows)'.dependencies]
windows-sys = \"0.59\"
";
        let p = check_core_dependencies_are_allowed(m);
        assert!(p.iter().any(|s| s.contains("windows-sys")), "got {p:?}");
    }

    #[test]
    fn ignores_dev_dependencies_in_core() {
        let m = "[dev-dependencies]
tempfile = \"3\"
";
        assert_eq!(check_core_dependencies_are_allowed(m), Vec::<String>::new());
    }

    #[test]
    fn ignores_non_workspace_crates() {
        let m = "[dependencies]\nserde.workspace = true\nfragcap-core.workspace = true\n";
        let d = parse_deps(m);
        assert!(d.contains("fragcap-core"));
        assert!(!d.contains("serde"));
    }
}
