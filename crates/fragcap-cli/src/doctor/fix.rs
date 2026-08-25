// SPDX-License-Identifier: Apache-2.0

//! The `doctor --fix` action layer: offer, confirm, perform, re-check.
//!
//! This module sits strictly above the pure classifier (slice S056). It never
//! changes what `doctor` decides; it consumes the [`Report`] the classifier
//! produced and offers to perform the remediations that report named, one at a
//! time, under the operator's confirmation. It may act only on actions carried by
//! a check in the current report ([`offered_actions`]), so it can never take a
//! step `doctor` did not first print (FR-003).
//!
//! The decision surface is separated from the side effects for testing (FR-017):
//! [`drive_actions`] is the pure loop over an injected report, an
//! [`ActionConfirm`] seam, and an [`ActionPerformer`] seam, so the offering,
//! confirmation, ordering, and outcome recording are driven from tests with no
//! terminal, no capture driver, no elevation, and no network. The real side
//! effects live in [`RealPerformer`] and are demonstrated at Tier 2, stated not
//! hidden.

use std::io::Write;

use super::action::{
    offered_actions, Action, ActionKind, ActionOutcome, Capabilities, ExtcapScope,
};
use super::{checks, probe, Report};
use crate::exit::{CliError, Exit};

/// A human confirmation for one action. Injected so the loop is driven by a
/// scripted answer in tests. `true` performs the action; `false` skips it.
pub trait ActionConfirm {
    /// Ask whether to perform `action`.
    fn confirm(&mut self, action: &Action, out: &mut dyn Write) -> bool;
}

/// Performs an action and reports the honest outcome. Injected so the loop is
/// tested with a double that takes no side effect.
pub trait ActionPerformer {
    /// Perform `action`, writing any progress to `out`, and return what happened.
    fn perform(&mut self, action: &Action, out: &mut dyn Write) -> ActionOutcome;
}

/// The real console confirmation: prints the action and reads a yes/no from stdin.
/// The default on an empty line or any non-affirmative token is No, the safe
/// default for a tool that may run elevated.
pub struct ConsoleConfirm;

impl ActionConfirm for ConsoleConfirm {
    fn confirm(&mut self, _action: &Action, out: &mut dyn Write) -> bool {
        let _ = write!(out, "  perform this action? [y/N] ");
        let _ = out.flush();
        let mut line = String::new();
        match std::io::stdin().read_line(&mut line) {
            Ok(0) | Err(_) => false,
            Ok(_) => {
                let answer = line.trim().to_ascii_lowercase();
                answer == "y" || answer == "yes"
            }
        }
    }
}

/// Pre-confirms every action, for `--fix --yes` (unattended interactive use).
pub struct YesConfirm;

impl ActionConfirm for YesConfirm {
    fn confirm(&mut self, _action: &Action, _out: &mut dyn Write) -> bool {
        true
    }
}

/// A scripted confirmation for tests: answers are consumed in order, and once
/// exhausted every further prompt is No.
pub struct ScriptedConfirm {
    answers: std::collections::VecDeque<bool>,
}

impl ScriptedConfirm {
    /// A confirmation that answers `answers` in order, then No.
    pub fn new(answers: impl IntoIterator<Item = bool>) -> ScriptedConfirm {
        ScriptedConfirm {
            answers: answers.into_iter().collect(),
        }
    }
}

impl ActionConfirm for ScriptedConfirm {
    fn confirm(&mut self, _action: &Action, _out: &mut dyn Write) -> bool {
        self.answers.pop_front().unwrap_or(false)
    }
}

