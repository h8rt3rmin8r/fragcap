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
use std::path::{Path, PathBuf};

#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
#[cfg(windows)]
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

use super::action::{
    offered_actions, Action, ActionKind, ActionOutcome, Capabilities, ExtcapScope,
};
pub(crate) use super::residue::register_session_owner;
use super::residue::{owner_is_active, registered_session_owners, SESSION_OWNER_REGISTRY};
use super::{checks, probe, Report};
use crate::emit::Emitter;
use crate::exit::{CliError, Exit};

const RECOVERY_LOCK: &str = "recovery.lock";

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
    fn perform(
        &mut self,
        action: &Action,
        out: &mut dyn Write,
        emitter: &mut Emitter,
    ) -> ActionOutcome;
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
    emitter: &mut Emitter,
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
        let outcome = performer.perform(&action, out, emitter);
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
pub fn run_fix(
    caps: Capabilities,
    yes: bool,
    color: bool,
    out: &mut dyn Write,
    emitter: &mut Emitter,
) -> Exit {
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
    let outcomes = drive_actions(
        &report,
        caps,
        confirm.as_mut(),
        &mut performer,
        out,
        emitter,
    );

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
    let _ = writeln!(
        out,
        "Capture: {}",
        if rechecked.capture_ready() {
            "ready"
        } else {
            "not ready"
        }
    );
    let _ = writeln!(
        out,
        "Deep Capture: {}",
        if rechecked.deep_capture_ready() {
            "ready"
        } else {
            "not ready"
        }
    );
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
    fn perform(
        &mut self,
        action: &Action,
        out: &mut dyn Write,
        emitter: &mut Emitter,
    ) -> ActionOutcome {
        match action.kind {
            ActionKind::RunDiscovery => to_outcome(
                crate::commands::targets::run_discovery_default(out, emitter),
            ),
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
    match recover_deep_capture_journals_with_legacy_confirmation(&root, out) {
        Ok(()) => {
            let _ = writeln!(out, "  replayed exact Deep Capture recovery actions");
            ActionOutcome::Performed
        }
        Err(errors) => ActionOutcome::Failed(format!(
            "exact Deep Capture recovery failed: {}",
            errors.join("; ")
        )),
    }
}

pub(crate) fn recover_deep_capture_journals(
    root: &Path,
    out: &mut dyn Write,
) -> Result<(), Vec<String>> {
    recover_deep_capture_journals_inner(root, out, false)
}

fn recover_deep_capture_journals_with_legacy_confirmation(
    root: &Path,
    out: &mut dyn Write,
) -> Result<(), Vec<String>> {
    recover_deep_capture_journals_inner(root, out, true)
}

fn recover_deep_capture_journals_inner(
    root: &Path,
    out: &mut dyn Write,
    legacy_confirmed: bool,
) -> Result<(), Vec<String>> {
    let _recovery_lock = RecoveryLock::acquire(root).map_err(|error| {
        vec![format!(
            "could not acquire Deep Capture recovery lock: {error}"
        )]
    })?;
    let mut failed = Vec::new();
    let mut roots = vec![root.to_path_buf()];
    let mut recoverable_entries = Vec::new();
    let mut active_bundles = Vec::new();
    match registered_session_owners(root) {
        Ok(entries) => {
            for entry in entries {
                match owner_is_active(&entry) {
                    Ok(true) => active_bundles.push(entry.bundle),
                    Ok(false) => {
                        roots.push(entry.bundle.clone());
                        recoverable_entries.push(entry);
                    }
                    Err(error)
                        if error.kind() == std::io::ErrorKind::Unsupported
                            && entry.lease_id.is_none()
                            && (legacy_confirmed || legacy_owner_is_terminal(&entry)) =>
                    {
                        roots.push(entry.bundle.clone());
                        recoverable_entries.push(entry);
                    }
                    Err(error)
                        if error.kind() == std::io::ErrorKind::Unsupported
                            && entry.lease_id.is_none() =>
                    {
                        active_bundles.push(entry.bundle.clone());
                        failed.push(format!(
                            "legacy session owner {} has no generation lease; run `fragcap doctor --fix` and confirm exact recovery",
                            entry.registry_path.display()
                        ));
                    }
                    Err(error) => failed.push(format!(
                        "could not determine session owner {}: {error}",
                        entry.registry_path.display()
                    )),
                }
            }
        }
        Err(error) => failed.push(format!("session owner registry is invalid: {error}")),
    }
    roots.sort();
    roots.dedup();
    for journal in roots
        .iter()
        .flat_map(|root| resource_journals(root))
        .filter(|journal| {
            !active_bundles
                .iter()
                .any(|bundle| journal.starts_with(bundle))
        })
    {
        match fragcap::deep_capture::recover_resource_journal(&journal, |action| {
            match action.kind {
                fragcap::deep_capture::ResourceKind::Trust => {
                    let thumbprint = action.target.strip_prefix("sha1:").ok_or_else(|| {
                        "trust recovery target lacks a SHA-1 thumbprint".to_string()
                    })?;
                    remove_ca_trust("CurrentUser/Root", thumbprint)?;
                    Ok("removed the exact current-user trust entry".to_string())
                }
                fragcap::deep_capture::ResourceKind::Route
                | fragcap::deep_capture::ResourceKind::Proxy
                | fragcap::deep_capture::ResourceKind::Capture => Ok(
                    "process-scoped resource cannot persist after the owning process ended"
                        .to_string(),
                ),
                _ => Err("no unrelated-resource-safe recovery adapter exists".to_string()),
            }
        }) {
            Ok(plan) => {
                for refusal in &plan.refusals {
                    let detail = format!(
                        "{} resource {} refused: {}",
                        journal.display(),
                        refusal.resource_id,
                        refusal.reason
                    );
                    let _ = writeln!(out, "  {detail}");
                    failed.push(detail);
                }
                match fragcap::deep_capture::read_resource_journal(&journal) {
                    Ok(prefix) => {
                        let latest = prefix.latest();
                        for action in &plan.actions {
                            match latest.get(action.resource_id.as_str()) {
                                Some(transition) if transition.state.terminal() => {
                                    let _ = writeln!(
                                        out,
                                        "  recovered {} resource {}",
                                        journal.display(),
                                        action.resource_id
                                    );
                                }
                                Some(transition) => failed.push(format!(
                                    "{} resource {} ended recovery in {}",
                                    journal.display(),
                                    action.resource_id,
                                    transition.state.as_str()
                                )),
                                None => failed.push(format!(
                                    "{} lost resource {} during recovery",
                                    journal.display(),
                                    action.resource_id
                                )),
                            }
                        }
                    }
                    Err(error) => failed.push(format!(
                        "could not verify replay of {}: {error}",
                        journal.display()
                    )),
                }
            }
            Err(error) => failed.push(format!("could not replay {}: {error}", journal.display())),
        }
    }
    if failed.is_empty() {
        for entry in recoverable_entries {
            if let Err(error) = std::fs::remove_file(&entry.registry_path) {
                if error.kind() != std::io::ErrorKind::NotFound {
                    return Err(vec![format!(
                        "could not retire session owner registry entry {}: {error}",
                        entry.registry_path.display()
                    )]);
                }
            }
        }
        let _ = std::fs::remove_dir(root.join(SESSION_OWNER_REGISTRY));
        Ok(())
    } else {
        Err(failed)
    }
}

fn legacy_owner_is_terminal(owner: &super::residue::SessionOwner) -> bool {
    let journal = owner.bundle.join(fragcap::deep_capture::RESOURCE_JOURNAL);
    fragcap::deep_capture::read_resource_journal(&journal).is_ok_and(|prefix| {
        prefix.status == fragcap::deep_capture::JournalStatus::Complete
            && prefix
                .latest()
                .into_values()
                .all(|transition| transition.state.terminal())
    })
}

struct RecoveryLock {
    path: PathBuf,
}

impl RecoveryLock {
    fn acquire(root: &Path) -> std::io::Result<Self> {
        std::fs::create_dir_all(root)?;
        let path = root.join(RECOVERY_LOCK);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            match std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&path)
            {
                Ok(mut file) => {
                    writeln!(file, "{}", std::process::id())?;
                    file.sync_all()?;
                    return Ok(Self { path });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    let owner = std::fs::read_to_string(&path)
                        .ok()
                        .and_then(|value| value.trim().parse::<u32>().ok());
                    let owner_is_active = match owner {
                        Some(owner) => active_process_ids()?.contains(&owner),
                        None => false,
                    };
                    if !owner_is_active {
                        match std::fs::remove_file(&path) {
                            Ok(()) => continue,
                            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                            Err(error) => return Err(error),
                        }
                    }
                    if std::time::Instant::now() >= deadline {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::WouldBlock,
                            "another live process is still recovering session resources",
                        ));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(error) => return Err(error),
            }
        }
    }
}

