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

/// The flags whose effective default is resolved by clap or `assemble.rs`
/// rather than left unstated. Mirrors, one entry per site:
/// - `--mode`: `cli.rs`'s `default_value_t = ModeArg::File` (S070; matches
///   `assemble.rs`'s `resolve_mode`, which no longer has a `None` arm)
/// - `--direction`: `cli.rs`'s `default_value_t = Direction::Both` (S070;
///   matches `assemble.rs:148`, which reads `args.direction` directly)
/// - `--roles`: stated in prose in `cli.rs`'s doc comment (mirrors
///   `assemble.rs:126,203`'s `.or_else(|| defaults.roles()...)`)
/// - `--wait`: stated in prose in `cli.rs`'s doc comment (mirrors
///   `assemble.rs:143`'s `acquisition_timeout: args.wait`, where `None` means
///   no timeout)
///
/// A future defaulted option is added here by hand, deliberately: the four
/// sites above are not expressible as a single structural walk of
/// `fragcap_cli::command()`, because "has a default" is a fact about
/// `assemble.rs`'s resolution logic, which clap's own `Arg` metadata only
/// reports for the two that use `default_value_t`. The comment above ties
/// each entry back to the exact site it mirrors so drift between this list
/// and `assemble.rs` is a one-file diff to check, not a re-audit.
const DEFAULTED_OPTIONS: &[&str] = &["--mode", "--direction", "--roles", "--wait"];

/// Extract the option/argument paragraphs of a `--help` (long) rendering.
///
/// clap's long help separates each entry with a blank line; a short (`-h`)
/// rendering does not, which is exactly the distinction
/// [`short_help_continuations`] depends on.
fn option_paragraphs(page: &str) -> Vec<&str> {
    page.split("\n\n").collect()
}