/// The pure action loop over an injected report and the two seams.
///
/// Offers each report-named action in order (elevation first), asking the
/// confirmation seam and performing on yes; a guidance-only action (a degraded
/// catalog fetch) is surfaced as advice without a prompt. Returns the outcome of
/// each offered action, in the order offered, so a test asserts the offering,
/// confirmation, and recording without any side effect. On a confirmed and
/// performed elevation the loop stops (the elevated child continues), which the
/// returned outcomes record for the caller to detect.
pub fn drive_actions(
    report: &Report,
    caps: Capabilities,
    confirm: &mut dyn ActionConfirm,
    performer: &mut dyn ActionPerformer,
    out: &mut dyn Write,
) -> Vec<(ActionKind, ActionOutcome)> {
    let mut outcomes = Vec::new();
    for action in offered_actions(report, caps) {
        let _ = writeln!(out, "- {}", action.label);
        if action.guidance_only() {
            // No performable form in this build: surface the guidance, record the
            // degraded fallback as what happened, and never prompt for a step it
            // cannot perform. The outcome line is printed like every other action's,
            // so the phase reports one honest result per action.
            report_outcome(out, &ActionOutcome::Degraded);
            outcomes.push((action.kind, ActionOutcome::Degraded));
            continue;
        }
        if !confirm.confirm(&action, out) {
            let _ = writeln!(out, "  skipped");
            outcomes.push((action.kind, ActionOutcome::Skipped));
            continue;
        }
        let outcome = performer.perform(&action, out);
        report_outcome(out, &outcome);
        let handed_off =
            action.kind == ActionKind::RelaunchElevated && outcome == ActionOutcome::Performed;
        outcomes.push((action.kind, outcome));
        if handed_off {
            let _ = writeln!(
                out,
                "  relaunched elevated; the elevated session continues. Stopping here."
            );
            break;
        }
    }
    outcomes
}

/// Print the honest outcome line for an action (P-9).
fn report_outcome(out: &mut dyn Write, outcome: &ActionOutcome) {
    let _ = match outcome {
        ActionOutcome::Performed => writeln!(out, "  done"),
        ActionOutcome::Skipped => writeln!(out, "  skipped"),
        ActionOutcome::Degraded => writeln!(out, "  done (limited form)"),
        ActionOutcome::Failed(reason) => writeln!(out, "  failed: {reason}"),
    };
}

/// The `--fix` shell entry: gather the environment, print the report, offer and
/// perform the actions it named, then re-check and print the updated verdict.
///
/// Not unit tested (it gathers the real environment and constructs the real
/// performer); the tested surface is [`drive_actions`]. The refusal gates
/// (`--json`, non-terminal stdout/stdin, `--yes` without `--fix`) are applied by
/// the command shell before this runs.
pub fn run_fix(caps: Capabilities, yes: bool, color: bool, out: &mut dyn Write) -> Exit {
    let report = checks::run(&probe::gather());
    let _ = write!(out, "{}", report.render_human_with(color));

    let offered = offered_actions(&report, caps);
    if offered.is_empty() {
        // Distinguish a ready machine from one that is blocked but for which the
        // action layer has nothing it can do, so the message is never misleading in
        // exactly the blocked case.
        if report.ready() {
            let _ = writeln!(out, "\nNothing to fix.");
        } else {
            let _ = writeln!(
                out,
                "\nNo automatic fixes are available for the remaining problems; \
                 see the remediations above."
            );
        }
        return report.exit();
    }

    let _ = writeln!(out, "\nProposed actions (only what doctor named above):");
    let mut confirm: Box<dyn ActionConfirm> = if yes {
        Box::new(YesConfirm)
    } else {
        Box::new(ConsoleConfirm)
    };
    let mut performer = RealPerformer { yes };
    let outcomes = drive_actions(&report, caps, confirm.as_mut(), &mut performer, out);

    // A confirmed, performed elevation hands off to the elevated child, which
    // re-checks in its own context; the non-elevated parent stops without a
    // re-run that would report the parent's still-unprivileged state.
    let handed_off = outcomes.iter().any(|(kind, outcome)| {
        *kind == ActionKind::RelaunchElevated && *outcome == ActionOutcome::Performed
    });
    if handed_off {
        return Exit::SUCCESS;
    }

    let rechecked = checks::run(&probe::gather());
    let _ = writeln!(out, "\nRe-checked:");
    if rechecked.ready() {
        let _ = writeln!(out, "Ready to capture.");
    } else {
        let failing = rechecked
            .checks
            .iter()
            .filter(|c| c.status == super::Status::Fail)
            .count();
        let _ = writeln!(
            out,
            "Not ready: {failing} blocking problem(s) remain; see the checks above."
        );
    }
    rechecked.exit()
}

/// The real performer: the Tier-2 side effects. Reuses the standalone command
/// paths (`extcap install`, `catalog update`, the discovery composition) so there
/// is one path to each effect (P-10), and does the platform work (open the vendor
/// page, fetch and launch the installer, relaunch elevated) for the rest.
pub struct RealPerformer {
    /// Whether `--yes` was set, so an elevated relaunch carries it forward and the
    /// elevated child stays unattended.
    yes: bool,
}

