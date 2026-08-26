// SPDX-License-Identifier: Apache-2.0

//! `doctor` over constructed environment states.
//!
//! The command is a pure classifier over injected `Inputs`, so the whole
//! section 26.3 matrix is exercised here with hand-built inputs and goldens, on
//! any target, without the environment it describes.

mod common;

use fragcap_cli::doctor::action::{offered_actions, ActionKind, Capabilities, ExtcapScope};
use fragcap_cli::doctor::{
    checks, DeepCaptureCa, DeepCaptureInputs, IfaceInfo, Inputs, NpcapInfo, Privilege,
    ProxyBackendInfo, Status, Subsystem,
};

fn ready() -> Inputs {
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
                name: "mitmdump".to_string(),
                version: Some("Mitmproxy: 12.1.0".to_string()),
            }),
            proxy_backend_error: None,
            analyzer_keylog_configured: true,
            ca: DeepCaptureCa::Absent,
            occupied_proxy_ports: Some(Vec::new()),
            orphaned_proxy_processes: Some(Vec::new()),
            stale_manifests: Vec::new(),
            stale_tls_key_logs: Vec::new(),
            sensitive_artifacts: Vec::new(),
        },
    }
}

#[test]
fn a_ready_machine_is_ok_ends_ready_and_exits_zero() {
    let report = checks::run(&ready());
    assert_eq!(report.exit().code(), 0);
    let human = report.render_human();
    assert!(human.contains("Ready to capture."), "{human}");
    common::assert_golden("doctor-ready.txt", human.as_bytes());
    common::assert_golden("doctor-ready.ndjson", report.render_json().as_bytes());
}

#[test]
fn absent_npcap_blocks_with_a_remediation_and_exits_one() {
    let mut inputs = ready();
    inputs.npcap = None;
    let report = checks::run(&inputs);
    assert_eq!(report.exit().code(), 1);
    let npcap = report.checks.iter().find(|c| c.name == "npcap").unwrap();
    assert_eq!(npcap.status, Status::Fail);
    assert!(
        npcap.remediation.as_ref().unwrap().contains("npcap.com"),
        "the remediation must say where to get npcap"
    );
}

#[test]
fn the_two_npcap_options_have_their_own_severities() {
    // Loopback is only needed with --loopback, so its absence warns and does not
    // block; the WinPcap option is unaffected and the machine stays ready.
    let mut loop_absent = ready();
    if let Some(info) = loop_absent.npcap.as_mut() {
        info.loopback_supported = Some(false);
    }
    let report = checks::run(&loop_absent);
    let loopback = report
        .checks
        .iter()
        .find(|c| c.name == "loopback adapter")
        .unwrap();
    let winpcap = report
        .checks
        .iter()
        .find(|c| c.name == "winpcap api mode")
        .unwrap();
    assert_eq!(loopback.status, Status::Warn);
    assert_eq!(
        winpcap.status,
        Status::Ok,
        "the WinPcap option is unaffected"
    );
    assert_eq!(
        report.exit().code(),
        0,
        "a missing loopback adapter does not block"
    );

    // The WinPcap API option, by contrast, blocks when absent.
    let mut api_absent = ready();
    if let Some(info) = api_absent.npcap.as_mut() {
        info.winpcap_api_mode = false;
    }
    let report = checks::run(&api_absent);
    let loopback = report
        .checks
        .iter()
        .find(|c| c.name == "loopback adapter")
        .unwrap();
    let winpcap = report
        .checks
        .iter()
        .find(|c| c.name == "winpcap api mode")
        .unwrap();
    assert_eq!(winpcap.status, Status::Fail);
    assert_eq!(
        loopback.status,
        Status::Ok,
        "the loopback option is unaffected"
    );
    assert_eq!(report.exit().code(), 1);
}

#[test]
fn an_absent_live_backend_blocks_and_an_absent_socket_table_only_warns() {
    let mut no_live = ready();
    no_live.live_available = None;
    let report = checks::run(&no_live);
    let live = report
        .checks
        .iter()
        .find(|c| c.name == "live backend")
        .unwrap();
    assert_eq!(live.status, Status::Fail);
    assert!(
        live.remediation.is_some(),
        "the live backend fail names a fix"
    );
    assert_eq!(report.exit().code(), 1, "no live backend blocks readiness");

    let mut no_attr = ready();
    no_attr.socket_table_available = None;
    let report = checks::run(&no_attr);
    let attr = report
        .checks
        .iter()
        .find(|c| c.name == "socket-table backend")
        .unwrap();
    assert_eq!(attr.status, Status::Warn);
    assert_eq!(
        report.exit().code(),
        0,
        "degraded attribution does not block"
    );
}