impl Drop for RecoveryLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(windows)]
fn active_process_ids() -> std::io::Result<std::collections::HashSet<u32>> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(std::io::Error::last_os_error());
    }
    let mut ids = std::collections::HashSet::new();
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..unsafe { std::mem::zeroed() }
    };
    let mut present = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
    while present {
        ids.insert(entry.th32ProcessID);
        present = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
    }
    unsafe { CloseHandle(snapshot) };
    Ok(ids)
}

#[cfg(not(windows))]
fn active_process_ids() -> std::io::Result<std::collections::HashSet<u32>> {
    Ok(std::iter::once(std::process::id()).collect())
}

fn resource_journals(root: &Path) -> Vec<PathBuf> {
    fn walk(path: &Path, depth: usize, result: &mut Vec<PathBuf>) {
        if depth > 3 {
            return;
        }
        let Ok(entries) = std::fs::read_dir(path) else {
            return;
        };
        for entry in entries.flatten().take(200) {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, depth + 1, result);
            } else if path
                .file_name()
                .is_some_and(|name| name == fragcap::deep_capture::RESOURCE_JOURNAL)
            {
                result.push(path);
            }
        }
    }
    let mut result = Vec::new();
    walk(root, 0, &mut result);
    result.sort();
    result
}

#[cfg(windows)]
fn remove_ca_trust(store: &str, thumbprint: &str) -> Result<(), String> {
    if store != "CurrentUser/Root" {
        return Err(format!(
            "refused native trust mutation outside CurrentUser/Root: {store}"
        ));
    }
    fragcap::deep_capture::NativeCertificateStore
        .remove_current_user_thumbprint(thumbprint)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(not(windows))]
