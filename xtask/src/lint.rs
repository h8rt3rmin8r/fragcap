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
];

/// Extensions treated as source, and therefore required to carry an SPDX
/// identifier as their first line.
const SOURCE_EXT: &[&str] = &["rs", "sh", "ps1", "psm1"];

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
        let first = text.lines().next().unwrap_or("");
        if !first.contains(SPDX) {
            out.push(Finding::new(
                1,
                "spdx",
                "first line is not the SPDX license identifier",
            ));
        }
    }

    out
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
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        if is_binary(&bytes) {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let is_source = SOURCE_EXT.contains(&ext);
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let shown = rel.to_string_lossy().replace('\\', "/");

        for f in check_bytes(&bytes, is_source) {
            println!("{}:{}: {}: {}", shown, f.line, f.rule, f.detail);
            total += 1;
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
    fn empty_file_is_not_flagged_for_newlines() {
        assert_eq!(check_bytes(b"", false), vec![]);
    }
}
