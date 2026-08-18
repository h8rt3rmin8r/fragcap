// SPDX-License-Identifier: Apache-2.0

//! `extcap` end to end (specification section 14.5): the three declaration
//! invocations against the extcap control grammar, and a `--capture --fifo` run
//! over the offline substrate reproducing the committed pcapng golden. No capture
//! driver, no elevation, no analyzer.

mod common;

use std::fs;

use common::{data, fixture};

/// Register a single-client `game.exe` target in a fresh local store (slice S058),
/// returning the `TempDir` (the caller keeps it alive), the store path, and the
/// selector handle. The extcap capture resolves this through the shared stored-target
/// seam, the same resolution `capture --target` uses.
fn registered_target() -> (tempfile::TempDir, String, String) {
    let dir = tempfile::tempdir().unwrap();
    let local_db = dir.path().join("targets.db").to_string_lossy().into_owned();
    let (code, _out, err) = common::run(&[
        "targets",
        "add",
        "Extcap Game",
        "--exe",
        "game.exe",
        "--socket-holder",
        "yes",
        "--handle",
        "extcap_game",
        "--db",
        &local_db,
    ]);
    assert_eq!(code, 0, "register the extcap target: {err}");
    (dir, local_db, "extcap_game".to_string())
}

/// The offline substrate flags an extcap (or `capture`) invocation is driven with at
/// tier 1: the stored-target selector resolved from `local_db`, plus the recorded
/// source, scripted attributor, and scripted timeline `capture` uses.
fn offline_substrate(selector: &str, local_db: &str) -> Vec<String> {
    vec![
        "--target".into(),
        selector.into(),
        "--local-db".into(),
        local_db.into(),
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
    assert!(out.contains("{call=--target}"), "{out}");
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
    let (_store, local_db, selector) = registered_target();
    let dir = tempfile::tempdir().unwrap();
    let fifo = dir.path().join("extcap.fcapng");

    let mut args: Vec<String> = vec![
        "extcap".into(),
        "--capture".into(),
        "--fifo".into(),
        fifo.to_string_lossy().into_owned(),
    ];
    args.extend(offline_substrate(&selector, &local_db));
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
    let (_store, local_db, selector) = registered_target();
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
    args.extend(offline_substrate(&selector, &local_db));
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let (code, _out, err) = common::run(&refs);
    assert_eq!(code, 0, "the direct extcap capture succeeds: {err}");
    let bytes = fs::read(&fifo).expect("the fifo stream was written");
    common::assert_golden("run.fcapng", &bytes);
}

// --- US2: the dialog options select the capture ----------------------------

/// Capture to a temp pcapng through `command`, returning the written bytes.
fn capture_bytes(command: &str, scope: &[&str]) -> Vec<u8> {
    let (_store, local_db, selector) = registered_target();
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
    args.extend(offline_substrate(&selector, &local_db));
    args.extend(scope.iter().map(|s| s.to_string()));
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let (code, _o, err) = common::run(&refs);
    assert_eq!(code, 0, "{command} capture succeeds: {err}");
    fs::read(&out).expect("capture written")
}

#[test]
fn the_roles_option_is_applied_through_the_overlay() {
    // The roles option reaches the same overlay `capture` uses (it scopes which stages
    // trigger). A stored target synthesizes a single `target`-role stage, so scoping to
    // `target` captures the gameplay and reproduces the golden; the option is accepted
    // and applied, not ignored.
    let bytes = capture_bytes("extcap", &["--roles", "target"]);
    common::assert_golden("run.fcapng", &bytes);
}

#[test]
fn an_unresolvable_target_is_a_configuration_error_before_capture() {
    // A target selector that matches nothing in the local store is a configuration
    // error (exit 2), reported before any capture starts, exactly as `capture` treats
    // it: an extcap capture never becomes a started-but-empty stream on a selection
    // that resolves to no target (P-4, P-9).
    let dir = tempfile::tempdir().unwrap();
    let fifo = dir.path().join("extcap.fcapng");
    let empty_db = dir.path().join("empty.db").to_string_lossy().into_owned();

    let (code, _out, err) = common::run(&[
        "extcap",
        "--capture",
        "--fifo",
        &fifo.to_string_lossy(),
        "--target",
        "no_such_target",
        "--local-db",
        &empty_db,
    ]);
    assert_eq!(code, 2, "an unresolvable target exits 2: {err}");
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
    let (code, _out, err) = common::run(&["extcap", "--capture", "--target", "extcap_game"]);
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

// --- extcap install / uninstall registration (slice 041) -------------------

#[test]
fn extcap_install_registers_the_binary_and_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path().to_string_lossy().into_owned();

    let (code, out, err) = common::run(&["extcap", "install", "--dir", &d]);
    assert_eq!(code, 0, "{err}");
    assert!(
        out.contains("registered"),
        "reports the registration: {out}"
    );
    assert_eq!(
        fs::read_dir(dir.path()).unwrap().count(),
        1,
        "exactly the registered binary is present"
    );

    // A second install refreshes and still succeeds (idempotent, FR-002).
    let (code, _out, err) = common::run(&["extcap", "install", "--dir", &d]);
    assert_eq!(code, 0, "second install is idempotent: {err}");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn extcap_uninstall_removes_and_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path().to_string_lossy().into_owned();

    common::run(&["extcap", "install", "--dir", &d]);
    let (code, out, err) = common::run(&["extcap", "uninstall", "--dir", &d]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("removed"), "reports the removal: {out}");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);

    // Uninstall when nothing is registered is a success no-op (FR-003).
    let (code, out, err) = common::run(&["extcap", "uninstall", "--dir", &d]);
    assert_eq!(code, 0, "absent uninstall is a no-op success: {err}");
    assert!(
        out.contains("nothing to do") || out.contains("not registered"),
        "{out}"
    );
}