fn remove_ca_trust(store: &str, thumbprint: &str) -> Result<(), String> {
    Err(format!(
        "cannot remove {thumbprint} from {store}: Windows certificate stores are unavailable"
    ))
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
        fn perform(
            &mut self,
            action: &Action,
            _out: &mut dyn Write,
            _emitter: &mut Emitter,
        ) -> ActionOutcome {
            self.performed.push(action.kind);
            (self.outcome)(action.kind)
        }
    }

    fn test_emitter(err: &mut Vec<u8>) -> Emitter<'_> {
        Emitter::new(
            err,
            crate::emit::Format::Human,
            crate::emit::Verbosity::Normal,
        )
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
        let mut err = Vec::new();
        let mut emitter = test_emitter(&mut err);
        let outcomes = drive_actions(
            &rpt,
            caps(),
            &mut confirm,
            &mut performer,
            &mut out,
            &mut emitter,
        );
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
        let mut err = Vec::new();
        let mut emitter = test_emitter(&mut err);
        let outcomes = drive_actions(
            &rpt,
            caps(),
            &mut confirm,
            &mut performer,
            &mut out,
            &mut emitter,
        );
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
        let mut err = Vec::new();
        let mut emitter = test_emitter(&mut err);
        let outcomes = drive_actions(
            &rpt,
            caps(),
            &mut confirm,
            &mut performer,
            &mut out,
            &mut emitter,
        );
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
        let mut err = Vec::new();
        let mut emitter = test_emitter(&mut err);
        let outcomes = drive_actions(
            &rpt,
            Capabilities {
                net: false,
                elevation: true,
            },
            &mut confirm,
            &mut performer,
            &mut out,
            &mut emitter,
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
    fn deep_capture_cleanup_preserves_artifacts_without_journal_authority() {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

        let dir = tempfile::tempdir().expect("tempdir");
        let session = dir.path().join("session-one");
        std::fs::create_dir_all(&session).expect("session dir");
        let keylog = session.join("tls-keylog.log");
        let app = session.join("application.jsonl");
        let clean_manifest = session.join("manifest.json");
        let nested = session.join("nested");
        std::fs::create_dir_all(&nested).expect("nested dir");
        let declared_app = nested.join("app-stream.jsonl");
        let stale = session.join("stale");
        std::fs::create_dir_all(&stale).expect("stale dir");
        let stale_manifest = stale.join("manifest.json");
        let unrelated_empty_dir = session.join("operator-empty-dir");
        std::fs::create_dir_all(&unrelated_empty_dir).expect("unrelated empty dir");
        let keep = session.join("notes.txt");
        std::fs::write(&keylog, "secret-adjacent").expect("keylog");
        std::fs::write(&app, "{}").expect("application");
        std::fs::write(&declared_app, "{}").expect("declared application");
        std::fs::write(
            &clean_manifest,
            r#"{"cleanup":{"status":"succeeded"},"artifacts":[{"role":"application-jsonl","path":"nested/app-stream.jsonl"}]}"#,
        )
        .expect("clean manifest");
        std::fs::write(&stale_manifest, r#"{"cleanup":{"status":"failed"}}"#)
            .expect("stale manifest");
        std::fs::write(&keep, "keep").expect("unrelated");

        std::env::set_var(crate::paths::SESSION_DIR_ENV, dir.path());
        let mut out = Vec::new();
        let outcome = cleanup_deep_capture(&mut out);
        std::env::remove_var(crate::paths::SESSION_DIR_ENV);

        assert_eq!(outcome, ActionOutcome::Performed);
        assert!(
            keylog.exists(),
            "completed retained key-log evidence remains"
        );
        assert!(
            app.exists(),
            "completed retained application evidence remains"
        );
        assert!(
            declared_app.exists(),
            "manifest-declared completed evidence remains"
        );
        assert!(
            clean_manifest.exists(),
            "successful manifests are historical records, not residue"
        );
        assert!(
            stale_manifest.exists(),
            "a manifest alone is not journal authority for deletion"
        );
        assert!(keep.exists(), "unrelated file remains");
        assert!(
            unrelated_empty_dir.exists(),
            "unrelated empty directories remain"
        );
    }

    #[test]
    fn session_owner_registry_round_trips_an_absolute_bundle() {
        let default = tempfile::tempdir().expect("default root");
        let custom = tempfile::tempdir().expect("custom root");
        let bundle = custom.path().join("custom-session");
        std::fs::create_dir(&bundle).expect("custom bundle");
        let _lease =
            register_session_owner(default.path(), &bundle).expect("register custom bundle");
        let roots = registered_session_owners(default.path()).expect("read registry");
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].bundle, bundle.canonicalize().unwrap());
        assert_eq!(roots[0].owner_pid, std::process::id());
        assert!(roots[0].registry_path.is_file());
    }

    #[test]
    fn startup_recovery_preserves_a_custom_root_owned_by_this_live_process() {
        let default = tempfile::tempdir().expect("default root");
        let custom = tempfile::tempdir().expect("custom root");
        let bundle = custom.path().join("active-session");
        std::fs::create_dir(&bundle).expect("custom bundle");
        let _lease =
            register_session_owner(default.path(), &bundle).expect("register custom bundle");

        recover_deep_capture_journals(default.path(), &mut std::io::sink())
            .expect("an active session is skipped rather than replayed");

        let roots = registered_session_owners(default.path()).expect("read registry");
        assert_eq!(
            roots.len(),
            1,
            "the active owner's entry remains registered"
        );
        assert_eq!(roots[0].bundle, bundle.canonicalize().unwrap());
        assert_eq!(roots[0].owner_pid, std::process::id());
    }

    #[test]
    fn startup_recovery_replays_only_an_abandoned_exact_journal() {
        use fragcap::deep_capture::{
            JournalStatus, ResourceJournal, ResourceKind, ResourceState, ResourceTransition,
        };

        let root = tempfile::tempdir().expect("root");
        let bundle = root.path().join("abandoned-session");
        std::fs::create_dir(&bundle).expect("bundle");
        let lease = register_session_owner(root.path(), &bundle).expect("owner");
        let mut journal = ResourceJournal::create(&bundle, "session", "plan").expect("journal");
        journal
            .append(ResourceTransition::new(
                "proxy-listener",
                ResourceKind::Proxy,
                "127.0.0.1:54321",
                "session:session",
                "close-loopback-listener",
                ResourceState::Pending,
                "obligation",
            ))
            .expect("pending");
        journal
            .append(ResourceTransition::new(
                "proxy-listener",
                ResourceKind::Proxy,
                "127.0.0.1:54321",
                "session:session",
                "close-loopback-listener",
                ResourceState::Applied,
                "bound",
            ))
            .expect("applied");
        drop(journal);
        drop(lease);

        recover_deep_capture_journals(root.path(), &mut std::io::sink()).expect("recover");

        let prefix = fragcap::deep_capture::read_resource_journal(
            &bundle.join(fragcap::deep_capture::RESOURCE_JOURNAL),
        )
        .expect("recovered journal");
        assert_eq!(prefix.status, JournalStatus::Complete);
        assert_eq!(
            prefix.latest()["proxy-listener"].state,
            ResourceState::Released
        );
        assert!(registered_session_owners(root.path())
            .expect("owners")
            .is_empty());
    }

    #[test]
    fn failed_exact_recovery_remains_journaled_and_registered() {
        use fragcap::deep_capture::{
            ResourceJournal, ResourceKind, ResourceState, ResourceTransition,
        };

        let root = tempfile::tempdir().expect("root");
        let bundle = root.path().join("failed-session");
        std::fs::create_dir(&bundle).expect("bundle");
        let lease = register_session_owner(root.path(), &bundle).expect("owner");
        let mut journal = ResourceJournal::create(&bundle, "session", "plan").expect("journal");
        journal
            .append(ResourceTransition::new(
                "artifact",
                ResourceKind::Artifact,
                "session/session-output",
                "session:session",
                "remove-artifact",
                ResourceState::Applied,
                "retained",
            ))
            .expect("applied");
        drop(journal);
        drop(lease);

        let errors = recover_deep_capture_journals(root.path(), &mut std::io::sink())
            .expect_err("artifact recovery has no broad deletion adapter");
        assert!(errors
            .iter()
            .any(|error| error.contains("ended recovery in failed")));
        let prefix = fragcap::deep_capture::read_resource_journal(
            &bundle.join(fragcap::deep_capture::RESOURCE_JOURNAL),
        )
        .expect("failed journal");
        assert_eq!(prefix.latest()["artifact"].state, ResourceState::Failed);
        assert_eq!(
            registered_session_owners(root.path())
                .expect("owners")
                .len(),
            1,
            "failed recovery retains exact retry authority"
        );
    }

    #[test]
    fn legacy_owner_requires_confirmation_before_recovery() {
        use fragcap::deep_capture::{
            ResourceJournal, ResourceKind, ResourceState, ResourceTransition,
        };

        let root = tempfile::tempdir().expect("root");
        let bundle = root.path().join("legacy-session");
        std::fs::create_dir(&bundle).expect("bundle");
        write_legacy_owner(root.path(), &bundle);
        let mut journal = ResourceJournal::create(&bundle, "legacy", "plan").expect("journal");
        journal
            .append(ResourceTransition::new(
                "proxy",
                ResourceKind::Proxy,
                "127.0.0.1:54321",
                "session:legacy",
                "close-loopback-listener",
                ResourceState::Applied,
                "bound",
            ))
            .expect("applied");
        drop(journal);

        let errors = recover_deep_capture_journals(root.path(), &mut std::io::sink())
            .expect_err("startup cannot infer legacy liveness");
        assert!(errors.iter().any(|error| error.contains("doctor --fix")));
        assert_eq!(registered_session_owners(root.path()).unwrap().len(), 1);

        recover_deep_capture_journals_with_legacy_confirmation(root.path(), &mut std::io::sink())
            .expect("confirmed exact recovery");
        assert!(registered_session_owners(root.path()).unwrap().is_empty());
    }

    #[test]
    fn terminal_legacy_owner_is_retired_without_liveness_guessing() {
        use fragcap::deep_capture::{
            ResourceJournal, ResourceKind, ResourceState, ResourceTransition,
        };

        let root = tempfile::tempdir().expect("root");
        let bundle = root.path().join("legacy-complete");
        std::fs::create_dir(&bundle).expect("bundle");
        write_legacy_owner(root.path(), &bundle);
        let mut journal = ResourceJournal::create(&bundle, "legacy", "plan").expect("journal");
        journal
            .append(ResourceTransition::new(
                "proxy",
                ResourceKind::Proxy,
                "127.0.0.1:54321",
                "session:legacy",
                "close-loopback-listener",
                ResourceState::Released,
                "closed",
            ))
            .expect("released");
        journal.finish().expect("complete journal");

        recover_deep_capture_journals(root.path(), &mut std::io::sink())
            .expect("terminal evidence is safe to retire");
        assert!(registered_session_owners(root.path()).unwrap().is_empty());
    }

    fn write_legacy_owner(root: &Path, bundle: &Path) {
        let registry = root.join(SESSION_OWNER_REGISTRY);
        std::fs::create_dir(&registry).expect("registry");
        let record = serde_json::json!({
            "bundle": bundle.canonicalize().expect("canonical bundle"),
            "owner_pid": std::process::id(),
        });
        std::fs::write(
            registry.join("legacy.json"),
            serde_json::to_vec(&record).expect("json"),
        )
        .expect("owner record");
    }
}
