// SPDX-License-Identifier: Apache-2.0

//! The pure per-check classifiers of specification section 26.3.
//!
//! Each is a function from the injected [`Inputs`] to one [`Check`], and
//! [`run`] assembles them in section order into a [`Report`]. The two npcap
//! options are separate checks, each naming its own remediation when absent
//! (the Licensing section's detect-never-install rule and slice S14 research
//! D-f), and the tracing check is a blocking fail only when the session is
//! elevated and the process-event session could not open.

use super::action::{Action, ActionKind, ExtcapScope};
use super::{Check, Inputs, Privilege, Report, Subsystem};
use fragcap::core::WIRESHARK_DOWNLOAD_URL;

const IDENTITY: &str = "Identity";
const PLATFORM: &str = "Platform";
const DRIVER: &str = "Capture driver";
const TRACING: &str = "Tracing";
const INTERFACES: &str = "Interfaces";
const INTEGRATION: &str = "Integration";
const PROFILES: &str = "Profiles";
const PREPARATION: &str = "Preparation";

/// Where to obtain npcap. fragcap never installs it (the Licensing section). The
/// Wireshark URL is single-sourced from [`WIRESHARK_DOWNLOAD_URL`], whose
/// installer also provides npcap, so one download resolves both.
fn npcap_source() -> String {
    format!(
        "fragcap does not install npcap; obtain it from https://npcap.com, or install Wireshark \
         ({WIRESHARK_DOWNLOAD_URL}), whose installer also provides npcap, then run doctor again"
    )
}

/// Run every classifier in section order.
pub fn run(inputs: &Inputs) -> Report {
    let mut checks = identity(inputs);
    checks.extend([
        os(inputs),
        subsystem(inputs),
        privilege(inputs),
        npcap(inputs),
        loopback(inputs),
        winpcap_api(inputs),
        live_backend(inputs),
        socket_table_backend(inputs),
        tracing(inputs),
    ]);
    checks.extend(interfaces(inputs));
    checks.push(integration(inputs));
    checks.push(profiles(inputs));
    // The two preparation checks surface a row only when there is something to do,
    // so a ready machine (a catalog present, at least one target entry) is
    // unchanged. Each carries the action the `--fix` layer performs (slice S056).
    checks.extend(catalog_store(inputs));
    checks.extend(target_entries(inputs));
    Report { checks }
}

/// The leading identity section: which fragcap produced this report and where it
/// keeps its data. Every row is informational (ok) and never changes the exit
/// status; a path is shown whether or not it exists yet, and an unresolvable one
/// renders as "undetermined" rather than an empty or wrong value.
fn identity(inputs: &Inputs) -> Vec<Check> {
    fn path_detail(path: &Option<std::path::PathBuf>) -> String {
        match path {
            Some(path) => path.display().to_string(),
            None => "undetermined".to_string(),
        }
    }
    // A store is created on first run, so its path is shown with whether it exists
    // yet; a missing store is normal before the first capture, not a fault.
    fn store_detail(path: &Option<std::path::PathBuf>, present: bool) -> String {
        match path {
            Some(path) => format!(
                "{} ({})",
                path.display(),
                if present { "present" } else { "absent" }
            ),
            None => "undetermined".to_string(),
        }
    }
    vec![
        Check::ok(IDENTITY, "version", inputs.fragcap_version.clone()),
        Check::ok(IDENTITY, "binary", path_detail(&inputs.binary_path)),
        Check::ok(IDENTITY, "profile dir", path_detail(&inputs.profile_dir)),
        Check::ok(
            IDENTITY,
            "catalog db",
            store_detail(&inputs.catalog_db_path, inputs.catalog_db_present),
        ),
        Check::ok(
            IDENTITY,
            "local db",
            store_detail(&inputs.local_db_path, inputs.local_db_present),
        ),
    ]
}

fn os(inputs: &Inputs) -> Check {
    Check::ok(PLATFORM, "os", inputs.os.clone())
}

