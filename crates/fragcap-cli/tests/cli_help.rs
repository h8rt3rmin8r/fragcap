// SPDX-License-Identifier: Apache-2.0

//! User-facing `--help` wraps, and carries no internal development vocabulary
//! and no argument-parser implementation note (issues #66, #67, #176, #177,
//! #178).
//!
//! The page set is enumerated from clap's own command tree through
//! [`fragcap_cli::command`], never from a hand-written list. That is the whole
//! correction over the previous guard: issue #67 closed with a test that
//! checked three pages out of twenty-nine against a hardcoded token list, so
//! nine leaking pages were never looked at, the bare `(S051)` form matched
//! nothing even on a page that was covered, and the regression came back as
//! issue #178. A new subcommand now inherits every assertion below on the day
//! it is declared.

mod common;

use common::run;
use regex::Regex;

/// The hard wrap limit, matching `max_term_width` on the root command.
const MAX_WIDTH: usize = 100;

/// Every `--help` path on the command surface, as a list of argv fragments.
///
/// Walks the clap command tree depth-first. clap's generated `help` subcommand
/// is skipped: it is not fragcap's text, it exists at every level, and it would
/// double the page count with pages nobody authored.
fn help_pages() -> Vec<Vec<String>> {
    fn walk(cmd: &clap::Command, prefix: &[String], out: &mut Vec<Vec<String>>) {
        out.push(prefix.to_vec());
        for sub in cmd.get_subcommands() {
            let name = sub.get_name();
            if name == "help" {
                continue;
            }
            let mut next = prefix.to_vec();
            next.push(name.to_string());
            walk(sub, &next, out);
        }
    }
    let mut out = Vec::new();
    walk(&fragcap_cli::command(), &[], &mut out);
    out
}

/// Render one page, or fail.
///
/// A page that does not render is a failure, never a skip: a page that dropped
/// out of coverage by breaking would leave the surface unguarded exactly where
/// something is already wrong (P-4, applied to documentation coverage).
fn render(page: &[String]) -> String {
    let mut args: Vec<&str> = page.iter().map(String::as_str).collect();
    args.push("--help");
    let (code, out, err) = run(&args);
    assert_eq!(
        code,
        0,
        "`fragcap {}--help` must render and exit 0; got {code}\nstderr:\n{err}",
        page.iter().map(|s| format!("{s} ")).collect::<String>()
    );
    assert!(
        !out.is_empty(),
        "`fragcap {}--help` rendered nothing",
        page.iter().map(|s| format!("{s} ")).collect::<String>()
    );
    out
}

/// A page's label for a failure message.
fn label(page: &[String]) -> String {
    if page.is_empty() {
        "fragcap --help".to_string()
    } else {
        format!("fragcap {} --help", page.join(" "))
    }
}

/// The internal-vocabulary patterns, matched against the whole page.
///
/// Patterns, not a token list. The previous guard held `["S15", "S16", "S17",
/// "slice S"]`, written when those were the current slices, so `S051` through
/// `S055` slipped past every entry but one and the bare parenthesised `(S051)`
/// form slipped past all of them.
///
/// The Cargo-feature entries match the *phrasing* that names a build feature to
/// a user, never the declared feature names. Four of the five workspace
/// features (`live`, `net`, `targets`, `etw`) are ordinary English words that
/// appear legitimately in help prose: matching `net` bare would fire on
/// "network" and `targets` on most of the `targets` pages. A rule that cries
/// wolf earns an exception list, and an exception list is what decayed into the
/// token set above.
fn leak_patterns() -> Vec<(Regex, &'static str)> {
    vec![
        (Regex::new(r"slice S\d+").unwrap(), "a slice identifier"),
        (
            Regex::new(r"\bS\d{2,3}\b").unwrap(),
            "a bare slice identifier",
        ),
        (
            Regex::new(r"[Ss]ection \d+\.\d+").unwrap(),
            "a specification section reference",
        ),
        (
            Regex::new(r"Appendix [A-Z]\b").unwrap(),
            "an appendix letter",
        ),
        (
            // Plus, not one digit. The constitution has eleven principles, so a
            // single-digit pattern silently exempts P-10 and P-11, which is the
            // hand-maintained-set failure this guard replaced, wearing a hat
            // (review of PR #189).
            Regex::new(r"\bP-\d+\b").unwrap(),
            "a constitution principle identifier",
        ),
        (
            Regex::new(r"`[A-Za-z][\w-]*` feature").unwrap(),
            "a Cargo feature name",
        ),
        (
            Regex::new(r"\bthe [a-z][\w-]* feature\b").unwrap(),
            "a Cargo feature name",
        ),
        (
            Regex::new(r"\bfeature `[A-Za-z][\w-]*`").unwrap(),
            "a Cargo feature name",
        ),
        (Regex::new(r"\bTier \d\b").unwrap(), "a bare tier number"),
    ]
}

