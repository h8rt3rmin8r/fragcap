// SPDX-License-Identifier: Apache-2.0

//! The documentation site task (specification section 22.6).
//!
//! `cargo xtask docs` is the single entry point for the site, local and in
//! continuous integration:
//!
//! - `docs` (no argument) starts the site locally with hot reload.
//! - `docs build` produces the static export and asserts it carries the
//!   `.nojekyll` marker and `CNAME` (the same build continuous integration
//!   runs).
//! - `docs check` runs the documentation linter (`scripts/lint-docs.sh check`)
//!   and checks the public CLI reference against the clap command tree under
//!   both the default and network-capable feature sets.
//!
//! Every subcommand returns the house 0/1/2 contract: 0 ran and passed, 1 ran
//! and failed, 2 could not run (a required tool, or the site app, is absent).
//! The site application arrives with sub-slice S18c-2; until then `docs build`
//! and `docs` report the app absent and exit 2 rather than a false pass, while
//! `docs check` already runs because the linter and the glossary exist.

use std::path::Path;
use std::process::Command;

/// A `pnpm` command. On Windows `pnpm` is a `.cmd` shim that the process
/// creation API cannot spawn directly, so it is run through `cmd /C`; elsewhere
/// it is a real executable on PATH.
fn pnpm() -> Command {
    if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.args(["/C", "pnpm"]);
        c
    } else {
        Command::new("pnpm")
    }
}

/// Whether pnpm is available (runs `pnpm --version` and checks success).
fn has_pnpm() -> bool {
    pnpm()
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Whether a command is available (runs `<cmd> --version` and checks success).
fn has(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run one focused CLI-reference contract variant.
fn cli_reference(root: &Path, net: bool) -> i32 {
    let mut command = Command::new(env!("CARGO"));
    command.current_dir(root).args([
        "test",
        "-p",
        "fragcap-cli",
        "--test",
        "cli_reference",
        "--locked",
    ]);
    if net {
        command.args(["--features", "net"]);
    }
    match command.status() {
        Ok(status) if status.success() => 0,
        Ok(_) => 1,
        Err(error) => {
            eprintln!("docs: could not run the CLI-reference check: {error}");
            2
        }
    }
}

/// `docs check`: run the documentation linter and CLI-reference contracts.
/// Returns the 0/1/2 code so the `ci` aggregate can branch on could-not-run.
pub fn check(root: &Path) -> i32 {
    if !has("bash") {
        eprintln!("docs: bash is required to run scripts/lint-docs.sh");
        return 2;
    }
    let lint = match Command::new("bash")
        .current_dir(root)
        .arg("scripts/lint-docs.sh")
        .arg("check")
        .status()
    {
        Ok(s) if s.success() => 0,
        Ok(s) => {
            if s.code() == Some(2) {
                2
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("docs: could not run the linter: {e}");
            2
        }
    };
    if lint != 0 {
        return lint;
    }

    for net in [false, true] {
        let result = cli_reference(root, net);
        if result != 0 {
            return result;
        }
    }
    0
}

/// `docs build`: build the static export with pnpm, then assert its markers.
pub fn build(root: &Path) -> i32 {
    let site = root.join("site");
    if !site.join("package.json").is_file() {
        eprintln!(
            "docs: the site/ application is not present yet (arrives with sub-slice S18c-2); \
             nothing to build."
        );
        return 2;
    }
    if !has_pnpm() {
        eprintln!("docs: pnpm is required to build the documentation site");
        return 2;
    }
    let built = pnpm().current_dir(&site).arg("build").status();
    if !matches!(built, Ok(s) if s.success()) {
        eprintln!("docs: the site build failed");
        return 1;
    }
    let out = site.join("out");
    let nojekyll = out.join(".nojekyll").is_file();
    let cname_ok = std::fs::read_to_string(out.join("CNAME"))
        .map(|c| c.trim() == "fragcap.com")
        .unwrap_or(false);
    if nojekyll && cname_ok {
        println!("docs: static export carries .nojekyll and CNAME (fragcap.com)");
        0
    } else {
        eprintln!(
            "docs: the static export is missing its .nojekyll marker or CNAME (fragcap.com); \
             without the marker the static host strips the asset directory (section 22.2)"
        );
        1
    }
}

/// `docs` (default): start the dev server with hot reload. This blocks until the
/// server is stopped, so it passes the child's exit through.
pub fn dev(root: &Path) -> i32 {
    let site = root.join("site");
    if !site.join("package.json").is_file() {
        eprintln!(
            "docs: the site/ application is not present yet (arrives with sub-slice S18c-2)."
        );
        return 2;
    }
    if !has_pnpm() {
        eprintln!("docs: pnpm is required to run the documentation dev server");
        return 2;
    }
    match pnpm().current_dir(&site).arg("dev").status() {
        Ok(s) if s.success() => 0,
        Ok(_) => 1,
        Err(e) => {
            eprintln!("docs: could not start the dev server: {e}");
            2
        }
    }
}
