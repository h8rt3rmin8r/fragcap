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

mod changelog;
mod conformance;
mod deps;
mod docs;
mod failure_matrix;
mod fuzz;
mod license;
mod lint;
mod notes;
mod publish;
mod skills;
mod spec;
mod threat_model;
mod wrappers;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// The target used to prove `fragcap-core` is platform-neutral. Chosen because
/// it has no capture backend, which is exactly the property under test.
const NEUTRAL_TARGET: &str = "x86_64-unknown-linux-gnu";

const USAGE: &str = "\
cargo xtask <command>

  lint       Repository conventions check
  deps       Dependency direction check
  license    Per-crate license, notice, and readme files for publication
  wrappers   Shell wrapper compliance against the ShruggieTech standards
  skills     Vendored skill set: .agents/skills, skills-lock.json, and git agree
  neutral    Build fragcap-core for a target with no capture backend
  msrv       Build at the declared minimum supported toolchain
  ci         Run the full local check set in order
  docs       Documentation site: docs (dev), docs build, docs check
  publish    Registry publication in dependency order (--execute to publish)
  notes      Print release notes for a version, from CHANGELOG.md
  changelog  Assemble changelog.d/ fragments (--check, or --release <ver> <date>)
  conformance Validate native HTTP/TLS evidence (--analyzer requires TShark)
  fuzz       Validate native parser fuzz surfaces, corpora, and CI mapping
  failure-matrix Validate native Deep Capture failure injection and recovery evidence
  threat-model Validate native Deep Capture threats and executable evidence
  spec       Specification currency: Applies-To vs workspace version, fragment spec-impact
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

