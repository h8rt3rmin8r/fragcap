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
use super::{Check, DeepCaptureCa, Inputs, Privilege, Report, Subsystem};
use fragcap::core::WIRESHARK_DOWNLOAD_URL;

const IDENTITY: &str = "Identity";
const PLATFORM: &str = "Platform";
const DRIVER: &str = "Capture driver";
const TRACING: &str = "Tracing";
const INTERFACES: &str = "Interfaces";
const INTEGRATION: &str = "Integration";
const PREPARATION: &str = "Preparation";
const DEEP_CAPTURE: &str = "Deep Capture";

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
    // The two preparation checks surface a row only when there is something to do,
    // so a ready machine (a catalog present, at least one target entry) is
    // unchanged. Each carries the action the `--fix` layer performs (slice S056).
    checks.extend(catalog_store(inputs));
    checks.extend(target_entries(inputs));
    checks.extend(deep_capture(inputs));
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
            // Scope the offered registration by elevation: machine-wide
            // registration writes a system directory that needs an elevated
            // session, so an elevated doctor offers machine-wide (the scope only it
            // can perform) and an unelevated one offers the per-user scope. Both are
            // reachable across runs, and the action label names the scope before the
            // operator confirms it.
            let scope = match inputs.privilege {
                Privilege::Elevated => ExtcapScope::Machine,
                Privilege::NotElevated => ExtcapScope::User,
            };
            Check::warn_action(
                INTEGRATION,
                "analyzer extcap",
                detail,
                "run `fragcap extcap install` to register the analyzer integration",
                Action::new(ActionKind::InstallExtcap(scope)),
            )
        }
    }
}

