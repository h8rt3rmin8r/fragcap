// SPDX-License-Identifier: Apache-2.0

//! Release notes derived from the changelog.
//!
//! Specification section 24.4 requires a release to carry notes derived from
//! the changelog. That means finding one section in `CHANGELOG.md` and
//! printing it, which is small enough to be tempting to do with a few lines
//! of shell inside the release workflow.
//!
//! It lives here instead for the reason the house rules give: a wrapper that
//! parses text is a missing capability in Rust. Parsing in the workflow would
//! also be untestable, and a notes extractor that silently produces an empty
//! body would publish a release with no notes and report success.

use std::fs;
use std::path::Path;

/// Pull the body of one version's section out of a changelog.
///
/// Matches the heading `## [<version>]` exactly, so `0.1.0` does not match
/// `## [10.1.0]`. Collects until the next second-level heading. Returns
/// `None` when the section is absent, and `None` when it is present but
/// empty, because an empty body is not usable release notes.
pub fn extract(changelog: &str, version: &str) -> Option<String> {
    let wanted = format!("[{version}]");
    let mut lines = changelog.lines();

    lines.by_ref().find(|line| {
        let l = line.trim_start();
        l.starts_with("## ") && l.contains(&wanted)
    })?;

    let mut body = String::new();
    for line in lines {
        if line.trim_start().starts_with("## ") {
            break;
        }
        body.push_str(line);
        body.push('\n');
    }

    let trimmed = body.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Print the notes for `version`, falling back to the `Unreleased` section.
///
/// The fallback exists because `CHANGELOG.md` is assembled from
/// `changelog.d/` fragments at release time, and a tag can be cut before that
/// assembly has renamed the section.
pub fn run(root: &Path, version: &str) -> usize {
    let path = root.join("CHANGELOG.md");
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("notes: could not read {}: {e}", path.display());
            return 1;
        }
    };

    if let Some(body) = extract(&text, version) {
        println!("{body}");
        return 0;
    }

    if let Some(body) = extract(&text, "Unreleased") {
        eprintln!("notes: no section for {version}, using Unreleased");
        println!("{body}");
        return 0;
    }

    eprintln!("notes: CHANGELOG.md has no section for {version} and no Unreleased section");
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
# Changelog

Preamble that is not part of any section.

## [Unreleased]

### Added

- Something unreleased.

## [0.2.0] - 2026-09-01

### Added

- The thing this release adds.

### Decisions

- **2026-09-01** A decision.

## [0.1.0] - 2026-08-08

### Added

- The first release.
";

    #[test]
    fn extracts_the_named_version() {
        let body = extract(SAMPLE, "0.2.0").unwrap();
        assert!(body.contains("The thing this release adds."));
        assert!(body.contains("A decision."));
    }

    #[test]
    fn stops_at_the_next_version() {
        let body = extract(SAMPLE, "0.2.0").unwrap();
        assert!(!body.contains("The first release."));
        assert!(!body.contains("Something unreleased."));
    }

    #[test]
    fn extracts_the_last_section() {
        let body = extract(SAMPLE, "0.1.0").unwrap();
        assert!(body.contains("The first release."));
    }

    #[test]
    fn extracts_unreleased() {
        let body = extract(SAMPLE, "Unreleased").unwrap();
        assert!(body.contains("Something unreleased."));
    }

    #[test]
    fn a_version_prefix_does_not_match_a_longer_version() {
        // "0.1.0" must not match "## [10.1.0]".
        let text = "## [10.1.0]\n\n- Ten.\n";
        assert_eq!(extract(text, "0.1.0"), None);
    }

    #[test]
    fn an_absent_section_is_none() {
        assert_eq!(extract(SAMPLE, "9.9.9"), None);
    }

    #[test]
    fn an_empty_section_is_none() {
        let text = "## [0.3.0]\n\n## [0.2.0]\n\n- Real.\n";
        assert_eq!(extract(text, "0.3.0"), None);
    }

    #[test]
    fn the_preamble_is_not_treated_as_a_section() {
        let body = extract(SAMPLE, "Unreleased").unwrap();
        assert!(!body.contains("Preamble"));
    }
}