#[test]
fn a_failed_registration_reports_an_error_and_never_success() {
    // A --dir that is an existing file cannot be created as a directory, so the
    // registration fails loudly rather than claiming success (FR-008, SC-005).
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("not-a-dir");
    fs::write(&blocker, b"x").unwrap();
    let (code, out, _err) =
        common::run(&["extcap", "install", "--dir", &blocker.to_string_lossy()]);
    assert_ne!(code, 0, "a failed registration is not exit 0");
    assert!(
        !out.contains("registered"),
        "no success line on failure: {out}"
    );
}

#[test]
fn the_new_subcommands_do_not_regress_the_bare_protocol_forms() {
    // Adding install/uninstall must not break the four analyzer invocations in
    // the bare top-level form Wireshark uses (route_extcap injects `extcap`).
    let (code, out, _e) = common::run(&["--extcap-interfaces"]);
    assert_eq!(code, 0);
    assert!(out.contains("interface {value=fragcap}"), "{out}");

    let (code, out, _e) = common::run(&["--extcap-interface", "fragcap", "--extcap-dlts"]);
    assert_eq!(code, 0);
    assert!(out.contains("dlt {number=1}"), "{out}");

    let (code, out, _e) = common::run(&["--extcap-interface", "fragcap", "--extcap-config"]);
    assert_eq!(code, 0);
    assert!(out.contains("{call=--target}"), "{out}");

    let (_store, local_db, selector) = registered_target();
    let tmp = tempfile::tempdir().unwrap();
    let fifo = tmp.path().join("x.fcapng");
    let mut args: Vec<String> = vec![
        "--capture".into(),
        "--extcap-interface".into(),
        "fragcap".into(),
        "--fifo".into(),
        fifo.to_string_lossy().into_owned(),
    ];
    args.extend(offline_substrate(&selector, &local_db));
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let (code, _o, err) = common::run(&refs);
    assert_eq!(code, 0, "the bare capture form still runs: {err}");
}

#[test]
fn extcap_scope_flags_target_the_right_directory_and_doctor_agrees() {
    // End-to-end agreement across scopes: install writes where doctor probes, so
    // the doctor integration check flips and names the scope. doctor reads the
    // per-user extcap dir from FRAGCAP_EXTCAP_DIR and the machine-wide dir from
    // FRAGCAP_SYSTEM_EXTCAP_DIR; this is the only test in this file that sets those
    // variables, so setting them process-wide does not race another test, and it
    // keeps the probe off the real system Wireshark directory (hermetic).
    let user = tempfile::tempdir().unwrap();
    let system = tempfile::tempdir().unwrap();
    std::env::set_var("FRAGCAP_EXTCAP_DIR", user.path());
    std::env::set_var("FRAGCAP_SYSTEM_EXTCAP_DIR", system.path());

    // Default (no scope flag) registers per-user; doctor names the current user.
    common::run(&["extcap", "install"]);
    assert_eq!(
        fs::read_dir(user.path()).unwrap().count(),
        1,
        "the default scope is per-user"
    );
    assert_eq!(fs::read_dir(system.path()).unwrap().count(), 0);
    let (_c, out, _e) = common::run(&["doctor"]);
    assert!(
        out.contains("installed for the current user in"),
        "doctor names the current-user scope: {out}"
    );
    common::run(&["extcap", "uninstall"]);
    let (_c, out, _e) = common::run(&["doctor"]);
    assert!(
        out.contains("not registered as a Wireshark extcap source"),
        "doctor reports it not registered after uninstall: {out}"
    );

    // --system registers into the machine-wide directory; doctor names it.
    let (code, _o, err) = common::run(&["extcap", "install", "--system"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(
        fs::read_dir(system.path()).unwrap().count(),
        1,
        "--system registers machine-wide"
    );
    assert_eq!(fs::read_dir(user.path()).unwrap().count(), 0);
    let (_c, out, _e) = common::run(&["doctor"]);
    assert!(
        out.contains("installed machine-wide in"),
        "doctor names the machine-wide scope: {out}"
    );
    let (code, out, err) = common::run(&["extcap", "uninstall", "--system"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("removed"), "{out}");
    assert_eq!(fs::read_dir(system.path()).unwrap().count(), 0);

    // --user is the explicit form of the default per-user scope.
    common::run(&["extcap", "install", "--user"]);
    assert_eq!(
        fs::read_dir(user.path()).unwrap().count(),
        1,
        "--user registers per-user"
    );
    assert_eq!(fs::read_dir(system.path()).unwrap().count(), 0);
    common::run(&["extcap", "uninstall", "--user"]);

    std::env::remove_var("FRAGCAP_EXTCAP_DIR");
    std::env::remove_var("FRAGCAP_SYSTEM_EXTCAP_DIR");
}

#[test]
fn extcap_scope_selectors_are_mutually_exclusive() {
    // At most one of --user / --system / --dir; two selectors is a clap usage
    // error (exit 2), never an ambiguous registration. No env vars, so this runs
    // in parallel with every other test.
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path().to_string_lossy().into_owned();

    let (code, _o, err) = common::run(&["extcap", "install", "--user", "--system"]);
    assert_eq!(code, 2, "--user with --system is a usage error: {err}");

    let (code, _o, err) = common::run(&["extcap", "install", "--system", "--dir", &d]);
    assert_eq!(code, 2, "--system with --dir is a usage error: {err}");

    let (code, _o, err) = common::run(&["extcap", "uninstall", "--user", "--dir", &d]);
    assert_eq!(
        code, 2,
        "uninstall --user with --dir is a usage error: {err}"
    );
}