/// Collapse every whitespace run to one space.
///
/// Load-bearing, not tidying. Help wraps since slice S062, so a leak can be
/// split across a line break: `fragcap extcap --help` rendered `specification
/// section` at the end of one line and `14.5` at the start of the next, and a
/// line-by-line scan reported that page clean while it still leaked. A guard
/// defeated by the wrapping shipped in the same slice would be issue #178
/// repeating itself one slice after being fixed.
fn normalize(page: &str) -> String {
    page.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn every_help_page_wraps_within_the_limit() {
    let mut failures = Vec::new();
    for page in help_pages() {
        let out = render(&page);
        for (i, line) in out.lines().enumerate() {
            let width = line.chars().count();
            if width > MAX_WIDTH {
                failures.push(format!(
                    "{}: line {} is {width} columns (limit {MAX_WIDTH}):\n    {line}",
                    label(&page),
                    i + 1
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} help line(s) exceed {MAX_WIDTH} columns:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn no_help_page_leaks_internal_vocabulary() {
    let patterns = leak_patterns();
    let mut failures = Vec::new();
    for page in help_pages() {
        let normalized = normalize(&render(&page));
        for (re, what) in &patterns {
            if let Some(m) = re.find(&normalized) {
                failures.push(format!("{}: leaks {what}: {:?}", label(&page), m.as_str()));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} internal-vocabulary leak(s) in user-facing help:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn no_help_page_leaks_parser_internals() {
    let mut failures = Vec::new();
    for page in help_pages() {
        let normalized = normalize(&render(&page));
        for note in ["value_parser", "value_delimiter", "Vec<String>"] {
            if normalized.contains(note) {
                failures.push(format!("{}: leaks the parser note {note}", label(&page)));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} parser-implementation note(s) in user-facing help:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn the_page_set_covers_the_whole_command_surface() {
    let pages = help_pages();
    // The root plus every subcommand at every depth. Asserted as a floor rather
    // than an exact count so adding a subcommand does not fail this test for
    // the wrong reason; the point is that enumeration reaches past the top
    // level, which the hand-written list it replaced did not.
    assert!(
        pages.len() >= 29,
        "expected the whole command surface, found {} page(s)",
        pages.len()
    );
    assert!(pages.iter().any(|p| p.is_empty()), "the root page");
    for expected in [
        vec!["capture"],
        vec!["targets"],
        vec!["targets", "add"],
        vec!["catalog", "seed-signatures"],
        vec!["extcap", "install"],
        vec!["technologies"],
    ] {
        assert!(
            pages.iter().any(|p| p == &expected),
            "the page set must reach `fragcap {}`",
            expected.join(" ")
        );
    }
    assert!(
        !pages.iter().any(|p| p.iter().any(|s| s == "help")),
        "clap's generated `help` command is not fragcap's text and is not a page"
    );
}

#[test]
fn launch_help_describes_the_stored_target_not_the_flag_argument() {
    let out = render(&["capture".to_string()]);
    let normalized = normalize(&out);
    assert!(
        normalized.contains("launcher"),
        "the --launch help describes the real managed launch: {out}"
    );
    assert!(
        !normalized.contains("deferred"),
        "the --launch help is not a 'deferred to slice' note: {out}"
    );
    // Issue #181: the operator read "requires a `--target` carrying a Steam app
    // id", passed the app id to `--target`, and was refused, because a bare
    // integer there is unconditionally a listing row index. The help must not
    // admit that reading.
    assert!(
        !normalized.contains("--target` carrying a Steam app id")
            && !normalized.contains("--target carrying a Steam app id"),
        "the --launch help must describe the stored target, not the flag argument: {out}"
    );
}

#[test]
fn a_bare_integer_is_documented_as_unconditionally_a_row_index() {
    let normalized = normalize(&render(&["capture".to_string()]));
    // Issue #181. Both the positional selector and `--target` already listed "a
    // 1-based row index" as one accepted form among handle and name, and that
    // phrasing is what invited the defect: a list of accepted forms reads as
    // though a number might also be tried as something else. `is_row_index`
    // gates first and never falls through, so the fact the page owes the reader
    // is the exclusivity, not the membership.
    //
    // Asserted twice, once for the positional and once for `--target`, since
    // they carry separate text and a fix applied to only one leaves the other
    // inviting the same reading.
    let says_only = normalized.matches("always a row index").count();
    assert!(
        says_only >= 2,
        "the positional selector and --target must each say a bare integer is \
         always a row index, never a handle, name, or app id; found {says_only} \
         statement(s) in:\n{normalized}"
    );
    assert!(
        normalized.contains("row index, not this"),
        "--id must keep its own statement of the collision: {normalized}"
    );
}
