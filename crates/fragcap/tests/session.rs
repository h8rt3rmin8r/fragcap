// SPDX-License-Identifier: Apache-2.0

//! Tier-1 tests for the capture session lifecycle, specification sections 10.4
//! through 10.6. Driven by scripted process events and packets, with no capture
//! driver, no elevation, and no game.

use std::time::Duration;

use fragcap::{
    CaptureSession, PacketDisposition, ProcessEvent, ProcessRecord, Profile, SessionConfig,
    SessionState, StopReason, Timestamp,
};

/// A single-stage identity profile, the shape `watch` and `tap` synthesize.
fn identity(match_body: &str) -> Profile {
    profile(&format!(
        r#"{{"role":"target","lifecycle":"session","terminal":true,"match":{match_body}}}"#
    ))
}

fn at(n: i64) -> Timestamp {
    Timestamp::from_nanos(n)
}

fn profile(stages: &str) -> Profile {
    let text = format!(
        r#"{{"schema":1,"kind":"profile","fidelity":"verified","game":{{"id":"t","name":"T"}},"stage":[{stages}]}}"#
    );
    Profile::parse(&text).unwrap_or_else(|d| {
        panic!(
            "test profile did not validate: {:?}",
            d.iter().map(|x| x.message.clone()).collect::<Vec<_>>()
        )
    })
}

/// Launcher (transient) then client (session, terminal, descended from launcher).
fn terminal_chain() -> Profile {
    profile(
        r#"{"role":"launcher","lifecycle":"transient","match":{"exe":"launcher.exe"}},{"role":"client","lifecycle":"session","terminal":true,"match":{"exe":"game.exe","descends_from":"launcher"}}"#,
    )
}

/// Launcher (transient) then client (session, not terminal).
fn nonterminal_chain() -> Profile {
    profile(
        r#"{"role":"launcher","lifecycle":"transient","match":{"exe":"launcher.exe"}},{"role":"client","lifecycle":"session","match":{"exe":"game.exe","descends_from":"launcher"}}"#,
    )
}

fn start(pid: u32, parent: u32, image: &str, when: i64) -> ProcessEvent {
    ProcessEvent::started(pid, parent, image, "cmd", at(when))
}

fn exit(pid: u32, when: i64) -> ProcessEvent {
    ProcessEvent::Exited { pid, at: at(when) }
}

#[test]
fn a_session_arms_before_any_target() {
    let mut s = CaptureSession::new(terminal_chain(), SessionConfig::default());
    assert_eq!(s.state(), SessionState::Arming);
    s.attach(at(0));
    assert_eq!(
        s.state(),
        SessionState::Watching,
        "the handle is open and the watcher attached before any process exists"
    );
}

