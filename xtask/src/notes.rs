// SPDX-License-Identifier: Apache-2.0

//! Short, separately authored release notes.
//!
//! A release page is a summary, not a second copy of `CHANGELOG.md`. Release
//! preparation therefore produces one AI-written file under `release-notes/`,
//! and this command validates that file before the tag workflow publishes it.

use std::fs;
use std::path::Path;

const MAX_CHARACTERS: usize = 1_400;
const MAX_NONEMPTY_LINES: usize = 12;

fn changelog_url(version: &str) -> String {
    format!("https://github.com/h8rt3rmin8r/fragcap/blob/v{version}/CHANGELOG.md")
}

fn required_closing_line(version: &str) -> String {
    format!(
        "For the complete change history, implementation details, and issue references, see the [full v{version} changelog]({}).",
        changelog_url(version)
    )
}

/// Validate a separately authored release summary.
///
/// The length budgets keep the rendered body within one ordinary desktop
/// screen. Requiring the fixed closing guidance keeps the complete record one
/// click away without letting the workflow fall back to copying that record.
pub fn validate(body: &str, version: &str) -> Result<(), Vec<String>> {
    let mut problems = Vec::new();
    let trimmed = body.trim();
    let lines: Vec<_> = trimmed.lines().collect();
    let nonempty = lines.iter().filter(|line| !line.trim().is_empty()).count();
    let characters = trimmed.chars().count();

    if trimmed.is_empty() {
        problems.push("release notes are empty".to_string());
        return Err(problems);
    }
    if lines.first().copied() != Some("# Highlights") {
        problems.push("release notes must begin with '# Highlights'".to_string());
    }
    if lines.iter().skip(1).any(|line| line.starts_with('#')) {
        problems.push("release notes may contain only the '# Highlights' heading".to_string());
    }
    if characters > MAX_CHARACTERS {
        problems.push(format!(
            "release notes contain {characters} characters; the limit is {MAX_CHARACTERS}"
        ));
    }
    if nonempty > MAX_NONEMPTY_LINES {
        problems.push(format!(
            "release notes contain {nonempty} non-empty lines; the limit is {MAX_NONEMPTY_LINES}"
        ));
    }
    if crate::changelog::normalize_markdown(trimmed).trim_end() != trimmed {
        problems.push(
            "release notes must keep each paragraph and list item on one source line".to_string(),
        );
    }
    if lines.last().copied() != Some(required_closing_line(version).as_str()) {
        problems
            .push("release notes must end with the standard full-changelog guidance".to_string());
    }
    for heading in [
        "### Added",
        "### Changed",
        "### Deprecated",
        "### Removed",
        "### Fixed",
        "### Security",
        "### Decisions",
    ] {
        if trimmed.contains(heading) {
            problems.push(format!(
                "release notes contain changelog section heading '{heading}'; summarize instead"
            ));
        }
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

fn release_section<'a>(changelog: &'a str, version: &str) -> Option<&'a str> {
    let wanted = format!("## [{version}]");
    let start = changelog
        .lines()
        .position(|line| line.starts_with(&wanted))?;
    let mut offset = 0usize;
    let mut body_start = None;
    for (index, line) in changelog.split_inclusive('\n').enumerate() {
        if index == start {
            body_start = Some(offset + line.len());
            break;
        }
        offset += line.len();
    }
    let body_start = body_start?;
    let rest = &changelog[body_start..];
    let body_end = rest
        .match_indices("\n## [")
        .next()
        .map(|(index, _)| index)
        .unwrap_or(rest.len());
    Some(rest[..body_end].trim())
}

fn content_units(markdown: &str) -> Vec<Vec<String>> {
    crate::changelog::normalize_markdown(markdown)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            line.strip_prefix("- ")
                .or_else(|| line.strip_prefix("* "))
                .or_else(|| line.strip_prefix("+ "))
                .unwrap_or(line)
        })
        .map(|line| {
            line.split(|character: char| !character.is_alphanumeric())
                .filter(|word| !word.is_empty())
                .map(str::to_lowercase)
                .collect::<Vec<_>>()
        })
        .filter(|words| !words.is_empty())
        .collect()
}

