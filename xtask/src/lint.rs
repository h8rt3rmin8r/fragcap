// SPDX-License-Identifier: Apache-2.0

//! Repository conventions check.
//!
//! Enforces the mechanical rules in `CONVENTIONS.md`, which constitution
//! principle P-8 requires be enforced by a linter rather than by review
//! attention.
//!
//! The checking logic is a pure function over file bytes so it can be tested
//! against known-bad input. That is the point of the design: a linter whose
//! matcher never fires is indistinguishable from a clean repository, and only
//! a test that feeds it a violation and demands a specific finding tells the
//! two apart.

use std::fs;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

/// Directories never linted: build output, version control internals, and
/// vendored third-party content. Listed explicitly rather than inferred, so
/// the exclusions are visible to a reader.
const EXCLUDED: &[&str] = &[
    ".git",
    "target",
    // The isolated fuzz workspace has its own Cargo output and unreviewed
    // campaign products. Its authored manifest, targets, dictionaries, and
    // promoted corpus remain linted; generated output never does.
    "fuzz/target",
    "fuzz/artifacts",
    "fuzz/coverage",
    "node_modules",
    "captures",
    ".agents/skills",
    ".claude/skills",
    // Machine-local git worktrees. A worktree here is a complete second
    // checkout of this repository, so linting it from the main checkout reports
    // every finding twice and reports the vendored content the exclusions above
    // exist to skip, because the exclusion paths no longer match once they are
    // nested. A worktree lints itself correctly when the linter is run inside
    // it, which is where the checking belongs. Added by slice S10, which found
    // it when a parallel worktree turned a clean run into 1306 violations
    // without a line of the slice's own code being involved.
    ".claude/worktrees",
    ".cursor/skills",
    ".opencode",
    ".specify",
    // The documentation site's dependency and build output directories. The
    // top-level `node_modules` entry above only matches the repository root, so
    // the site's own must be named; `.next`, `.source`, and `out` are generated
    // build artifacts carrying minified third-party code (CRLF, em dashes) that
    // is not this repository's to lint. The site's authored source (TypeScript,
    // MDX, CSS) is still linted. Added by slice S18c-2.
    "site/node_modules",
    "site/.next",
    "site/.source",
    "site/out",
    // The glossary content tree is generated from docs/glossary/ by
    // scripts/prebuild.mjs and gitignored; docs/glossary/ is the single source
    // and is linted there. Linting the derived copy would double every finding
    // and depends on the site build having run. Added by slice S18c-2.
    "site/content/docs/glossary",
    // The disclaimer module is generated from README.md by scripts/prebuild.mjs
    // and gitignored; README.md is the single source and is linted there. The
    // generated file carries its own SPDX header but does not exist in a fresh
    // checkout until the site build runs. Added for issue #39.
    "site/app/(home)/disclaimer/disclaimer.generated.ts",
    // next-env.d.ts is a Next.js-generated, gitignored declaration file with no
    // SPDX header of its own; it is a build product, not hand-authored source.
    // tsconfig.json is committed and stays under the encoding checks: .gitattributes
    // normalizes it to LF, and Next 16 does not rewrite it (it already carries the
    // options Next wants), so excluding it would only hide a real LF violation.
    "site/next-env.d.ts",
];

/// Extensions treated as source, and therefore required to carry an SPDX
/// identifier as their first line (CONVENTIONS.md). The documentation site adds
/// the TypeScript, TSX, ES module, and CSS faces of the same rule; content files
/// (Markdown, MDX, JSON) are not source and are exempt.
const SOURCE_EXT: &[&str] = &["rs", "sh", "ps1", "psm1", "ts", "tsx", "mjs", "css"];