fn subsystem(inputs: &Inputs) -> Check {
    match inputs.subsystem {
        Subsystem::Native => Check::ok(PLATFORM, "subsystem", "native"),
        Subsystem::Wsl => Check::warn(
            PLATFORM,
            "subsystem",
            "running under WSL; capture of Windows traffic is constrained",
        ),
    }
}

fn privilege(inputs: &Inputs) -> Check {
    match inputs.privilege {
        Privilege::Elevated => Check::ok(PLATFORM, "privilege", "elevated"),
        Privilege::NotElevated => Check::warn_action(
            PLATFORM,
            "privilege",
            "not elevated; some interfaces and process tracing need an elevated session",
            "relaunch fragcap from an elevated session",
            Action::new(ActionKind::RelaunchElevated),
        ),
    }
}

fn npcap(inputs: &Inputs) -> Check {
    match &inputs.npcap {
        // The probe falls back to the literal "installed" when it cannot read a
        // real version; do not dress that up as "version installed", which would
        // claim a version it does not have (P-9).
        Some(info) if info.version == "installed" => Check::ok(DRIVER, "npcap", "installed"),
        Some(info) => Check::ok(DRIVER, "npcap", format!("version {}", info.version)),
        None => Check::fail_action(
            DRIVER,
            "npcap",
            "npcap is not installed",
            npcap_source(),
            Action::new(ActionKind::ObtainNpcap),
        ),
    }
}

fn loopback(inputs: &Inputs) -> Check {
    match &inputs.npcap {
        None => Check::skip(DRIVER, "loopback adapter", "npcap is not installed"),
        // Three-valued driver truth. Loopback is only needed with --loopback, so
        // neither absence nor an undetermined state blocks standalone doctor; the
        // run/extcap path that actually requests --loopback treats its absence as
        // blocking on that path. The old signal guessed from an unrelated file and
        // is gone; and current npcap installs loopback automatically, with no
        // installer checkbox, so the wording no longer tells the operator to pick
        // one.
        Some(info) => match info.loopback_supported {
            Some(true) => Check::ok(DRIVER, "loopback adapter", "loopback capture supported"),
            Some(false) => Check::warn(
                DRIVER,
                "loopback adapter",
                "loopback capture is not available; it is only needed when capturing \
                 loopback traffic with --loopback",
            ),
            None => Check::warn(
                DRIVER,
                "loopback adapter",
                "loopback support could not be determined",
            ),
        },
    }
}

/// Whether the live capture backend is compiled into this binary. Its absence
/// is blocking: the binary cannot capture at all, and reporting anything softer
/// would let the readiness verdict stay green over a binary that fails every
/// capture (P-9, and the interfaces check's "no interfaces" symptom).
fn live_backend(inputs: &Inputs) -> Check {
    match inputs.live_available {
        Some(_) => Check::ok(DRIVER, "live backend", "live capture backend is built in"),
        None => Check::fail(
            DRIVER,
            "live backend",
            "the live capture backend is not built into this binary",
            "install a build with the `live` feature (the official release enables it); \
             this binary cannot capture without it",
        ),
    }
}

/// Whether the socket-table attribution backend is compiled into this binary.
/// Absence degrades attribution rather than preventing capture, so it warns.
fn socket_table_backend(inputs: &Inputs) -> Check {
    match inputs.socket_table_available {
        Some(_) => Check::ok(
            DRIVER,
            "socket-table backend",
            "socket-table attribution is built in",
        ),
        None => Check::warn(
            DRIVER,
            "socket-table backend",
            "the socket-table attribution backend is not built into this binary; \
             attribution is degraded",
        ),
    }
}