/// Run cargo and return whether it succeeded along with its combined output.
///
/// Needed where a caller has to tell one failure from another rather than
/// only see that something failed. The output is echoed as well as returned,
/// so an automated log still shows the work.
fn cargo_captured(args: &[&str]) -> (bool, String) {
    match Command::new(env!("CARGO"))
        .current_dir(repo_root())
        .args(args)
        .output()
    {
        Ok(out) => {
            let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
            text.push_str(&String::from_utf8_lossy(&out.stderr));
            print!("{text}");
            (out.status.success(), text)
        }
        Err(e) => (false, format!("could not run cargo: {e}")),
    }
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

        "license" => match license::run(&root) {
            Ok(0) => {
                println!("license: every publishable crate carries its own license text");
                ExitCode::SUCCESS
            }
            Ok(n) => {
                eprintln!("license: {n} problem(s)");
                ExitCode::from(1)
            }
            Err(e) => {
                eprintln!("license: could not run: {e}");
                ExitCode::from(2)
            }
        },

        "wrappers" => match wrappers::run(&root) {
            Ok(0) => {
                println!("wrappers: the shell scripts are compliant");
                ExitCode::SUCCESS
            }
            Ok(n) => {
                eprintln!("wrappers: {n} check(s) failed");
                ExitCode::from(1)
            }
            Err(e) => {
                eprintln!("wrappers: could not run: {e}");
                ExitCode::from(2)
            }
        },

        "skills" => match skills::run(&root) {
            Ok(0) => {
                println!("skills: the vendored set, the lock, and git agree");
                ExitCode::SUCCESS
            }
            Ok(n) => {
                eprintln!("skills: {n} disagreement(s)");
                ExitCode::from(1)
            }
            Err(e) => {
                eprintln!("skills: could not run: {e}");
                ExitCode::from(2)
            }
        },

        "spec" => match spec::run(&root) {
            Ok(0) => {
                println!("spec: the specification and the workspace agree on the version");
                ExitCode::SUCCESS
            }
            Ok(n) => {
                eprintln!("spec: {n} problem(s)");
                ExitCode::from(1)
            }
            Err(e) => {
                eprintln!("spec: could not run: {e}");
                ExitCode::from(2)
            }
        },

        "conformance" => {
            let analyzer = std::env::args().any(|argument| argument == "--analyzer");
            match conformance::run(&root, analyzer) {
                Ok(0) => ExitCode::SUCCESS,
                Ok(_) => ExitCode::from(1),
                Err(error) => {
                    eprintln!("conformance: could not run: {error}");
                    ExitCode::from(2)
                }
            }
        }

        "threat-model" => match threat_model::run(&root) {
            Ok(0) => ExitCode::SUCCESS,
            Ok(_) => ExitCode::from(1),
            Err(error) => {
                eprintln!("threat-model: could not run: {error}");
                ExitCode::from(2)
            }
        },

        "fuzz" => match fuzz::run(&root) {
            Ok(0) => ExitCode::SUCCESS,
            Ok(_) => ExitCode::from(1),
            Err(error) => {
                eprintln!("fuzz: could not run: {error}");
                ExitCode::from(2)
            }
        },

        "failure-matrix" => match failure_matrix::run(&root) {
            Ok(0) => ExitCode::SUCCESS,
            Ok(_) => ExitCode::from(1),
            Err(error) => {
                eprintln!("failure-matrix: could not run: {error}");
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

            // Every crate with a platform backend, not just core. Until slice
            // S09 this built `fragcap-core` alone, while the specification
            // claimed that `fragcap-capture` also builds for a target with no
            // capture backend. That claim was true and nothing checked it,
            // which the S09 analyze gate caught. Slice S10 added
            // `fragcap-attr` for the same reason and by the same route: its
            // socket table backend is Windows-only, and nothing would
            // otherwise have checked that the crate still builds where that
            // backend does not exist.
            //
            // Slice S11 gave that crate a second Windows-only backend, the ETW
            // process watcher, so the same build now covers both. Nothing had
            // to change here, which is the point of having added it.
            //
            // All three are built with default features, which is what leaves
            // the live source, the socket table, and the process watcher
            // compiled out.
            let mut ok = true;
            for crate_name in ["fragcap-core", "fragcap-capture", "fragcap-attr"] {
                if cargo(&["build", "-p", crate_name, "--target", NEUTRAL_TARGET]) {
                    println!(
                        "neutral: {crate_name} builds for {NEUTRAL_TARGET} (constitution P-2)"
                    );
                } else {
                    eprintln!("neutral: {crate_name} does NOT build for {NEUTRAL_TARGET}");
                    ok = false;
                }
            }
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }

        // Builds at the declared minimum, not with the pinned toolchain.
        //
        // Until S02 this ran an ordinary build and reported success, which
        // checked the pinned toolchain and said nothing about the minimum. That
        // was harmless while the dependency graph was empty and every minimum
        // passed trivially. It stopped being harmless the moment a real
        // dependency arrived, so the check now either uses the minimum
        // toolchain or exits 2 to say it could not run.
        "msrv" => {
            let msrv = match workspace_msrv(&root) {
                Some(v) => v,
                None => {
                    eprintln!("msrv: could not read rust-version from the workspace manifest");
                    return ExitCode::from(2);
                }
            };
            println!("msrv: declared minimum supported version is {msrv}");

            let installed = Command::new("rustup")
                .args(["toolchain", "list"])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).contains(&msrv))
                .unwrap_or(false);

            if !installed {
                // Exit 2, never 0. A check that did not run must not be
                // indistinguishable from one that passed.
                eprintln!("msrv: toolchain {msrv} is not installed, so the check did not run.");
                eprintln!("msrv: install it with: rustup toolchain install {msrv}");
                return ExitCode::from(2);
            }

            // Through rustup, not through `env!("CARGO")`. The latter is the
            // pinned toolchain's own cargo binary, which does not understand a
            // `+toolchain` directive because that is a rustup shim feature.
            // A separate target directory, for two reasons. This process is
            // `target/debug/xtask.exe`, and a workspace build would try to
            // replace the running binary and fail on Windows. It also keeps a
            // second toolchain's artifacts from thrashing the main cache.
            let built = Command::new("rustup")
                .current_dir(&root)
                .args([
                    "run",
                    &msrv,
                    "cargo",
                    "build",
                    "--workspace",
                    "--locked",
                    "--target-dir",
                    "target/msrv",
                ])
                .status();

            if matches!(built, Ok(s) if s.success()) {
                println!("msrv: the workspace builds at {msrv}");
                ExitCode::SUCCESS
            } else {
                eprintln!("msrv: the workspace does NOT build at {msrv}");
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
            println!("ci: running license");
            match license::run(&root) {
                Ok(0) => {}
                Ok(n) => {
                    eprintln!("ci: license reported {n} problem(s)");
                    return ExitCode::from(1);
                }
                Err(e) => {
                    eprintln!("ci: license could not run: {e}");
                    return ExitCode::from(2);
                }
            }
            println!("ci: running wrappers");
            match wrappers::run(&root) {
                Ok(0) => {}
                Ok(n) => {
                    eprintln!("ci: wrappers reported {n} failed check(s)");
                    return ExitCode::from(1);
                }
                Err(e) => {
                    eprintln!("ci: wrappers could not run: {e}");
                    return ExitCode::from(2);
                }
            }
            println!("ci: running skills");
            match skills::run(&root) {
                Ok(0) => {}
                Ok(n) => {
                    eprintln!("ci: skills reported {n} disagreement(s)");
                    return ExitCode::from(1);
                }
                Err(e) => {
                    eprintln!("ci: skills could not run: {e}");
                    return ExitCode::from(2);
                }
            }
            println!("ci: running docs check");
            match docs::check(&root) {
                0 => {}
                2 => {
                    eprintln!("ci: docs check could not run");
                    return ExitCode::from(2);
                }
                _ => {
                    eprintln!("ci: docs check reported failures");
                    return ExitCode::from(1);
                }
            }
            println!("ci: running spec");
            match spec::run(&root) {
                Ok(0) => {}
                Ok(n) => {
                    eprintln!("ci: spec reported {n} problem(s)");
                    return ExitCode::from(1);
                }
                Err(e) => {
                    eprintln!("ci: spec could not run: {e}");
                    return ExitCode::from(2);
                }
            }
            println!("ci: running native Deep Capture threat model");
            match threat_model::run(&root) {
                Ok(0) => {}
                Ok(n) => {
                    eprintln!("ci: threat model reported {n} problem(s)");
                    return ExitCode::from(1);
                }
                Err(e) => {
                    eprintln!("ci: threat model could not run: {e}");
                    return ExitCode::from(2);
                }
            }
            println!("ci: running native parser fuzz inventory");
            match fuzz::run(&root) {
                Ok(0) => {}
                Ok(n) => {
                    eprintln!("ci: fuzz inventory reported {n} problem(s)");
                    return ExitCode::from(1);
                }
                Err(e) => {
                    eprintln!("ci: fuzz inventory could not run: {e}");
                    return ExitCode::from(2);
                }
            }
            println!("ci: running native Deep Capture failure matrix");
            match failure_matrix::run(&root) {
                Ok(0) => {}
                Ok(n) => {
                    eprintln!("ci: failure matrix reported {n} problem(s)");
                    return ExitCode::from(1);
                }
                Err(e) => {
                    eprintln!("ci: failure matrix could not run: {e}");
                    return ExitCode::from(2);
                }
            }
            println!("ci: running native HTTP/TLS conformance evidence");
            match conformance::run(&root, false) {
                Ok(0) => {}
                Ok(n) => {
                    eprintln!("ci: conformance reported {n} problem(s)");
                    return ExitCode::from(1);
                }
                Err(e) => {
                    eprintln!("ci: conformance could not run: {e}");
                    return ExitCode::from(2);
                }
            }
            println!("ci: all checks passed");
            ExitCode::SUCCESS
        }

        // The single entry point for the documentation site (section 22.6):
        // `docs` (dev server), `docs build` (static export), `docs check`
        // (linter), each returning the 0/1/2 contract.
        "docs" => {
            let code = match std::env::args().nth(2).as_deref() {
                Some("build") => docs::build(&root),
                Some("check") => docs::check(&root),
                None => docs::dev(&root),
                Some(other) => {
                    eprintln!(
                        "docs: unknown subcommand: {other}\n\nUse: docs | docs build | docs check"
                    );
                    2
                }
            };
            ExitCode::from(code as u8)
        }
        "notes" => match std::env::args().nth(2) {
            Some(version) => match notes::run(&root, &version) {
                0 => ExitCode::SUCCESS,
                _ => ExitCode::from(1),
            },
            None => {
                eprintln!("notes: a version is required, for example: cargo xtask notes 0.2.0");
                ExitCode::from(2)
            }
        },

        // Fold the `changelog.d/` fragments into `CHANGELOG.md`. `--check`
        // previews the assembled body; `--release <version> <date>` rewrites
        // the file and removes the consumed fragments. Both share one tested
        // transform in `changelog`.
        "changelog" => {
            let sub = std::env::args().nth(2);
            let mode = match sub.as_deref() {
                Some("--check") | None => Some(changelog::Mode::Check),
                Some("--release") => match (std::env::args().nth(3), std::env::args().nth(4)) {
                    (Some(version), Some(date)) => Some(changelog::Mode::Release { version, date }),
                    _ => {
                        eprintln!(
                            "changelog: --release needs a version and a date, for example: \
                                 changelog --release 0.2.0 2026-08-12"
                        );
                        return ExitCode::from(2);
                    }
                },
                Some(other) => {
                    eprintln!(
                        "changelog: unknown argument: {other}\n\n\
                         Use: changelog [--check] | changelog --release <version> <date>"
                    );
                    return ExitCode::from(2);
                }
            };
            match mode.map(|m| changelog::run(&root, m)) {
                Some(Ok(0)) => ExitCode::SUCCESS,
                Some(Ok(_)) => ExitCode::from(1),
                Some(Err(e)) => {
                    eprintln!("changelog: could not run: {e}");
                    ExitCode::from(2)
                }
                None => ExitCode::from(2),
            }
        }

        // Publishing changes the outside world and cannot be undone, so
        // `--execute` is required. Without it this prints the plan.
        "publish" => {
            let execute = std::env::args().any(|a| a == "--execute");
            match publish::run(&root, execute) {
                0 => ExitCode::SUCCESS,
                _ => ExitCode::from(1),
            }
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
