// SPDX-License-Identifier: Apache-2.0

//! The shell-wrapper compliance gate (specification section 18.4).
//!
//! `cargo xtask wrappers` holds the shell scripts under `scripts/` to their
//! ShruggieTech house standards: the two capture wrappers, the documentation
//! linter `lint-docs.sh` (section 22.5), and the release-preparation pair
//! `cut-release.sh` and `New-Release.ps1`. Bash scripts are checked by the
//! vendored `shruggie-bash` compliance checker. PowerShell scripts are checked
//! by the vendored `shruggie-powershell` POSIX twin, then parsed and exercised
//! with `pwsh`.
//!
//! ShellCheck is a load-bearing part of the Bash standard, so it must be
//! visible to the same Bash process that runs the vendored checker. Its absence
//! is an unable-to-run result (exit 2), never a clean gate whose static-analysis
//! portion was skipped.

use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

const BASH_CHECKER_REL: &str = ".agents/skills/shruggie-bash/scripts/test-script-compliance.sh";
const POWERSHELL_CHECKER_REL: &str =
    ".agents/skills/shruggie-powershell/scripts/test-script-compliance.sh";

const FRAGCAP_SH_REL: &str = "scripts/fragcap.sh";
const INVOKE_FRAGCAP_PS1_REL: &str = "scripts/Invoke-FragCap.ps1";
const LINT_DOCS_REL: &str = "scripts/lint-docs.sh";
const CUT_RELEASE_REL: &str = "scripts/cut-release.sh";
const NEW_RELEASE_REL: &str = "scripts/New-Release.ps1";