/// Whether the first paragraph of a doc-comment block (the part before the
/// first blank `///` line, or the whole block if there is none) itself reads
/// as more than one sentence: a `.` followed by whitespace and a capital
/// letter, outside backticked code. A single period with no following
/// capital (`catalog.db`, `%APPDATA%\fragcap\catalog.db`) does not count.
fn first_paragraph_has_two_sentences(doc_lines: &[&str]) -> bool {
    let mut first_paragraph = Vec::new();
    for line in doc_lines {
        if line.trim().is_empty() {
            break;
        }
        first_paragraph.push(*line);
    }
    let joined = first_paragraph.join(" ");
    let mut in_backtick = false;
    let chars: Vec<char> = joined.chars().collect();
    for i in 0..chars.len() {
        match chars[i] {
            '`' => in_backtick = !in_backtick,
            '.' if !in_backtick => {
                let rest: String = chars[i + 1..].iter().collect();
                let trimmed = rest.trim_start();
                if trimmed
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_uppercase())
                    && rest.starts_with(char::is_whitespace)
                {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// The paragraph whose first line names `flag`, or `None`.
fn find_flag_paragraph<'a>(page: &'a str, flag: &str) -> Option<&'a str> {
    option_paragraphs(page).into_iter().find(|p| {
        p.lines()
            .next()
            .is_some_and(|l| l.trim_start().starts_with(flag))
    })
}

#[test]
fn every_defaulted_option_states_its_default() {
    let out = render(&["capture".to_string()]);
    let mut failures = Vec::new();
    for flag in DEFAULTED_OPTIONS {
        let Some(block) = find_flag_paragraph(&out, flag) else {
            failures.push(format!("{flag}: no `capture --help` block found at all"));
            continue;
        };
        let normalized = normalize(block).to_lowercase();
        if !normalized.contains("[default:") && !normalized.contains("default") {
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

/// One `///`-commented block in `cli.rs`, as consecutive doc-comment lines
/// (with their `///` prefix and up to one leading space stripped), plus the
/// 1-based source line the block starts on.
struct DocBlock {
    start_line: usize,
    lines: Vec<String>,
}

/// Every doc-comment block in `cli.rs` that documents an `#[arg(...)]` field
/// or a `Subcommand`/`ValueEnum` variant: a run of consecutive `///` lines
/// immediately followed by a non-`///` line (the attribute or the item
/// itself). A run immediately followed by another `///` run with only a
/// blank *non-doc* line between is not something this grammar produces, so
/// it is not specially handled.
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
/// `next_line`) belongs to something clap actually renders as `--help` text.
///
/// A struct's or enum's own outer doc comment never renders when that type is
/// used as an enum variant's payload or via `#[command(subcommand)]`/
/// `#[command(flatten)]`: clap renders the *variant*'s doc or the flattened
/// field's own doc instead (verified: `doctor --help`'s about text is the
/// `Doctor` variant's doc, not `DoctorArgs`' own struct doc, which is never
/// shown anywhere). A `#[command(subcommand)]` or `#[command(flatten)]` field
/// likewise renders nothing of its own doc comment; only its target type's
/// content does. Excluding both keeps this check aimed at strings clap
/// actually publishes.
fn renders_as_user_help(lines: &[&str], next_line: usize) -> bool {
    // A doc comment on a struct or enum is followed by zero or more attribute
    // lines (`#[derive(...)]`, `#[command(...)]`) before the declaration
    // itself; skip past them to find what is actually being documented.
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
    if declaration.starts_with("pub struct")
        || declaration.starts_with("pub enum")
        || declaration.starts_with("impl ")
    {
        return false;
    }
    true
}

/// This is a source check, not a rendered-output check, and deliberately so:
/// `capture -h`'s own column width (driven by the longest flag name on the
/// page) can force clap to wrap even a single, legitimately short sentence
/// (measured: `--target`'s existing, already-split summary wraps on
/// `capture -h` purely from column width) and a page with especially long
/// flag names (measured: `extcap -h`, whose `--extcap-interfaces` is 21
/// characters) can push clap into its own next-line layout for every entry on
/// that page, including its auto-generated `-h, --help` text, which is not
/// authorable at all. Neither is the defect FR-011 exists to catch; both are
/// widths, already FR-001/FR-002's job on the existing wrap test. What FR-011
/// actually asks (per plan.md Design section 9, and the short/long split
/// FR-009 of S062 established) is a source fact: a doc comment conveying more
/// than one sentence must have a blank `///` line separating the first
/// sentence (what `-h` shows) from the rest (what only `--help` shows).
#[test]
fn every_short_help_summary_is_one_line() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cli.rs"),
    )
    .expect("cli.rs must be readable");
    let mut failures = Vec::new();
    for block in doc_blocks(&source) {
        let has_blank_split = block.lines.iter().any(|l| l.trim().is_empty());
        let refs: Vec<&str> = block.lines.iter().map(String::as_str).collect();
        if !has_blank_split && first_paragraph_has_two_sentences(&refs) {
            failures.push(format!(
                "cli.rs:{}: doc comment has more than one sentence with no blank `///` \
                 line splitting the `-h` summary from the `--help` detail:\n{}",
                block.start_line,
                block.lines.join(" ")
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
/// tree, and every real subcommand name at any depth (excluding clap's
/// generated `help`).
fn command_surface() -> (
    std::collections::BTreeSet<String>,
    std::collections::BTreeSet<String>,
) {
    fn walk(
        cmd: &clap::Command,
        flags: &mut std::collections::BTreeSet<String>,
        words: &mut std::collections::BTreeSet<String>,
    ) {
        for arg in cmd.get_arguments() {
            if let Some(long) = arg.get_long() {
                flags.insert(format!("--{long}"));
            }
            if let Some(short) = arg.get_short() {
                flags.insert(format!("-{short}"));
            }
        }
        for sub in cmd.get_subcommands() {
            if sub.get_name() == "help" {
                continue;
            }
            words.insert(sub.get_name().to_string());
            walk(sub, flags, words);
        }
    }
    let mut flags = std::collections::BTreeSet::new();
    let mut words = std::collections::BTreeSet::new();
    walk(&fragcap_cli::command(), &mut flags, &mut words);
    // clap adds these automatically; they exist on every page but are not
    // declared `Arg`s this walk would otherwise see.
    for f in ["-h", "--help", "-V", "--version"] {
        flags.insert(f.to_string());
    }
    (flags, words)
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

#[test]
fn every_cross_reference_resolves() {
    let (flags, words) = command_surface();
    let backtick_re = Regex::new(r"`([^`]+)`").unwrap();
    let mut failures = Vec::new();
    for page in help_pages() {
        let normalized = normalize(&render(&page));
        for cap in backtick_re.captures_iter(&normalized) {
            let inner = &cap[1];
            let tokens: Vec<&str> = inner.split_whitespace().collect();
            // A bare word is only checked as a command reference if the whole
            // backtick span reads like an invocation, i.e. its own first word
            // is itself a real command word. This is the shape rule that
            // keeps a purely technical or value-string backtick (`` `target`
            // ``, `` `all` ``, `` `yes` ``) from being misread as a stale
            // command reference: none of those spans starts with a real
            // command word, so bare-word checking never engages for them.
            let span_is_invocation = tokens
                .first()
                .is_some_and(|w| looks_like_command_word(w) && words.contains(*w));
            for (idx, token) in tokens.iter().enumerate() {
                let token = token.trim_matches(|c: char| matches!(c, ',' | '.' | ';' | ':'));
                // A word immediately after a flag is that flag's *value*
                // (`--tier signature`), never a command word, even inside an
                // otherwise invocation-shaped span.
                let preceded_by_flag = idx > 0 && looks_like_flag(tokens[idx - 1]);
                if looks_like_flag(token) {
                    if !flags.contains(token) {
                        failures.push(format!(
                            "{}: `{token}` (from `{inner}`) names no real flag",
                            label(&page)
                        ));
                    }
                } else if span_is_invocation
                    && !preceded_by_flag
                    && looks_like_command_word(token)
                    && !words.contains(token)
                {
                    failures.push(format!(
                        "{}: `{token}` (from `{inner}`) names no real command",
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

/// The short-flag set (`-x` tokens) `capture`'s own grammar block in
/// specification section 17.2 documents, parsed from the fenced `text` block
/// under the `### 17.2 Capture Invocation` heading.
fn spec_capture_short_flags() -> std::collections::BTreeSet<String> {
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

    let short_flag_re = Regex::new(r"^\s*(-[A-Za-z]),").unwrap();
    block
        .lines()
        .filter_map(|line| short_flag_re.captures(line))
        .map(|c| c[1].to_string())
        .collect()
}

#[test]
fn capture_short_flags_match_the_specification() {
    let spec_flags = spec_capture_short_flags();
    let capture = fragcap_cli::command()
        .find_subcommand("capture")
        .expect("`capture` must exist")
        .clone();
    let mut binary_flags: std::collections::BTreeSet<String> = capture
        .get_arguments()
        .filter_map(|a| a.get_short())
        .map(|c| format!("-{c}"))
        .collect();
    // clap injects `-h`/`--help` at parse/build time; it is not present on an
    // unbuilt `Command`'s own `get_arguments()`, but every subcommand carries
    // it in practice (confirmed by rendering `capture -h`), and it is not a
    // fact about this crate's own grammar that could drift.
    binary_flags.insert("-h".to_string());

    let only_in_spec: Vec<_> = spec_flags.difference(&binary_flags).collect();
    let only_in_binary: Vec<_> = binary_flags.difference(&spec_flags).collect();
    assert!(
        only_in_spec.is_empty() && only_in_binary.is_empty(),
        "capture's short flags disagree with specification section 17.2: \
         spec-only {only_in_spec:?}, binary-only {only_in_binary:?}"
    );
}
