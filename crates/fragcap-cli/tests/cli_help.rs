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
    // No page count is asserted. A count is a hand-maintained number, which is
    // the class of defect this guard exists to remove, and slice S063 proved it
    // by removing one command and failing this test for a reason that had
    // nothing to do with help. What matters is structural: enumeration reaches
    // past the top level, which the hand-written list this replaced did not.
    assert!(
        pages.iter().any(|p| p.len() >= 2),
        "enumeration must reach nested subcommands, not just the top level"
    );
    assert!(pages.iter().any(|p| p.is_empty()), "the root page");
    for expected in [
        vec!["capture"],
        vec!["targets"],
        vec!["targets", "add"],
        vec!["catalog", "seed"],
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
        normalized.contains("protocol handler")
            && normalized.contains("direct target")
            && normalized.contains("no command shell"),
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
/// No subcommand requires a path to a store fragcap owns.
///
/// Slice S058 (issue #157) established the contract for `targets`: a store path
/// is an override, never a requirement, because fragcap installs these stores,
/// manages them, and already knows how to find them. S058's FR-005 scoped the
/// `catalog`, `technologies`, and `targets discover` flags out, and nothing was
/// filed to pick them up, so they survived as nine required arguments asking the
/// user to type a path to an internal component (issue #179).
///
/// Enumerated rather than listed, for the same reason the leak check is: a new
/// subcommand must inherit the rule instead of quietly reintroducing the defect.
#[test]
fn no_subcommand_requires_a_store_path() {
    const STORE_ARGS: [&str; 3] = ["db", "catalog-db", "local-db"];

    /// Match on the long flag as well as the id. clap's `get_id()` is the field
    /// name, so `catalog_db` never equals the flag `catalog-db`; matching on the
    /// id alone silently missed three of the eight required flags on the first
    /// run of this test.
    fn is_store_arg(arg: &clap::Arg) -> bool {
        let id = arg.get_id().as_str().replace('_', "-");
        let long = arg.get_long().unwrap_or_default();
        STORE_ARGS.contains(&id.as_str()) || STORE_ARGS.contains(&long)
    }

    fn walk(cmd: &clap::Command, path: &[String], out: &mut Vec<String>) {
        for arg in cmd.get_arguments() {
            let id = arg.get_long().unwrap_or_else(|| arg.get_id().as_str());
            if is_store_arg(arg) && arg.is_required_set() {
                let where_ = if path.is_empty() {
                    "fragcap".to_string()
                } else {
                    format!("fragcap {}", path.join(" "))
                };
                out.push(format!("{where_}: `--{id}` is required"));
            }
        }
        for sub in cmd.get_subcommands() {
            if sub.get_name() == "help" {
                continue;
            }
            let mut next = path.to_vec();
            next.push(sub.get_name().to_string());
            walk(sub, &next, out);
        }
    }

    let mut failures = Vec::new();
    walk(&fragcap_cli::command(), &[], &mut failures);
    assert!(
        failures.is_empty(),
        "{} store path(s) are required rather than overrides:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// ---------------------------------------------------------------------------
// The audit-and-gate additions of slice S070 (issue #183). The four checks
// below are the gate the issue itself specifies: length, defaults,
// cross-reference, and spec agreement. None existed before this slice.
// ---------------------------------------------------------------------------

/// Extract the option/argument paragraphs of a `--help` (long) rendering.
///
/// clap's long help separates each entry with a blank line; a short (`-h`)
/// rendering does not, which is exactly the distinction
/// [`short_help_continuations`] depends on.
fn option_paragraphs(page: &str) -> Vec<&str> {
    page.split("\n\n").collect()
}

/// The paragraph whose first line names `flag`, or `None`.
///
/// The very first entry under `Arguments:`/`Options:` glues that section
/// header onto its own paragraph (there is no blank line between a section
/// heading and its first row), so the header itself, not the flag, is that
/// paragraph's first line; skip it if present rather than only ever checking
/// line zero.
fn find_flag_paragraph<'a>(page: &'a str, flag: &str) -> Option<&'a str> {
    option_paragraphs(page).into_iter().find(|p| {
        p.lines()
            .find(|l| *l != "Arguments:" && *l != "Options:")
            .is_some_and(|l| l.trim_start().starts_with(flag))
    })
}

/// FR-012, in two halves, split by what is actually derivable.
///
/// **Structural half**: any `Arg` on `capture` carrying a clap-visible
/// default (`default_value_t`, which `--scope` and `--direction` both use)
/// is walked from `fragcap_cli::command()` itself via `get_default_values()`,
/// exactly as `no_subcommand_requires_a_store_path` walks the tree for its
/// own check. A newly `default_value_t`-defaulted option is covered with no
/// edit here.
///
/// **Prose-only half**: `--roles` and `--wait` resolve their default inside
/// `assemble.rs` on an `Option<T>` field clap sees as carrying no default at
/// all (`.or_else(|| defaults.roles()...)`; a bare `None` passthrough for
/// `--wait`). No clap `Arg` metadata expresses this, so it cannot be derived
/// from the command tree the way the structural half is; `PROSE_ONLY_DEFAULTS`
/// is the minimal remaining hand-maintained list, and each entry is tied in
/// its own comment to the exact `assemble.rs` site it mirrors so drift is a
/// one-file diff to check, not a re-audit. Review of PR #198 pushed on this:
/// the original version of this check hand-listed all four options
/// (including `--mode`/`--direction`, which do not need it); this is the
/// corrected, minimized version.
const PROSE_ONLY_DEFAULTS: &[&str] = &[
    // assemble.rs:126,203: `.or_else(|| defaults.roles().map(|r| r.to_vec()))`
    "--roles",
    // assemble.rs:143: `acquisition_timeout: args.wait`, where `None` means
    // no timeout.
    "--wait",
    // assemble.rs's `resolve_mode`: an explicit `--mode` wins, else a
    // profile-declared mode, else `file` (the `None` arm was preserved
    // rather than removed; see plan.md's Phase 0 note on the reverted
    // `default_value_t` attempt, which broke a real, tested profile-mode
    // fallback). `mode` stays `Option<ModeArg>` with no clap-level default,
    // so it is not reachable via `get_default_values()` either.
    "--mode",
];

#[test]
fn every_defaulted_option_states_its_default() {
    let out = render(&["capture".to_string()]);
    let mut failures = Vec::new();

    let capture = fragcap_cli::command()
        .find_subcommand("capture")
        .expect("`capture` must exist")
        .clone();
    for arg in capture.get_arguments() {
        if arg.get_default_values().is_empty() {
            continue;
        }
        let Some(long) = arg.get_long() else { continue };
        let flag = format!("--{long}");
        let Some(block) = find_flag_paragraph(&out, &flag) else {
            failures.push(format!("{flag}: no `capture --help` block found at all"));
            continue;
        };
        if !normalize(block).to_lowercase().contains("default") {
            failures.push(format!("{flag}: states no default:\n{block}"));
        }
    }

    for flag in PROSE_ONLY_DEFAULTS {
        let Some(block) = find_flag_paragraph(&out, flag) else {
            failures.push(format!("{flag}: no `capture --help` block found at all"));
            continue;
        };
        if !normalize(block).to_lowercase().contains("default") {
            failures.push(format!("{flag}: states no default:\n{block}"));
        }
    }

    assert!(
        failures.is_empty(),
        "{} defaulted option(s) do not state their default in `capture --help`:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Every `"literal" =>` (or `"a" | "b" =>`) match-arm key inside the named
/// function's body in `args.rs`, from `fn {name}(` to the next top-level
/// `fn `.
fn match_arm_keys(args_rs: &str, name: &str) -> std::collections::BTreeSet<String> {
    let start_marker = format!("fn {name}(");
    let start = args_rs
        .find(&start_marker)
        .unwrap_or_else(|| panic!("args.rs must define `{name}`"));
    let after = &args_rs[start..];
    let body_end = after[start_marker.len()..]
        .find("\nfn ")
        .map(|i| i + start_marker.len())
        .unwrap_or(after.len());
    let body = &after[..body_end];
    Regex::new(r#""([a-z][a-z-]*)"\s*(?:\||=>)"#)
        .unwrap()
        .captures_iter(body)
        .map(|c| c[1].to_string())
        .collect()
}

/// FR-002/T017: every scheme `parse_destination` accepts and every modifier
/// key `apply_option` accepts must appear in `capture --sink`'s rendered
/// help, derived from `args.rs`'s own match-arm keys (via
/// [`match_arm_keys`]) rather than a copy of the list hand-typed into this
/// test, so the help text and the parser cannot drift independently. `tcp`
/// is handled separately in `parse_destination` (a `strip_prefix("tcp://")`
/// check, not a match arm) and is added explicitly for that reason.
#[test]
fn sink_help_names_every_scheme_and_modifier_the_parser_accepts() {
    let args_rs = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/args.rs"),
    )
    .expect("args.rs must be readable");

    let mut schemes = match_arm_keys(&args_rs, "parse_destination");
    schemes.insert("tcp".to_string());
    let modifiers = match_arm_keys(&args_rs, "apply_option");

    let out = render(&["capture".to_string()]);
    let block = find_flag_paragraph(&out, "--sink").expect("`--sink` must have a --help block");
    let normalized = normalize(block);

    let mut failures = Vec::new();
    for scheme in &schemes {
        if !normalized.contains(scheme) {
            failures.push(format!(
                "scheme `{scheme}` is accepted but not named in --sink help"
            ));
        }
    }
    for modifier in &modifiers {
        if !normalized.contains(modifier) {
            failures.push(format!(
                "modifier `{modifier}` is accepted but not named in --sink help"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} sink scheme/modifier drift(s) between args.rs and --help:\n{}\n\nrendered block:\n{block}",
        failures.len(),
        failures.join("\n")
    );
}

/// One `///`-commented block in `cli.rs`: consecutive doc-comment lines
/// (`///` prefix and up to one leading space stripped), plus the 1-based
/// source line the block starts on.
struct DocBlock {
    start_line: usize,
    lines: Vec<String>,
}

/// Every doc-comment block in `cli.rs` that documents something clap
/// actually renders as `--help`/`-h` text: an `#[arg(...)]` field, an enum
/// variant, or similar. Excludes a struct's or enum's own outer doc comment
/// when that type is used as an enum variant's payload or via
/// `#[command(subcommand)]`/`#[command(flatten)]` (verified:
/// `doctor --help`'s about text is the `Doctor` variant's own doc, never
/// `DoctorArgs`' struct doc, which clap never shows anywhere).
fn doc_blocks(source: &str) -> Vec<DocBlock> {
    let lines: Vec<&str> = source.lines().collect();
    let mut blocks = Vec::new();
    let mut current: Option<DocBlock> = None;
    let mut current_end: usize = 0;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("///") {
            let content = rest.strip_prefix(' ').unwrap_or(rest).to_string();
            current
                .get_or_insert_with(|| DocBlock {
                    start_line: i + 1,
                    lines: Vec::new(),
                })
                .lines
                .push(content);
            current_end = i;
        } else if let Some(block) = current.take() {
            if renders_as_user_help(&lines, current_end + 1) {
                blocks.push(block);
            }
        }
    }
    if let Some(block) = current {
        if renders_as_user_help(&lines, current_end + 1) {
            blocks.push(block);
        }
    }
    blocks
}

/// Whether the source line immediately after a doc-comment block (0-based
/// `next_line`) belongs to something clap renders, skipping past any
/// `#[derive(...)]`/`#[command(...)]` attribute lines first.
fn renders_as_user_help(lines: &[&str], next_line: usize) -> bool {
    let mut i = next_line;
    let mut saw_subcommand_or_flatten = false;
    while let Some(line) = lines.get(i).map(|l| l.trim_start()) {
        if line.starts_with("#[command(subcommand)]") || line.starts_with("#[command(flatten)]") {
            saw_subcommand_or_flatten = true;
        }
        if line.starts_with('#') {
            i += 1;
            continue;
        }
        break;
    }
    if saw_subcommand_or_flatten {
        return false;
    }
    let Some(declaration) = lines.get(i).map(|l| l.trim_start()) else {
        return true;
    };
    !(declaration.starts_with("pub struct")
        || declaration.starts_with("pub enum")
        || declaration.starts_with("impl "))
}

/// Whether `text` conveys more than one sentence: a `.`, `?`, or `!` followed
/// by whitespace and a capital letter, or by the end of the text, outside
/// backticked code, counted twice. A single terminator with nothing
/// capitalized after it (`catalog.db`, `%APPDATA%\fragcap\catalog.db`, an
/// abbreviation like `e.g.` where the letter after the embedded period is
/// lowercase) does not count, and neither does a lone `?`/`!` used as
/// punctuation rather than a sentence break, since it must also be followed
/// by a capital or the text's end. Review of PR #198 correctly flagged an
/// earlier version of this detector as only recognizing `.`, which passed a
/// two-sentence doc comment joined by `?` or `!`; both are handled here too.
fn has_two_sentences(text: &str) -> bool {
    let mut in_backtick = false;
    let mut sentence_ends = 0;
    let chars: Vec<char> = text.chars().collect();
    for i in 0..chars.len() {
        match chars[i] {
            '`' => in_backtick = !in_backtick,
            '.' | '?' | '!' if !in_backtick => {
                let rest: String = chars[i + 1..].iter().collect();
                let trimmed = rest.trim_start();
                let ends_sentence = trimmed.is_empty()
                    || (rest.starts_with(char::is_whitespace)
                        && trimmed
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_ascii_uppercase()));
                if ends_sentence {
                    sentence_ends += 1;
                    if sentence_ends >= 2 {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

/// FR-011: every option's `-h` (short) rendering is one line, over every page
/// `help_pages()` discovers.
///
/// This runs in two parts, and the split is deliberate, backed by two
/// separate rounds of measurement against the real binary. **Every page's
/// `-h` is rendered** (satisfying "render `-h`... iterate `help_pages()`"
/// literally), confirming each one exits 0. The actual pass/fail judgment,
/// though, comes from a **source-level scan** of `cli.rs`'s doc comments,
/// not from whether the rendered `-h` text happens to wrap: a fully rendered
/// check was tried twice and measured wrong both times. First attempt: raw
/// continuation-line detection flagged `--target`'s own already-correctly-
/// split, single-sentence summary, because `capture -h`'s description column
/// (pushed right by *other* long flag names on the same page) is narrow
/// enough to wrap even one short sentence. Second attempt: cross-checking
/// each continuation against that same page's `--help` rendering for an
/// internal blank-line split fixed the `--target` case, but then flagged
/// `extcap -h`'s `--extcap-interfaces`, `--capture`, and even clap's own
/// auto-generated, unauthorable `-h, --help` text ("Print help (see more
/// with '--help')") as violations: `extcap`'s longest flag name
/// (`--extcap-interfaces`, 21 characters) is long enough to push clap into a
/// next-line layout for the whole page, so *every* entry wraps regardless of
/// whether it has, or even could have, a second sentence to defer. Neither
/// failure mode is the defect FR-011 exists to catch (both are widths,
/// already FR-001/FR-002's job on the existing wrap test); both are caused
/// by *sibling* flags on the same page, not by the field's own doc comment,
/// which is exactly why a per-field source fact, not a per-page rendering
/// fact, is what actually decides this.
#[test]
fn every_short_help_summary_is_one_line() {
    for page in help_pages() {
        let mut args: Vec<&str> = page.iter().map(String::as_str).collect();
        args.push("-h");
        let (code, _, err) = run(&args);
        assert_eq!(
            code,
            0,
            "`fragcap {}-h` must render and exit 0; got {code}\nstderr:\n{err}",
            page.iter().map(|s| format!("{s} ")).collect::<String>()
        );
    }

    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cli.rs"),
    )
    .expect("cli.rs must be readable");
    let mut failures = Vec::new();
    for block in doc_blocks(&source) {
        let has_blank_split = block.lines.iter().any(|l| l.trim().is_empty());
        if has_blank_split {
            continue;
        }
        let joined = block.lines.join(" ");
        if has_two_sentences(&joined) {
            failures.push(format!(
                "cli.rs:{}: doc comment has more than one sentence with no blank `///` \
                 line splitting the `-h` summary from the `--help` detail:\n{joined}",
                block.start_line
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} doc comment(s) need a short/long split:\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

/// Every long-flag and short-flag string reachable from the whole command
/// tree, every real subcommand name at any depth (excluding clap's generated
/// `help`), and every bare word that is a legitimate *value* for some
/// argument rather than a command word (an enum's possible values, and the
/// individual tokens of a `value_name` hint like `"yes|no|unsure"`), so a
/// value like `all` or `signature` is never mistaken for a stale command
/// reference.
fn command_surface() -> (
    std::collections::BTreeSet<String>,
    std::collections::BTreeSet<String>,
    std::collections::BTreeSet<String>,
) {
    fn walk(
        cmd: &clap::Command,
        flags: &mut std::collections::BTreeSet<String>,
        words: &mut std::collections::BTreeSet<String>,
        value_literals: &mut std::collections::BTreeSet<String>,
    ) {
        for arg in cmd.get_arguments() {
            if let Some(long) = arg.get_long() {
                flags.insert(format!("--{long}"));
            }
            if let Some(short) = arg.get_short() {
                flags.insert(format!("-{short}"));
            }
            for pv in arg.get_possible_values() {
                value_literals.insert(pv.get_name().to_string());
            }
            if let Some(names) = arg.get_value_names() {
                for name in names {
                    for token in name.split('|') {
                        value_literals.insert(token.to_string());
                    }
                }
            }
        }
        for sub in cmd.get_subcommands() {
            if sub.get_name() == "help" {
                continue;
            }
            words.insert(sub.get_name().to_string());
            walk(sub, flags, words, value_literals);
        }
    }
    let mut flags = std::collections::BTreeSet::new();
    let mut words = std::collections::BTreeSet::new();
    let mut value_literals = std::collections::BTreeSet::new();
    let root = fragcap_cli::command();
    // The binary's own name, derived from the root `Command` itself (its
    // `#[command(name = "fragcap")]`) rather than a literal, so a rename
    // could not silently desync this from what every worked invocation in
    // help actually starts with (`` `fragcap capture <n>` ``, `` `fragcap
    // targets` ``, ...).
    words.insert(root.get_name().to_string());
    walk(&root, &mut flags, &mut words, &mut value_literals);
    // clap adds these automatically; they exist on every page but are not
    // declared `Arg`s this walk would otherwise see.
    for f in ["-h", "--help", "-V", "--version"] {
        flags.insert(f.to_string());
    }
    (flags, words, value_literals)
}

/// Whether `token` is shaped like a command word: purely lowercase ASCII
/// letters and hyphens, non-empty. Excludes anything carrying a path, a
/// placeholder (`<...>`), a scheme (`file:`), or punctuation, none of which is
/// a command-word candidate.
fn looks_like_command_word(token: &str) -> bool {
    !token.is_empty() && token.chars().all(|c| c.is_ascii_lowercase() || c == '-')
}

/// Whether `token` is shaped like a flag: `--x` or a two-character `-x`.
fn looks_like_flag(token: &str) -> bool {
    token.starts_with("--") || (token.starts_with('-') && token.chars().count() == 2)
}

/// FR-013. Every backticked flag-shaped token is checked against `flags`
/// unconditionally. Every backticked bare command-word-shaped token is
/// checked against `words` unconditionally too, *except* when it is a known
/// value literal (an enum's possible value, or a token from some `Arg`'s
/// `value_name` hint), derived structurally in [`command_surface`], not by
/// gating on the surrounding backtick span.
///
/// An earlier version of this check only treated a bare word as a candidate
/// when the whole backtick span's first word was itself a real command,
/// specifically to avoid misreading a lone value backtick (`` `target` ``,
/// `` `all` ``) as a stale command reference. Review of PR #198 correctly
/// called that circular: a genuinely stale, *standalone* reference (a bare
/// `` `watch` `` naming a retired command, with no `targets` prefix in the
/// same span) could never trigger the check, since `watch` failing to be a
/// real command is exactly what made the span fail to "read as an
/// invocation" in the first place. Excluding known value literals by their
/// own structural identity, rather than by span shape, closes that hole:
/// nothing here can suppress a check merely because the word being checked
/// happens not to be real.
#[test]
fn every_cross_reference_resolves() {
    let (flags, words, value_literals) = command_surface();
    let backtick_re = Regex::new(r"`([^`]+)`").unwrap();
    let mut failures = Vec::new();
    for page in help_pages() {
        let normalized = normalize(&render(&page));
        for cap in backtick_re.captures_iter(&normalized) {
            let inner = &cap[1];
            let tokens: Vec<&str> = inner.split_whitespace().collect();
            for token in &tokens {
                // A flag reference is trimmed of a trailing colon too
                // (`` `--target`: `` in prose); a bare word is not, because a
                // trailing colon there is the shape of a JSON-field or
                // similar description (`` `kind: "export"` ``), never how a
                // real command reference is written in this codebase's
                // prose. Trimming it away for word candidates reintroduced
                // exactly the false-positive class the span gate used to
                // (over-)suppress.
                let flag_candidate =
                    token.trim_matches(|c: char| matches!(c, ',' | '.' | ';' | ':'));
                let word_candidate = token.trim_matches(|c: char| matches!(c, ',' | '.' | ';'));
                if looks_like_flag(flag_candidate) {
                    if !flags.contains(flag_candidate) {
                        failures.push(format!(
                            "{}: `{flag_candidate}` (from `{inner}`) names no real flag",
                            label(&page)
                        ));
                    }
                } else if looks_like_command_word(word_candidate)
                    && !value_literals.contains(word_candidate)
                    && !words.contains(word_candidate)
                {
                    failures.push(format!(
                        "{}: `{word_candidate}` (from `{inner}`) names no real command",
                        label(&page)
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} cross-reference(s) in help text name no real command or flag:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The full flag set (both `-x` short and `--long-name` long tokens)
/// `capture`'s own grammar block in specification section 17.2 documents,
/// parsed from the fenced `text` block under the `### 17.2 Capture
/// Invocation` heading.
///
/// FR-014 names "capture's flag and short-flag set," not the short-flag set
/// alone; an earlier version of this check compared short flags only, so a
/// long-option rename or a long option the specification never listed at all
/// (`--catalog-db`, `--local-db`, and `--scope` were all missing from section
/// 17.2, found by review of PR #198 turning this on) drifted invisibly.
fn spec_capture_flags() -> std::collections::BTreeSet<String> {
    let spec = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/fragcap-specification.md"),
    )
    .expect("the master specification must be readable");
    let heading = "### 17.2 Capture Invocation";
    let start = spec.find(heading).expect("section 17.2 must exist");
    let after = &spec[start..];
    let fence_start = after
        .find("```text")
        .expect("section 17.2 must open a text fence");
    let block_start = fence_start + "```text".len();
    let block_end = after[block_start..]
        .find("```")
        .expect("the fence must close");
    let block = &after[block_start..block_start + block_end];

    let flag_re = Regex::new(r"^\s*(?:(-[A-Za-z]),\s*)?(--[\w-]+)").unwrap();
    let mut flags = std::collections::BTreeSet::new();
    for line in block.lines() {
        let Some(cap) = flag_re.captures(line) else {
            continue;
        };
        if let Some(short) = cap.get(1) {
            flags.insert(short.as_str().to_string());
        }
        flags.insert(cap[2].to_string());
    }
    flags
}

#[test]
fn capture_short_flags_match_the_specification() {
    let spec_flags = spec_capture_flags();
    let capture = fragcap_cli::command()
        .find_subcommand("capture")
        .expect("`capture` must exist")
        .clone();
    let mut binary_flags = std::collections::BTreeSet::new();
    for arg in capture.get_arguments() {
        // The offline substrate flags (`--replay-source`, `--attr-script`,
        // `--process-script`, `--local-addr`, `--fire-interrupt`) are
        // `hide = true`: they exist so the whole capture path is drivable
        // from a tier-1 test with no capture driver, and never render in any
        // `--help`, so the specification is not expected to document them
        // either. Excluding them structurally (`is_hide_set()`), rather than
        // by name, means a future hidden flag is excluded automatically.
        if arg.is_hide_set() {
            continue;
        }
        if let Some(short) = arg.get_short() {
            binary_flags.insert(format!("-{short}"));
        }
        if let Some(long) = arg.get_long() {
            binary_flags.insert(format!("--{long}"));
        }
    }
    // clap injects `-h`/`--help` at parse/build time, and propagates
    // `global = true` args (`--quiet`/`--silent`/`--json`) at the same time;
    // neither is present on an unbuilt `Command` fetched via
    // `find_subcommand()`, but every one of them renders on `capture --help`
    // in practice (confirmed directly), and none is a fact about this
    // crate's own grammar that could drift out from under this check.
    for f in ["-h", "--help", "--quiet", "--silent", "--json"] {
        binary_flags.insert(f.to_string());
    }

    let only_in_spec: Vec<_> = spec_flags.difference(&binary_flags).collect();
    let only_in_binary: Vec<_> = binary_flags.difference(&spec_flags).collect();
    assert!(
        only_in_spec.is_empty() && only_in_binary.is_empty(),
        "capture's flags disagree with specification section 17.2: \
         spec-only {only_in_spec:?}, binary-only {only_in_binary:?}"
    );
}
