// SPDX-License-Identifier: Apache-2.0

//! `extcap` end to end (specification section 14.5): the three declaration
//! invocations against the extcap control grammar, and a `--capture --fifo` run
//! over the offline substrate reproducing the committed pcapng golden. No capture
//! driver, no elevation, no analyzer.

mod common;

use std::fs;

use common::{data, fixture};

/// The offline substrate flags an extcap capture is driven with at tier 1, the
/// same recorded source, scripted attributor, and scripted timeline `run` uses.
fn offline_substrate() -> Vec<String> {
    vec![
        "--profile".into(),
        data("game.toml"),
        "--replay-source".into(),
        fixture("udp-gameplay.pcap"),
        "--attr-script".into(),
        fixture("udp-gameplay.script"),
        "--process-script".into(),
        data("game.procscript"),
        "--local-addr".into(),
        "192.0.2.10".into(),
    ]
}

/// Whether a declaration line conforms to the extcap control grammar: a keyword
/// followed by one or more `{key=value}` groups (values carry no `}`).
fn line_conforms(line: &str) -> bool {
    let Some((keyword, rest)) = line.split_once(' ') else {
        return false;
    };
    if keyword.is_empty()
        || !keyword
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return false;
    }
    let mut rest = rest.trim();
    if rest.is_empty() {
        return false;
    }
    while !rest.is_empty() {
        if !rest.starts_with('{') {
            return false;
        }
        let Some(end) = rest.find('}') else {
            return false;
        };
        let group = &rest[1..end];
        let Some((key, _value)) = group.split_once('=') else {
            return false;
        };
        if key.is_empty() || key.contains('{') {
            return false;
        }
        rest = &rest[end + 1..];
    }
    true
}

fn assert_all_lines_conform(stream: &str) {
    for line in stream.lines().filter(|l| !l.trim().is_empty()) {
        assert!(
            line_conforms(line),
            "line does not conform to extcap grammar: {line}"
        );
    }
}

// --- US1: the discovery contract -------------------------------------------

#[test]
fn extcap_interfaces_declares_the_fragcap_source() {
    let (code, out, err) = common::run(&["extcap", "--extcap-interfaces"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("interface {value=fragcap}"), "{out}");
    assert!(
        out.lines().any(|l| l.starts_with("extcap {version=")),
        "{out}"
    );
    assert_all_lines_conform(&out);
}

#[test]
fn extcap_dlts_declares_ethernet() {
    let (code, out, err) =
        common::run(&["extcap", "--extcap-dlts", "--extcap-interface", "fragcap"]);
    assert_eq!(code, 0, "{err}");
    assert!(
        out.contains("dlt {number=1}{name=EN10MB}{display=Ethernet}"),
        "{out}"
    );
    assert_all_lines_conform(&out);
}

#[test]
fn extcap_config_declares_exactly_the_four_options() {
    let (code, out, err) =
        common::run(&["extcap", "--extcap-config", "--extcap-interface", "fragcap"]);
    assert_eq!(code, 0, "{err}");
    // Exactly four arg lines, with the four run flag names as their calls.
    let arg_lines: Vec<&str> = out.lines().filter(|l| l.starts_with("arg ")).collect();
    assert_eq!(arg_lines.len(), 4, "four options: {out}");
    assert!(out.contains("{call=--profile}"), "{out}");
    assert!(out.contains("{call=--roles}"), "{out}");
    assert!(out.contains("{call=--direction}"), "{out}");
    assert!(out.contains("{call=--loopback}"), "{out}");
    // The direction selector declares both (default), in, out.
    assert!(
        out.contains("value {arg=2}{value=both}{display=Both}{default=true}"),
        "{out}"
    );
    assert!(
        out.contains("{value=in}") && out.contains("{value=out}"),
        "{out}"
    );
    assert_all_lines_conform(&out);
}

// --- US1: the FIFO stream reproduces the golden ----------------------------

#[test]
fn an_extcap_capture_reproduces_the_run_golden_and_conserves() {
    let dir = tempfile::tempdir().unwrap();
    let fifo = dir.path().join("extcap.fcapng");

    let mut args: Vec<String> = vec![
        "extcap".into(),
        "--capture".into(),
        "--fifo".into(),
        fifo.to_string_lossy().into_owned(),
    ];
    args.extend(offline_substrate());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let (code, _out, err) = common::run(&refs);
    assert_eq!(code, 0, "the extcap capture succeeds: {err}");

    // FR-005 / SC-002: the FIFO stream is byte-identical to a plain `run --out`
    // capture of the same input, so it reproduces the committed pcapng golden.
    let bytes = fs::read(&fifo).expect("the fifo stream was written");
    common::assert_golden("run.fcapng", &bytes);

    // FR-011 / SC-006: the completion summary carries the conservation identity.
    assert!(
        err.contains("capture complete"),
        "the summary is printed: {err}"
    );
    let captured = summary_count(&err, "packets captured");
    let attributed = summary_count(&err, "attributed");
    let unattributed = summary_count(&err, "unattributed");
    assert_eq!(captured, 24, "{err}");
    assert_eq!(
        attributed + unattributed,
        captured,
        "every captured packet is in one attribution state: {err}"
    );
    assert_eq!(summary_count(&err, "buffer dropped"), 0, "{err}");
    assert_eq!(summary_count(&err, "sink dropped"), 0, "{err}");
}

/// Read a `label   N` count out of the human completion summary.
fn summary_count(summary: &str, label: &str) -> u64 {
    for line in summary.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix(label) {
            if let Some(value) = rest.split_whitespace().next() {
                if let Ok(n) = value.parse::<u64>() {
                    return n;
                }
            }
        }
    }
    panic!("summary has no `{label}` count: {summary}");
}

