// SPDX-License-Identifier: Apache-2.0

//! Tier 2: the live capture source against a real capture driver.
//!
//! Specification section 25.2. Needs Windows, npcap installed with loopback
//! capture support, and an elevated shell. No game and no profiled process is
//! involved: the test generates its own traffic and looks for it coming back.
//!
//! **A test here that finds no driver prints why and returns.** Rust's harness
//! has no skip, and a test that failed on a machine without npcap would make the
//! `live` feature unusable for local development, which would in turn mean
//! nobody compiles it until continuous integration does. A run that skipped
//! everything has proved nothing, so read the output rather than the exit code.

#![cfg(all(windows, feature = "live"))]

use std::net::UdpSocket;
use std::time::{Duration, Instant};

use fragcap_capture::live::{self, LiveOptions, LiveSource, BOOTSTRAP_FILTER};
use fragcap_core::interface::InterfaceRecord;
use fragcap_core::traits::PacketSource;

/// The loopback adapter, if npcap was installed with loopback capture support.
///
/// Returning `None` is the "cannot run here" signal every test below checks.
fn loopback_interface() -> Option<InterfaceRecord> {
    let inventory = live::enumerate().ok()?;
    inventory
        .interfaces
        .into_iter()
        .find(|record| record.is_loopback)
}

fn skip(reason: &str) {
    eprintln!("skipped: {reason}");
}

// FR-041, FR-042. The report is honest about what it could and could not
// determine, on whatever machine this runs on.
#[test]
fn driver_detection_reports_what_it_observed_and_nothing_more() {
    let report = live::detect_driver();
    eprintln!("driver report: {report:?}");

    assert_eq!(
        report.version, None,
        "the binding exposes no version and one must not be invented"
    );
    assert_eq!(
        report.winpcap_compatible, None,
        "compatibility mode is not determinable through libpcap"
    );
    if !report.present {
        assert_eq!(
            report.loopback_supported, None,
            "with no driver, nothing was observed about its options"
        );
    }
}

// FR-001, FR-003. Enumeration answers without opening a handle, so an operator
// can be told what exists before anything is captured.
#[test]
fn enumeration_describes_the_machines_interfaces() {
    let Ok(inventory) = live::enumerate() else {
        skip("interface enumeration failed, most likely no capture driver");
        return;
    };
    assert!(
        !inventory.interfaces.is_empty(),
        "a machine with a capture driver has at least one interface"
    );
    for record in &inventory.interfaces {
        assert!(
            !record.name.is_empty(),
            "an interface with no name is unusable"
        );
    }
    eprintln!(
        "enumerated {} interfaces, default route source {:?}",
        inventory.interfaces.len(),
        inventory.default_route_source
    );
}

// SC-001. The claim this whole slice exists to make: fragcap reads a frame from
// a real interface, with the driver's own timestamp and length.
#[test]
fn a_frame_this_test_sent_comes_back_with_the_drivers_own_numbers() {
    let Some(record) = loopback_interface() else {
        skip("no loopback adapter; npcap may be installed without loopback support");
        return;
    };

    let mut source = match LiveSource::open(&record, LiveOptions::default()) {
        Ok(source) => source,
        Err(e) => {
            skip(&format!(
                "could not open {}: {e}. Elevation is required",
                record.name
            ));
            return;
        }
    };

    // A distinctive payload, so that a frame belonging to something else on
    // loopback cannot be mistaken for this test's.
    const MARKER: &[u8] = b"fragcap-s09-tier2-probe";
    let socket = UdpSocket::bind("127.0.0.1:0").expect("binding loopback cannot fail");
    let target = socket.local_addr().expect("a bound socket has an address");

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut seen = None;
    while Instant::now() < deadline && seen.is_none() {
        socket
            .send_to(MARKER, target)
            .expect("sending to ourselves on loopback cannot fail");

        while let Ok(Some(packet)) = source.next_packet(Duration::from_millis(100)) {
            if packet
                .data
                .windows(MARKER.len())
                .any(|window| window == MARKER)
            {
                seen = Some(packet);
                break;
            }
        }
    }

    let packet = seen.expect("the probe frame must be captured within five seconds");

    // The driver's timestamp, not one fragcap applied. Section 12.7.
    assert!(
        packet.ts.as_nanos() > 0,
        "a driver timestamp of zero means fragcap read the wrong field"
    );
    // The on-wire length, kept separate from what was retained.
    assert!(packet.orig_len as usize >= packet.captured_len());
    assert!(
        !packet.is_truncated(),
        "the default snapshot length retains a whole loopback frame"
    );
}

// FR-016. A quiet interface must not end a run.
#[test]
fn a_quiet_interface_reports_nothing_rather_than_failing() {
    let Some(record) = loopback_interface() else {
        skip("no loopback adapter");
        return;
    };
    let mut source = match LiveSource::open(&record, LiveOptions::default()) {
        Ok(source) => source,
        Err(e) => {
            skip(&format!("could not open {}: {e}", record.name));
            return;
        }
    };

    // Nothing is being sent, so within a short timeout the honest answer is
    // "nothing arrived" rather than an error. A handful of frames from other
    // processes on loopback are possible and are equally acceptable.
    match source.next_packet(Duration::from_millis(50)) {
        Ok(_) => {}
        Err(e) => panic!("a quiet interface must not produce an error: {e}"),
    }
}

// FR-017. The driver's counters reach fragcap unaltered.
#[test]
fn the_backends_counters_are_relayed() {
    let Some(record) = loopback_interface() else {
        skip("no loopback adapter");
        return;
    };
    let Ok(source) = LiveSource::open(&record, LiveOptions::default()) else {
        skip("could not open the loopback adapter");
        return;
    };
    let stats = source.stats();
    eprintln!("backend reported {stats:?}");
    // Nothing stronger is assertable: what the driver reports depends on the
    // machine. What matters is that reading them does not fail and does not
    // fold fragcap's own accounting in, which the type system already enforces.
    assert_eq!(
        stats.total_dropped(),
        stats.kernel_dropped + stats.interface_dropped
    );
}

// FR-036. The bootstrap filter is installed before any packet is delivered.
#[test]
fn the_bootstrap_filter_is_accepted_by_the_backend() {
    let Some(record) = loopback_interface() else {
        skip("no loopback adapter");
        return;
    };
    // `open` installs it, so a backend that rejected the expression would fail
    // here rather than silently capturing more than section 12.2 phase one
    // describes.
    match LiveSource::open(&record, LiveOptions::default()) {
        Ok(_) => {}
        Err(e) => {
            let text = e.to_string();
            assert!(
                !text.contains("filter"),
                "the backend rejected the bootstrap filter {BOOTSTRAP_FILTER:?}: {e}"
            );
            skip(&format!("could not open the loopback adapter: {e}"));
        }
    }
}
