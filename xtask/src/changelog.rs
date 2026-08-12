// SPDX-License-Identifier: Apache-2.0

//! Changelog assembly from `changelog.d/` fragments (specification 24.4).
//!
//! `CHANGELOG.md` is assembled from the one-file-per-change fragments under
//! `changelog.d/` at release time. Every pull request adds a fragment instead
//! of editing the changelog, so parallel branches never conflict on the same
//! few lines; this command is what folds them back together.
//!
//! It lives in the task runner rather than in the release shell script for the
//! reason the house rules give: a wrapper that parses text is a missing
//! capability in Rust. The assembly is a text transform with a canonical
//! section order and a merge rule, both of which are worth a unit test, and a
//! shell reimplementation in two dialects (Bash and PowerShell) would be two
//! copies of the same untested logic.
//!
//! Two entry points share the transform: `--check` prints the assembled body
//! and changes nothing (for a dry run), and `--release <version> <date>`
//! rewrites `CHANGELOG.md`, moving the assembled body into a dated version
//! section and emptying `[Unreleased]`, then removes the consumed fragments.

use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

/// The canonical section order. `highlights` leads: it carries the curated
/// release summary the notes lead with, and it sits above `decisions` so the
/// notes extractor's Decisions cutoff (see `notes`) keeps it in the release
/// body. `decisions` is last because it is the verbose internal-rationale
/// section the release notes deliberately drop.
const SECTION_ORDER: &[&str] = &[
    "highlights",
    "added",
    "changed",
    "deprecated",
    "removed",
    "fixed",
    "security",
    "decisions",
];

/// The repository the version link references point at. Matches the existing
/// `[Unreleased]:` reference in `CHANGELOG.md`.
const REPO: &str = "https://github.com/h8rt3rmin8r/fragcap";

/// `"added"` becomes `"### Added"`. Only the first letter is capitalized, so
/// every key in `SECTION_ORDER` renders as it appears in the changelog.
fn section_heading(key: &str) -> String {
    let mut chars = key.chars();
    let title = match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    };
    format!("### {title}")
}

/// Split a changelog body into `(section_key, content)` chunks, one per
/// `### Heading`. The key is the heading text lowercased; the content is every
/// line after the heading up to the next heading, outer whitespace trimmed.
/// Text before the first heading is ignored, so a fragment that carries its own
/// `### Added` line and one that is bare both parse sensibly.
fn split_sections(body: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut current: Option<(String, String)> = None;

    for line in body.lines() {
        if let Some(rest) = line.trim_start().strip_prefix("### ") {
            if let Some((key, content)) = current.take() {
                out.push((key, content));
            }
            current = Some((rest.trim().to_lowercase(), String::new()));
        } else if let Some((_, content)) = current.as_mut() {
            content.push_str(line);
            content.push('\n');
        }
    }
    if let Some((key, content)) = current.take() {
        out.push((key, content));
    }

    out.into_iter()
        .map(|(key, content)| (key, content.trim().to_string()))
        .collect()
}

/// Strip a leading `### <section>` line (and the blank line after it) from a
/// fragment body, when present. A fragment's section is its file name, so a
/// body that repeats it as its first heading would otherwise emit a second
/// `### Added` under the one this assembler writes. A body that leads with its
/// own title heading (`### Profile schema ...`) is left untouched, so that
/// heading survives as a titled sub-block, and a decisions body full of
/// `### <date>: ...` sub-headings is opaque content and is never re-sectioned.
fn strip_leading_section_header(content: &str, section: &str) -> String {
    let want = format!("### {section}").to_lowercase();
    let mut lines = content.lines().peekable();
    if let Some(first) = lines.peek() {
        if first.trim().to_lowercase() == want {
            lines.next();
            // Also drop one blank separator line, if any.
            if lines.peek().is_some_and(|l| l.trim().is_empty()) {
                lines.next();
            }
            return lines.collect::<Vec<_>>().join("\n").trim().to_string();
        }
    }
    content.trim().to_string()
}