fn winpcap_api(inputs: &Inputs) -> Check {
    match &inputs.npcap {
        None => Check::skip(DRIVER, "winpcap api mode", "npcap is not installed"),
        Some(info) if info.winpcap_api_mode => {
            Check::ok(DRIVER, "winpcap api mode", "WinPcap API compatible")
        }
        Some(_) => Check::fail_action(
            DRIVER,
            "winpcap api mode",
            "WinPcap API compatibility mode is not installed",
            "reinstall npcap with the \"Install Npcap in WinPcap API-compatible Mode\" option \
             enabled",
            Action::new(ActionKind::RelaunchNpcapInstaller),
        ),
    }
}

fn tracing(inputs: &Inputs) -> Check {
    match inputs.etw_available {
        // The tracing capability is not built into this binary. A non-blocking
        // skip: attribution still works from the socket table.
        None => Check::skip(
            TRACING,
            "process events",
            "process-event tracing is not built into this binary",
        ),
        Some(true) => Check::ok(TRACING, "process events", "process-event session available"),
        // Elevated and the session could not open: attribution is degraded, so
        // this blocks. Not elevated and it could not open: not blocking, because
        // the session was never entitled to open one.
        Some(false) => match inputs.privilege {
            Privilege::Elevated => Check::fail(
                TRACING,
                "process events",
                "the process-event session could not open while elevated",
                "ensure no other tool holds the kernel process trace session, then run doctor \
                 again",
            ),
            Privilege::NotElevated => Check::warn(
                TRACING,
                "process events",
                "the process-event session is unavailable without elevation",
            ),
        },
    }
}

fn interfaces(inputs: &Inputs) -> Vec<Check> {
    // A probe that could not run is not an observed-empty machine. When
    // enumeration was attempted and failed, say so distinctly rather than
    // reporting "no interfaces were found", which would present a failed
    // observation as a successful one (P-9).
    if let Some(error) = &inputs.interface_error {
        return vec![Check::warn(
            INTERFACES,
            "adapters",
            format!("interface enumeration failed: {error}"),
        )];
    }
    if inputs.interfaces.is_empty() {
        // When the live backend is absent, the empty set is a consequence of the
        // missing backend, not an npcap/adapter fault. Name the real cause so the
        // operator is not sent chasing an adapter red herring.
        let detail = if inputs.live_available.is_none() {
            "no interfaces were enumerated because the live capture backend is not built \
             into this binary (see the live backend check above)"
        } else {
            "no interfaces were found"
        };
        return vec![Check::warn(INTERFACES, "adapters", detail)];
    }
    inputs
        .interfaces
        .iter()
        .map(|iface| {
            let marker = if iface.is_virtual { " (virtual)" } else { "" };
            // The interface name is folded into the detail: a Check's name is a
            // fixed &'static str slot, and the name is what tells two adapters
            // apart and says which interface an address belongs to.
            if iface.up {
                let addr = iface.addr.as_deref().unwrap_or("no address");
                Check::ok(
                    INTERFACES,
                    "adapter",
                    format!("{}  {}{}", iface.name, addr, marker),
                )
            } else {
                // Never a blocking failure: an interface being down is a warning,
                // because another interface may still carry the traffic.
                Check::warn(
                    INTERFACES,
                    "adapter",
                    format!("{}  down{}", iface.name, marker),
                )
            }
        })
        .collect()
}