/// Extensions whose files are binary regardless of content and are never linted
/// as text. Content sniffing catches most binaries by an embedded null, but a
/// file can be binary and carry no null in its first bytes: the vendored
/// `brand/brand-guide.pdf` begins with a prose-like header and was linted as
/// text. Skipping by extension as well as by content lets a vendored binary
/// asset sit beside editable text (the brand `README.md`, tokens, and SVG
/// masters) without a directory exclusion that would also stop the linter
/// checking that text.
const BINARY_EXT: &[&str] = &[
    "pdf", "ttf", "otf", "woff", "woff2", "png", "jpg", "jpeg", "gif", "ico", "webp",
];

/// Whether a file extension marks a binary asset the linter never reads as text.
fn is_binary_ext(ext: &str) -> bool {
    BINARY_EXT.contains(&ext.to_ascii_lowercase().as_str())
}

const SPDX: &str = "SPDX-License-Identifier: Apache-2.0";

/// One rule violation at one location.
#[derive(Debug, PartialEq, Eq)]
pub struct Finding {
    pub line: usize,
    pub rule: &'static str,
    pub detail: String,
}

impl Finding {
    fn new(line: usize, rule: &'static str, detail: impl Into<String>) -> Self {
        Finding {
            line,
            rule,
            detail: detail.into(),
        }
    }
}

/// True when the bytes look like a binary file. Content sniffing rather than
/// an extension list, so an unexpected binary does not produce thousands of
/// spurious findings.
pub fn is_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(8192).any(|b| *b == 0)
}

/// Check one file's bytes against every rule. Pure, and therefore testable.
///
/// `is_source` selects whether the SPDX identifier rule applies.
pub fn check_bytes(bytes: &[u8], is_source: bool) -> Vec<Finding> {
    let mut out = Vec::new();

    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        out.push(Finding::new(1, "bom", "file begins with a byte order mark"));
    }

    if bytes.contains(&b'\r') {
        let line = bytes
            .split(|b| *b == b'\n')
            .position(|l| l.contains(&b'\r'))
            .map(|i| i + 1)
            .unwrap_or(1);
        out.push(Finding::new(
            line,
            "crlf",
            "carriage return present; use LF",
        ));
    }

    let text = String::from_utf8_lossy(bytes);

    if !bytes.is_empty() {
        if !bytes.ends_with(b"\n") {
            let n = text.lines().count().max(1);
            out.push(Finding::new(
                n,
                "final-newline",
                "no newline at end of file",
            ));
        } else if bytes.ends_with(b"\n\n") {
            let n = text.lines().count().max(1);
            out.push(Finding::new(
                n,
                "final-newline",
                "more than one trailing newline",
            ));
        }
    }

    for (i, line) in text.lines().enumerate() {
        let n = i + 1;

        if line.len() != line.trim_end().len() {
            out.push(Finding::new(
                n,
                "trailing-whitespace",
                "line ends with whitespace",
            ));
        }

        // Written as escapes on purpose. A literal character here would make
        // this file violate the rule it implements.
        if line.contains('\u{2014}') {
            out.push(Finding::new(
                n,
                "em-dash",
                "em-dash present; use a comma or parentheses",
            ));
        }
        if line.contains('\u{2013}') {
            out.push(Finding::new(
                n,
                "en-dash",
                "en-dash present; use a standard hyphen",
            ));
        }
    }

    if is_source && !bytes.is_empty() {
        // A shell script's shebang must be the first line for the kernel to
        // honor it, which conflicts with the SPDX-first-line rule. When the
        // first line is a shebang, the SPDX identifier is required on the second
        // line instead, so the identifier still sits at the top of the file.
        let mut lines = text.lines();
        let first = lines.next().unwrap_or("");
        let carries_spdx = if first.starts_with("#!") {
            lines.next().unwrap_or("").contains(SPDX)
        } else {
            first.contains(SPDX)
        };
        if !carries_spdx {
            out.push(Finding::new(
                1,
                "spdx",
                "first line is not the SPDX license identifier (or the second line, after a shebang)",
            ));
        }
    }

    out
}

