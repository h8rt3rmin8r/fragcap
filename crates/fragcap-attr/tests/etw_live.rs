// SPDX-License-Identifier: Apache-2.0

//! Tier 2: the ETW watcher against the machine it is running on.
//!
//! Specification section 25.2 puts anything needing elevation in tier 2, so
//! none of this runs in the ordinary check set. Every test here is `#[ignore]`
//! and the whole file is behind the `etw` feature.
//!
//! Run from an elevated terminal with:
//!
//! ```text
//! cargo test -p fragcap-attr --features etw -- --ignored
//! ```
//!
//! **As of slice S11 none of this has ever executed.** Saying so is the point:
//! the project already carries one capability whose tests have never run,
//! because no continuous integration runner has npcap installed, and reporting
//! an unverified success is worse than reporting a known gap.

#![cfg(all(windows, feature = "etw"))]

use std::time::{Duration, Instant};

use fragcap_attr::etw::EtwWatcher;
use fragcap_core::traits::ProcessWatcher;
use fragcap_core::{Ancestry, CommandLine, ProcessId, ProcessTree};

/// A session name unlikely to collide, and never `NT Kernel Logger`.
fn session_name(suffix: &str) -> String {
    format!("fragcap-test-{}-{}", std::process::id(), suffix)
}

/// Collect events for a bounded time, folding them as a run would.
fn drain(
    watcher: &EtwWatcher,
    rx: &std::sync::mpsc::Receiver<fragcap_core::ProcessEvent>,
    for_: Duration,
) -> ProcessTree {
    let mut tree = ProcessTree::new();
    tree.apply_snapshot(&watcher.snapshot());
    let deadline = Instant::now() + for_;
    while Instant::now() < deadline {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(e) => tree.apply(e),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => break,
        }
    }
    tree
}

#[test]
#[ignore = "tier 2: needs an elevated session"]
fn a_child_this_test_spawns_is_observed_at_its_creation() {
    let watcher = EtwWatcher::start(&session_name("child")).expect("elevated session");
    let rx = watcher.subscribe();

    // Deliberately short-lived. Any implementation that polled would miss it,
    // which is the property specification section 10.1 is built on and the one
    // worth having a test fail over.
    let mut child = std::process::Command::new("cmd.exe")
        .args(["/c", "exit", "0"])
        .spawn()
        .expect("spawn");
    let child_pid = child.id();
    let _ = child.wait();

    let tree = drain(&watcher, &rx, Duration::from_secs(5));

    let me = std::process::id();
    let node = tree
        .resolve(
            ProcessId(child_pid),
            fragcap_core::packet::Timestamp::from_nanos(i64::MAX / 2),
        )
        .or_else(|| {
            tree.nodes()
                .find(|n| n.pid() == ProcessId(child_pid))
                .map(|n| n.id())
        })
        .expect("the child was observed");

    let node = tree.node(node).unwrap();
    assert_eq!(node.image_name().to_lowercase(), "cmd.exe");
    assert_eq!(
        node.ancestry(),
        Ancestry::Observed,
        "creation-time ancestry"
    );
    assert!(
        matches!(node.command_line(), CommandLine::Observed(c) if c.contains("exit")),
        "the start event carries the command line verbatim"
    );

    // The parent recorded at the instant of creation is this test process.
    let parent = node.parent().expect("parent resolved");
    assert_eq!(tree.node(parent).unwrap().pid(), ProcessId(me));

    // And its exit was seen too.
    assert!(node.exited().is_some(), "the exit event arrived");
}

#[test]
#[ignore = "tier 2: needs an elevated session"]
fn the_startup_snapshot_finds_this_process() {
    let watcher = EtwWatcher::start(&session_name("snap")).expect("elevated session");
    let me = std::process::id();

    let snap = watcher.snapshot();
    assert!(snap.iter().any(|r| r.pid == me), "this process is running");

    // No snapshot record carries a command line, because obtaining one needs a
    // memory-read right P-1 forbids.
    assert!(snap.iter().all(|r| !r.command_line.is_available()));
}

#[test]
#[ignore = "tier 2: needs an elevated session"]
fn the_watcher_reports_what_the_kernel_lost() {
    let watcher = EtwWatcher::start(&session_name("report")).expect("elevated session");
    let report = watcher.report();
    assert!(report.running);

    let stopped = watcher.stop();
    assert!(!stopped.running);
}

#[test]
#[ignore = "tier 2: needs an elevated session"]
fn two_watchers_coexist_rather_than_contending_for_one_session() {
    // The property FR-005 is about. If this fails with ERROR_NO_SYSTEM_RESOURCES
    // the machine is already at its system-logger limit, which is a real
    // condition and not a defect in fragcap.
    let a = EtwWatcher::start(&session_name("a")).expect("first session");
    let b = EtwWatcher::start(&session_name("b")).expect("second session");
    assert!(a.report().running);
    assert!(b.report().running);
}

#[test]
#[ignore = "tier 2: needs an elevated session"]
fn a_second_watcher_by_the_same_name_is_refused_rather_than_taking_over() {
    let name = session_name("dup");
    let _first = EtwWatcher::start(&name).expect("first session");
    match EtwWatcher::start(&name) {
        Err(fragcap_attr::WatcherError::SessionUnavailable { code, .. }) => {
            assert_eq!(code, 183, "ERROR_ALREADY_EXISTS");
        }
        Ok(_) => panic!("fragcap must not reuse a session it did not create"),
        Err(other) => panic!("unexpected: {other}"),
    }
}

/// Runs unelevated, which is the ordinary developer's machine.
///
/// Not ignored, because it needs no privilege by definition, and because the
/// property it checks is the one an operator meets first.
#[test]
fn without_elevation_the_watcher_says_so_rather_than_degrading() {
    match EtwWatcher::start(&session_name("unelevated")) {
        Err(fragcap_attr::WatcherError::NotElevated) => {
            // The expected outcome on an unelevated machine.
        }
        Ok(w) => {
            // The session is elevated. Nothing to assert about the refusal, so
            // say so rather than passing silently over an untested path.
            eprintln!("this session is elevated; the refusal path was not exercised");
            let _ = w.stop();
        }
        Err(other) => panic!("expected NotElevated or success, got: {other}"),
    }
}