/// Merge an `[Unreleased]` body and a set of fragments into one changelog body
/// in canonical section order.
///
/// The existing `[Unreleased]` content is kept, so legacy inline entries
/// written before the fragment system are never lost. A fragment's section is
/// its file name (the `.added` in `S17-steam.added.md`), not any heading inside
/// it: the fragments are authored two ways, some leading with `### Added` and
/// some with their own title, and only the file name is a reliable signal.
/// Returns an error naming the first unknown section, because a typo like
/// `.add.md` must fail loudly rather than drop the entry.
pub fn assemble(unreleased_body: &str, fragments: &[(String, String)]) -> Result<String, String> {
    // One bucket per known section, in `SECTION_ORDER` positions, preserving
    // the order chunks arrive: existing [Unreleased] first, then fragments as
    // given (the caller sorts by filename for determinism).
    let mut buckets: Vec<Vec<String>> = vec![Vec::new(); SECTION_ORDER.len()];

    // The [Unreleased] body carries real `### Section` headings, so split it.
    let mut chunks: Vec<(String, String)> = split_sections(unreleased_body);
    // A fragment's section is its file name; its body is opaque apart from a
    // redundant leading section header, which is stripped.
    for (section, content) in fragments {
        let key = section.to_lowercase();
        let body = strip_leading_section_header(content, &key);
        chunks.push((key, body));
    }

    for (key, content) in &chunks {
        if content.is_empty() {
            continue;
        }
        match SECTION_ORDER.iter().position(|s| *s == key) {
            Some(idx) => buckets[idx].push(content.clone()),
            None => return Err(format!("unknown changelog section '### {key}'")),
        }
    }

    let mut body = String::new();
    for (idx, section) in SECTION_ORDER.iter().enumerate() {
        if buckets[idx].is_empty() {
            continue;
        }
        body.push_str(&section_heading(section));
        body.push_str("\n\n");
        body.push_str(&buckets[idx].join("\n\n"));
        body.push_str("\n\n");
    }

    Ok(body.trim_end().to_string())
}

/// Extract the body of the `## [Unreleased]` section from a changelog. Returns
/// the empty string when the section is absent or empty.
fn unreleased_body(changelog: &str) -> String {
    let mut lines = changelog.lines();
    if lines
        .by_ref()
        .find(|l| l.trim_start().starts_with("## [Unreleased]"))
        .is_none()
    {
        return String::new();
    }

    let mut body = String::new();
    for line in lines {
        let trimmed = line.trim_start();
        // A version heading or a link-reference definition ends the section.
        if trimmed.starts_with("## ") || (trimmed.starts_with('[') && trimmed.contains("]:")) {
            break;
        }
        body.push_str(line);
        body.push('\n');
    }
    body.trim().to_string()
}

/// Rewrite a changelog: replace the `[Unreleased]` body with a fresh empty one,
/// insert a `## [version] - date` section carrying `assembled` beneath it, and
/// add the version's link reference. Pure over strings so the splice is tested
/// without touching the filesystem.
fn rewrite(changelog: &str, version: &str, date: &str, assembled: &str) -> String {
    let mut out = String::new();
    // 0: before [Unreleased]; 1: inside the old body (skipped); 2: after.
    let mut state = 0u8;

    for line in changelog.lines() {
        match state {
            0 => {
                out.push_str(line);
                out.push('\n');
                if line.trim_start().starts_with("## [Unreleased]") {
                    out.push('\n');
                    out.push_str(&format!("## [{version}] - {date}\n\n"));
                    out.push_str(assembled);
                    out.push_str("\n\n");
                    state = 1;
                }
            }
            1 => {
                let trimmed = line.trim_start();
                if trimmed.starts_with("## ")
                    || (trimmed.starts_with('[') && trimmed.contains("]:"))
                {
                    out.push_str(line);
                    out.push('\n');
                    state = 2;
                }
                // Otherwise this is old [Unreleased] body: consumed.
            }
            _ => {
                out.push_str(line);
                out.push('\n');
            }
        }
    }

    // Add the version's link reference beneath the [Unreleased] one, once.
    let vlink = format!("[{version}]: {REPO}/releases/tag/v{version}");
    if !out.contains(&vlink) {
        if let Some(pos) = out.find("[Unreleased]:") {
            let line_end = out[pos..].find('\n').map(|i| pos + i).unwrap_or(out.len());
            out.insert_str(line_end, &format!("\n{vlink}"));
        } else {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&vlink);
            out.push('\n');
        }
    }

    out
}

/// Read the fragments under `changelog.d/`, sorted by file name for a
/// deterministic assembly. `README.md` is documentation, not a fragment.
fn read_fragments(root: &Path) -> io::Result<Vec<(String, String)>> {
    let dir = root.join("changelog.d");
    let mut paths: Vec<_> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .filter(|p| p.file_name().is_some_and(|n| n != "README.md"))
        .collect();
    paths.sort();

    let mut out = Vec::new();
    for path in paths {
        // The section is the second-to-last dot-separated component of the
        // file name: `S17-steam.added.md` -> `added`.
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let section = name
            .strip_suffix(".md")
            .and_then(|s| s.rsplit('.').next())
            .unwrap_or("")
            .to_lowercase();
        let content = fs::read_to_string(&path)?;
        out.push((section, content));
    }
    Ok(out)
}