impl ActionPerformer for RealPerformer {
    fn perform(&mut self, action: &Action, out: &mut dyn Write) -> ActionOutcome {
        match action.kind {
            ActionKind::RunDiscovery => {
                to_outcome(crate::commands::targets::run_discovery_default(out))
            }
            ActionKind::InstallExtcap(scope) => to_outcome(crate::commands::extcap::install_scope(
                matches!(scope, ExtcapScope::Machine),
                out,
            )),
            // Offline: creates the store and loads the compiled-in detection
            // signatures, so there is no degraded form to branch on.
            ActionKind::InitializeCatalog => {
                to_outcome(crate::commands::catalog::initialize_default(out))
            }
            ActionKind::CleanupDeepCapture => cleanup_deep_capture(out),
            ActionKind::ObtainNpcap | ActionKind::RelaunchNpcapInstaller => {
                if action.degraded {
                    open_download_page(out)
                } else {
                    fetch_and_launch_installer(out)
                }
            }
            ActionKind::RelaunchElevated => relaunch_elevated(self.yes, out),
        }
    }
}

/// Map a command result to an outcome, keeping a failure honest (P-9). Only an
/// `Exit::SUCCESS` is a performed action; a completed command that reported a
/// non-zero exit (an expected failure or a usage error) is a failed action, never
/// a performed one.
fn to_outcome(result: Result<Exit, CliError>) -> ActionOutcome {
    match result {
        Ok(exit) if exit == Exit::SUCCESS => ActionOutcome::Performed,
        Ok(exit) => ActionOutcome::Failed(format!("the action exited with code {}", exit.code())),
        Err(e) => ActionOutcome::Failed(e.message().to_string()),
    }
}

fn cleanup_deep_capture(out: &mut dyn Write) -> ActionOutcome {
    let Some(root) = crate::paths::deep_capture_session_dir() else {
        return ActionOutcome::Failed(
            "Deep Capture session storage path could not be determined".to_string(),
        );
    };
    if !root.is_dir() {
        let _ = writeln!(out, "  no Deep Capture session storage found");
        return ActionOutcome::Performed;
    }
    let root = match root.canonicalize() {
        Ok(root) => root,
        Err(err) => {
            return ActionOutcome::Failed(format!(
                "could not resolve Deep Capture session storage: {err}"
            ))
        }
    };
    let mut removed = 0usize;
    let mut failed = Vec::new();
    for path in deep_capture_cleanup_candidates(&root) {
        if !path.starts_with(&root) {
            failed.push(format!(
                "refused path outside session storage: {}",
                path.display()
            ));
            continue;
        }
        match std::fs::remove_file(&path) {
            Ok(()) => {
                removed += 1;
                let _ = writeln!(out, "  removed {}", path.display());
            }
            Err(err) => failed.push(format!("{}: {err}", path.display())),
        }
    }
    remove_empty_session_dirs(&root);
    if failed.is_empty() {
        let _ = writeln!(out, "  removed {removed} Deep Capture residue file(s)");
        ActionOutcome::Performed
    } else {
        ActionOutcome::Failed(format!(
            "removed {removed} file(s), failed to remove {} file(s): {}",
            failed.len(),
            failed.join("; ")
        ))
    }
}

fn deep_capture_cleanup_candidates(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    fn walk(dir: &std::path::Path, depth: usize, out: &mut Vec<std::path::PathBuf>) {
        if depth > 3 || out.len() > 200 {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            if out.len() > 200 {
                break;
            }
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            if file_type.is_dir() {
                walk(&path, depth + 1, out);
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let cleanup_manifest =
                name == "manifest.json" && super::probe::manifest_cleanup_unfinished(&path);
            let sensitive_sidecar = matches!(
                name,
                "application.jsonl"
                    | "http.har"
                    | "tls-keylog.log"
                    | "sslkeylog.log"
                    | "proxy.jsonl"
                    | "process-trace.jsonl"
            );
            if cleanup_manifest || sensitive_sidecar {
                out.push(path);
            }
        }
    }

    let mut out = Vec::new();
    walk(root, 0, &mut out);
    out
}

fn remove_empty_session_dirs(root: &std::path::Path) {
    fn walk(dir: &std::path::Path) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                let path = entry.path();
                walk(&path);
                let _ = std::fs::remove_dir(&path);
            }
        }
    }
    walk(root);
}

