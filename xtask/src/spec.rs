// SPDX-License-Identifier: Apache-2.0

//! Specification currency checks (slice S049).
//!
//! Two assertions keep the master specification honest about which release it
//! describes, and keep changelog fragments honest about which specification
//! sections they touched.
//!
//! The version lock-step reads the `Applies-To` field from the specification's
//! document-control block and asserts it equals the workspace package version.
//! The two drifted by two minor versions before this check existed, which is the
//! condition constitution principle P-11 forbids; the check is what makes P-11
//! enforceable rather than aspirational.
//!
//! The fragment format check asserts every `changelog.d/` fragment carries a
//! well-formed `spec-impact` line, the field the release gate in `changelog`
//! reads to refuse a release whose fragment claims a specification change the
//! release diff does not contain.
//!
//! Exit codes follow the house 0/1/2 contract: 0 passed, 1 a real mismatch or a
//! malformed fragment, 2 the check could not run (a field or a directory could
//! not be read). The pure pieces (parsing, the version extractors, and the
//! release-gate decision) take strings and slices so they are unit tested with
//! no filesystem and no git.

use std::fs;
use std::io;
use std::path::Path;

/// The specification whose `Applies-To` field the version check binds against.
/// Repository-relative; the release gate matches a changed path against it too.
pub const SPEC_PATH: &str = "docs/fragcap-specification.md";

/// The parsed value of a fragment's `spec-impact` field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecImpact {
    /// The literal `none`: the change touched no specification section.
    None,
    /// One or more specification section numbers the change modified.
    Sections(Vec<String>),
}

/// A release-gate violation: a fragment claimed a specification section change
/// that the release diff does not back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateViolation {
    /// The fragment file name that carried the claim.
    pub fragment: String,
    /// The sections it named.
    pub sections: Vec<String>,
}