fn contains_words(haystack: &[String], needle: &[String]) -> bool {
    needle.len() <= haystack.len()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn copies_changelog_text(body: &str, changelog: &str, version: &str) -> bool {
    const MIN_SUBSTANTIVE_WORDS: usize = 8;

    let Some(section) = release_section(changelog, version) else {
        return false;
    };
    let closing = required_closing_line(version);
    let summary = body
        .trim()
        .strip_prefix("# Highlights")
        .unwrap_or(body)
        .trim()
        .strip_suffix(&closing)
        .unwrap_or(body)
        .trim();
    let summary_units = content_units(summary);
    let changelog_units = content_units(section);
    let summary_all = summary_units.iter().flatten().collect::<Vec<_>>();
    let changelog_all = changelog_units.iter().flatten().collect::<Vec<_>>();

    if !summary_all.is_empty() && summary_all == changelog_all {
        return true;
    }

    summary_units.iter().any(|summary_unit| {
        changelog_units.iter().any(|changelog_unit| {
            let shorter = summary_unit.len().min(changelog_unit.len());
            shorter >= MIN_SUBSTANTIVE_WORDS
                && (contains_words(summary_unit, changelog_unit)
                    || contains_words(changelog_unit, summary_unit))
        })
    })
}

/// Print the validated release notes for `version`.
pub fn run(root: &Path, version: &str) -> usize {
    let path = root.join("release-notes").join(format!("v{version}.md"));
    let body = match fs::read_to_string(&path) {
        Ok(body) => body,
        Err(error) => {
            eprintln!(
                "notes: could not read {}: {error}; release preparation must add an AI-written summary",
                path.display()
            );
            return 1;
        }
    };

    if let Err(problems) = validate(&body, version) {
        for problem in problems {
            eprintln!("notes: {}: {problem}", path.display());
        }
        return 1;
    }

    let changelog_path = root.join("CHANGELOG.md");
    let changelog = match fs::read_to_string(&changelog_path) {
        Ok(changelog) => changelog,
        Err(error) => {
            eprintln!(
                "notes: could not read {}: {error}",
                changelog_path.display()
            );
            return 1;
        }
    };
    if release_section(&changelog, version).is_none() {
        eprintln!("notes: CHANGELOG.md has no [{version}] release section");
        return 1;
    }
    if copies_changelog_text(&body, &changelog, version) {
        eprintln!(
            "notes: {} copies substantive changelog text; synthesize the highlights instead",
            path.display()
        );
        return 1;
    }

    print!("{}", body.trim_end());
    println!();
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid(version: &str) -> String {
        format!(
            "# Highlights\n\nA concise AI-written overview.\n\n- One important outcome.\n- Another important outcome.\n\n{}\n",
            required_closing_line(version)
        )
    }

    #[test]
    fn accepts_a_short_summary_with_full_changelog_guidance() {
        assert_eq!(validate(&valid("0.9.0"), "0.9.0"), Ok(()));
    }

    #[test]
    fn rejects_a_changelog_shaped_body() {
        let body = valid("0.9.0").replace(
            "A concise AI-written overview.",
            "### Added\n\nA copied changelog section.",
        );
        let problems = validate(&body, "0.9.0").unwrap_err().join("\n");
        assert!(problems.contains("only the '# Highlights' heading"));
        assert!(problems.contains("summarize instead"));
    }

    #[test]
    fn rejects_missing_or_wrong_changelog_guidance() {
        let body = valid("0.9.0").replace("v0.9.0", "v0.8.0");
        assert!(validate(&body, "0.9.0")
            .unwrap_err()
            .iter()
            .any(|problem| problem.contains("full-changelog guidance")));
    }

    #[test]
    fn rejects_notes_beyond_the_screen_budget() {
        let long = format!(
            "# Highlights\n\n{}\n\n{}\n",
            "x".repeat(MAX_CHARACTERS),
            required_closing_line("0.9.0")
        );
        assert!(validate(&long, "0.9.0")
            .unwrap_err()
            .iter()
            .any(|problem| problem.contains("characters")));

        let mut many_lines = String::from("# Highlights\n\n");
        for _ in 0..MAX_NONEMPTY_LINES {
            many_lines.push_str("- item\n");
        }
        many_lines.push('\n');
        many_lines.push_str(&required_closing_line("0.9.0"));
        assert!(validate(&many_lines, "0.9.0")
            .unwrap_err()
            .iter()
            .any(|problem| problem.contains("non-empty lines")));
    }

    #[test]
    fn rejects_hard_wrapped_release_notes() {
        let body = valid("0.9.0").replace(
            "A concise AI-written overview.",
            "A concise AI-written\noverview.",
        );
        assert!(validate(&body, "0.9.0")
            .unwrap_err()
            .iter()
            .any(|problem| problem.contains("one source line")));
    }

    #[test]
    fn rejects_a_carbon_copy_of_the_release_changelog() {
        let changelog = "# Changelog\n\n## [0.9.0] - 2026-09-05\n\n### Added\n\n- One result.\n- Another result.\n\n## [0.8.0] - 2026-08-30\n\nOlder.\n";
        let body = format!(
            "# Highlights\n\n- One result.\n- Another result.\n\n{}\n",
            required_closing_line("0.9.0")
        );
        assert!(copies_changelog_text(&body, changelog, "0.9.0"));
        assert!(!copies_changelog_text(&valid("0.9.0"), changelog, "0.9.0"));
    }

    #[test]
    fn rejects_selected_or_reordered_changelog_entries() {
        let changelog = "# Changelog\n\n## [0.9.0] - 2026-09-05\n\n### Added\n\n- Capture now preserves every accepted datagram with exact direction and sequence identity.\n- Release archives now contain independently verified checksums for every supported Windows target.\n\n## [0.8.0] - 2026-08-30\n\nOlder.\n";
        let reordered = format!(
            "# Highlights\n\n- Release archives now contain independently verified checksums for every supported Windows target.\n- Capture now preserves every accepted datagram with exact direction and sequence identity.\n\n{}\n",
            required_closing_line("0.9.0")
        );
        let excerpt = format!(
            "# Highlights\n\n- Preserves every accepted datagram with exact direction and sequence identity.\n\n{}\n",
            required_closing_line("0.9.0")
        );

        assert!(copies_changelog_text(&reordered, changelog, "0.9.0"));
        assert!(copies_changelog_text(&excerpt, changelog, "0.9.0"));
    }
}
