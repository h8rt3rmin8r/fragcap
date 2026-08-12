// SPDX-License-Identifier: Apache-2.0

//! `doctor` over constructed environment states.
//!
//! The command is a pure classifier over injected `Inputs`, so the whole
//! section 26.3 matrix is exercised here with hand-built inputs and goldens, on
//! any target, without the environment it describes.

mod common;

use fragcap_cli::doctor::{checks, IfaceInfo, Inputs, NpcapInfo, Privilege, Status, Subsystem};

fn ready() -> Inputs {
    Inputs {
        os: "Windows 11".to_string(),
        subsystem: Subsystem::Native,
        privilege: Privilege::Elevated,
        npcap: Some(NpcapInfo {
            version: "1.79".to_string(),
            loopback_adapter: true,
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
        extcap_installed: true,
        extcap_dir: Some(std::path::PathBuf::from(
            "C:\\Users\\gamer\\AppData\\Roaming\\Wireshark\\extcap",
        )),
        bundled_count: 0,
        user_count: 2,
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
        info.loopback_adapter = false;
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
        loopback_adapter: false,
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