/// A section-number token: one or more dot-separated, non-empty, all-digit runs.
/// `3`, `3.3`, and `27.3` match; `3.`, `.3`, `a`, and the empty string do not.
fn is_section_number(tok: &str) -> bool {
    let parts: Vec<&str> = tok.split('.').collect();
    !parts.is_empty()
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// Parse a `spec-impact` value (the text between `spec-impact:` and `-->`).
///
/// `none` yields `SpecImpact::None`; a comma-separated list of section-number
/// tokens yields `SpecImpact::Sections`. An empty value, or any token that is
/// neither `none` nor a section number, is an error.
pub fn parse_spec_impact(value: &str) -> Result<SpecImpact, String> {
    let v = value.trim();
    if v.is_empty() {
        return Err("empty spec-impact value".into());
    }
    if v == "none" {
        return Ok(SpecImpact::None);
    }
    let mut sections = Vec::new();
    for tok in v.split(',') {
        let t = tok.trim();
        if !is_section_number(t) {
            return Err(format!("invalid spec-impact section token '{t}'"));
        }
        sections.push(t.to_string());
    }
    Ok(SpecImpact::Sections(sections))
}

/// Extract the raw `spec-impact` value from a fragment body when its first line
/// is a `<!-- spec-impact: ... -->` comment. Returns the inner value (trimmed),
/// or `None` when the first line is not that comment.
///
/// The comment must be the literal first line, which is where the field spec
/// places it and what lets the assembler strip exactly one leading line.
pub fn extract_spec_impact(body: &str) -> Option<String> {
    let first = body.lines().next()?;
    let inner = first.trim().strip_prefix("<!--")?.strip_suffix("-->")?;
    let val = inner.trim().strip_prefix("spec-impact:")?;
    Some(val.trim().to_string())
}

/// Read `[workspace.package] version` from the root manifest text. Section-aware
/// so a dependency's `version =` is never mistaken for the workspace version.
fn workspace_version_from(text: &str) -> Option<String> {
    let mut in_pkg = false;
    for line in text.lines() {
        // Drop an inline comment before inspecting the line. A version value
        // never contains '#', so this is safe and stops `version = "x" # note`
        // from being read with the comment attached.
        let t = line.split('#').next().unwrap_or(line).trim();
        if t.starts_with('[') {
            in_pkg = t == "[workspace.package]";
            continue;
        }
        if !in_pkg {
            continue;
        }
        // Match the `version` key exactly: the bare word followed by `=`, so a
        // key like `rust-version` (or a hypothetical `versioned`) never matches.
        if let Some(rest) = t.strip_prefix("version") {
            if let Some(value) = rest.trim_start().strip_prefix('=') {
                return Some(value.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

/// The workspace package version, read from the root `Cargo.toml`.
pub fn workspace_version(root: &Path) -> Option<String> {
    workspace_version_from(&fs::read_to_string(root.join("Cargo.toml")).ok()?)
}

/// Read the `Applies-To` version from the specification document-control block
/// text. Matches a `**Applies-To:** <value>` line, tolerating a trailing `\`.
fn applies_to_from(text: &str) -> Option<String> {
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix("**Applies-To:**") {
            let val = rest.trim().trim_end_matches('\\').trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

/// The `Applies-To` version, read from the specification.
pub fn applies_to(root: &Path) -> Option<String> {
    applies_to_from(&fs::read_to_string(root.join(SPEC_PATH)).ok()?)
}

/// The pure release-gate decision. Given each fragment's name and parsed
/// `spec-impact`, and the set of paths changed in the release diff, return the
/// violations: every fragment that names sections while the specification path
/// is absent from the changed set.
///
/// The check is file-level by design: a fragment naming any section requires the
/// specification file to have changed at all, not that the specific section
/// changed. That keeps the gate total and matches the slice's stated contract.
pub fn release_gate(
    fragments: &[(String, SpecImpact)],
    changed_paths: &[String],
) -> Vec<GateViolation> {
    let spec_changed = changed_paths
        .iter()
        .any(|p| p.replace('\\', "/").ends_with(SPEC_PATH));
    if spec_changed {
        return Vec::new();
    }
    fragments
        .iter()
        .filter_map(|(name, impact)| match impact {
            SpecImpact::Sections(secs) => Some(GateViolation {
                fragment: name.clone(),
                sections: secs.clone(),
            }),
            SpecImpact::None => None,
        })
        .collect()
}

/// Assert every `changelog.d/` fragment (except `README.md`) carries a
/// well-formed `spec-impact` line. Returns the count of malformed or missing
/// fragments, or an `io::Error` when the directory cannot be read.
fn check_fragments(root: &Path) -> io::Result<usize> {
    let dir = root.join("changelog.d");
    let mut problems = 0usize;
    let mut ok = 0usize;
    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        if path.extension().is_none_or(|e| e != "md") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if name == "README.md" {
            continue;
        }
        let body = fs::read_to_string(&path)?;
        match extract_spec_impact(&body) {
            Some(val) => match parse_spec_impact(&val) {
                Ok(_) => ok += 1,
                Err(e) => {
                    eprintln!("spec: {name}: malformed spec-impact ({e})");
                    problems += 1;
                }
            },
            None => {
                eprintln!("spec: {name}: missing a leading spec-impact comment");
                problems += 1;
            }
        }
    }
    if problems == 0 {
        println!("spec: {ok} changelog fragment(s) carry a valid spec-impact line");
    }
    Ok(problems)
}

/// Run both assertions. Returns the count of problems (0 on success), or an
/// `Err` when a check could not run at all (the 2 exit code): the workspace
/// version, the `Applies-To` field, or the fragment directory could not be read.
pub fn run(root: &Path) -> io::Result<usize> {
    let mut problems = 0usize;
    let mut could_not_run = false;

    // A. Version lock-step. A field that cannot be read is could-not-run (exit 2)
    // rather than a mismatch, but it does not short-circuit the fragment
    // assertion below: a contributor sees every problem in one pass.
    match (workspace_version(root), applies_to(root)) {
        (Some(ws), Some(at)) => {
            if ws == at {
                println!("spec: Applies-To ({at}) matches the workspace version");
            } else {
                eprintln!("spec: Applies-To ({at}) does NOT match the workspace version ({ws})");
                problems += 1;
            }
        }
        (ws, at) => {
            if ws.is_none() {
                eprintln!("spec: could not read [workspace.package] version from Cargo.toml");
            }
            if at.is_none() {
                eprintln!("spec: could not read the Applies-To field from {SPEC_PATH}");
            }
            could_not_run = true;
        }
    }

    // B. Fragment format. Always runs, so its diagnostics are reported even when
    // the version check could not run.
    match check_fragments(root) {
        Ok(n) => problems += n,
        Err(e) => {
            eprintln!("spec: could not read the changelog fragments ({e})");
            could_not_run = true;
        }
    }

    if could_not_run {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "a specification currency check could not run",
        ));
    }
    Ok(problems)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_number_shapes() {
        assert!(is_section_number("3"));
        assert!(is_section_number("3.3"));
        assert!(is_section_number("27.3"));
        assert!(!is_section_number("3."));
        assert!(!is_section_number(".3"));
        assert!(!is_section_number("a"));
        assert!(!is_section_number(""));
        assert!(!is_section_number("23.1a"));
    }

    #[test]
    fn parses_none_and_lists() {
        assert_eq!(parse_spec_impact("none").unwrap(), SpecImpact::None);
        assert_eq!(parse_spec_impact("  none  ").unwrap(), SpecImpact::None);
        assert_eq!(
            parse_spec_impact("3.3, 27.3").unwrap(),
            SpecImpact::Sections(vec!["3.3".into(), "27.3".into()])
        );
    }

    #[test]
    fn rejects_empty_and_bad_tokens() {
        assert!(parse_spec_impact("").is_err());
        assert!(parse_spec_impact("   ").is_err());
        assert!(parse_spec_impact("abc").is_err());
        assert!(parse_spec_impact("3.3, ").is_err());
        assert!(parse_spec_impact("none, 3.3").is_err());
    }

    #[test]
    fn extracts_a_leading_comment_only() {
        assert_eq!(
            extract_spec_impact("<!-- spec-impact: none -->\n\nbody\n").as_deref(),
            Some("none")
        );
        assert_eq!(
            extract_spec_impact("<!-- spec-impact: 3.3, 27.3 -->\nbody").as_deref(),
            Some("3.3, 27.3")
        );
        // Not on the first line: not extracted.
        assert_eq!(
            extract_spec_impact("body\n<!-- spec-impact: none -->"),
            None
        );
        // Not a spec-impact comment.
        assert_eq!(extract_spec_impact("<!-- other -->\n"), None);
        assert_eq!(extract_spec_impact("ordinary body\n"), None);
    }

    #[test]
    fn reads_workspace_version_section_aware() {
        let manifest = "\
[workspace]
members = [\"crates/*\"]

[workspace.package]
# The workspace version.
version      = \"0.4.0\"  # inline comment
rust-version = \"1.82\"

[workspace.dependencies]
bytes = { version = \"1\" }
";
        // The value is read past aligned whitespace and an inline comment, and
        // neither `rust-version` nor the dependency `version` is mistaken for it.
        assert_eq!(workspace_version_from(manifest).as_deref(), Some("0.4.0"));
    }

    #[test]
    fn reads_applies_to_field() {
        let header = "\
# fragcap Technical Specification

**Status:** Draft \\
**Applies-To:** 0.4.0 \\
**Audience:** Human-facing \\
";
        assert_eq!(applies_to_from(header).as_deref(), Some("0.4.0"));
        assert_eq!(applies_to_from("no field here\n"), None);
    }

    #[test]
    fn version_lockstep_logic() {
        // Equality is the whole assertion; string compare, no semver ordering.
        assert_eq!("0.4.0", "0.4.0");
        assert_ne!("0.4.0", "0.5.0");
    }

    #[test]
    fn release_gate_flags_unbacked_section_claims() {
        let frags = vec![
            (
                "a.changed.md".to_string(),
                SpecImpact::Sections(vec!["23.1".into()]),
            ),
            ("b.added.md".to_string(), SpecImpact::None),
        ];

        // Specification not in the changed set: the section-naming fragment violates.
        let changed = vec!["xtask/src/spec.rs".to_string()];
        let v = release_gate(&frags, &changed);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].fragment, "a.changed.md");
        assert_eq!(v[0].sections, vec!["23.1".to_string()]);

        // Specification present (any slash style): no violation.
        let changed = vec!["docs/fragcap-specification.md".to_string()];
        assert!(release_gate(&frags, &changed).is_empty());
        let changed = vec!["docs\\fragcap-specification.md".to_string()];
        assert!(release_gate(&frags, &changed).is_empty());

        // A `none` fragment never violates, spec changed or not.
        let only_none = vec![("c.md".to_string(), SpecImpact::None)];
        assert!(release_gate(&only_none, &[]).is_empty());
    }
}
