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
/// `## [10.1.0]`. Collects until the next second-level heading, and also stops
/// at the `### Decisions` subsection: decisions record verbose internal
/// rationale for changing pinned artifacts, which belongs in the changelog file
/// but would bloat the release notes (a single decision fragment runs to
/// kilobytes). The curated `### Highlights` and the user-facing Added, Changed,
/// and Fixed sections all sit above Decisions, so they are kept. Returns `None`
/// when the section is absent, and `None` when it is present but empty, because
/// an empty body is not usable release notes.
pub fn extract(changelog: &str, version: &str) -> Option<String> {
    let wanted = format!("[{version}]");
    let mut lines = changelog.lines();

    lines.by_ref().find(|line| {
        let l = line.trim_start();
        l.starts_with("## ") && l.contains(&wanted)
    })?;

    let mut body = String::new();
    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with("## ") {
            break;
        }
        if trimmed == "### Decisions" {
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

/// Pull just the `### Highlights` subsection out of one version's section.
///
/// Finds `## [<version>]`, then the `### Highlights` heading within it, and
/// collects until the next subsection (`###`) or version (`##`). The heading
/// itself is dropped; the curated bullets are what the release notes want.
/// Returns `None` when the version has no Highlights block, so the caller can
/// fall back to the fuller body. This is what keeps a release page crisp: for a
/// release with eighteen essay-length fragments, the full Added list runs to
/// over a thousand lines, and the Highlights are the two dozen that matter.
pub fn extract_highlights(changelog: &str, version: &str) -> Option<String> {
    let wanted = format!("[{version}]");
    let mut lines = changelog.lines();

    lines.by_ref().find(|line| {
        let l = line.trim_start();
        l.starts_with("## ") && l.contains(&wanted)
    })?;

    let mut in_highlights = false;
    let mut body = String::new();
    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with("## ") {
            break;
        }
        if trimmed.starts_with("### ") {
            if trimmed == "### Highlights" {
                in_highlights = true;
            } else if in_highlights {
                break;
            }
            continue;
        }
        if in_highlights {
            body.push_str(line);
            body.push('\n');
        }
    }

    let trimmed = body.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// A link to the full changelog at the release tag.
fn changelog_link(version: &str) -> String {
    format!("https://github.com/h8rt3rmin8r/fragcap/blob/v{version}/CHANGELOG.md")
}

/// The release-notes body for a real version: the curated Highlights plus a link
/// to the full changelog at the tag, or the fuller Added/Changed/Fixed body when
/// the version carries no Highlights block. Returns `None` when the version has
/// no section at all.
pub fn notes(changelog: &str, version: &str) -> Option<String> {
    if let Some(highlights) = extract_highlights(changelog, version) {
        Some(format!(
            "{highlights}\n\nFull changelog: {}",
            changelog_link(version)
        ))
    } else {
        extract(changelog, version)
    }
}

/// Print the notes for `version`, falling back to the `Unreleased` section.
///
/// For a real version, the notes are the curated Highlights plus a link to the
/// full changelog; a version with no Highlights keeps the fuller body. The
/// Unreleased fallback exists because `CHANGELOG.md` is assembled from
/// `changelog.d/` fragments at release time, and a tag can be cut before that
/// assembly has renamed the section; it carries no tag link, because the content
/// is not yet published under a version.
pub fn run(root: &Path, version: &str) -> usize {
    let path = root.join("CHANGELOG.md");
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("notes: could not read {}: {e}", path.display());
            return 1;
        }
    };

    if let Some(body) = notes(&text, version) {
        println!("{body}");
        return 0;
    }

    if let Some(body) =
        extract_highlights(&text, "Unreleased").or_else(|| extract(&text, "Unreleased"))
    {
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
    }

    #[test]
    fn excludes_the_decisions_section() {
        // Decisions are verbose internal rationale and must not reach the
        // release notes, even though they stay in CHANGELOG.md.
        let body = extract(SAMPLE, "0.2.0").unwrap();
        assert!(!body.contains("A decision."));
        assert!(!body.contains("### Decisions"));
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

    const HIGHLIGHTED: &str = "\
## [0.2.0] - 2026-09-01

### Highlights

- A curated highlight.
- Another highlight.

### Added

- A verbose added item that should not reach the notes.

### Decisions

- **2026-09-01** A decision.
";

    #[test]
    fn extract_highlights_returns_only_the_highlights_bullets() {
        let h = extract_highlights(HIGHLIGHTED, "0.2.0").unwrap();
        assert!(h.contains("A curated highlight."));
        assert!(h.contains("Another highlight."));
        assert!(
            !h.contains("verbose added item"),
            "the Added list is trimmed"
        );
        assert!(!h.contains("### Highlights"), "the heading is dropped");
        assert!(!h.contains("A decision."), "decisions are excluded");
    }

    #[test]
    fn extract_highlights_is_none_without_a_highlights_block() {
        // SAMPLE's 0.2.0 has Added but no Highlights.
        assert_eq!(extract_highlights(SAMPLE, "0.2.0"), None);
    }

    #[test]
    fn notes_prefers_highlights_and_appends_a_tag_link() {
        let body = notes(HIGHLIGHTED, "0.2.0").unwrap();
        assert!(body.contains("A curated highlight."));
        assert!(
            !body.contains("verbose added item"),
            "the full Added list is trimmed away when Highlights exist"
        );
        assert!(
            body.contains("blob/v0.2.0/CHANGELOG.md"),
            "links to the full changelog at the tag: {body}"
        );
    }

    #[test]
    fn notes_falls_back_to_the_full_body_without_highlights() {
        // SAMPLE 0.2.0 has no Highlights: keep the current Added/Changed/Fixed
        // body, and do not append a link (the body is already the full notes).
        let body = notes(SAMPLE, "0.2.0").unwrap();
        assert!(body.contains("The thing this release adds."));
        assert!(!body.contains("A decision."), "decisions still excluded");
        assert!(!body.contains("blob/"), "no link in the fallback body");
    }
}
