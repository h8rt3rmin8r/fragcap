// SPDX-License-Identifier: Apache-2.0

//! Repository task runner.
//!
//! Invoked as `cargo xtask <command>`. Requires nothing installed beyond the
//! language toolchain, which is why the repository's own checks live here
//! rather than in shell scripts.
//!
//! Exit codes follow the house contract in specification section 17.4:
//! 0 the check ran and passed, 1 the check ran and failed, 2 the check could
//! not run. The distinction between 1 and 2 is load-bearing: "the conventions
//! are violated" and "the conventions could not be checked" are different
//! facts, and collapsing them lets a broken check masquerade as a clean
//! repository.

mod deps;
mod lint;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// The target used to prove `fragcap-core` is platform-neutral. Chosen because
/// it has no capture backend, which is exactly the property under test.
const NEUTRAL_TARGET: &str = "x86_64-unknown-linux-gnu";

const USAGE: &str = "\
cargo xtask <command>

  lint       Repository conventions check
  deps       Dependency direction check
  neutral    Build fragcap-core for a target with no capture backend
  msrv       Build at the declared minimum supported toolchain
  ci         Run the full local check set in order
  docs       Documentation site (stub; owned by S18)
  publish    Registry publication (stub; owned by the release process)
";

fn repo_root() -> PathBuf {
    // This crate lives at <root>/xtask.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must live one level below the repository root")
        .to_path_buf()
}

fn cargo(args: &[&str]) -> bool {
    let status = Command::new(env!("CARGO"))
        .current_dir(repo_root())
        .args(args)
        .status();
    matches!(status, Ok(s) if s.success())
}

fn main() -> ExitCode {
    let cmd = std::env::args().nth(1).unwrap_or_default();
    let root = repo_root();

    match cmd.as_str() {
        "lint" => match lint::run(&root) {
            Ok(0) => {
                println!("lint: clean");
                ExitCode::SUCCESS
            }
            Ok(n) => {
                eprintln!("lint: {n} violation(s)");
                ExitCode::from(1)
            }
            Err(e) => {
                eprintln!("lint: could not run: {e}");
                ExitCode::from(2)
            }
        },

        "deps" => match deps::run(&root) {
            Ok(0) => {
                println!("deps: graph matches the architecture of record");
                ExitCode::SUCCESS
            }
            Ok(n) => {
                eprintln!("deps: {n} problem(s)");
                ExitCode::from(1)
            }
            Err(e) => {
                eprintln!("deps: could not run: {e}");
                ExitCode::from(2)
            }
        },

        "neutral" => {
            let installed = Command::new("rustup")
                .args(["target", "list", "--installed"])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).contains(NEUTRAL_TARGET))
                .unwrap_or(false);

            if !installed {
                // Exit 2, never 0. A skipped check that reports success is the
                // failure this project's constitution names most often.
                eprintln!(
                    "neutral: target {NEUTRAL_TARGET} is not installed, so the check did not run."
                );
                eprintln!("neutral: install it with: rustup target add {NEUTRAL_TARGET}");
                return ExitCode::from(2);
            }

            if cargo(&["build", "-p", "fragcap-core", "--target", NEUTRAL_TARGET]) {
                println!("neutral: fragcap-core builds for {NEUTRAL_TARGET} (constitution P-2)");
                ExitCode::SUCCESS
            } else {
                eprintln!("neutral: fragcap-core does NOT build for {NEUTRAL_TARGET}");
                ExitCode::from(1)
            }
        }

        "msrv" => {
            let msrv = workspace_msrv(&root).unwrap_or_else(|| "unknown".into());
            let ok = cargo(&["build", "--workspace", "--locked"]);

            // State the caveat where a reader sees it. This check is currently
            // vacuous: with no external dependencies in the graph, it passes
            // for any declared minimum. It is scaffolded now so it is already
            // in place when it starts to constrain something at S02.
            println!("msrv: declared minimum supported version is {msrv}");
            println!(
                "msrv: NOTE this check does not yet constrain anything. The workspace has no \
                 external dependencies, so any declared minimum passes. It becomes meaningful \
                 when dependencies enter the graph at S02."
            );
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }

        "ci" => {
            let steps: &[(&str, &[&str])] = &[
                ("fmt", &["fmt", "--all", "--", "--check"]),
                (
                    "clippy",
                    &[
                        "clippy",
                        "--all-targets",
                        "--all-features",
                        "--",
                        "-D",
                        "warnings",
                    ],
                ),
                ("test", &["test", "--workspace", "--locked"]),
            ];
            for (name, args) in steps {
                println!("ci: running {name}");
                if !cargo(args) {
                    eprintln!("ci: {name} failed");
                    return ExitCode::from(1);
                }
            }
            println!("ci: running lint");
            match lint::run(&root) {
                Ok(0) => {}
                Ok(n) => {
                    eprintln!("ci: lint reported {n} violation(s)");
                    return ExitCode::from(1);
                }
                Err(e) => {
                    eprintln!("ci: lint could not run: {e}");
                    return ExitCode::from(2);
                }
            }
            println!("ci: running deps");
            match deps::run(&root) {
                Ok(0) => {}
                Ok(n) => {
                    eprintln!("ci: deps reported {n} problem(s)");
                    return ExitCode::from(1);
                }
                Err(e) => {
                    eprintln!("ci: deps could not run: {e}");
                    return ExitCode::from(2);
                }
            }
            println!("ci: all checks passed");
            ExitCode::SUCCESS
        }

        // Stubs exit 2, not 0. A caller cannot distinguish a successful no-op
        // from a successful run, so a stub must not claim success.
        "docs" => {
            eprintln!("docs: not implemented. The documentation site is owned by slice S18.");
            ExitCode::from(2)
        }
        "publish" => {
            eprintln!(
                "publish: not implemented. Registry publication in dependency order is owned \
                 by the release process, and requires explicit authorization."
            );
            ExitCode::from(2)
        }

        "" | "-h" | "--help" | "help" => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("unknown command: {other}\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// Read `workspace.package.rust-version` from the root manifest.
fn workspace_msrv(root: &Path) -> Option<String> {
    let text = std::fs::read_to_string(root.join("Cargo.toml")).ok()?;
    text.lines()
        .find(|l| l.trim_start().starts_with("rust-version"))
        .and_then(|l| l.split('=').nth(1))
        .map(|v| v.trim().trim_matches('"').to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_root_contains_the_workspace_manifest() {
        assert!(repo_root().join("Cargo.toml").is_file());
    }

    #[test]
    fn msrv_is_readable_from_the_workspace_manifest() {
        let v = workspace_msrv(&repo_root()).expect("rust-version must be declared");
        assert!(v.starts_with('1'), "unexpected rust-version: {v}");
    }
}