/// The stable vendor URL for the Windows installer that provides npcap. Wireshark
/// publishes a version-independent "latest" alias, so this is not a versioned URL
/// that rots. fragcap fetches the vendor's own signed installer and stores nothing
/// of it in any fragcap artifact (amended constitution Licensing rule 2). Compiled
/// only in a `net`-capable build, where the fetch action is offered in its primary
/// form.
#[cfg(feature = "net")]
const NPCAP_INSTALLER_URL: &str =
    "https://www.wireshark.org/download/win64/Wireshark-latest-x64.exe";

/// The degraded npcap action: name the official download location and, on Windows,
/// open it in the default browser. Prints the link unconditionally so the operator
/// always has it even when a browser cannot be opened.
fn open_download_page(out: &mut dyn Write) -> ActionOutcome {
    use fragcap::core::WIRESHARK_DOWNLOAD_URL;
    let _ = writeln!(
        out,
        "  obtain npcap from https://npcap.com, or install Wireshark ({WIRESHARK_DOWNLOAD_URL}), \
         whose installer also provides npcap"
    );
    #[cfg(windows)]
    {
        let opened = std::process::Command::new("cmd")
            .args(["/C", "start", "", WIRESHARK_DOWNLOAD_URL])
            .spawn();
        if let Err(e) = opened {
            let _ = writeln!(out, "  (could not open a browser: {e})");
        }
    }
    ActionOutcome::Degraded
}

/// A per-invocation nonce for the installer's temporary filename, so the launched
/// path is not predictable.
#[cfg(feature = "net")]
fn installer_nonce() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

/// The primary npcap action (a `net`-capable build): fetch the vendor's own signed
/// installer to a temporary file and launch it. Tier 2 (not run in continuous
/// integration). Without network capability this arm is never offered as primary
/// (the action degrades), so the fallback keeps the build honest.
#[cfg(feature = "net")]
fn fetch_and_launch_installer(out: &mut dyn Write) -> ActionOutcome {
    let _ = writeln!(
        out,
        "  fetching the vendor installer from {NPCAP_INSTALLER_URL}"
    );
    let mut body: Vec<u8> = Vec::new();
    let response = match http_req::request::get(NPCAP_INSTALLER_URL, &mut body) {
        Ok(response) => response,
        Err(e) => return ActionOutcome::Failed(format!("download failed: {e}")),
    };
    if !response.status_code().is_success() {
        return ActionOutcome::Failed(format!(
            "download failed: server returned {}",
            response.status_code()
        ));
    }
    // A per-invocation unique name in the temp directory, so a fetch does not
    // clobber an existing file and the path an installer is launched from is not a
    // predictable, pre-creatable target. `create_new` refuses to overwrite an
    // existing file, so a name collision fails loudly rather than reusing a file
    // this run did not write.
    let dir = std::env::temp_dir();
    let unique = format!(
        "fragcap-npcap-installer-{}-{}.exe",
        std::process::id(),
        installer_nonce()
    );
    let path = dir.join(unique);
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(mut file) => {
            if let Err(e) = std::io::Write::write_all(&mut file, &body) {
                return ActionOutcome::Failed(format!("could not write the installer: {e}"));
            }
        }
        Err(e) => {
            return ActionOutcome::Failed(format!("could not create the installer file: {e}"))
        }
    }
    #[cfg(windows)]
    {
        match std::process::Command::new(&path).spawn() {
            Ok(_) => {
                let _ = writeln!(out, "  launched the vendor installer; follow its prompts");
                ActionOutcome::Performed
            }
            Err(e) => ActionOutcome::Failed(format!("could not launch the installer: {e}")),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = out;
        ActionOutcome::Failed("launching the installer is Windows-only".to_string())
    }
}

/// The fallback when the build cannot fetch. Never reached as a primary action (it
/// degrades to [`open_download_page`] first); present so every build compiles.
#[cfg(not(feature = "net"))]
fn fetch_and_launch_installer(out: &mut dyn Write) -> ActionOutcome {
    open_download_page(out)
}