/// The catalog store, surfaced only when it is absent (slice S056). A present
/// catalog needs no row here (its path already appears in the identity section),
/// so a ready machine is unchanged. Absence is a warning, not a block: detection
/// is degraded without the catalog but capture still works, and the `--fix`
/// layer can create it.
///
/// The remediation is offline, and always was available. This check fires on
/// absence, never on emptiness, and an absent store is exactly what the
/// first-run bootstrap creates and what the compiled-in signature document
/// fills. It used to name `catalog update`, a network fetch that no shipped
/// binary could run and that degraded to telling the user to rebuild fragcap
/// from source (issue #175).
fn catalog_store(inputs: &Inputs) -> Option<Check> {
    if inputs.catalog_db_present {
        return None;
    }
    Some(Check::warn_action(
        PREPARATION,
        "catalog store",
        "the catalog store is not present; technology detection is degraded until it exists",
        "run `fragcap catalog seed` to create it and load the detection signatures",
        Action::new(ActionKind::InitializeCatalog),
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

pub(crate) fn deep_capture(inputs: &Inputs) -> Vec<Check> {
    let dc = &inputs.deep_capture;
    let mut checks = Vec::new();
    checks.push(match &dc.proxy_backend {
        Some(backend) => {
            let detail = match &backend.version {
                Some(version) => format!("{} {version}", backend.name),
                None => format!("{} present, version undetermined", backend.name),
            };
            Check::ok(DEEP_CAPTURE, "proxy backend", detail)
        }
        None => {
            let detail = dc.proxy_backend_error.as_deref().unwrap_or(
                "no supported proxy backend found; Deep Capture proxy inspection is unavailable",
            );
            Check::fail(
                DEEP_CAPTURE,
                "proxy backend",
                detail,
                "use a Windows build containing the native Deep Capture runtime",
            )
        }
    });
    checks.push(loopback_check(
        "IPv4 loopback listener",
        "127.0.0.1",
        &dc.ipv4_loopback,
    ));
    checks.push(loopback_check(
        "IPv6 loopback listener",
        "::1",
        &dc.ipv6_loopback,
    ));
    checks.push(match &dc.ca {
        DeepCaptureCa::Absent => Check::ok(
            DEEP_CAPTURE,
            "local CA trust",
            "no fragcap Deep Capture CA trust found",
        ),
        DeepCaptureCa::CurrentUser { thumbprint } => Check::ok(
            DEEP_CAPTURE,
            "local CA trust",
            format!("trusted in current-user store ({thumbprint})"),
        ),
        DeepCaptureCa::WrongStore { store, thumbprint } => Check::fail(
            DEEP_CAPTURE,
            "local CA trust",
            format!("fragcap CA is trusted in unexpected store {store} ({thumbprint})"),
            "remove only the exact unexpected trust entry using that store's administration tools",
        ),
        DeepCaptureCa::Mismatched {
            expected,
            actual,
            store,
        } => match store {
            Some(store) => Check::fail(
                DEEP_CAPTURE,
                "local CA trust",
                format!("manifest expects {expected}, but {store} contains {actual}"),
                "inspect the manifest and remove only the exact mismatched trust entry",
            ),
            None => Check::warn(
                DEEP_CAPTURE,
                "local CA trust",
                format!("manifest expects {expected}, but bundled CA material is {actual}"),
            ),
        },
        DeepCaptureCa::Unknown(reason) => Check::fail(
            DEEP_CAPTURE,
            "local CA trust",
            format!("CA trust state could not be determined: {reason}"),
            "resolve the reported bundle or trust-store error before Deep Capture",
        ),
    });
    checks.push(if dc.analyzer_keylog_configured {
        Check::ok(
            DEEP_CAPTURE,
            "analyzer key log",
            "TLS key-log path is visible to this session",
        )
    } else {
        Check::warn(
            DEEP_CAPTURE,
            "analyzer key log",
            "TLS key-log path is not configured; analyzer decryption may need manual setup",
        )
    });
    checks.extend(native_residue_checks(&dc.native_residue));
    checks.push(match &dc.session_dir {
        Some(path) if dc.session_dir_present => Check::ok(
            DEEP_CAPTURE,
            "session storage",
            format!("{} (present)", path.display()),
        ),
        Some(path) => Check::ok(
            DEEP_CAPTURE,
            "session storage",
            format!("{} (absent)", path.display()),
        ),
        None => Check::fail(
            DEEP_CAPTURE,
            "session storage",
            "Deep Capture session storage path is undetermined",
            "configure a writable current-user data directory before Deep Capture",
        ),
    });
    if !matches!(dc.ipv4_loopback, crate::doctor::LoopbackReadiness::Ready)
        && !matches!(dc.ipv6_loopback, crate::doctor::LoopbackReadiness::Ready)
    {
        checks.push(Check::fail(
            DEEP_CAPTURE,
            "loopback availability",
            "neither exact IPv4 nor exact IPv6 loopback can host the native proxy",
            "restore at least one exact loopback family before Deep Capture",
        ));
    }
    checks
}

fn native_residue_checks(inventory: &super::residue::NativeResidueInventory) -> Vec<Check> {
    if !inventory.limitations.is_empty() {
        return vec![Check::fail(
            DEEP_CAPTURE,
            "native inventory",
            format!(
                "inventory is incomplete: {}",
                inventory
                    .limitations
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            "resolve the reported inventory limitations before Deep Capture",
        )];
    }
    if inventory.findings.is_empty() {
        return vec![Check::ok(
            DEEP_CAPTURE,
            "native inventory",
            "no native Deep Capture session residue found",
        )];
    }
    inventory
        .findings
        .iter()
        .map(|finding| {
            let name = format!(
                "native resource {}/{}",
                finding.session_id, finding.resource_id
            );
            let detail = format!(
                "session={} resource={} kind={} state={} health={} authority={}: {}",
                finding.session_id,
                finding.resource_id,
                finding.kind,
                finding.state,
                finding.health.as_str(),
                finding.ownership_authority,
                finding.detail
            );
            match finding.health {
                super::residue::ResidueHealth::Healthy | super::residue::ResidueHealth::Active => {
                    Check::ok(DEEP_CAPTURE, name, detail)
                }
                super::residue::ResidueHealth::Stale
                | super::residue::ResidueHealth::CleanupFailed
                | super::residue::ResidueHealth::Unknown
                    if finding.recoverable =>
                {
                    Check::fail_action(
                        DEEP_CAPTURE,
                        name,
                        detail,
                        "run `fragcap doctor --fix` to replay this exact recovery authority",
                        Action::new(ActionKind::CleanupDeepCapture),
                    )
                }
                _ => Check::fail(
                    DEEP_CAPTURE,
                    name,
                    detail,
                    "inspect the reported journal refusal before starting Deep Capture",
                ),
            }
        })
        .collect()
}

fn loopback_check(
    name: &'static str,
    address: &'static str,
    readiness: &crate::doctor::LoopbackReadiness,
) -> Check {
    match readiness {
        crate::doctor::LoopbackReadiness::Ready => Check::ok(
            DEEP_CAPTURE,
            name,
            format!("exact ephemeral bind to {address} succeeded"),
        ),
        crate::doctor::LoopbackReadiness::Unavailable(reason) => Check::warn(
            DEEP_CAPTURE,
            name,
            format!("exact ephemeral bind to {address} failed: {reason}"),
        ),
        crate::doctor::LoopbackReadiness::Undetermined => Check::warn(
            DEEP_CAPTURE,
            name,
            format!("exact loopback bind readiness for {address} is undetermined"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::{
        DeepCaptureInputs, IfaceInfo, LoopbackReadiness, NpcapInfo, ProxyBackendInfo, Status,
    };
    use crate::exit::Exit;

    fn ready_inputs() -> Inputs {
        Inputs {
            fragcap_version: "0.0.0-test".to_string(),
            binary_path: Some(std::path::PathBuf::from(
                "C:\\Program Files\\fragcap\\fragcap.exe",
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
            target_entry_count: Some(3),
            deep_capture: DeepCaptureInputs {
                session_dir: Some(std::path::PathBuf::from(
                    "C:\\Users\\gamer\\AppData\\Roaming\\fragcap\\sessions",
                )),
                session_dir_present: false,
                proxy_backend: Some(ProxyBackendInfo {
                    name: "fragcap-native".to_string(),
                    version: Some("0.8.0".to_string()),
                }),
                proxy_backend_error: None,
                ipv4_loopback: LoopbackReadiness::Ready,
                ipv6_loopback: LoopbackReadiness::Ready,
                analyzer_keylog_configured: true,
                ca: DeepCaptureCa::Absent,
                native_residue: Default::default(),
            },
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
        // The identity section is the four leading rows, all ok. The retired profile
        // directory row and the Profiles section were removed with the profile-file
        // surface (slice S057); the section is version, binary, catalog db, local db.
        let identity: Vec<_> = report
            .checks
            .iter()
            .filter(|c| c.section == IDENTITY)
            .collect();
        assert_eq!(identity.len(), 4, "version, binary, catalog db, local db");
        assert!(identity.iter().all(|c| c.status == Status::Ok));
        assert_eq!(report.checks[0].section, IDENTITY, "identity leads");
        assert!(
            report.checks[0].detail.contains("0.0.0-test"),
            "version row"
        );
        // The retired profile surface is gone: no identity row names a profile
        // directory, and no section is named Profiles.
        assert!(
            !report.checks.iter().any(|c| c.name == "profile dir"),
            "no profile dir row"
        );
        assert!(
            !report.checks.iter().any(|c| c.section == "Profiles"),
            "no Profiles section"
        );
        // Identity is informational: an unresolvable path is an ok note, and the
        // machine stays ready.
        let mut inputs = ready_inputs();
        inputs.binary_path = None;
        inputs.catalog_db_path = None;
        inputs.local_db_path = None;
        let report = run(&inputs);
        assert!(report.ready(), "unresolvable paths do not block");
        let binary = report.checks.iter().find(|c| c.name == "binary").unwrap();
        assert_eq!(binary.status, Status::Ok);
        assert_eq!(binary.detail, "undetermined");
    }

    #[test]
    fn deep_capture_ready_inputs_are_non_blocking_and_name_storage() {
        let checks = deep_capture(&ready_inputs());
        assert!(checks.iter().all(|check| check.status == Status::Ok));
        assert!(checks
            .iter()
            .any(|check| check.name == "session storage" && check.detail.contains("sessions")));
    }

    #[test]
    fn deep_capture_residue_warns_and_offers_cleanup() {
        let mut inputs = ready_inputs();
        inputs
            .deep_capture
            .native_residue
            .findings
            .push(crate::doctor::residue::ResourceFinding {
                session_id: "old".to_string(),
                bundle: std::path::PathBuf::from("C:\\fragcap\\sessions\\old"),
                resource_id: "proxy".to_string(),
                kind: "proxy".to_string(),
                state: "applied".to_string(),
                health: crate::doctor::residue::ResidueHealth::Stale,
                recoverable: true,
                ownership_authority: "resource-journal".to_string(),
                detail: "exact journal recovery action is available".to_string(),
            });
        let checks = deep_capture(&inputs);
        let check = checks
            .iter()
            .find(|check| check.name == "native resource old/proxy")
            .unwrap();
        assert_eq!(check.status, Status::Fail);
        assert_eq!(
            check.action.as_ref().map(|action| action.kind),
            Some(ActionKind::CleanupDeepCapture)
        );
    }

    #[test]
    fn native_resource_check_identity_does_not_depend_on_list_position() {
        let finding =
            |session_id: &str, resource_id: &str| crate::doctor::residue::ResourceFinding {
                session_id: session_id.to_string(),
                bundle: std::path::PathBuf::from(format!("C:\\sessions\\{session_id}")),
                resource_id: resource_id.to_string(),
                kind: "proxy".to_string(),
                state: "applied".to_string(),
                health: crate::doctor::residue::ResidueHealth::Stale,
                recoverable: true,
                ownership_authority: "resource-journal".to_string(),
                detail: "exact journal recovery action is available".to_string(),
            };
        let mut inventory = crate::doctor::residue::NativeResidueInventory::default();
        inventory.findings.push(finding("stable", "proxy"));
        let before = native_residue_checks(&inventory)
            .into_iter()
            .next()
            .expect("check")
            .name;
        inventory.findings.insert(0, finding("earlier", "route"));
        let after = native_residue_checks(&inventory)
            .into_iter()
            .find(|check| check.name.contains("stable/proxy"))
            .expect("stable check")
            .name;

        assert_eq!(before, "native resource stable/proxy");
        assert_eq!(after, before);
    }

    #[test]
    fn deep_capture_ca_wrong_store_blocks_without_unsafe_cleanup() {
        let mut inputs = ready_inputs();
        inputs.deep_capture.ca = DeepCaptureCa::WrongStore {
            store: "local-machine".to_string(),
            thumbprint: "sha256:example".to_string(),
        };
        let checks = deep_capture(&inputs);
        let ca = checks
            .iter()
            .find(|check| check.name == "local CA trust")
            .unwrap();
        assert_eq!(ca.status, Status::Fail);
        assert!(ca.detail.contains("local-machine"));
        assert!(ca.action.is_none());
    }

    #[test]
    fn unobserved_ca_mismatch_warns_without_cleanup() {
        let mut inputs = ready_inputs();
        inputs.deep_capture.ca = DeepCaptureCa::Mismatched {
            expected: "00112233445566778899AABBCCDDEEFF00112233".to_string(),
            actual: "112233445566778899AABBCCDDEEFF0011223344".to_string(),
            store: None,
        };
        let checks = deep_capture(&inputs);
        let ca = checks
            .iter()
            .find(|check| check.name == "local CA trust")
            .unwrap();
        assert_eq!(ca.status, Status::Warn);
        assert_eq!(ca.action, None);
        assert!(ca
            .detail
            .contains("112233445566778899AABBCCDDEEFF0011223344"));
    }
}