#[test]
fn not_elevated_and_no_interfaces_warn_without_blocking() {
    let mut inputs = ready();
    inputs.privilege = Privilege::NotElevated;
    inputs.interfaces.clear();
    let report = checks::run(&inputs);
    assert_eq!(report.exit().code(), 0, "warnings do not block");
    assert!(report.checks.iter().any(|c| c.status == Status::Warn));
}

#[test]
fn tracing_unavailable_while_elevated_blocks() {
    let mut inputs = ready();
    inputs.etw_available = Some(false);
    inputs.privilege = Privilege::Elevated;
    let report = checks::run(&inputs);
    assert_eq!(report.exit().code(), 1);
    let tracing = report
        .checks
        .iter()
        .find(|c| c.name == "process events")
        .unwrap();
    assert_eq!(tracing.status, Status::Fail);
    assert!(tracing.remediation.is_some());
}

#[test]
fn every_failing_check_names_a_remediation() {
    let mut inputs = ready();
    inputs.npcap = Some(NpcapInfo {
        version: "1.79".to_string(),
        loopback_supported: Some(false),
        winpcap_api_mode: false,
    });
    inputs.etw_available = Some(false);
    let report = checks::run(&inputs);
    let fails: Vec<_> = report
        .checks
        .iter()
        .filter(|c| c.status == Status::Fail)
        .collect();
    assert!(fails.len() >= 2, "several checks should fail here");
    for check in fails {
        assert!(
            check.remediation.as_ref().is_some_and(|r| !r.is_empty()),
            "{} has no remediation",
            check.name
        );
    }
}

#[test]
fn the_json_form_is_one_record_per_check() {
    let report = checks::run(&ready());
    let json = report.render_json();
    let lines: Vec<&str> = json.lines().collect();
    assert_eq!(lines.len(), report.checks.len(), "one record per check");
    for line in lines {
        assert!(line.starts_with("{\"section\":"), "record shape: {line}");
        assert!(line.contains("\"status\":"));
    }
}

#[test]
fn the_plain_human_report_has_no_color_and_fits_eighty_columns() {
    let report = checks::run(&ready());
    let human = report.render_human();
    assert!(
        !human.contains('\u{1b}'),
        "render_human must be plain: no ANSI escapes"
    );
    for line in human.lines() {
        assert!(
            line.chars().count() <= 80,
            "line exceeds 80 columns ({}): {line:?}",
            line.chars().count()
        );
    }
}

#[test]
fn the_colored_form_wraps_the_status_words_and_leaves_json_plain() {
    let report = checks::run(&ready());
    // The colored human form carries ANSI; the plain form does not; the JSON
    // form is never colorized.
    assert!(report.render_human_with(true).contains('\u{1b}'));
    assert!(!report.render_human_with(false).contains('\u{1b}'));
    assert!(!report.render_json().contains('\u{1b}'));
}

#[test]
fn the_identity_section_appears_first_in_both_forms() {
    let report = checks::run(&ready());
    let human = report.render_human();
    assert!(human.starts_with("Identity\n"), "identity leads:\n{human}");
    for want in ["version", "binary", "catalog db", "local db"] {
        assert!(
            human.contains(want),
            "identity row {want} missing:\n{human}"
        );
    }
    // The retired profile directory row and the Profiles section were removed with
    // the profile-file surface (slice S057).
    assert!(
        !human.contains("profile dir"),
        "no profile dir row:\n{human}"
    );
    assert!(
        !human.contains("\nProfiles\n"),
        "no Profiles section:\n{human}"
    );
    let json = report.render_json();
    assert_eq!(json.lines().count(), report.checks.len());
    assert!(json
        .lines()
        .next()
        .unwrap()
        .contains("\"section\":\"Identity\""));
}

#[test]
fn the_command_runs_end_to_end_and_returns_a_valid_exit() {
    // The real probe is machine-dependent, so only the shape and exit class are
    // asserted here; classification is covered above over injected inputs.
    let (code, out, _err) = common::run(&["doctor"]);
    assert!(code == 0 || code == 1, "doctor exits 0 or 1, got {code}");
    assert!(!out.is_empty(), "the report is written to stdout");

    let (code, out, _err) = common::run(&["doctor", "--json"]);
    assert!(code == 0 || code == 1);
    assert!(
        out.contains("\"section\":"),
        "json records on stdout: {out}"
    );
}

// --- The --fix action layer (slice S056) ---

#[test]
fn fix_is_refused_with_json() {
    // --fix is interactive; combined with --json it is a usage error (exit 2) and
    // performs no action (FR-007, SC-004).
    let (code, out, _err) = common::run(&["doctor", "--fix", "--json"]);
    assert_eq!(code, 2, "--fix --json is a usage error");
    assert!(
        !out.contains("Proposed actions"),
        "no action phase runs: {out}"
    );
}