/// Relaunch this same `doctor --fix` invocation elevated. Tier 2 (Windows). The
/// elevated child re-checks in its own context; the caller stops the parent on a
/// performed elevation.
#[cfg(windows)]
fn relaunch_elevated(yes: bool, out: &mut dyn Write) -> ActionOutcome {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;

    let Ok(exe) = std::env::current_exe() else {
        return ActionOutcome::Failed("could not determine the fragcap binary".to_string());
    };
    let to_wide =
        |s: &std::ffi::OsStr| -> Vec<u16> { s.encode_wide().chain(std::iter::once(0)).collect() };
    let verb: Vec<u16> = "runas\0".encode_utf16().collect();
    let file = to_wide(exe.as_os_str());
    // Carry `--yes` across the handoff so an unattended `doctor --fix --yes` stays
    // unattended in the elevated child rather than prompting for every action.
    let command_line = if yes {
        "doctor --fix --yes\0"
    } else {
        "doctor --fix\0"
    };
    let params: Vec<u16> = command_line.encode_utf16().collect();
    // SAFETY: all string pointers are to NUL-terminated wide buffers that outlive
    // the call; the null hwnd (0) and null directory are the documented values,
    // and SW_SHOWNORMAL is a valid show flag, for a shell verb invocation. On the
    // windows-sys 0.36 line HWND/HINSTANCE are isize.
    let result = unsafe {
        ShellExecuteW(
            0,
            verb.as_ptr(),
            file.as_ptr(),
            params.as_ptr(),
            std::ptr::null(),
            1, // SW_SHOWNORMAL
        )
    };
    // ShellExecuteW returns a value greater than 32 on success.
    if result > 32 {
        let _ = writeln!(out, "  requested an elevated relaunch");
        ActionOutcome::Performed
    } else {
        ActionOutcome::Failed("the elevation request was declined or failed".to_string())
    }
}