fn integration(inputs: &Inputs) -> Check {
    // Registration can be per-user (%APPDATA%\Wireshark\extcap) or machine-wide
    // (Wireshark's system extcap directory, where the MSI's machine-wide option
    // installs it). The check is ok when either scope holds the binary, and the
    // detail names which scope, so a second user on a machine-wide install is not
    // told it is missing (specification section 14.5, FR-009).
    let user = inputs.extcap_dir.as_ref().map(|d| d.display().to_string());
    let system = inputs
        .extcap_system_dir
        .as_ref()
        .map(|d| d.display().to_string());
    match (inputs.extcap_installed, inputs.extcap_system_installed) {
        (true, true) => {
            let detail = match (&user, &system) {
                (Some(u), Some(s)) => {
                    format!("installed for the current user in {u} and machine-wide in {s}")
                }
                (Some(u), None) => {
                    format!("installed for the current user in {u} and machine-wide")
                }
                (None, Some(s)) => format!("installed machine-wide in {s}"),
                (None, None) => "installed for the current user and machine-wide".to_string(),
            };
            Check::ok(INTEGRATION, "analyzer extcap", detail)
        }
        (true, false) => {
            let detail = match &user {
                Some(u) => format!("installed for the current user in {u}"),
                None => "installed for the current user".to_string(),
            };
            Check::ok(INTEGRATION, "analyzer extcap", detail)
        }
        (false, true) => {
            let detail = match &system {
                Some(s) => format!("installed machine-wide in {s}"),
                None => "installed machine-wide".to_string(),
            };
            Check::ok(INTEGRATION, "analyzer extcap", detail)
        }
        (false, false) => {
            // Optional: a warning, never a block. Wireshark's extcap framework is
            // always present; what is not yet in place is fragcap's registration as
            // one of its sources, which `fragcap extcap install` (slice 041) will do.
            // doctor does no Wireshark-presence detection, so if Wireshark itself
            // is absent it cannot tell; the download link is named unconditionally,
            // matching the npcap detect-and-link posture, and its installer also
            // provides npcap.
            let detail = match &user {
                Some(dir) => format!(
                    "not registered as a Wireshark extcap source; run `fragcap extcap install` to \
                     register it in {dir} (optional). Get Wireshark from {WIRESHARK_DOWNLOAD_URL}; \
                     its installer also provides npcap"
                ),
                None => format!(
                    "not registered as a Wireshark extcap source; run `fragcap extcap install` to \
                     register it (optional). Get Wireshark from {WIRESHARK_DOWNLOAD_URL}; its \
                     installer also provides npcap"
                ),
            };
            Check::warn_action(
                INTEGRATION,
                "analyzer extcap",
                detail,
                "run `fragcap extcap install` to register the analyzer integration",
                Action::new(ActionKind::InstallExtcap(ExtcapScope::User)),
            )
        }
    }
}

fn profiles(inputs: &Inputs) -> Check {
    Check::ok(
        PROFILES,
        "profiles",
        format!(
            "bundled: {}, user: {}",
            inputs.bundled_count, inputs.user_count
        ),
    )
}

/// The shipped catalog store, surfaced only when it is absent (slice S056). A
/// present catalog needs no row here (its path already appears in the identity
/// section), so a ready machine is unchanged. Absence is a warning, not a block:
/// detection is degraded without the catalog but capture still works, and the
/// `--fix` layer can fetch it.
fn catalog_store(inputs: &Inputs) -> Option<Check> {
    if inputs.catalog_db_present {
        return None;
    }
    Some(Check::warn_action(
        PREPARATION,
        "catalog store",
        "the shipped catalog store is not present; technology detection is degraded until it \
         is fetched",
        "run `fragcap catalog update` to fetch the current published catalog",
        Action::new(ActionKind::FetchCatalog),
    ))
}