/// The command-surface file whose doc comments clap publishes verbatim.
const HELP_SOURCE: &str = "crates/fragcap-cli/src/cli.rs";

/// Internal development vocabulary that must never reach user-facing `--help`.
///
/// clap derives every `about` and every option description from the `///` doc
/// comments in [`HELP_SOURCE`], so provenance written for a maintainer is
/// published to anyone holding the binary. Issue #67 closed this once by
/// scrubbing the text; issue #178 records it coming straight back, because
/// nothing asserted it. The guard test in `cli_help.rs` asserts it over
/// rendered output; this asserts it over the source, so the cheap check catches
/// it too and catches a leak in a doc comment clap does not publish today but
/// would after a later refactor.
///
/// A slice identifier means something to `specs/` and to nobody else. A
/// specification section number is actionable only with the specification open,
/// which is not the state of a reader in a terminal. A Cargo feature name is a
/// build-time switch a user of a released binary cannot act on at all.
///
/// Each entry is (pattern kind, what it is). The match is deliberately narrow:
/// it runs only on `///` lines of one file, so a `//` maintainer comment above
/// the item is the sanctioned place to keep the provenance.
fn help_doc_leak(text: &str) -> Option<&'static str> {
    // `slice S051`, and the bare `(S051)` form the previous guard missed.
    if let Some(i) = text.find("slice S") {
        if text[i + 7..].starts_with(|c: char| c.is_ascii_digit()) {
            return Some("a slice identifier");
        }
    }
    let bytes = text.as_bytes();
    for (i, w) in bytes.iter().enumerate() {
        if *w != b'S' {
            continue;
        }
        // Word boundary before, two or three digits, word boundary after.
        if i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_') {
            continue;
        }
        let digits = bytes[i + 1..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count();
        if (2..=3).contains(&digits) {
            let after = bytes.get(i + 1 + digits);
            if after.is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_') {
                return Some("a bare slice identifier");
            }
        }
    }
    // `section 17.2`, `Section 14.5`.
    for lead in ["section ", "Section "] {
        if let Some(i) = text.find(lead) {
            let rest = &text[i + lead.len()..];
            let digits = rest.chars().take_while(char::is_ascii_digit).count();
            if digits > 0 && rest[digits..].starts_with('.') {
                let after = &rest[digits + 1..];
                if after.starts_with(|c: char| c.is_ascii_digit()) {
                    return Some("a specification section reference");
                }
            }
        }
    }
    if let Some(i) = text.find("Appendix ") {
        if text[i + 9..].starts_with(|c: char| c.is_ascii_uppercase()) {
            return Some("an appendix letter");
        }
    }
    if let Some(i) = text.find("Tier ") {
        if text[i + 5..].starts_with(|c: char| c.is_ascii_digit()) {
            return Some("a bare tier number");
        }
    }
    // A constitution principle identifier: `P-1` through `P-11` today, and the
    // pattern must not assume one digit. Absent entirely until review of PR
    // #189 caught that this rule promised parity with the rendered guard and did
    // not deliver it.
    if let Some(i) = text.find("P-") {
        let rest = &text[i + 2..];
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        let before_ok = i == 0 || !text.as_bytes()[i - 1].is_ascii_alphanumeric();
        let after = rest.as_bytes().get(digits);
        if digits > 0 && before_ok && after.is_none_or(|b| !b.is_ascii_alphanumeric()) {
            return Some("a constitution principle identifier");
        }
    }
    // A Cargo feature named to a user. Matched by the *phrasing*, never by the
    // declared feature names: `live`, `net`, `targets`, and `etw` are ordinary
    // words, and matching `net` bare would fire on "network" and `targets` on
    // most of the targets help. A rule that cries wolf earns an exception list,
    // and an exception list is what decayed into issue #178.
    //
    // Three forms, matching the rendered guard: the two backticked ones, and the
    // unquoted `the <word> feature`. The last was missing until review of PR
    // #189, so a doc comment saying "the net feature" passed the cheap gate and
    // failed only the expensive one.
    if text.contains("` feature") || text.contains("feature `") {
        return Some("a Cargo feature name");
    }
    if let Some(i) = text.find("the ") {
        let rest = &text[i + 4..];
        let word: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if !word.is_empty() && rest[word.len()..].starts_with(" feature") {
            return Some("a Cargo feature name");
        }
    }
    None
}

