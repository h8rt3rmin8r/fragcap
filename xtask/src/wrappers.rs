// SPDX-License-Identifier: Apache-2.0

//! The shell-wrapper compliance gate (specification section 18.4).
//!
//! `cargo xtask wrappers` holds the shell scripts under `scripts/` to their
//! ShruggieTech house standards: the two wrappers and the documentation linter
//! `lint-docs.sh` (section 22.5), all under the Bash standard, plus the
//! PowerShell wrapper. The PowerShell wrapper is checked by the
//! vendored `Test-ScriptCompliance.ps1` (its POSIX twin, so only bash is
//! needed); the Bash wrapper is checked by [`check_bash`], authored here because
//! no Bash checker is vendored. Both scripts are then checked for syntax
//! (`bash -n`, a PowerShell parse) and for their help and dry-run seams. The
//! exit contract is the house 0/1/2: 0 both compliant, 1 a check failed, 2 the
//! gate could not run (bash or pwsh absent), the last so a skipped gate never
//! reads as a clean one.

use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

/// The four section headings the Bash standard requires, in order.
const BASH_HEADINGS: &[&str] = &[
    "# Declare Functions",
    "# Declare Variables and Arrays",
    "# Execute Operations",
    "# End of script",
];

/// A section divider: `#` followed by exactly 79 underscores (80 columns).
fn is_divider(line: &str) -> bool {
    line.len() == 80 && line.starts_with('#') && line[1..].bytes().all(|b| b == b'_')
}

/// Whether a character is an emoji or pictograph the house standard forbids.
fn is_emoji(c: char) -> bool {
    let u = c as u32;
    (0x2600..=0x27BF).contains(&u)
        || (0x2B00..=0x2BFF).contains(&u)
        || (0xFE00..=0xFE0F).contains(&u)
        || u == 0x200D
        || u >= 0x1F000
}

/// Check a Bash wrapper's bytes against the ShruggieTech Bash structure, and
/// return one rule key per failing check (empty when compliant).
///
/// A pure function over bytes, like `lint::check_bytes`, so it is tested against
/// known-bad input rather than only exercised on the real script.
pub fn check_bash(bytes: &[u8]) -> Vec<&'static str> {
    let mut out = Vec::new();

    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        out.push("bom");
    }
    if bytes.contains(&b'\r') {
        out.push("crlf");
    }
    // Reject invalid UTF-8 rather than lossily replacing it: a malformed byte in
    // a comment must not pass the structural checks unseen (CONVENTIONS.md
    // requires UTF-8). The structural checks below still run over the lossy text
    // so a file with both problems reports both.
    if std::str::from_utf8(bytes).is_err() {
        out.push("utf8");
    }

    let text = String::from_utf8_lossy(bytes);
    let lines: Vec<&str> = text.lines().collect();

    if lines.first().copied() != Some("#!/usr/bin/env bash") {
        out.push("shebang");
    }
    if lines.get(1).map(|l| l.contains("SPDX-License-Identifier")) != Some(true) {
        out.push("spdx");
    }
    if !lines.iter().any(|l| {
        let t = l.trim_start();
        t.starts_with("set -") && t.contains("pipefail")
    }) {
        out.push("strict-mode");
    }
    if !lines.iter().any(|l| l.contains("IFS=")) {
        out.push("ifs");
    }
    // The man-page help block, by two of its conventional headings.
    if !text.contains("# NAME") || !text.contains("# SYNOPSIS") {
        out.push("help-block");
    }
    // The fixtures specification section 18.3 requires of the Bash wrapper.
    // Checking for the definitions keeps the compliance gate from passing a
    // wrapper that has dropped a mandated idiom (Codex review of PR #35).
    for fixture in ["print_help", "has_cmd", "safe_run", "log_"] {
        if !text.contains(fixture) {
            out.push(match fixture {
                "print_help" => "fixture-print_help",
                "has_cmd" => "fixture-has_cmd",
                "safe_run" => "fixture-safe_run",
                _ => "fixture-log",
            });
        }
    }
    // The four headings appear in order, each under a divider.
    let mut heading_idx = 0usize;
    let mut last_divider = false;
    for line in &lines {
        if is_divider(line) {
            last_divider = true;
            continue;
        }
        if last_divider && heading_idx < BASH_HEADINGS.len() && *line == BASH_HEADINGS[heading_idx]
        {
            heading_idx += 1;
        }
        last_divider = false;
    }
    if heading_idx != BASH_HEADINGS.len() {
        out.push("layout");
    }
    // `# End of script` is the last content line.
    if lines.iter().rev().find(|l| !l.trim().is_empty()).copied() != Some("# End of script") {
        out.push("end-of-script");
    }
    if text.chars().any(is_emoji) {
        out.push("emoji");
    }

    out
}

