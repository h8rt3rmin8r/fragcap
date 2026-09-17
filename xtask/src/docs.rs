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
use std::process::{Command, Stdio};

/// Documentation tooling is non-interactive and cannot open console windows.
fn hidden_command(executable: &str) -> Command {
    let mut command = Command::new(executable);
    command.stdin(Stdio::null());
    if cfg!(windows) {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        }
    }
    command
}

/// A `pnpm` command. On Windows `pnpm` is a `.cmd` shim that the process
/// creation API cannot spawn directly, so it is run through `cmd /C`; elsewhere
/// it is a real executable on PATH.
fn pnpm() -> Command {
    let mut command = if cfg!(windows) {
        let mut c = hidden_command("cmd");
        c.args(["/C", "pnpm"]);
        c
    } else {
        hidden_command("pnpm")
    };
    command.env("PNPM_CONFIG_CONFIRM_MODULES_PURGE", "false");
    command
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
    hidden_command(cmd)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run one focused CLI-reference contract variant.
fn cli_reference(root: &Path, net: bool) -> i32 {
    let mut command = hidden_command(env!("CARGO"));
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
    match crate::docs_coverage::run(root) {
        Ok(problems) if problems.is_empty() => {
            println!(
                "docs: thirteen-topic native product contract passes (not independent acceptance)"
            );
        }
        Ok(problems) => {
            for problem in problems {
                eprintln!("docs-coverage: {problem}");
            }
            return 1;
        }
        Err(error) => {
            eprintln!("docs-coverage: could not run: {error}");
            return 2;
        }
    }
    if !has("bash") {
        eprintln!("docs: bash is required to run scripts/lint-docs.sh");
        return 2;
    }
    let lint = match hidden_command("bash")
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
    let built = pnpm()
        .env("CI", "true")
        .current_dir(&site)
        .arg("build")
        .status();
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
    match dev_status(pnpm().current_dir(&site).arg("dev")) {
        Ok(s) if s.success() => 0,
        Ok(_) => 1,
        Err(e) => {
            eprintln!("docs: could not start the dev server: {e}");
            2
        }
    }
}

fn dev_status(command: &mut Command) -> std::io::Result<std::process::ExitStatus> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Threading::{CREATE_NO_WINDOW, CREATE_SUSPENDED};

        // Reuse the controlled Windows launcher's non-inheritable kill-on-close
        // job. Suspension prevents cmd from launching descendants before assignment.
        // Windows closes the parent's job handle even on abrupt Ctrl+C termination.
        let job = crate::windows_integration::WindowsJob::new()?;
        command.creation_flags(CREATE_NO_WINDOW | CREATE_SUSPENDED);
        let mut child = command.spawn()?;
        if let Err(error) = job
            .assign(&child)
            .and_then(|()| crate::windows_integration::resume_child(child.id()))
        {
            // Assignment may have failed, so closing the job alone cannot reap the
            // suspended child. Use only its existing owned creation handle.
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
        let status = child.wait();
        drop(job); // Also reap descendants if the pnpm shim exits before its server.
        status
    }
    #[cfg(not(windows))]
    command.status()
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::process::Child;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    const ROLE: &str = "FRAGCAP_DOCS_JOB_FIXTURE";
    const DIRECTORY: &str = "FRAGCAP_DOCS_JOB_DIRECTORY";
    const MODE: &str = "FRAGCAP_DOCS_JOB_MODE";

    fn fixture_command(directory: &Path, role: &str, mode: &str) -> Command {
        let mut command = hidden_command(std::env::current_exe().unwrap().to_str().unwrap());
        command
            .args(["--exact", "docs::tests::dev_job_fixture", "--nocapture"])
            .env(ROLE, role)
            .env(DIRECTORY, directory)
            .env(MODE, mode)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        command
    }

    fn wait_for_marker(path: &Path) -> u16 {
        let start = Instant::now();
        loop {
            if let Ok(text) = std::fs::read_to_string(path) {
                if let Ok(port) = text.parse() {
                    return port;
                }
            }
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "fixture not ready"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    // Ordinary harness discovery runs this test without a role and launches nothing.
    // Every subprocess fixture has a ten-second lifetime even if ownership regresses.
    #[test]
    fn dev_job_fixture() {
        let Ok(role) = std::env::var(ROLE) else {
            return;
        };
        let directory = std::path::PathBuf::from(std::env::var_os(DIRECTORY).unwrap());
        let mode = std::env::var(MODE).unwrap();
        if role == "parent" {
            let status = dev_status(&mut fixture_command(&directory, "shim", &mode)).unwrap();
            assert!(status.success());
            return;
        }
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut leaf = if role == "shim" {
            Some(fixture_command(&directory, "leaf", &mode).spawn().unwrap())
        } else {
            None
        };
        std::fs::write(
            directory.join(&role),
            listener.local_addr().unwrap().port().to_string(),
        )
        .unwrap();
        if role == "shim" && mode == "normal" {
            wait_for_marker(&directory.join("leaf"));
            return;
        }
        std::thread::sleep(Duration::from_secs(10));
        if let Some(child) = leaf.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    struct Parent(Child);

    impl Drop for Parent {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    fn assert_dev_tree_cleanup(mode: &str) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "fragcap-docs-job-{}-{nonce}-{mode}",
            std::process::id()
        ));
        std::fs::create_dir(&directory).unwrap();
        let mut parent = Parent(fixture_command(&directory, "parent", mode).spawn().unwrap());
        let ports = [
            wait_for_marker(&directory.join("shim")),
            wait_for_marker(&directory.join("leaf")),
        ];
        if mode == "abrupt" {
            // Simulate abrupt xtask termination, which cannot rely on Rust Drop or
            // a console event reaching the hidden cmd/Node descendants.
            parent.0.kill().unwrap();
        }
        let start = Instant::now();
        loop {
            if let Some(status) = parent.0.try_wait().unwrap() {
                if mode == "normal" {
                    assert!(status.success());
                }
                break;
            }
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "parent did not exit"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        for port in ports {
            let start = Instant::now();
            loop {
                if TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).is_ok() {
                    break;
                }
                assert!(
                    start.elapsed() < Duration::from_secs(2),
                    "orphaned dev descendant still owns port {port}"
                );
                std::thread::sleep(Duration::from_millis(20));
            }
        }
        std::fs::remove_file(directory.join("shim")).unwrap();
        std::fs::remove_file(directory.join("leaf")).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn abrupt_dev_parent_exit_releases_shim_and_server_ports() {
        assert_dev_tree_cleanup("abrupt");
    }

    #[test]
    fn normal_dev_shim_exit_releases_descendant_server_port() {
        assert_dev_tree_cleanup("normal");
    }
}