fn is_excluded(path: &Path, root: &Path) -> bool {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let s = rel.to_string_lossy().replace('\\', "/");
    EXCLUDED
        .iter()
        .any(|e| s == *e || s.starts_with(&format!("{e}/")))
}

fn collect(dir: &Path, root: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if is_excluded(&path, root) {
            continue;
        }
        if path.is_dir() {
            collect(&path, root, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

/// Walk the repository and report every violation. Returns the finding count.
/// Capture-binding calls fragcap must never make.
///
/// Constitution P-1 permits the NDIS capture driver and nothing that modifies
/// traffic. The `pcap` crate binds a library that can also transmit, and slice
/// S09 plan decision D-8 concluded that transmission is not on the section 19.3
/// denylist, so the dependency is acceptable. That argument is correct and it
/// is also exactly the kind of argument that decays, so this makes it
/// mechanical: fragcap's own source may not name a transmit call, and a change
/// that wants to has to delete this list deliberately.
///
/// Slice S10 added the second group for the same reason. Constitution P-1
/// requires that any process handle state its access rights explicitly at the
/// call site, and a request carrying memory rights fails review. Naming a
/// process is the classic reason to open one, and S10 research R-2 chose the
/// toolhelp enumeration path specifically because it opens no handle against
/// any target process at all. That removes the thing a reviewer has to check
/// rather than documenting it, and this list is what keeps it removed: fragcap
/// opens no process, so there are no access rights to audit.
///
/// A slice that genuinely needs a process handle deletes the relevant entry
/// deliberately and argues for it, which is the point.
const FORBIDDEN_CALLS: &[(&str, &str)] = &[
    (
        "sendpacket",
        "transmits a frame; fragcap observes and never modifies traffic (P-1)",
    ),
    (
        "inject(",
        "transmits a frame; fragcap observes and never modifies traffic (P-1)",
    ),
    (
        "openprocess",
        "opens a handle against a target process; fragcap names processes by query-only enumeration (P-1)",
    ),
    (
        "readprocessmemory",
        "reads another process's memory; on the section 19.3 denylist (P-1)",
    ),
    (
        "writeprocessmemory",
        "writes another process's memory; on the section 19.3 denylist (P-1)",
    ),
    // Slice S11 added the access-right constants, which are complementary to
    // the three calls above rather than a duplicate of them. A right can be
    // named where the call is not: passed to a helper, stored in a constant, or
    // handed to a binding that opens the handle on the caller's behalf. The
    // calls are how a handle is obtained; these are what it would carry.
    //
    // S11 originally added these to permit `openprocess` with
    // PROCESS_QUERY_LIMITED_INFORMATION, which P-1 does allow. It withdrew that
    // during integration and kept S10's stronger rule instead: fragcap opens no
    // process at all, so there are no rights to audit. These entries exist so
    // that a slice which deletes the `openprocess` line still cannot quietly
    // ask for memory.
    (
        "process_vm_read",
        "reads another process's memory; P-1 forbids a handle carrying memory rights",
    ),
    (
        "process_vm_write",
        "writes another process's memory; P-1 forbids a handle carrying memory rights",
    ),
    (
        "process_vm_operation",
        "operates on another process's memory; P-1 forbids a handle carrying memory rights",
    ),
    (
        "process_all_access",
        "includes memory rights; name the narrowest right the call needs (P-1)",
    ),
    (
        "certutil",
        "spawns an external certificate utility; native Deep Capture owns exact trust effects (P-1)",
    ),
];

/// Files that must never appear in the repository.
///
/// The npcap software development kit and driver are not redistributable. The
/// constitution's licensing section forbids vendoring or bundling either, and
/// slice S09 success criterion SC-010 says the rule is verified mechanically
/// rather than remembered.
const FORBIDDEN_ARTIFACTS: &[&str] = &[
    "wpcap.lib",
    "packet.lib",
    "wpcap.dll",
    "packet.dll",
    "npcap-sdk",
];

fn is_production_proxy_surface(path: &str) -> bool {
    matches!(
        path,
        "crates/fragcap-cli/src/cli.rs"
            | "crates/fragcap-cli/src/commands/deep_capture.rs"
            | "crates/fragcap-cli/src/doctor/mod.rs"
            | "crates/fragcap-cli/src/doctor/probe.rs"
            | "crates/fragcap-cli/src/doctor/checks.rs"
    ) || path.starts_with(".github/workflows/")
        || path == "Cargo.toml"
}

fn external_proxy_violation(path: &str, text: &str) -> Option<&'static str> {
    if !is_production_proxy_surface(path) {
        return None;
    }
    let lower = text.to_ascii_lowercase();
    if lower.contains("mitmdump") || lower.contains("mitmproxy") {
        return Some("external proxy implementation name");
    }
    if path.ends_with(".py") || lower.contains("python.exe") || lower.contains("python3") {
        return Some("Python proxy runtime");
    }
    None
}

fn platform_trigger_violation(text: &str) -> Option<&'static str> {
    let trigger = text.split("\nconcurrency:").next().unwrap_or(text);
    if trigger
        .lines()
        .any(|line| matches!(line.trim(), "paths:" | "paths-ignore:"))
    {
        return Some("whole-workspace platform workflow must not use path filters");
    }
    if !trigger.contains("workflow_dispatch:")
        || !trigger.contains("push:")
        || !trigger.contains("branches: [main]")
        || !trigger.contains("pull_request:")
    {
        return Some("platform workflow must run manually, on main pushes, and on pull requests");
    }
    None
}

pub fn run(root: &Path) -> std::io::Result<usize> {
    let mut files = Vec::new();
    collect(root, root, &mut files)?;
    files.sort();

    let mut total = 0usize;
    for path in &files {
        let rel = path.strip_prefix(root).unwrap_or(path);
        let shown = rel.to_string_lossy().replace(MAIN_SEPARATOR, "/");
        let lowered = shown.to_lowercase();
        for artifact in FORBIDDEN_ARTIFACTS {
            if lowered.contains(artifact) {
                println!(
                    "{shown}: capture-driver-artifact: npcap binaries and its \
                     software development kit are never vendored (constitution \
                     licensing section)"
                );
                total += 1;
            }
        }
    }

    for path in files {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
        if is_binary_ext(&ext) {
            continue;
        }
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        if is_binary(&bytes) {
            continue;
        }
        let is_source = SOURCE_EXT.contains(&ext.as_str());
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let shown = rel.to_string_lossy().replace('\\', "/");

        for f in check_bytes(&bytes, is_source) {
            println!("{}:{}: {}: {}", shown, f.line, f.rule, f.detail);
            total += 1;
        }

        if let Ok(text) = std::str::from_utf8(&bytes) {
            if let Some(detail) = external_proxy_violation(&shown, text) {
                println!(
                    "{shown}: native-proxy-cutover: production and release inputs contain {detail}"
                );
                total += 1;
            }
            if shown == ".github/workflows/platform.yml" {
                if let Some(detail) = platform_trigger_violation(text) {
                    println!("{shown}: platform-trigger-coverage: {detail}");
                    total += 1;
                }
            }
        }

        // The command surface's doc comments are user-facing help (issues #176,
        // #178). Scoped to one file and to `///` lines, so a `//` maintainer
        // comment above the item stays the sanctioned home for provenance.
        if shown == HELP_SOURCE {
            if let Ok(text) = std::str::from_utf8(&bytes) {
                for (line_no, line) in text.lines().enumerate() {
                    let trimmed = line.trim_start();
                    let Some(doc) = trimmed.strip_prefix("///") else {
                        continue;
                    };
                    if let Some(what) = help_doc_leak(doc) {
                        println!(
                            "{}:{}: help-vocabulary: {what} in a doc comment clap publishes",
                            shown,
                            line_no + 1
                        );
                        total += 1;
                    }
                }
            }
        }

        // P-1, mechanically. Only fragcap's own Rust source is checked: this
        // file names the calls it forbids, and the specification and the plan
        // discuss them, so matching prose would report itself.
        if is_source && ext == "rs" && !shown.starts_with("xtask/") {
            if let Ok(text) = std::str::from_utf8(&bytes) {
                for (line_no, line) in text.lines().enumerate() {
                    // Case-insensitive since slice S10. The `pcap` binding
                    // names its calls in snake case and the platform bindings
                    // name theirs in Pascal case, so a list written in one
                    // casing would silently miss the other. Every entry in
                    // FORBIDDEN_CALLS is therefore written lowercase.
                    let code = line.split("//").next().unwrap_or("").to_ascii_lowercase();
                    for (call, why) in FORBIDDEN_CALLS {
                        if code.contains(call) {
                            println!("{}:{}: forbidden-call: {call} {why}", shown, line_no + 1);
                            total += 1;
                        }
                    }
                }
            }
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(bytes: &[u8], is_source: bool) -> Vec<&'static str> {
        check_bytes(bytes, is_source)
            .into_iter()
            .map(|f| f.rule)
            .collect()
    }

    // The load-bearing test. If clean input produced findings, or if the
    // matchers never fired, every other test below could still pass while the
    // linter was useless.
    #[test]
    fn clean_source_produces_no_findings() {
        let ok = b"// SPDX-License-Identifier: Apache-2.0\n\nfn main() {}\n";
        assert_eq!(check_bytes(ok, true), vec![]);
    }

    #[test]
    fn clean_prose_produces_no_findings() {
        assert_eq!(check_bytes(b"# Title\n\nSome prose.\n", false), vec![]);
    }

    #[test]
    fn detects_byte_order_mark() {
        let b = b"\xEF\xBB\xBF// SPDX-License-Identifier: Apache-2.0\n";
        assert!(rules(b, true).contains(&"bom"));
    }

    #[test]
    fn detects_carriage_return() {
        assert!(rules(b"# Title\r\n", false).contains(&"crlf"));
    }

    #[test]
    fn detects_trailing_whitespace() {
        assert!(rules(b"# Title   \n", false).contains(&"trailing-whitespace"));
    }

    #[test]
    fn detects_missing_final_newline() {
        assert!(rules(b"# Title", false).contains(&"final-newline"));
    }

    #[test]
    fn detects_multiple_final_newlines() {
        assert!(rules(b"# Title\n\n", false).contains(&"final-newline"));
    }

    #[test]
    fn detects_em_dash() {
        let b = "# A \u{2014} B\n".as_bytes();
        assert!(rules(b, false).contains(&"em-dash"));
    }

    #[test]
    fn detects_en_dash() {
        let b = "# A \u{2013} B\n".as_bytes();
        assert!(rules(b, false).contains(&"en-dash"));
    }

    #[test]
    fn detects_missing_spdx_in_source() {
        assert!(rules(b"fn main() {}\n", true).contains(&"spdx"));
    }

    #[test]
    fn a_shebang_moves_the_spdx_requirement_to_the_second_line() {
        // A shell script's shebang must be line 1, so its SPDX sits on line 2.
        let ok = b"#!/usr/bin/env bash\n# SPDX-License-Identifier: Apache-2.0\n\necho hi\n";
        assert!(
            !rules(ok, true).contains(&"spdx"),
            "shebang then SPDX is clean"
        );
        // A shebang with no SPDX on line 2 still fails.
        let bad = b"#!/usr/bin/env bash\necho hi\n";
        assert!(
            rules(bad, true).contains(&"spdx"),
            "shebang without SPDX fails"
        );
    }

    #[test]
    fn does_not_require_spdx_in_prose() {
        assert!(!rules(b"# Title\n", false).contains(&"spdx"));
    }

    #[test]
    fn reports_the_offending_line_number() {
        let b = b"# SPDX-License-Identifier: Apache-2.0\nclean\nbad   \n";
        let f = check_bytes(b, false);
        let ws: Vec<_> = f
            .iter()
            .filter(|f| f.rule == "trailing-whitespace")
            .collect();
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0].line, 3);
    }

    #[test]
    fn binary_content_is_recognized() {
        assert!(is_binary(&[0x00, 0x01, 0x02]));
        assert!(!is_binary(b"plain text\n"));
    }

    #[test]
    fn native_proxy_gate_rejects_external_runtime_only_on_production_surfaces() {
        assert_eq!(
            external_proxy_violation(
                "crates/fragcap-cli/src/commands/deep_capture.rs",
                "Command::new(\"mitmdump\")",
            ),
            Some("external proxy implementation name")
        );
        assert_eq!(
            external_proxy_violation("docs/history.md", "mitmdump was the former backend"),
            None
        );
    }

    #[test]
    fn platform_trigger_gate_rejects_filters_and_missing_events() {
        let clean = "on:\n  workflow_dispatch:\n  push:\n    branches: [main]\n  pull_request:\n\nconcurrency:\n";
        assert_eq!(platform_trigger_violation(clean), None);
        assert_eq!(
            platform_trigger_violation(
                &clean.replace("  pull_request:\n", "  pull_request:\n    paths:\n")
            ),
            Some("whole-workspace platform workflow must not use path filters")
        );
        assert_eq!(
            platform_trigger_violation(
                &clean.replace("  pull_request:\n", "  pull_request:\n    paths-ignore:\n")
            ),
            Some("whole-workspace platform workflow must not use path filters")
        );
        assert_eq!(
            platform_trigger_violation(&clean.replace("  pull_request:\n", "")),
            Some("platform workflow must run manually, on main pushes, and on pull requests")
        );
    }

    #[test]
    fn binary_extensions_are_skipped_regardless_of_content() {
        // A PDF whose header is prose-like carries no null in its first bytes,
        // so content sniffing alone would lint it as text. The extension guard
        // is what skips it, while leaving text assets beside it (md, css, svg)
        // under the walk. Case-insensitive.
        assert!(is_binary_ext("pdf"));
        assert!(is_binary_ext("PDF"));
        assert!(is_binary_ext("woff2"));
        assert!(is_binary_ext("ico"));
        assert!(!is_binary_ext("md"));
        assert!(!is_binary_ext("svg"));
        assert!(!is_binary_ext("css"));
        assert!(!is_binary_ext(""));
    }

    #[test]
    fn isolated_fuzz_products_are_excluded_but_authored_inputs_are_not() {
        let root = Path::new("repo");
        assert!(is_excluded(Path::new("repo/fuzz/target/debug"), root));
        assert!(is_excluded(Path::new("repo/fuzz/artifacts/http1"), root));
        assert!(is_excluded(Path::new("repo/fuzz/coverage/http1"), root));
        assert!(!is_excluded(
            Path::new("repo/fuzz/fuzz_targets/http1.rs"),
            root
        ));
        assert!(!is_excluded(
            Path::new("repo/fuzz/corpus/http1/request-get"),
            root
        ));
    }

    #[test]
    fn empty_file_is_not_flagged_for_newlines() {
        assert_eq!(check_bytes(b"", false), vec![]);
    }
}