/// Whether bash is available to run the checkers and syntax checks.
fn has_bash() -> bool {
    Command::new("bash")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Whether pwsh (PowerShell 7) is available. It is required, not optional: the
/// vendored POSIX twin validates the PowerShell script's structure, but only a
/// real PowerShell parser catches a syntax-broken `.ps1`, so its absence makes
/// the gate unable to run rather than a false pass.
fn has_pwsh() -> bool {
    Command::new("pwsh")
        .args(["-NoProfile", "-Command", "$PSVersionTable.PSVersion.Major"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run a command, returning whether it succeeded and its combined output.
fn run_cmd(cmd: &mut Command) -> (bool, String) {
    match cmd.output() {
        Ok(out) => {
            let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
            text.push_str(&String::from_utf8_lossy(&out.stderr));
            (out.status.success(), text)
        }
        Err(e) => (false, e.to_string()),
    }
}

/// Run the wrapper compliance gate. Returns the count of failing checks; an
/// `Err` means the gate could not run (exit 2).
pub fn run(root: &Path) -> io::Result<usize> {
    if !has_bash() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "bash is required to run the wrapper compliance checkers",
        ));
    }
    // pwsh is required too: the vendored POSIX twin validates the PowerShell
    // script's structure, but only a real PowerShell parser catches a
    // syntax-broken .ps1. Its absence is an unable-to-run (exit 2), never a
    // false pass (Codex review of PR #35).
    if !has_pwsh() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "pwsh (PowerShell 7) is required to parse and run the PowerShell wrapper",
        ));
    }

    let sh = root.join("scripts").join("fragcap.sh");
    let ps1 = root.join("scripts").join("Invoke-FragCap.ps1");
    let ps_twin = root.join(".agents/skills/shruggie-powershell/scripts/test-script-compliance.sh");
    // Relative paths, run with the working directory set to the repository root,
    // resolve under native bash, Git Bash, and WSL bash alike; an absolute
    // drive-letter path is not one WSL bash can resolve.
    const SH_REL: &str = "scripts/fragcap.sh";
    const PS1_REL: &str = "scripts/Invoke-FragCap.ps1";
    const TWIN_REL: &str = ".agents/skills/shruggie-powershell/scripts/test-script-compliance.sh";

    let mut fails = 0usize;

    // 1. The authored Bash structural checker.
    match fs::read(&sh) {
        Ok(bytes) => {
            let findings = check_bash(&bytes);
            if findings.is_empty() {
                println!("wrappers: OK  fragcap.sh structure");
            } else {
                for rule in findings {
                    eprintln!("wrappers: FAIL fragcap.sh: {rule}");
                    fails += 1;
                }
            }
        }
        Err(_) => {
            eprintln!("wrappers: FAIL fragcap.sh is missing");
            fails += 1;
        }
    }

    // 2. bash -n syntax check of the Bash wrapper.
    if sh.exists() {
        let (ok, out) = run_cmd(Command::new("bash").current_dir(root).arg("-n").arg(SH_REL));
        if ok {
            println!("wrappers: OK  fragcap.sh parses (bash -n)");
        } else {
            eprintln!("wrappers: FAIL fragcap.sh does not parse:\n{out}");
            fails += 1;
        }
    }

    // 3. The vendored PowerShell checker (its POSIX twin, so only bash is needed).
    if ps_twin.exists() && ps1.exists() {
        let (ok, out) = run_cmd(
            Command::new("bash")
                .current_dir(root)
                .arg(TWIN_REL)
                .arg(PS1_REL),
        );
        if ok {
            println!("wrappers: OK  Invoke-FragCap.ps1 (vendored checker)");
        } else {
            eprintln!("wrappers: FAIL Invoke-FragCap.ps1 (vendored checker):\n{out}");
            fails += 1;
        }
    } else {
        eprintln!("wrappers: FAIL the PowerShell wrapper or vendored checker is missing");
        fails += 1;
    }

    // 4. The Bash wrapper's help and dry-run seams.
    if sh.exists() {
        let (ok, _) = run_cmd(
            Command::new("bash")
                .current_dir(root)
                .arg(SH_REL)
                .arg("--help"),
        );
        if ok {
            println!("wrappers: OK  fragcap.sh --help");
        } else {
            eprintln!("wrappers: FAIL fragcap.sh --help did not exit 0");
            fails += 1;
        }
        let (ok, out) = run_cmd(Command::new("bash").current_dir(root).arg(SH_REL).args([
            "--dry-run",
            "--profile",
            "eso",
            "-o",
            "t-{profile}-{date}.fcapng",
            "--frobnicate",
        ]));
        let assembles = ok
            && out.contains("fragcap run")
            && out.contains("--profile eso")
            && out.contains("--out t-eso-")
            && out.contains("--json")
            && out.contains("--frobnicate");
        if assembles {
            println!("wrappers: OK  fragcap.sh --dry-run assembles the invocation");
        } else {
            eprintln!("wrappers: FAIL fragcap.sh --dry-run did not assemble as expected:\n{out}");
            fails += 1;
        }
    }

    // 5. A real PowerShell parse of the wrapper (syntax the POSIX twin cannot
    // check), then its help and dry-run seams. pwsh is required (guarded above).
    if ps1.exists() {
        let parse = "$e=$null; \
             [void][System.Management.Automation.Language.Parser]::ParseFile(\
             (Resolve-Path 'scripts/Invoke-FragCap.ps1').Path,[ref]$null,[ref]$e); \
             if ($e -and $e.Count -gt 0) { $e | ForEach-Object { $_.Message } | Write-Output; exit 1 }";
        let (ok, out) =
            run_cmd(
                Command::new("pwsh")
                    .current_dir(root)
                    .args(["-NoProfile", "-Command", parse]),
            );
        if ok {
            println!("wrappers: OK  Invoke-FragCap.ps1 parses (PowerShell)");
        } else {
            eprintln!("wrappers: FAIL Invoke-FragCap.ps1 does not parse:\n{out}");
            fails += 1;
        }
        let (ok, _) = run_cmd(
            Command::new("pwsh")
                .args(["-NoProfile", "-File"])
                .arg(&ps1)
                .arg("-Help"),
        );
        if ok {
            println!("wrappers: OK  Invoke-FragCap.ps1 -Help");
        } else {
            eprintln!("wrappers: FAIL Invoke-FragCap.ps1 -Help did not exit 0");
            fails += 1;
        }
        let (ok, out) = run_cmd(
            Command::new("pwsh")
                .args(["-NoProfile", "-File"])
                .arg(&ps1)
                .args([
                    "-DryRun",
                    "-Profile",
                    "eso",
                    "-Out",
                    "t-{profile}-{date}.fcapng",
                    "-Frobnicate",
                ]),
        );
        let assembles = ok && out.contains("fragcap run") && out.contains("--json");
        if assembles {
            println!("wrappers: OK  Invoke-FragCap.ps1 -DryRun assembles the invocation");
        } else {
            eprintln!(
                "wrappers: FAIL Invoke-FragCap.ps1 -DryRun did not assemble as expected:\n{out}"
            );
            fails += 1;
        }
    }

    // 6. The documentation linter is a third `scripts/*.sh` under the same
    // ShruggieTech Bash standard, so the same structural checker and syntax
    // check hold it. It has no dry-run seam; its `--help` must still exit 0.
    let lint_docs = root.join("scripts").join("lint-docs.sh");
    const LINT_DOCS_REL: &str = "scripts/lint-docs.sh";
    match fs::read(&lint_docs) {
        Ok(bytes) => {
            let findings = check_bash(&bytes);
            if findings.is_empty() {
                println!("wrappers: OK  lint-docs.sh structure");
            } else {
                for rule in findings {
                    eprintln!("wrappers: FAIL lint-docs.sh: {rule}");
                    fails += 1;
                }
            }
        }
        Err(_) => {
            eprintln!("wrappers: FAIL lint-docs.sh is missing");
            fails += 1;
        }
    }
    if lint_docs.exists() {
        let (ok, out) = run_cmd(
            Command::new("bash")
                .current_dir(root)
                .arg("-n")
                .arg(LINT_DOCS_REL),
        );
        if ok {
            println!("wrappers: OK  lint-docs.sh parses (bash -n)");
        } else {
            eprintln!("wrappers: FAIL lint-docs.sh does not parse:\n{out}");
            fails += 1;
        }
        let (ok, _) = run_cmd(
            Command::new("bash")
                .current_dir(root)
                .arg(LINT_DOCS_REL)
                .arg("--help"),
        );
        if ok {
            println!("wrappers: OK  lint-docs.sh --help");
        } else {
            eprintln!("wrappers: FAIL lint-docs.sh --help did not exit 0");
            fails += 1;
        }
    }

    Ok(fails)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal compliant Bash skeleton, for the checker's positive case.
    const OK: &str = "#!/usr/bin/env bash\n\
# SPDX-License-Identifier: Apache-2.0\n\
# NAME\n\
# SYNOPSIS\n\
set -euo pipefail\n\
IFS=$'\\n\\t'\n\
#_______________________________________________________________________________\n\
# Declare Functions\n\
    print_help() { :; }\n\
    has_cmd() { :; }\n\
    safe_run() { :; }\n\
    log_info() { :; }\n\
#_______________________________________________________________________________\n\
# Declare Variables and Arrays\n\
#_______________________________________________________________________________\n\
# Execute Operations\n\
#_______________________________________________________________________________\n\
# End of script\n";

    #[test]
    fn a_compliant_skeleton_passes() {
        assert_eq!(check_bash(OK.as_bytes()), Vec::<&str>::new());
    }

    #[test]
    fn a_missing_shebang_fails() {
        let bad = OK.replacen("#!/usr/bin/env bash", "# not a shebang", 1);
        assert!(check_bash(bad.as_bytes()).contains(&"shebang"));
    }

    #[test]
    fn a_missing_spdx_fails() {
        let bad = OK.replacen("# SPDX-License-Identifier: Apache-2.0\n", "", 1);
        assert!(check_bash(bad.as_bytes()).contains(&"spdx"));
    }

    #[test]
    fn a_missing_strict_mode_fails() {
        let bad = OK.replacen("set -euo pipefail\n", "", 1);
        assert!(check_bash(bad.as_bytes()).contains(&"strict-mode"));
    }

    #[test]
    fn a_dropped_divider_or_heading_fails_the_layout() {
        let bad = OK.replacen("# Execute Operations\n", "", 1);
        assert!(check_bash(bad.as_bytes()).contains(&"layout"));
    }

    #[test]
    fn an_emoji_fails() {
        let bad = OK.replacen("# NAME", "# NAME \u{1F600}", 1);
        assert!(check_bash(bad.as_bytes()).contains(&"emoji"));
    }

    #[test]
    fn end_of_script_must_be_last() {
        let bad = format!("{OK}echo trailing\n");
        assert!(check_bash(bad.as_bytes()).contains(&"end-of-script"));
    }

    #[test]
    fn a_missing_safe_run_fixture_fails() {
        // The `\` line continuations in OK strip the source indentation, so the
        // fixture lines sit at column zero in the actual string.
        let bad = OK.replacen("safe_run() { :; }\n", "", 1);
        assert!(check_bash(bad.as_bytes()).contains(&"fixture-safe_run"));
    }

    #[test]
    fn invalid_utf8_fails() {
        let mut bad = OK.as_bytes().to_vec();
        // Splice an invalid UTF-8 byte into the file.
        bad.insert(30, 0xFF);
        assert!(check_bash(&bad).contains(&"utf8"));
    }
}