/// Elevation is a Windows concept; on other targets it cannot run. (The action is
/// not offered on non-Windows platforms, so this is a defensive fallback.)
#[cfg(not(windows))]
fn relaunch_elevated(_yes: bool, _out: &mut dyn Write) -> ActionOutcome {
    ActionOutcome::Failed("relaunching elevated is Windows-only".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::action::{Action, ActionKind, ExtcapScope};
    use crate::doctor::{Check, Report};
    use std::sync::{Mutex, OnceLock};

    const S: &str = "Section";

    /// A performer that records the actions it was asked to perform and returns a
    /// canned outcome per kind, so the loop is tested with no side effect.
    struct ScriptedPerformer {
        outcome: fn(ActionKind) -> ActionOutcome,
        performed: Vec<ActionKind>,
    }

    impl ActionPerformer for ScriptedPerformer {
        fn perform(&mut self, action: &Action, _out: &mut dyn Write) -> ActionOutcome {
            self.performed.push(action.kind);
            (self.outcome)(action.kind)
        }
    }

    fn report(actions: Vec<Action>) -> Report {
        let checks = actions
            .into_iter()
            .enumerate()
            .map(|(i, a)| {
                let name = Box::leak(format!("c{i}").into_boxed_str());
                Check::warn_action(S, name, "d", "r", a)
            })
            .collect();
        Report { checks }
    }

    fn caps() -> Capabilities {
        Capabilities {
            net: true,
            elevation: true,
        }
    }

    #[test]
    fn a_confirmed_action_is_performed_and_a_declined_one_is_skipped() {
        let rpt = report(vec![
            Action::new(ActionKind::InstallExtcap(ExtcapScope::User)),
            Action::new(ActionKind::RunDiscovery),
        ]);
        let mut confirm = ScriptedConfirm::new([true, false]);
        let mut performer = ScriptedPerformer {
            outcome: |_| ActionOutcome::Performed,
            performed: Vec::new(),
        };
        let mut out = Vec::new();
        let outcomes = drive_actions(&rpt, caps(), &mut confirm, &mut performer, &mut out);
        assert_eq!(
            outcomes,
            vec![
                (
                    ActionKind::InstallExtcap(ExtcapScope::User),
                    ActionOutcome::Performed
                ),
                (ActionKind::RunDiscovery, ActionOutcome::Skipped),
            ]
        );
        // The declined action was never handed to the performer.
        assert_eq!(
            performer.performed,
            vec![ActionKind::InstallExtcap(ExtcapScope::User)]
        );
    }

    #[test]
    fn a_failed_action_is_recorded_as_failed_not_performed() {
        let rpt = report(vec![Action::new(ActionKind::RunDiscovery)]);
        let mut confirm = ScriptedConfirm::new([true]);
        let mut performer = ScriptedPerformer {
            outcome: |_| ActionOutcome::Failed("boom".to_string()),
            performed: Vec::new(),
        };
        let mut out = Vec::new();
        let outcomes = drive_actions(&rpt, caps(), &mut confirm, &mut performer, &mut out);
        assert_eq!(
            outcomes,
            vec![(
                ActionKind::RunDiscovery,
                ActionOutcome::Failed("boom".to_string())
            )]
        );
        assert!(String::from_utf8_lossy(&out).contains("failed: boom"));
    }

    #[test]
    fn a_performed_elevation_stops_the_loop() {
        // Elevation is offered first; a performed elevation stops the loop so the
        // parent does not keep acting after handing off (the later action is not
        // offered).
        let rpt = report(vec![
            Action::new(ActionKind::RunDiscovery),
            Action::new(ActionKind::RelaunchElevated),
        ]);
        let mut confirm = ScriptedConfirm::new([true, true]);
        let mut performer = ScriptedPerformer {
            outcome: |_| ActionOutcome::Performed,
            performed: Vec::new(),
        };
        let mut out = Vec::new();
        let outcomes = drive_actions(&rpt, caps(), &mut confirm, &mut performer, &mut out);
        assert_eq!(
            outcomes.first().map(|(k, _)| *k),
            Some(ActionKind::RelaunchElevated)
        );
        assert_eq!(outcomes.len(), 1, "the loop stopped after the handoff");
        assert_eq!(performer.performed, vec![ActionKind::RelaunchElevated]);
    }

    #[test]
    fn an_offline_action_is_performed_even_without_network_capability() {
        // The catalog action used to be net-gated, so in a default build it was
        // surfaced as guidance, recorded Degraded, and never performed. It is now
        // an offline initialize-and-seed, so a build with no network capability
        // still performs it. That is the whole of issue #175's dead end: the
        // remediation a shipped binary was offered was one it could not run.
        let rpt = report(vec![Action::new(ActionKind::InitializeCatalog)]);
        let mut confirm = ScriptedConfirm::new([true]);
        let mut performer = ScriptedPerformer {
            outcome: |_| ActionOutcome::Performed,
            performed: Vec::new(),
        };
        let mut out = Vec::new();
        let outcomes = drive_actions(
            &rpt,
            Capabilities {
                net: false,
                elevation: true,
            },
            &mut confirm,
            &mut performer,
            &mut out,
        );
        assert_eq!(
            outcomes,
            vec![(ActionKind::InitializeCatalog, ActionOutcome::Performed)]
        );
        assert_eq!(
            performer.performed,
            vec![ActionKind::InitializeCatalog],
            "an offline action reaches the performer even with net: false"
        );
    }

    #[test]
    fn deep_capture_cleanup_removes_only_known_session_files() {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

        let dir = tempfile::tempdir().expect("tempdir");
        let session = dir.path().join("session-one");
        std::fs::create_dir_all(&session).expect("session dir");
        let keylog = session.join("tls-keylog.log");
        let app = session.join("application.jsonl");
        let clean_manifest = session.join("manifest.json");
        let stale = session.join("stale");
        std::fs::create_dir_all(&stale).expect("stale dir");
        let stale_manifest = stale.join("manifest.json");
        let keep = session.join("notes.txt");
        std::fs::write(&keylog, "secret-adjacent").expect("keylog");
        std::fs::write(&app, "{}").expect("application");
        std::fs::write(&clean_manifest, r#"{"cleanup":{"status":"succeeded"}}"#)
            .expect("clean manifest");
        std::fs::write(&stale_manifest, r#"{"cleanup":{"status":"failed"}}"#)
            .expect("stale manifest");
        std::fs::write(&keep, "keep").expect("unrelated");

        std::env::set_var(crate::paths::SESSION_DIR_ENV, dir.path());
        let mut out = Vec::new();
        let outcome = cleanup_deep_capture(&mut out);
        std::env::remove_var(crate::paths::SESSION_DIR_ENV);

        assert_eq!(outcome, ActionOutcome::Performed);
        assert!(!keylog.exists(), "known key-log residue removed");
        assert!(!app.exists(), "known application sidecar removed");
        assert!(
            clean_manifest.exists(),
            "successful manifests are historical records, not residue"
        );
        assert!(!stale_manifest.exists(), "unfinished manifest removed");
        assert!(keep.exists(), "unrelated file remains");
    }
}
