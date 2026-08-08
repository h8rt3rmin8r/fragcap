// SPDX-License-Identifier: Apache-2.0

//! The claim in specification section 25.1, exercised.
//!
//! "Substituting a replay `PacketSource` reading a recorded capture file and a
//! scripted `FlowAttributor` returning predetermined attributions converts the
//! pipeline into a deterministic function from fixture input to output."
//!
//! This is the first test in the project that puts a source, the header parser,
//! and an attributor together, and it runs with no capture driver, no elevated
//! privilege, and no game. Slices S02, S03, and S04 meet here.
//!
//! It lives in the facade because the facade is the crate that legitimately
//! depends on both `fragcap-capture` and `fragcap-attr`. Putting it in either
//! of those would have meant a dev-dependency between them, which is exactly
//! the edge constitution P-3 and `cargo xtask deps` exist to prevent, and which
//! a dev-dependency would have slipped past unnoticed.

use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use fragcap::{
    AttributionScript, AttributionState, CapturedPacket, Direction, FlowAttributor, HeaderParser,
    InterfaceAddrs, PacketSource, ReplaySource, ScriptedAttributor, SourceError,
};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
}

fn ip(s: &str) -> IpAddr {
    s.parse().expect("test address must parse")
}

/// What one run over a fixture produced.
struct Run {
    packets: Vec<CapturedPacket>,
    attributed: usize,
    unattributed: usize,
    not_attempted: usize,
}

/// Read a fixture, parse every packet, and resolve every flow against the
/// fixture's own script.
///
/// This loop is the shape S08 will build the pipeline around, and it is worth
/// noticing how little there is to it: the seams do the work.
fn run(name: &str, local: &[IpAddr]) -> Run {
    let dir = fixtures_dir();
    let mut source = ReplaySource::open(dir.join(format!("{name}.pcap")))
        .unwrap_or_else(|e| panic!("{name}.pcap must open: {e}"));
    let script = AttributionScript::load(dir.join(format!("{name}.script")))
        .unwrap_or_else(|e| panic!("{name}.script must load: {e}"));
    // Boxed, which is how S08 will hold it. Anything the pipeline needs from an
    // attributor has to be reachable through the seam, and this is where that
    // gets proved rather than assumed.
    let attributor: Box<dyn FlowAttributor> = Box::new(ScriptedAttributor::new(script));
    let mut parser = HeaderParser::new(InterfaceAddrs::new(local.iter().copied()));
    let link = source.link_type();

    let mut out = Run {
        packets: Vec::new(),
        attributed: 0,
        unattributed: 0,
        not_attempted: 0,
    };

    loop {
        let raw = match source.next_packet(Duration::from_millis(0)) {
            Ok(Some(raw)) => raw,
            Ok(None) => panic!("a replay source never times out"),
            Err(SourceError::Closed) => break,
            Err(e) => panic!("{name}: unexpected source failure: {e}"),
        };
        let mut packet = CapturedPacket::from_raw(raw);
        parser.apply(link, &mut packet);
        if let Some(key) = packet.flow.as_ref() {
            // The packet's own instant, through the seam. Specification section
            // 11.4: capture and socket table observation are not synchronized,
            // so the question is who owned this flow then, not now.
            packet.attribution = attributor.resolve(key, packet.ts);
        }
        match packet.attribution_state() {
            AttributionState::Resolved => out.attributed += 1,
            AttributionState::Unresolved => out.unattributed += 1,
            AttributionState::NotAttempted => out.not_attempted += 1,
        }
        out.packets.push(packet);
    }

    assert!(
        source.replay_stats().read_whole_file(),
        "{name} did not read whole: {}",
        source.replay_stats()
    );
    out
}

// SC-001. The claim itself.
#[test]
fn the_pipeline_runs_over_a_fixture_with_no_capture_driver() {
    let r = run("tcp-session", &[ip("192.0.2.10")]);
    assert_eq!(r.packets.len(), 6);
    assert_eq!(
        r.attributed, 6,
        "every packet of an attributed flow resolves"
    );
    assert_eq!(r.unattributed, 0);
    assert_eq!(r.not_attempted, 0);
    for packet in &r.packets {
        let a = packet.attribution.as_ref().expect("resolved");
        assert_eq!(a.pid, 4242);
        assert_eq!(&*a.process, "game.exe");
    }
}

#[test]
fn every_fixture_in_the_corpus_runs_end_to_end() {
    // Not just the convenient one. A fixture whose script no longer loads, or
    // whose capture no longer reads, fails here rather than in whichever later
    // slice first tried to use it.
    let cases: Vec<(&str, Vec<IpAddr>)> = vec![
        ("tcp-session", vec![ip("192.0.2.10")]),
        ("udp-gameplay", vec![ip("192.0.2.10")]),
        ("ipv6-mixed", vec![ip("2001:db8::10")]),
        ("fragmented", vec![ip("192.0.2.10")]),
        ("loopback", vec![ip("127.0.0.1")]),
        ("malformed", vec![ip("192.0.2.10"), ip("2001:db8::10")]),
        ("port-reuse", vec![ip("192.0.2.10")]),
        ("burst", vec![ip("192.0.2.10")]),
    ];
    assert_eq!(cases.len(), 8, "section 25.3 names eight fixtures");
    for (name, local) in cases {
        let r = run(name, &local);
        assert!(!r.packets.is_empty(), "{name} produced no packets");
        assert_eq!(
            r.attributed + r.unattributed + r.not_attempted,
            r.packets.len(),
            "{name}: every packet is in exactly one attribution state"
        );
    }
}