#[test]
fn fix_is_refused_without_an_interactive_terminal() {
    // In the test process stdout is not a terminal, so --fix is refused (exit 2),
    // even with --yes, and no action runs (FR-008, SC-004).
    let (code, out, _err) = common::run(&["doctor", "--fix"]);
    assert_eq!(code, 2, "--fix needs a terminal");
    assert!(!out.contains("Proposed actions"), "no action phase: {out}");

    let (code, _out, _err) = common::run(&["doctor", "--fix", "--yes"]);
    assert_eq!(code, 2, "--fix --yes still needs a terminal stdout");
}

#[test]
fn yes_without_fix_is_a_usage_error() {
    let (code, _out, _err) = common::run(&["doctor", "--yes"]);
    assert_eq!(code, 2, "--yes has no effect without --fix");
}

#[test]
fn a_blocked_machine_surfaces_actions_bound_to_its_findings() {
    // Each finding the report names carries the action --fix would offer, and the
    // selection is a subset of those (FR-003), with elevation first (FR-014) and
    // the network actions degraded when the capability is absent (FR-012, FR-016).
    let mut inputs = ready();
    inputs.npcap = None; // -> ObtainNpcap
    inputs.privilege = Privilege::NotElevated; // -> RelaunchElevated
    inputs.extcap_installed = false;
    inputs.extcap_system_installed = false; // -> InstallExtcap(User)
    inputs.catalog_db_present = false; // -> FetchCatalog
    inputs.target_entry_count = Some(0); // -> RunDiscovery
    let report = checks::run(&inputs);

    let net = offered_actions(
        &report,
        Capabilities {
            net: true,
            elevation: true,
        },
    );
    let kinds: Vec<ActionKind> = net.iter().map(|a| a.kind).collect();
    assert_eq!(
        kinds[0],
        ActionKind::RelaunchElevated,
        "elevation offered first"
    );
    assert!(kinds.contains(&ActionKind::ObtainNpcap));
    assert!(kinds.contains(&ActionKind::InstallExtcap(ExtcapScope::User)));
    assert!(kinds.contains(&ActionKind::InitializeCatalog));
    assert!(kinds.contains(&ActionKind::RunDiscovery));
    assert!(
        net.iter().all(|a| !a.degraded),
        "net-capable: no degradation"
    );

    let off = offered_actions(
        &report,
        Capabilities {
            net: false,
            elevation: true,
        },
    );
    let npcap = off
        .iter()
        .find(|a| a.kind == ActionKind::ObtainNpcap)
        .unwrap();
    assert!(npcap.degraded, "npcap fetch degrades without net");
    // The catalog action does not degrade at all since slice S063. It creates
    // the store and loads the compiled-in detection signatures, which needs no
    // network, so there is nothing to degrade from. It was net-gated until then,
    // which meant that in every released build it degraded to guidance telling
    // the user to rebuild fragcap from source (issue #175).
    let catalog = off
        .iter()
        .find(|a| a.kind == ActionKind::InitializeCatalog)
        .unwrap();
    assert!(!catalog.degraded, "the catalog action is offline");
    assert!(
        !catalog.guidance_only(),
        "an offline action is performable, not guidance"
    );
    // A non-network action is unaffected by the capability.
    let discovery = off
        .iter()
        .find(|a| a.kind == ActionKind::RunDiscovery)
        .unwrap();
    assert!(!discovery.degraded);
}

#[test]
fn a_ready_machine_offers_no_actions() {
    // The ready fixture has a catalog and target entries, so the preparation checks
    // stay silent and there is nothing to fix.
    let report = checks::run(&ready());
    assert!(offered_actions(
        &report,
        Capabilities {
            net: true,
            elevation: true
        }
    )
    .is_empty());
}

#[test]
fn deep_capture_residue_is_machine_readable_and_offers_cleanup() {
    let mut inputs = ready();
    inputs.deep_capture.stale_tls_key_logs = vec![std::path::PathBuf::from(
        "C:\\Users\\gamer\\AppData\\Roaming\\fragcap\\sessions\\s1\\tls-keylog.log",
    )];
    let report = checks::run(&inputs);
    let keylog = report
        .checks
        .iter()
        .find(|check| check.section == "Deep Capture" && check.name == "tls key logs")
        .unwrap();
    assert_eq!(keylog.status, Status::Warn);
    assert_eq!(
        keylog.action.as_ref().map(|action| action.kind),
        Some(ActionKind::CleanupDeepCapture)
    );
    let json = report.render_json();
    assert!(
        json.lines().any(|line| {
            line.contains("\"section\":\"Deep Capture\"")
                && line.contains("\"name\":\"tls key logs\"")
                && line.contains("\"status\":\"warn\"")
        }),
        "Deep Capture residue is present in machine output: {json}"
    );
}