/// The fragment file paths, sorted, for removal after a release assembly.
fn fragment_paths(root: &Path) -> io::Result<Vec<std::path::PathBuf>> {
    let dir = root.join("changelog.d");
    let mut paths: Vec<_> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .filter(|p| p.file_name().is_some_and(|n| n != "README.md"))
        .collect();
    paths.sort();
    Ok(paths)
}

/// Whether a string is a plain `X.Y.Z` release version: exactly three
/// dot-separated, non-empty, all-ASCII-digit components. The branch name, the
/// changelog heading, and the release tag link are all built from this, so a
/// malformed value such as `1.2.3junk` or `1.2.3.4` must be rejected before any
/// of them is written.
fn is_release_version(v: &str) -> bool {
    let parts: Vec<&str> = v.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// Whether a string is an ISO `YYYY-MM-DD` date with a plausible month and day.
/// This is a typo guard, not a full calendar check: it rejects `2026-8-12` and
/// `Aug 12` while accepting any real date.
fn is_iso_date(d: &str) -> bool {
    let parts: Vec<&str> = d.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    let (y, m, day) = (parts[0], parts[1], parts[2]);
    if y.len() != 4 || m.len() != 2 || day.len() != 2 {
        return false;
    }
    if !y
        .bytes()
        .chain(m.bytes())
        .chain(day.bytes())
        .all(|b| b.is_ascii_digit())
    {
        return false;
    }
    let month: u32 = m.parse().unwrap_or(0);
    let dom: u32 = day.parse().unwrap_or(0);
    (1..=12).contains(&month) && (1..=31).contains(&dom)
}

/// What `run` should do.
pub enum Mode {
    /// Print the assembled body; change nothing.
    Check,
    /// Rewrite `CHANGELOG.md` and remove the consumed fragments.
    Release { version: String, date: String },
}

/// Assemble the changelog. Returns the count of problems (0 on success), or an
/// `Err` when the command could not run at all (exit 2).
pub fn run(root: &Path, mode: Mode) -> io::Result<usize> {
    // Validate release metadata before touching the filesystem, so a mistyped
    // version or date fails without writing a malformed heading or, worse,
    // deleting every fragment while reporting success. A direct
    // `cargo xtask changelog --release <version> <date>` call reaches this too.
    if let Mode::Release { version, date } = &mode {
        if !is_release_version(version) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid release version '{version}' (expected X.Y.Z)"),
            ));
        }
        if !is_iso_date(date) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid release date '{date}' (expected YYYY-MM-DD)"),
            ));
        }
    }

    let changelog_path = root.join("CHANGELOG.md");
    let changelog = fs::read_to_string(&changelog_path)?;
    let fragments = read_fragments(root)?;

    let assembled = match assemble(&unreleased_body(&changelog), &fragments) {
        Ok(body) => body,
        Err(e) => {
            eprintln!("changelog: {e}");
            return Ok(1);
        }
    };

    if assembled.trim().is_empty() {
        eprintln!("changelog: nothing to assemble (no [Unreleased] content and no fragments)");
        return Ok(1);
    }

    match mode {
        Mode::Check => {
            println!("{assembled}");
            Ok(0)
        }
        Mode::Release { version, date } => {
            let rewritten = rewrite(&changelog, &version, &date, &assembled);
            fs::write(&changelog_path, rewritten)?;

            for path in fragment_paths(root)? {
                let rel = path.strip_prefix(root).unwrap_or(&path);
                let staged = Command::new("git")
                    .current_dir(root)
                    .arg("rm")
                    .arg("--quiet")
                    .arg(rel)
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                if !staged {
                    // Not in git yet, or git absent: remove the file directly.
                    let _ = fs::remove_file(&path);
                }
            }

            println!(
                "changelog: assembled {} fragment(s) into [{}] - {} and reset [Unreleased]",
                fragments.len(),
                version,
                date
            );
            Ok(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_headed_sections() {
        let body = "### Added\n\n- one\n\n### Fixed\n\n- two\n";
        let chunks = split_sections(body);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].0, "added");
        assert!(chunks[0].1.contains("- one"));
        assert_eq!(chunks[1].0, "fixed");
    }

    #[test]
    fn assembles_in_canonical_order() {
        let frags = vec![
            ("fixed".into(), "### Fixed\n\n- a fix\n".into()),
            ("added".into(), "### Added\n\n- a feature\n".into()),
        ];
        let body = assemble("", &frags).unwrap();
        let added_at = body.find("### Added").unwrap();
        let fixed_at = body.find("### Fixed").unwrap();
        assert!(added_at < fixed_at, "Added must precede Fixed");
        assert!(body.contains("- a feature"));
        assert!(body.contains("- a fix"));
    }

    #[test]
    fn merges_two_fragments_under_one_heading() {
        let frags = vec![
            ("added".into(), "### Added\n\n- first\n".into()),
            ("added".into(), "### Added\n\n- second\n".into()),
        ];
        let body = assemble("", &frags).unwrap();
        assert_eq!(body.matches("### Added").count(), 1);
        assert!(body.contains("- first"));
        assert!(body.contains("- second"));
    }

    #[test]
    fn keeps_existing_unreleased_content() {
        let unreleased = "### Added\n\n- legacy inline entry\n";
        let frags = vec![("added".into(), "### Added\n\n- fragment entry\n".into())];
        let body = assemble(unreleased, &frags).unwrap();
        assert!(body.contains("- legacy inline entry"));
        assert!(body.contains("- fragment entry"));
        assert_eq!(body.matches("### Added").count(), 1);
    }

    #[test]
    fn highlights_lead_and_decisions_trail() {
        let frags = vec![
            (
                "decisions".into(),
                "### Decisions\n\n- **2026** a call\n".into(),
            ),
            ("added".into(), "### Added\n\n- a feature\n".into()),
            (
                "highlights".into(),
                "### Highlights\n\n- the summary\n".into(),
            ),
        ];
        let body = assemble("", &frags).unwrap();
        let hi = body.find("### Highlights").unwrap();
        let add = body.find("### Added").unwrap();
        let dec = body.find("### Decisions").unwrap();
        assert!(hi < add && add < dec);
    }

    #[test]
    fn an_unknown_section_is_an_error() {
        let frags = vec![("add".into(), "### Add\n\n- typo\n".into())];
        assert!(assemble("", &frags).is_err());
    }

    #[test]
    fn release_version_accepts_only_x_y_z() {
        assert!(is_release_version("0.2.0"));
        assert!(is_release_version("10.20.30"));
        assert!(!is_release_version("1.2.3junk"));
        assert!(!is_release_version("1.2.3.4"));
        assert!(!is_release_version("1.2"));
        assert!(!is_release_version("v1.2.3"));
        assert!(!is_release_version("1.2."));
        assert!(!is_release_version(""));
    }

    #[test]
    fn iso_date_rejects_typos() {
        assert!(is_iso_date("2026-08-12"));
        assert!(!is_iso_date("2026-8-12"));
        assert!(!is_iso_date("Aug 12"));
        assert!(!is_iso_date("2026-13-01"));
        assert!(!is_iso_date("2026-00-10"));
        assert!(!is_iso_date("2026-08-32"));
        assert!(!is_iso_date("2026/08/12"));
    }

    #[test]
    fn release_mode_rejects_bad_metadata_before_touching_files() {
        // A nonexistent root would give a NotFound on the first read; an
        // InvalidInput proves the metadata guard ran first, before any write or
        // fragment deletion.
        let root = Path::new("this/path/does/not/exist");
        let bad_version = run(
            root,
            Mode::Release {
                version: "1.2.3.4".into(),
                date: "2026-08-12".into(),
            },
        );
        assert_eq!(bad_version.unwrap_err().kind(), io::ErrorKind::InvalidInput);
        let bad_date = run(
            root,
            Mode::Release {
                version: "0.2.0".into(),
                date: "2026-8-12".into(),
            },
        );
        assert_eq!(bad_date.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn rewrite_moves_unreleased_into_a_dated_section() {
        let changelog = "\
# Changelog

## [Unreleased]

### Added

- old entry

[Unreleased]: https://github.com/h8rt3rmin8r/fragcap/commits/main
";
        let assembled = "### Added\n\n- old entry";
        let out = rewrite(changelog, "0.2.0", "2026-08-12", assembled);
        assert!(out.contains("## [Unreleased]"));
        assert!(out.contains("## [0.2.0] - 2026-08-12"));
        // The old body is gone from under [Unreleased] and now under the version.
        let unreleased_at = out.find("## [Unreleased]").unwrap();
        let version_at = out.find("## [0.2.0]").unwrap();
        let entry_at = out.find("- old entry").unwrap();
        assert!(unreleased_at < version_at && version_at < entry_at);
        // The link reference for the version is present, and Unreleased's survives.
        assert!(out.contains("[0.2.0]: https://github.com/h8rt3rmin8r/fragcap/releases/tag/v0.2.0"));
        assert!(out.contains("[Unreleased]: https://github.com/h8rt3rmin8r/fragcap/commits/main"));
    }

    #[test]
    fn unreleased_body_reads_only_that_section() {
        let changelog = "\
# Preamble

## [Unreleased]

### Added

- the entry

## [0.1.0] - 2026-08-08

### Added

- older
";
        let body = unreleased_body(changelog);
        assert!(body.contains("- the entry"));
        assert!(!body.contains("- older"));
    }
}