// SC-002, through the whole stack rather than the reader alone.
#[test]
fn two_runs_over_one_fixture_agree_exactly() {
    let a = run("udp-gameplay", &[ip("192.0.2.10")]);
    let b = run("udp-gameplay", &[ip("192.0.2.10")]);
    assert_eq!(
        a.packets, b.packets,
        "a golden comparison against a varying input is not a test"
    );
    assert_eq!(
        (a.attributed, a.unattributed),
        (b.attributed, b.unattributed)
    );
}

// The case no other test in the project can currently express.
#[test]
fn port_reuse_changes_owner_part_way_through_the_capture() {
    let r = run("port-reuse", &[ip("192.0.2.10")]);
    assert_eq!(r.attributed, r.packets.len(), "all six resolve");

    let owners: Vec<u32> = r
        .packets
        .iter()
        .map(|p| p.attribution.as_ref().expect("resolved").pid)
        .collect();
    assert_eq!(
        owners,
        vec![100, 100, 100, 200, 200, 200],
        "the same local port belongs to two processes at two times, and only \
         the script's windows distinguish them"
    );

    // The flow key is identical throughout, which is the whole difficulty: the
    // capture alone cannot tell the two sessions apart.
    let keys: std::collections::HashSet<_> = r.packets.iter().filter_map(|p| p.flow).collect();
    assert_eq!(keys.len(), 1);
}

#[test]
fn loopback_resolves_an_owner_while_leaving_direction_undetermined() {
    let r = run("loopback", &[ip("127.0.0.1")]);
    assert_eq!(r.attributed, r.packets.len(), "an owner is still knowable");
    for packet in &r.packets {
        assert_eq!(
            packet.direction, None,
            "section 12.6 resolves this from the attributed endpoint in a \
             later slice, and guessing now would be wrong half the time"
        );
    }
}

#[test]
fn fragments_resolve_without_anything_being_reassembled() {
    let r = run("fragmented", &[ip("192.0.2.10")]);
    assert_eq!(r.packets.len(), 3);
    assert_eq!(
        r.attributed, 3,
        "the later fragments resolve from what the first one said"
    );
    // Nothing was joined: the three payloads are still three payloads.
    let lengths: Vec<usize> = r.packets.iter().map(|p| p.captured_len()).collect();
    assert_eq!(lengths.len(), 3);
    assert!(
        lengths.iter().all(|n| *n > 0),
        "reassembly would have collapsed these"
    );
}

#[test]
fn a_malformed_fixture_attributes_nothing_and_loses_nothing() {
    let r = run("malformed", &[ip("192.0.2.10"), ip("2001:db8::10")]);
    assert_eq!(r.attributed, 0, "nothing here parses to a flow");
    assert_eq!(r.unattributed, 0);
    assert_eq!(
        r.not_attempted,
        r.packets.len(),
        "no flow key means attribution was never attempted, which is not the \
         same as attempted and failed"
    );
    // P-4: every one of them is still here.
    assert_eq!(r.packets.len(), 5);
}

#[test]
fn direction_reflects_the_interface_address_set() {
    let r = run("tcp-session", &[ip("192.0.2.10")]);
    let outbound = r
        .packets
        .iter()
        .filter(|p| p.direction == Some(Direction::Outbound))
        .count();
    let inbound = r
        .packets
        .iter()
        .filter(|p| p.direction == Some(Direction::Inbound))
        .count();
    assert_eq!((outbound, inbound), (3, 3), "the fixture alternates");

    // With the peer's address instead, every direction inverts. The packets are
    // identical; only what fragcap believes about the host changed.
    let flipped = run("tcp-session", &[ip("198.51.100.5")]);
    let flipped_outbound = flipped
        .packets
        .iter()
        .filter(|p| p.direction == Some(Direction::Outbound))
        .count();
    assert_eq!(flipped_outbound, 3);
    assert_ne!(
        r.packets[0].direction, flipped.packets[0].direction,
        "direction is a statement about the capturing host, not the wire"
    );
}

// C5's regression guard. Task T025a asked for an endpoint declaration per
// scripted flow, and `ipv6-mixed` had one for its TCP flow and not its UDP one.
// Nothing caught it, because nothing checked.
#[test]
fn every_scripted_flow_declares_its_local_endpoint() {
    let dir = fixtures_dir();
    for name in [
        "tcp-session",
        "udp-gameplay",
        "ipv6-mixed",
        "fragmented",
        "loopback",
        "malformed",
        "port-reuse",
        "burst",
    ] {
        let script = AttributionScript::load(dir.join(format!("{name}.script")))
            .unwrap_or_else(|e| panic!("{name}.script must load: {e}"));
        let declared: std::collections::HashSet<_> = script
            .endpoints()
            .iter()
            .map(|e| (e.addr, e.proto))
            .collect();
        for entry in script.entries() {
            assert!(
                declared.contains(&(entry.local, entry.proto)),
                "{name}.script scripts a flow on {} but never declares it as an \
                 active endpoint, so an attributor cannot report it",
                entry.local
            );
        }
    }
}

#[test]
fn a_scripted_attributor_is_usable_through_the_seam_alone() {
    // What S08 will hold: a boxed source and a boxed attributor, neither
    // naming the other's crate.
    let dir = fixtures_dir();
    let source: Box<dyn PacketSource> =
        Box::new(ReplaySource::open(dir.join("tcp-session.pcap")).expect("the fixture opens"));
    let attributor: Box<dyn FlowAttributor> = Box::new(ScriptedAttributor::new(
        AttributionScript::load(dir.join("tcp-session.script")).expect("the script loads"),
    ));
    assert_eq!(source.link_type(), fragcap::LinkType::ETHERNET);
    assert!(attributor.active_endpoints().len() == 1);
}