#[test]
fn a_startup_snapshot_acquires_an_already_running_target() {
    // Attach-to-running (section 15.7): a process already present at arm, in the
    // startup snapshot with no later start event, acquires the target.
    let mut s = CaptureSession::new(identity(r#"{"exe":"eso64.exe"}"#), SessionConfig::default());
    s.attach(at(0));
    assert_eq!(s.state(), SessionState::Watching);

    s.apply_snapshot(
        &[ProcessRecord::new(1234, 0, "C:\\Games\\ESO\\eso64.exe")],
        at(0),
    );
    assert_eq!(
        s.state(),
        SessionState::Capturing,
        "a snapshot process matching the identity acquires the target at arm"
    );
}

#[test]
fn a_startup_snapshot_with_no_match_keeps_watching() {
    let mut s = CaptureSession::new(identity(r#"{"exe":"eso64.exe"}"#), SessionConfig::default());
    s.attach(at(0));
    s.apply_snapshot(
        &[ProcessRecord::new(1, 0, "C:\\Windows\\explorer.exe")],
        at(0),
    );
    assert_eq!(
        s.state(),
        SessionState::Watching,
        "no snapshot process matches, so the session keeps waiting"
    );
}

#[test]
fn a_filename_only_snapshot_cannot_be_disambiguated_by_a_path_anchor() {
    // Honesty (P-9): the Windows toolhelp startup snapshot carries only the
    // executable file name, never the full path (opening a process to read its
    // path is the handle the no-handle P-1 choice precludes). A path anchor is
    // matched against the full path, so it cannot match a snapshot node. An
    // already-running target that needs a path anchor is therefore not attached
    // from the snapshot; it is caught when it next produces a start event, whose
    // image the platform does supply. Modelled with file-name-only records, as
    // toolhelp gives them.
    let mut s = CaptureSession::new(
        identity(r#"{"exe":"SkyrimSE.exe","path_contains":"Mod Organizer 2"}"#),
        SessionConfig::default(),
    );
    s.attach(at(0));
    s.apply_snapshot(
        &[
            ProcessRecord::new(10, 0, "SkyrimSE.exe"),
            ProcessRecord::new(20, 0, "SkyrimSE.exe"),
        ],
        at(0),
    );
    assert_eq!(
        s.state(),
        SessionState::Watching,
        "a path anchor cannot match a file-name-only snapshot, so no attach happens"
    );
}

#[test]
fn an_out_of_order_snapshot_resolves_descends_from() {
    // Review of PR #84: a startup snapshot has no creation-order guarantee, so a
    // child can be listed before its parent. Folding parent-first means a
    // descends_from stage still binds when both are already running. The client
    // (descended from the launcher) is listed first; the launcher second.
    let mut s = CaptureSession::new(terminal_chain(), SessionConfig::default());
    s.attach(at(0));
    s.apply_snapshot(
        &[
            // Child before parent: game.exe (pid 200, parent 100) precedes
            // launcher.exe (pid 100). File-name-only, as toolhelp gives.
            ProcessRecord::new(200, 100, "game.exe"),
            ProcessRecord::new(100, 0, "launcher.exe"),
        ],
        at(0),
    );
    assert_eq!(
        s.state(),
        SessionState::Capturing,
        "the descended client acquires the target even though it preceded its parent \
         in the snapshot"
    );
}

#[test]
fn packets_before_a_match_are_discarded_and_counted() {
    let mut s = CaptureSession::new(terminal_chain(), SessionConfig::default());
    s.attach(at(0));
    assert_eq!(s.on_packet(100), PacketDisposition::Discarded);
    assert_eq!(s.on_packet(200), PacketDisposition::Discarded);
    assert_eq!(
        s.stats().watching_discarded,
        2,
        "P-4: the discard is counted"
    );
    assert_eq!(s.stats().retained, 0);
}

#[test]
fn the_first_match_begins_capturing_with_no_traffic_lost_at_the_boundary() {
    let mut s = CaptureSession::new(terminal_chain(), SessionConfig::default());
    s.attach(at(0));

    // Before any match: discarded.
    assert_eq!(s.on_packet(100), PacketDisposition::Discarded);

    // The launcher matches; the first match moves Watching to Capturing.
    s.on_process_event(start(100, 0, "C:\\L\\launcher.exe", 1));
    assert_eq!(s.state(), SessionState::Capturing);

    // From here packets are retained. The handle was already open, so the packet
    // that coincides with the transition is not lost.
    assert_eq!(s.on_packet(100), PacketDisposition::Retained);
    assert_eq!(s.on_packet(100), PacketDisposition::Retained);
    assert_eq!(s.stats().watching_discarded, 1);
    assert_eq!(s.stats().retained, 2);
    assert_eq!(s.stats().retained_bytes, 200);
}

#[test]
fn every_packet_offered_while_armed_is_accounted_for() {
    // Session conservation: observed equals retained plus watching-discards, and
    // every packet is one or the other. Nothing offered while armed is lost.
    let mut s = CaptureSession::new(terminal_chain(), SessionConfig::default());
    s.attach(at(0));
    let mut offered = 0u64;

    for _ in 0..5 {
        s.on_packet(64);
        offered += 1;
    }
    s.on_process_event(start(100, 0, "C:\\L\\launcher.exe", 1)); // -> Capturing
    for _ in 0..7 {
        s.on_packet(64);
        offered += 1;
    }

    assert_eq!(s.stats().observed(), offered);
    assert_eq!(
        s.stats().watching_discarded + s.stats().retained,
        offered,
        "conservation: every packet is retained or a counted discard"
    );
    assert_eq!(s.stats().watching_discarded, 5);
    assert_eq!(s.stats().retained, 7);
}

#[test]
fn no_target_by_the_acquisition_timeout_completes_without_capturing() {
    let cfg = SessionConfig {
        acquisition_timeout: Some(Duration::from_secs(30)),
        ..SessionConfig::default()
    };
    let mut s = CaptureSession::new(terminal_chain(), cfg);
    s.attach(at(0));
    s.on_packet(100); // discarded while watching
    s.on_tick(at(30_000_000_000)); // 30s later, still no match

    assert_eq!(s.state(), SessionState::Complete);
    assert_eq!(s.stop_reason(), Some(StopReason::AcquisitionTimeout));
    assert_eq!(s.stats().retained, 0);
}

#[test]
fn acquisition_timeout_remains_active_until_the_terminal_stage_binds() {
    let cfg = SessionConfig {
        acquisition_timeout: Some(Duration::from_secs(30)),
        exact_stage_ownership: true,
        ..SessionConfig::default()
    };
    let mut s = CaptureSession::new(terminal_chain(), cfg);
    s.attach(at(0));
    s.on_process_event(start(100, 0, "C:\\L\\launcher.exe", 1));
    assert_eq!(s.state(), SessionState::Capturing);

    s.on_tick(at(30_000_000_000));

    assert_eq!(s.state(), SessionState::Draining);
    assert_eq!(s.stop_reason(), Some(StopReason::AcquisitionTimeout));
    s.on_tick(at(31_000_000_000));
    assert_eq!(
        s.stop_reason(),
        Some(StopReason::AcquisitionTimeout),
        "later ticks cannot overwrite the terminal launch outcome"
    );
}

#[test]
fn a_second_match_for_one_stage_is_explicitly_ambiguous() {
    let mut s = CaptureSession::new(
        identity(r#"{"exe":"game.exe"}"#),
        SessionConfig {
            exact_stage_ownership: true,
            ..SessionConfig::default()
        },
    );
    s.attach(at(0));
    s.on_process_event(start(100, 0, "C:\\A\\game.exe", 1));
    s.on_process_event(start(200, 0, "C:\\B\\game.exe", 2));

    assert_eq!(s.state(), SessionState::Draining);
    assert_eq!(s.stop_reason(), Some(StopReason::AmbiguousStageMatch));
    assert_eq!(
        s.role_bindings().len(),
        1,
        "the competing process is observed but never promoted to stage ownership"
    );
}

#[test]
fn ordinary_profiles_keep_multi_process_roles_and_watching_only_wait_semantics() {
    let cfg = SessionConfig {
        acquisition_timeout: Some(Duration::from_secs(30)),
        ..SessionConfig::default()
    };
    let mut s = CaptureSession::new(terminal_chain(), cfg);
    s.attach(at(0));
    s.on_process_event(start(100, 0, "C:\\L\\launcher.exe", 1));
    s.on_process_event(start(101, 0, "C:\\L2\\launcher.exe", 2));
    s.on_tick(at(30_000_000_000));

    assert_eq!(s.state(), SessionState::Capturing);
    assert_eq!(s.stop_reason(), None);
    assert_eq!(s.role_bindings().len(), 2);
}

#[test]
fn the_duration_bound_stops_capture() {
    let cfg = SessionConfig {
        duration: Some(Duration::from_secs(10)),
        ..SessionConfig::default()
    };
    let mut s = CaptureSession::new(terminal_chain(), cfg);
    s.attach(at(0));
    s.on_process_event(start(100, 0, "C:\\L\\launcher.exe", 1)); // -> Capturing
    s.on_tick(at(10_000_000_000));
    assert_eq!(s.state(), SessionState::Draining);
    assert_eq!(s.stop_reason(), Some(StopReason::DurationReached));
    s.finalize();
    assert_eq!(s.state(), SessionState::Complete);
}

#[test]
fn a_volume_bound_stops_capture() {
    let cfg = SessionConfig {
        packet_bound: Some(2),
        ..SessionConfig::default()
    };
    let mut s = CaptureSession::new(terminal_chain(), cfg);
    s.attach(at(0));
    s.on_process_event(start(100, 0, "C:\\L\\launcher.exe", 1)); // -> Capturing
    assert_eq!(s.on_packet(100), PacketDisposition::Retained);
    assert_eq!(
        s.on_packet(100),
        PacketDisposition::Retained,
        "the packet that reaches the bound is still retained"
    );
    assert_eq!(s.state(), SessionState::Draining);
    assert_eq!(s.stop_reason(), Some(StopReason::VolumeReached));
}

// Review of PR #26 (Codex P2). A zero volume bound (`--max-packets 0` /
// `--max-bytes 0`) is met before any packet is retained, so the per-packet volume
// check never fires it. The session must still rest in Capturing after
// acquisition (the offline driver detects acquisition by that state), and
// on_volume_reached then produces the promised VolumeReached stop rather than a
// later source-exhausted reason.
#[test]
fn a_zero_volume_bound_stops_via_on_volume_reached() {
    let cfg = SessionConfig {
        packet_bound: Some(0),
        ..SessionConfig::default()
    };
    let mut s = CaptureSession::new(terminal_chain(), cfg);
    s.attach(at(0));
    s.on_process_event(start(100, 0, "C:\\L\\launcher.exe", 1)); // -> Capturing
    assert_eq!(
        s.state(),
        SessionState::Capturing,
        "acquisition rests in Capturing so the offline driver detects it"
    );
    s.on_volume_reached();
    assert_eq!(s.state(), SessionState::Draining);
    assert_eq!(s.stop_reason(), Some(StopReason::VolumeReached));
}

#[test]
fn the_terminal_stage_exit_stops_capture() {
    let mut s = CaptureSession::new(terminal_chain(), SessionConfig::default());
    s.attach(at(0));
    s.on_process_event(start(100, 0, "C:\\L\\launcher.exe", 1)); // launcher -> Capturing
    s.on_process_event(start(200, 100, "C:\\G\\game.exe", 2)); // client (terminal), under launcher
    s.on_process_event(exit(200, 3)); // terminal exits
    assert_eq!(s.stop_reason(), Some(StopReason::TerminalStageExited));
    assert_eq!(s.state(), SessionState::Draining);
}

#[test]
fn all_matched_processes_exiting_stops_capture() {
    let mut s = CaptureSession::new(nonterminal_chain(), SessionConfig::default());
    s.attach(at(0));
    s.on_process_event(start(100, 0, "C:\\L\\launcher.exe", 1)); // launcher (transient)
    s.on_process_event(start(200, 100, "C:\\G\\game.exe", 2)); // client (session, not terminal)
    s.on_process_event(exit(100, 3)); // launcher exits: transient, normal
    assert_eq!(
        s.state(),
        SessionState::Capturing,
        "a transient launcher exit does not end capture on its own"
    );
    s.on_process_event(exit(200, 4)); // last non-service process exits
    assert_eq!(s.stop_reason(), Some(StopReason::AllProcessesExited));
}

#[test]
fn an_interrupt_while_watching_is_a_clean_cancellation() {
    // Watch mode (section 15.7): an operator interrupt before any target is
    // acquired is a clean cancellation, not a failure to acquire. The live path
    // maps StopReason::Interrupt to exit zero.
    let mut s = CaptureSession::new(identity(r#"{"exe":"eso64.exe"}"#), SessionConfig::default());
    s.attach(at(0));
    assert_eq!(s.state(), SessionState::Watching);
    s.on_interrupt();
    assert_eq!(
        s.stop_reason(),
        Some(StopReason::Interrupt),
        "an interrupt while watching is the interrupt reason, not acquisition timeout"
    );
}

#[test]
fn an_interrupt_is_a_normal_stop() {
    let mut s = CaptureSession::new(terminal_chain(), SessionConfig::default());
    s.attach(at(0));
    s.on_process_event(start(100, 0, "C:\\L\\launcher.exe", 1));
    s.on_interrupt();
    assert_eq!(s.stop_reason(), Some(StopReason::Interrupt));
    assert_eq!(s.state(), SessionState::Draining);
    s.finalize();
    assert_eq!(
        s.state(),
        SessionState::Complete,
        "an interrupt yields a complete, valid capture, not an abort"
    );
}

#[test]
fn a_sink_error_stops_capture() {
    let mut s = CaptureSession::new(terminal_chain(), SessionConfig::default());
    s.attach(at(0));
    s.on_process_event(start(100, 0, "C:\\L\\launcher.exe", 1));
    s.on_sink_error();
    assert_eq!(s.stop_reason(), Some(StopReason::SinkError));
    s.finalize();
    assert_eq!(s.state(), SessionState::Complete);
}

#[test]
fn every_stop_condition_reaches_complete_through_draining() {
    // The uniform-shutdown property: whatever the reason, the session drains to
    // Complete after finalize. The two reasons an operator raises directly stand
    // in for the set; the timed and event-driven reasons are covered by their
    // own tests above.
    for reason in [StopReason::Interrupt, StopReason::SinkError] {
        let mut s = CaptureSession::new(terminal_chain(), SessionConfig::default());
        s.attach(at(0));
        s.on_process_event(start(100, 0, "C:\\L\\launcher.exe", 1));
        match reason {
            StopReason::Interrupt => s.on_interrupt(),
            StopReason::SinkError => s.on_sink_error(),
            _ => unreachable!(),
        }
        assert_eq!(s.stop_reason(), Some(reason), "{reason:?}");
        s.finalize();
        assert_eq!(
            s.state(),
            SessionState::Complete,
            "{reason:?} reaches Complete"
        );
    }
}

#[test]
fn a_service_process_does_not_keep_the_all_exited_condition_from_firing() {
    // A profile with a service stage that outlives the session. The client is
    // the only non-service stage; when it exits, capture stops even though the
    // service is still live (section 10.4: a service is never awaited).
    let p = profile(
        r#"{"role":"client","lifecycle":"session","match":{"exe":"game.exe"}},{"role":"platform","lifecycle":"service","match":{"exe":"platform.exe"}}"#,
    );
    let mut s = CaptureSession::new(p, SessionConfig::default());
    s.attach(at(0));
    s.on_process_event(start(100, 0, "C:\\G\\game.exe", 1)); // client -> Capturing
    s.on_process_event(start(200, 0, "C:\\P\\platform.exe", 2)); // service, still live
    s.on_process_event(exit(100, 3)); // the only non-service process exits
    assert_eq!(
        s.stop_reason(),
        Some(StopReason::AllProcessesExited),
        "a live service does not keep the session alive"
    );
}

#[test]
fn a_service_match_does_not_begin_capturing_or_disable_the_timeout() {
    // A persistent service that appears while Watching must not acquire the
    // target (section 10.4: a service is never awaited during acquisition), so
    // the acquisition timeout still governs and no service noise is retained.
    let p = profile(
        r#"{"role":"client","lifecycle":"session","match":{"exe":"game.exe"}},{"role":"platform","lifecycle":"service","match":{"exe":"platform.exe"}}"#,
    );
    let cfg = SessionConfig {
        acquisition_timeout: Some(Duration::from_secs(30)),
        ..SessionConfig::default()
    };
    let mut s = CaptureSession::new(p, cfg);
    s.attach(at(0));
    s.on_process_event(start(200, 0, "C:\\P\\platform.exe", 1)); // service appears first
    assert_eq!(
        s.state(),
        SessionState::Watching,
        "a service does not begin capturing"
    );
    assert_eq!(s.on_packet(100), PacketDisposition::Discarded); // still discarding
    s.on_tick(at(30_000_000_000)); // timeout with no real target
    assert_eq!(s.state(), SessionState::Complete);
    assert_eq!(s.stop_reason(), Some(StopReason::AcquisitionTimeout));
    assert_eq!(s.stats().retained, 0, "no service noise was retained");
}

#[test]
fn an_exit_delivered_before_its_start_still_stops_a_terminal() {
    // ETW can deliver an exit before its matching start; the tree holds the exit
    // and joins it when the start arrives, so the process is bound already
    // exited. A terminal in that state must still stop capture rather than being
    // recorded as live.
    let mut s = CaptureSession::new(terminal_chain(), SessionConfig::default());
    s.attach(at(0));
    s.on_process_event(start(100, 0, "C:\\L\\launcher.exe", 1)); // launcher -> Capturing
    s.on_process_event(exit(200, 5)); // the client's exit arrives first
    s.on_process_event(start(200, 100, "C:\\G\\game.exe", 2)); // terminal, joins the held exit
    assert_eq!(
        s.stop_reason(),
        Some(StopReason::TerminalStageExited),
        "a terminal bound already exited still stops capture"
    );
}

#[test]
fn packets_offered_after_a_stop_are_counted_out_of_window() {
    // After a stop moves the session to Draining, a packet still in flight is
    // discarded but counted, never silently dropped (P-4).
    let mut s = CaptureSession::new(terminal_chain(), SessionConfig::default());
    s.attach(at(0));
    s.on_process_event(start(100, 0, "C:\\L\\launcher.exe", 1)); // -> Capturing
    assert_eq!(s.on_packet(100), PacketDisposition::Retained);
    s.on_interrupt(); // -> Draining
    assert_eq!(s.on_packet(100), PacketDisposition::Discarded);
    assert_eq!(
        s.stats().discarded_out_of_window,
        1,
        "P-4: the post-stop discard is counted, not silently dropped"
    );
    assert_eq!(
        s.stats().observed(),
        2,
        "conservation holds across all three counters"
    );
}

#[test]
fn a_role_outside_the_scope_neither_acquires_nor_stamps() {
    // Specification FR-011b: --roles scopes which stages trigger and are
    // captured. Scoped to the launcher, the terminal client stage is treated as
    // if it were absent from the profile: it never binds, so it never stamps a
    // role and its match cannot stop the run, while the in-scope launcher
    // acquires the target and stamps exactly as an unscoped run would.
    let scope = Some(vec!["launcher".to_string()]);
    let mut s = CaptureSession::new_scoped(terminal_chain(), SessionConfig::default(), scope);
    s.attach(at(0));

    // The in-scope launcher binds and acquires the target.
    s.on_process_event(start(100, 0, "C:\\L\\launcher.exe", 1));
    assert_eq!(
        s.state(),
        SessionState::Capturing,
        "the in-scope role acquires"
    );
    let bindings = s.role_bindings();
    assert_eq!(bindings.len(), 1);
    assert_eq!(
        bindings[0].1.as_deref(),
        Some("launcher"),
        "the in-scope role is stamped"
    );

    // The out-of-scope client would match (it descends from the bound launcher
    // and is terminal), but scope keeps it from binding: no new stamp, and the
    // run does not stop on a terminal it was told to ignore.
    s.on_process_event(start(200, 100, "C:\\G\\game.exe", 2));
    assert_eq!(
        s.role_bindings().len(),
        1,
        "the out-of-scope role does not stamp"
    );
    assert!(
        s.stop_reason().is_none(),
        "an out-of-scope terminal match does not stop the run"
    );
    assert_eq!(s.state(), SessionState::Capturing);
}