/// Registered target entries, surfaced only when the store holds none (slice
/// S056). `Some(0)` is a real empty store and carries the discovery action; a
/// store with entries needs no row, so a ready machine is unchanged; `None`
/// (undetermined) surfaces nothing rather than a fabricated empty state (P-9).
/// Absence of entries is a warning, not a block: a capture can still name a target
/// by other means, and the `--fix` layer can run discovery.
fn target_entries(inputs: &Inputs) -> Option<Check> {
    if inputs.target_entry_count != Some(0) {
        return None;
    }
    Some(Check::warn_action(
        PREPARATION,
        "target entries",
        "no target entries are registered yet; run discovery to find installed titles",
        "run `fragcap targets` (or `fragcap targets discover`) to register installed titles",
        Action::new(ActionKind::RunDiscovery),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::{IfaceInfo, NpcapInfo, Status};
    use crate::exit::Exit;

    fn ready_inputs() -> Inputs {
        Inputs {
            fragcap_version: "0.0.0-test".to_string(),
            binary_path: Some(std::path::PathBuf::from(
                "C:\\Program Files\\fragcap\\fragcap.exe",
            )),
            profile_dir: Some(std::path::PathBuf::from(
                "C:\\Users\\gamer\\AppData\\Roaming\\fragcap\\profiles",
            )),
            catalog_db_path: Some(std::path::PathBuf::from(
                "C:\\Users\\gamer\\AppData\\Roaming\\fragcap\\catalog.db",
            )),
            catalog_db_present: true,
            local_db_path: Some(std::path::PathBuf::from(
                "C:\\Users\\gamer\\AppData\\Roaming\\fragcap\\local.db",
            )),
            local_db_present: true,
            os: "Windows 11".to_string(),
            subsystem: Subsystem::Native,
            privilege: Privilege::Elevated,
            npcap: Some(NpcapInfo {
                version: "1.79".to_string(),
                loopback_supported: Some(true),
                winpcap_api_mode: true,
            }),
            etw_available: Some(true),
            live_available: Some(true),
            socket_table_available: Some(true),
            interfaces: vec![IfaceInfo {
                name: "Ethernet".to_string(),
                addr: Some("192.0.2.10".to_string()),
                up: true,
                is_virtual: false,
            }],
            interface_error: None,
            extcap_installed: true,
            extcap_dir: Some(std::path::PathBuf::from(
                "C:\\Users\\gamer\\AppData\\Roaming\\Wireshark\\extcap",
            )),
            extcap_system_installed: false,
            extcap_system_dir: Some(std::path::PathBuf::from(
                "C:\\Program Files\\Wireshark\\extcap",
            )),
            bundled_count: 0,
            user_count: 2,
            target_entry_count: Some(3),
        }
    }

    #[test]
    fn a_ready_machine_passes_and_exits_zero() {
        let report = run(&ready_inputs());
        assert!(report.ready());
        assert_eq!(report.exit(), Exit::SUCCESS);
        assert!(report.checks.iter().all(|c| c.status != Status::Fail));
    }

    #[test]
    fn absent_npcap_fails_and_skips_its_options() {
        let mut inputs = ready_inputs();
        inputs.npcap = None;
        let report = run(&inputs);
        assert_eq!(report.exit(), Exit::FAILURE);
        assert_eq!(npcap(&inputs).status, Status::Fail);
        assert_eq!(loopback(&inputs).status, Status::Skip);
        assert_eq!(winpcap_api(&inputs).status, Status::Skip);
    }

    #[test]
    fn the_two_npcap_options_are_independent_with_their_own_severities() {
        // Loopback is only needed with --loopback, so its absence warns and does
        // not block; the WinPcap API option is unaffected and the machine stays
        // ready.
        let mut inputs = ready_inputs();
        if let Some(info) = inputs.npcap.as_mut() {
            info.loopback_supported = Some(false);
        }
        let report = run(&inputs);
        assert_eq!(loopback(&inputs).status, Status::Warn);
        assert_eq!(
            winpcap_api(&inputs).status,
            Status::Ok,
            "the WinPcap API check is unaffected by the loopback option"
        );
        assert!(
            report.ready(),
            "a missing loopback adapter alone does not block readiness"
        );

        // The WinPcap API option, by contrast, is a blocking fail when absent.
        let mut inputs = ready_inputs();
        if let Some(info) = inputs.npcap.as_mut() {
            info.winpcap_api_mode = false;
        }
        assert_eq!(winpcap_api(&inputs).status, Status::Fail);
        assert_eq!(loopback(&inputs).status, Status::Ok);
        assert_eq!(run(&inputs).exit(), Exit::FAILURE);
    }

    #[test]
    fn an_absent_live_backend_blocks_and_names_the_cause() {
        let mut inputs = ready_inputs();
        inputs.live_available = None;
        let report = run(&inputs);
        assert_eq!(live_backend(&inputs).status, Status::Fail);
        assert_eq!(report.exit(), Exit::FAILURE);
        // The empty-interface message points at the missing backend, not npcap.
        inputs.interfaces.clear();
        let ifaces = interfaces(&inputs);
        assert!(
            ifaces
                .iter()
                .any(|c| c.detail.contains("live capture backend")),
            "the empty-interface message names the missing backend: {ifaces:?}"
        );
    }

    #[test]
    fn an_absent_socket_table_backend_only_warns() {
        let mut inputs = ready_inputs();
        inputs.socket_table_available = None;
        let report = run(&inputs);
        assert_eq!(socket_table_backend(&inputs).status, Status::Warn);
        assert!(
            report.ready(),
            "a missing socket-table backend degrades attribution but does not block"
        );
    }

    #[test]
    fn a_real_npcap_version_is_shown_and_the_fallback_is_not_dressed_up() {
        // A real version renders as "version X".
        let inputs = ready_inputs();
        assert!(npcap(&inputs).detail.contains("version 1.79"));

        // The "installed" fallback renders plainly, never "version installed".
        let mut inputs = ready_inputs();
        if let Some(info) = inputs.npcap.as_mut() {
            info.version = "installed".to_string();
        }
        let detail = npcap(&inputs).detail;
        assert_eq!(detail, "installed");
        assert!(!detail.contains("version"));
    }

    #[test]
    fn every_failing_check_carries_a_remediation() {
        let mut inputs = ready_inputs();
        inputs.npcap = Some(NpcapInfo {
            version: "1.79".to_string(),
            loopback_supported: Some(false),
            winpcap_api_mode: false,
        });
        inputs.privilege = Privilege::Elevated;
        inputs.etw_available = Some(false);
        let report = run(&inputs);
        for check in report.checks.iter().filter(|c| c.status == Status::Fail) {
            assert!(
                check.remediation.as_ref().is_some_and(|r| !r.is_empty()),
                "{} has no remediation",
                check.name
            );
        }
    }

    #[test]
    fn tracing_is_a_skip_when_not_built_and_blocks_only_when_elevated() {
        let mut inputs = ready_inputs();
        inputs.etw_available = None;
        assert_eq!(tracing(&inputs).status, Status::Skip);

        inputs.etw_available = Some(false);
        inputs.privilege = Privilege::Elevated;
        assert_eq!(tracing(&inputs).status, Status::Fail);

        inputs.privilege = Privilege::NotElevated;
        assert_eq!(
            tracing(&inputs).status,
            Status::Warn,
            "unavailable without elevation is not a blocking problem"
        );
    }

    #[test]
    fn missing_optional_integration_only_warns() {
        let mut inputs = ready_inputs();
        inputs.extcap_installed = false;
        let report = run(&inputs);
        assert_eq!(integration(&inputs).status, Status::Warn);
        assert!(report.ready(), "an optional warning does not block");
    }

    #[test]
    fn the_integration_check_names_the_extcap_directory() {
        // Installed: the detail names the directory (FR-009, SC-004).
        let inputs = ready_inputs();
        let dir = inputs.extcap_dir.as_ref().unwrap().display().to_string();
        let check = integration(&inputs);
        assert_eq!(check.status, Status::Ok);
        assert!(
            check.detail.contains(&dir),
            "installed names the dir: {}",
            check.detail
        );

        // Not installed: still names the directory, and only warns.
        let mut absent = ready_inputs();
        absent.extcap_installed = false;
        let check = integration(&absent);
        assert_eq!(check.status, Status::Warn);
        assert!(
            check.detail.contains(&dir),
            "not-installed names the dir: {}",
            check.detail
        );

        // Location undetermined: the report says so rather than naming a wrong path.
        let mut unknown = ready_inputs();
        unknown.extcap_dir = None;
        assert!(integration(&unknown).detail.contains("installed"));
        unknown.extcap_installed = false;
        assert!(integration(&unknown).detail.contains("optional"));
    }

    #[test]
    fn integration_recognizes_both_extcap_scopes() {
        // Per-user only: ok, names the current-user scope (SC-002).
        let mut user_only = ready_inputs();
        user_only.extcap_installed = true;
        user_only.extcap_system_installed = false;
        let check = integration(&user_only);
        assert_eq!(check.status, Status::Ok);
        assert!(check.detail.contains("current user"), "{}", check.detail);

        // Machine-wide only: ok, names the machine-wide scope (SC-001). This is
        // the case a second user hit before this slice; it must not warn.
        let mut system_only = ready_inputs();
        system_only.extcap_installed = false;
        system_only.extcap_system_installed = true;
        let check = integration(&system_only);
        assert_eq!(
            check.status,
            Status::Ok,
            "machine-wide must be ok: {}",
            check.detail
        );
        assert!(check.detail.contains("machine-wide"), "{}", check.detail);
        let system_dir = system_only
            .extcap_system_dir
            .as_ref()
            .unwrap()
            .display()
            .to_string();
        assert!(
            check.detail.contains(&system_dir),
            "names system dir: {}",
            check.detail
        );

        // Both scopes: ok, names both.
        let mut both = ready_inputs();
        both.extcap_installed = true;
        both.extcap_system_installed = true;
        let check = integration(&both);
        assert_eq!(check.status, Status::Ok);
        assert!(check.detail.contains("current user"), "{}", check.detail);
        assert!(check.detail.contains("machine-wide"), "{}", check.detail);

        // Neither: the optional warning, unchanged.
        let mut neither = ready_inputs();
        neither.extcap_installed = false;
        neither.extcap_system_installed = false;
        let check = integration(&neither);
        assert_eq!(check.status, Status::Warn);
        assert!(check.detail.contains("optional"), "{}", check.detail);
    }

    #[test]
    fn doctor_points_at_wireshark_from_the_integration_and_npcap_guidance() {
        // The not-registered integration guidance names the Wireshark download
        // URL and stays an optional warning (#107). The URL is single-sourced, so
        // the assertion is against the constant, not a repeated literal.
        let url = fragcap::core::WIRESHARK_DOWNLOAD_URL;

        let mut not_registered = ready_inputs();
        not_registered.extcap_installed = false;
        not_registered.extcap_system_installed = false;
        let check = integration(&not_registered);
        assert_eq!(check.status, Status::Warn);
        assert!(
            check.detail.contains(url),
            "not-registered names the Wireshark URL: {}",
            check.detail
        );
        assert!(check.detail.contains("extcap install"), "{}", check.detail);

        // Still named when the extcap directory cannot be resolved.
        let mut no_dir = ready_inputs();
        no_dir.extcap_installed = false;
        no_dir.extcap_system_installed = false;
        no_dir.extcap_dir = None;
        assert!(integration(&no_dir).detail.contains(url));

        // The npcap-absent remediation single-sources the same Wireshark URL.
        let mut no_npcap = ready_inputs();
        no_npcap.npcap = None;
        let remediation = npcap(&no_npcap).remediation.expect("npcap fail remediates");
        assert!(
            remediation.contains(url),
            "npcap remediation single-sources the Wireshark URL: {remediation}"
        );
    }

    #[test]
    fn no_interfaces_warns_without_blocking() {
        let mut inputs = ready_inputs();
        inputs.interfaces.clear();
        let report = run(&inputs);
        assert!(report
            .checks
            .iter()
            .any(|c| c.section == INTERFACES && c.status == Status::Warn));
        assert!(report.ready());
    }

    #[test]
    fn a_failed_enumeration_is_reported_distinctly_not_as_empty() {
        // A probe that could not run must not be presented as an observed-empty
        // machine: it names the failure and never says "no interfaces were found".
        let mut inputs = ready_inputs();
        inputs.interfaces.clear();
        inputs.interface_error = Some("backend: enumeration failed".to_string());
        let ifaces = interfaces(&inputs);
        assert_eq!(ifaces.len(), 1);
        assert_eq!(ifaces[0].status, Status::Warn);
        assert!(
            ifaces[0].detail.contains("enumeration failed"),
            "{}",
            ifaces[0].detail
        );
        assert!(!ifaces[0].detail.contains("no interfaces were found"));
    }

    #[test]
    fn each_interface_row_names_its_adapter() {
        // The adapter name is in the detail so two interfaces are distinguishable
        // and each address is tied to its interface.
        let mut inputs = ready_inputs();
        inputs.interfaces = vec![
            IfaceInfo {
                name: "Ethernet".to_string(),
                addr: Some("192.0.2.10".to_string()),
                up: true,
                is_virtual: false,
            },
            IfaceInfo {
                name: "Wi-Fi".to_string(),
                addr: Some("192.0.2.20".to_string()),
                up: false,
                is_virtual: false,
            },
        ];
        let ifaces = interfaces(&inputs);
        assert_eq!(ifaces.len(), 2);
        assert!(ifaces[0].detail.contains("Ethernet") && ifaces[0].detail.contains("192.0.2.10"));
        assert_eq!(ifaces[1].status, Status::Warn, "a down interface warns");
        assert!(ifaces[1].detail.contains("Wi-Fi") && ifaces[1].detail.contains("down"));
    }

    #[test]
    fn loopback_is_three_valued_and_never_blocks() {
        // Determined present, determined absent, and undetermined are ok, warn,
        // warn respectively, and none of them blocks readiness (loopback is only
        // needed with --loopback).
        for (state, want) in [
            (Some(true), Status::Ok),
            (Some(false), Status::Warn),
            (None, Status::Warn),
        ] {
            let mut inputs = ready_inputs();
            if let Some(info) = inputs.npcap.as_mut() {
                info.loopback_supported = state;
            }
            assert_eq!(loopback(&inputs).status, want, "state {state:?}");
            assert!(
                run(&inputs).ready(),
                "loopback never blocks: state {state:?}"
            );
        }
        // Undetermined says so and never claims loopback is not installed.
        let mut inputs = ready_inputs();
        if let Some(info) = inputs.npcap.as_mut() {
            info.loopback_supported = None;
        }
        let detail = loopback(&inputs).detail;
        assert!(detail.contains("could not be determined"), "{detail}");
        assert!(!detail.contains("not installed"), "{detail}");
    }

    #[test]
    fn the_identity_section_leads_and_is_informational() {
        let report = run(&ready_inputs());
        // The first four rows are the identity section, all ok.
        let identity: Vec<_> = report
            .checks
            .iter()
            .filter(|c| c.section == IDENTITY)
            .collect();
        assert_eq!(
            identity.len(),
            5,
            "version, binary, profile dir, catalog db, local db"
        );
        assert!(identity.iter().all(|c| c.status == Status::Ok));
        assert_eq!(report.checks[0].section, IDENTITY, "identity leads");
        assert!(
            report.checks[0].detail.contains("0.0.0-test"),
            "version row"
        );
        // Identity is informational: an unresolvable path is an ok note, and the
        // machine stays ready.
        let mut inputs = ready_inputs();
        inputs.binary_path = None;
        inputs.profile_dir = None;
        inputs.catalog_db_path = None;
        inputs.local_db_path = None;
        let report = run(&inputs);
        assert!(report.ready(), "unresolvable paths do not block");
        let binary = report.checks.iter().find(|c| c.name == "binary").unwrap();
        assert_eq!(binary.status, Status::Ok);
        assert_eq!(binary.detail, "undetermined");
    }
}