// --- US1: Wireshark invokes the binary directly, with no subcommand ---------

#[test]
fn a_direct_extcap_interfaces_invocation_is_routed() {
    // Wireshark runs `<binary> --extcap-interfaces`, not `<binary> extcap ...`,
    // so the protocol flags must be accepted at the top level (Codex review of
    // PR #34). The subcommand-form tests above would hide a regression here.
    let (code, out, err) = common::run(&["--extcap-interfaces"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("interface {value=fragcap}"), "{out}");
    assert_all_lines_conform(&out);
}

#[test]
fn a_direct_extcap_dlts_invocation_is_routed() {
    let (code, out, err) = common::run(&["--extcap-interface", "fragcap", "--extcap-dlts"]);
    assert_eq!(code, 0, "{err}");
    assert!(
        out.contains("dlt {number=1}{name=EN10MB}{display=Ethernet}"),
        "{out}"
    );
}

#[test]
fn a_direct_extcap_capture_without_a_subcommand_reproduces_the_golden() {
    let dir = tempfile::tempdir().unwrap();
    let fifo = dir.path().join("extcap.fcapng");
    // The Wireshark capture form: no `extcap` subcommand, protocol flags first.
    let mut args: Vec<String> = vec![
        "--capture".into(),
        "--extcap-interface".into(),
        "fragcap".into(),
        "--fifo".into(),
        fifo.to_string_lossy().into_owned(),
    ];
    args.extend(offline_substrate());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let (code, _out, err) = common::run(&refs);
    assert_eq!(code, 0, "the direct extcap capture succeeds: {err}");
    let bytes = fs::read(&fifo).expect("the fifo stream was written");
    common::assert_golden("run.fcapng", &bytes);
}

// --- US2: the dialog options select the capture ----------------------------

/// Capture to a temp pcapng through `command`, returning the written bytes.
fn capture_bytes(command: &str, scope: &[&str]) -> Vec<u8> {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.fcapng");
    let mut args: Vec<String> = vec![command.into()];
    if command == "extcap" {
        args.push("--capture".into());
        args.push("--fifo".into());
        args.push(out.to_string_lossy().into_owned());
    } else {
        args.push("--out".into());
        args.push(out.to_string_lossy().into_owned());
    }
    args.extend(offline_substrate());
    args.extend(scope.iter().map(|s| s.to_string()));
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let (code, _o, err) = common::run(&refs);
    assert_eq!(code, 0, "{command} capture succeeds: {err}");
    fs::read(&out).expect("capture written")
}

#[test]
fn extcap_options_scope_the_capture_like_the_run_flags() {
    // The same scoping flags select the same capture through the extcap dialog as
    // through the run command line (FR-006, SC-003), whatever each flag does there.
    let scope = ["--roles", "client", "--direction", "in", "--loopback"];
    let via_run = capture_bytes("run", &scope);
    let via_extcap = capture_bytes("extcap", &scope);
    assert_eq!(
        via_run, via_extcap,
        "extcap parity with run for the scoped capture"
    );
}

#[test]
fn the_roles_option_is_applied_through_the_overlay() {
    // The roles option reaches the same overlay `run` uses (it scopes which stages
    // trigger). Scoping to the profiled `client` role captures the gameplay and
    // reproduces the golden; the option is accepted and applied, not ignored.
    let bytes = capture_bytes("extcap", &["--roles", "client"]);
    common::assert_golden("run.fcapng", &bytes);
}

#[test]
fn a_malformed_profile_is_a_configuration_error_before_capture() {
    // A profile that fails validation is a configuration error (exit 2), reported
    // before any capture starts, exactly as `run` treats it: an extcap capture
    // never becomes a started-but-empty stream on a bad profile. (A profile that
    // is simply not found is exit 1 by the resolver's own mapping, the same as
    // `run`; validation failure is the exit-2 configuration error tested here.)
    let dir = tempfile::tempdir().unwrap();
    let fifo = dir.path().join("extcap.fcapng");
    let bad = dir.path().join("bad.toml");
    // schema declared but no [game] and no stage: the profile validator reports
    // diagnostics rather than parsing a capture target.
    fs::write(&bad, "schema = 1\n").unwrap();

    let (code, _out, err) = common::run(&[
        "extcap",
        "--capture",
        "--fifo",
        &fifo.to_string_lossy(),
        "--profile",
        &bad.to_string_lossy(),
    ]);
    assert_eq!(code, 2, "a malformed profile exits 2: {err}");
    assert!(
        !fifo.exists() || fs::read(&fifo).unwrap().is_empty(),
        "no capture was written"
    );
}

// --- Error contract (SC-005) -----------------------------------------------

#[test]
fn extcap_with_no_mode_flag_is_a_usage_error() {
    let (code, _out, err) = common::run(&["extcap"]);
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("--extcap-interfaces") || err.contains("--capture"),
        "{err}"
    );
}

#[test]
fn capture_without_a_fifo_is_a_usage_error() {
    let (code, _out, err) = common::run(&["extcap", "--capture", "--profile", &data("game.toml")]);
    assert_eq!(code, 2, "{err}");
    assert!(err.to_lowercase().contains("fifo"), "{err}");
}

#[test]
fn a_declaration_without_a_selected_interface_is_a_usage_error() {
    let (code, _out, err) = common::run(&["extcap", "--extcap-dlts"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("--extcap-interface"), "{err}");
}

#[test]
fn an_unknown_extcap_interface_is_a_usage_error() {
    let (code, _out, err) = common::run(&[
        "extcap",
        "--extcap-dlts",
        "--extcap-interface",
        "not-fragcap",
    ]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("not a fragcap extcap interface"), "{err}");
}