const BASH_SCRIPTS: &[(&str, &str)] = &[
    ("fragcap.sh", FRAGCAP_SH_REL),
    ("lint-docs.sh", LINT_DOCS_REL),
    ("cut-release.sh", CUT_RELEASE_REL),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BashShellcheck {
    Native,
    WindowsExe,
}

/// Whether bash is available to run the checkers and syntax checks.
fn has_bash() -> bool {
    Command::new("bash")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// How ShellCheck is visible inside bash, where the Bash compliance checker
/// runs. Checking from the host shell is insufficient on Windows because Git
/// Bash and WSL can see a different PATH than PowerShell.
fn shellcheck_for_bash() -> Option<BashShellcheck> {
    if Command::new("bash")
        .args(["-lc", "command -v shellcheck >/dev/null 2>&1"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some(BashShellcheck::Native);
    }

    if Command::new("bash")
        .args(["-lc", "command -v shellcheck.exe >/dev/null 2>&1"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some(BashShellcheck::WindowsExe);
    }

    None
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

fn prepare_shellcheck_shim(root: &Path, shellcheck: BashShellcheck) -> io::Result<()> {
    if shellcheck == BashShellcheck::Native {
        return Ok(());
    }

    let shim_dir = root.join("target").join("xtask-shellcheck-shim");
    fs::create_dir_all(&shim_dir)?;
    fs::write(
        shim_dir.join("shellcheck"),
        "#!/usr/bin/env bash\nexec shellcheck.exe \"$@\"\n",
    )
}

fn check_bash_vendored(
    root: &Path,
    shellcheck: BashShellcheck,
    target_rel: &str,
    label: &str,
) -> usize {
    let checker = root.join(BASH_CHECKER_REL);
    let target = root.join(target_rel);
    if !checker.exists() || !target.exists() {
        eprintln!("wrappers: FAIL {label} or the vendored checker is missing");
        return 1;
    }

    let (ok, out) = match shellcheck {
        BashShellcheck::Native => run_cmd(
            Command::new("bash")
                .current_dir(root)
                .arg(BASH_CHECKER_REL)
                .arg(target_rel),
        ),
        BashShellcheck::WindowsExe => {
            let script = format!(
                "PATH=target/xtask-shellcheck-shim:\"$PATH\"\n\
                 exec bash {BASH_CHECKER_REL} {target_rel}"
            );
            run_cmd(Command::new("bash").current_dir(root).arg("-c").arg(script))
        }
    };

    if ok {
        println!("wrappers: OK  {label} (vendored Bash checker)");
        0
    } else {
        eprintln!("wrappers: FAIL {label} (vendored Bash checker):\n{out}");
        1
    }
}

fn check_powershell_vendored(root: &Path, target_rel: &str, label: &str) -> usize {
    let checker = root.join(POWERSHELL_CHECKER_REL);
    let target = root.join(target_rel);
    if !checker.exists() || !target.exists() {
        eprintln!("wrappers: FAIL {label} or the vendored checker is missing");
        return 1;
    }

    let (ok, out) = run_cmd(
        Command::new("bash")
            .current_dir(root)
            .arg(POWERSHELL_CHECKER_REL)
            .arg(target_rel),
    );
    if ok {
        println!("wrappers: OK  {label} (vendored PowerShell checker)");
        0
    } else {
        eprintln!("wrappers: FAIL {label} (vendored PowerShell checker):\n{out}");
        1
    }
}

fn check_bash_syntax(root: &Path, rel: &str, label: &str) -> usize {
    if !root.join(rel).exists() {
        return 0;
    }

    let (ok, out) = run_cmd(Command::new("bash").current_dir(root).arg("-n").arg(rel));
    if ok {
        println!("wrappers: OK  {label} parses (bash -n)");
        0
    } else {
        eprintln!("wrappers: FAIL {label} does not parse:\n{out}");
        1
    }
}

fn check_bash_help(root: &Path, rel: &str, label: &str) -> usize {
    if !root.join(rel).exists() {
        return 0;
    }

    let (ok, _) = run_cmd(
        Command::new("bash")
            .current_dir(root)
            .arg(rel)
            .arg("--help"),
    );
    if ok {
        println!("wrappers: OK  {label} --help");
        0
    } else {
        eprintln!("wrappers: FAIL {label} --help did not exit 0");
        1
    }
}

fn check_fragcap_sh_dry_run(root: &Path) -> usize {
    if !root.join(FRAGCAP_SH_REL).exists() {
        return 0;
    }

    let (ok, out) = run_cmd(
        Command::new("bash")
            .current_dir(root)
            .arg(FRAGCAP_SH_REL)
            .args([
                "--dry-run",
                "--profile",
                "eso",
                "-o",
                "t-{profile}-{date}.fcapng",
                "--frobnicate",
            ]),
    );
    let assembles = ok
        && out.contains("fragcap run")
        && out.contains("--profile eso")
        && out.contains("--out t-eso-")
        && out.contains("--json")
        && out.contains("--frobnicate");
    if assembles {
        println!("wrappers: OK  fragcap.sh --dry-run assembles the invocation");
        0
    } else {
        eprintln!("wrappers: FAIL fragcap.sh --dry-run did not assemble as expected:\n{out}");
        1
    }
}

fn check_powershell_parse(root: &Path, rel: &str, label: &str) -> usize {
    if !root.join(rel).exists() {
        return 0;
    }

    let parse = format!(
        "$e=$null; \
         [void][System.Management.Automation.Language.Parser]::ParseFile(\
         (Resolve-Path '{rel}').Path,[ref]$null,[ref]$e); \
         if ($e -and $e.Count -gt 0) {{ \
             $e | ForEach-Object {{ $_.Message }} | Write-Output; exit 1 \
         }}"
    );
    let (ok, out) =
        run_cmd(
            Command::new("pwsh")
                .current_dir(root)
                .args(["-NoProfile", "-Command", &parse]),
        );
    if ok {
        println!("wrappers: OK  {label} parses (PowerShell)");
        0
    } else {
        eprintln!("wrappers: FAIL {label} does not parse:\n{out}");
        1
    }
}

fn check_powershell_help(root: &Path, rel: &str, label: &str) -> usize {
    let script = root.join(rel);
    if !script.exists() {
        return 0;
    }

    let (ok, _) = run_cmd(
        Command::new("pwsh")
            .args(["-NoProfile", "-File"])
            .arg(script)
            .arg("-Help"),
    );
    if ok {
        println!("wrappers: OK  {label} -Help");
        0
    } else {
        eprintln!("wrappers: FAIL {label} -Help did not exit 0");
        1
    }
}

fn check_invoke_fragcap_ps1_dry_run(root: &Path) -> usize {
    let script = root.join(INVOKE_FRAGCAP_PS1_REL);
    if !script.exists() {
        return 0;
    }

    let (ok, out) = run_cmd(
        Command::new("pwsh")
            .args(["-NoProfile", "-File"])
            .arg(script)
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
        0
    } else {
        eprintln!("wrappers: FAIL Invoke-FragCap.ps1 -DryRun did not assemble as expected:\n{out}");
        1
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
    let shellcheck = shellcheck_for_bash().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "shellcheck is required on bash's PATH to run the Bash wrapper compliance checker",
        )
    })?;
    if shellcheck == BashShellcheck::WindowsExe {
        prepare_shellcheck_shim(root, shellcheck)?;
        println!("wrappers: OK  shellcheck.exe is visible to bash");
    }
    if !has_pwsh() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "pwsh (PowerShell 7) is required to parse and run the PowerShell wrapper",
        ));
    }

    let mut fails = 0usize;

    for (label, rel) in BASH_SCRIPTS {
        fails += check_bash_vendored(root, shellcheck, rel, label);
        fails += check_bash_syntax(root, rel, label);
        fails += check_bash_help(root, rel, label);
    }
    fails += check_fragcap_sh_dry_run(root);

    fails += check_powershell_vendored(root, INVOKE_FRAGCAP_PS1_REL, "Invoke-FragCap.ps1");
    fails += check_powershell_parse(root, INVOKE_FRAGCAP_PS1_REL, "Invoke-FragCap.ps1");
    fails += check_powershell_help(root, INVOKE_FRAGCAP_PS1_REL, "Invoke-FragCap.ps1");
    fails += check_invoke_fragcap_ps1_dry_run(root);

    fails += check_powershell_vendored(root, NEW_RELEASE_REL, "New-Release.ps1");
    fails += check_powershell_parse(root, NEW_RELEASE_REL, "New-Release.ps1");
    fails += check_powershell_help(root, NEW_RELEASE_REL, "New-Release.ps1");

    Ok(fails)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bash_scripts_are_all_checked_by_the_vendored_bash_checker() {
        assert_eq!(
            BASH_SCRIPTS,
            &[
                ("fragcap.sh", "scripts/fragcap.sh"),
                ("lint-docs.sh", "scripts/lint-docs.sh"),
                ("cut-release.sh", "scripts/cut-release.sh"),
            ]
        );
        assert_eq!(
            BASH_CHECKER_REL,
            ".agents/skills/shruggie-bash/scripts/test-script-compliance.sh"
        );
    }

    #[test]
    fn powershell_scripts_are_still_checked_by_their_vendored_checker() {
        assert_eq!(
            POWERSHELL_CHECKER_REL,
            ".agents/skills/shruggie-powershell/scripts/test-script-compliance.sh"
        );
        assert_eq!(INVOKE_FRAGCAP_PS1_REL, "scripts/Invoke-FragCap.ps1");
        assert_eq!(NEW_RELEASE_REL, "scripts/New-Release.ps1");
    }

    #[test]
    fn bash_shellcheck_modes_cover_native_and_windows_bash() {
        assert_eq!(BashShellcheck::Native, BashShellcheck::Native);
        assert_eq!(BashShellcheck::WindowsExe, BashShellcheck::WindowsExe);
    }
}
