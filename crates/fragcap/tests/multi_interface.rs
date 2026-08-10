// SPDX-License-Identifier: Apache-2.0

//! Tier 1: several interfaces, told apart, with no capture driver anywhere.
//!
//! Slice S09, user story 2. Two replay sources stand in for two interfaces,
//! which is the whole point of the seam: everything this project can verify
//! about multi-interface capture is verified here, on any machine, before a
//! line of Windows-specific code runs.
//!
//! What is asserted, in the order it matters:
//!
//! 1. Every packet reaches output labelled with the interface it arrived on.
//! 2. Two interfaces with different link types are each parsed against their
//!    own, not against a capture-wide one.
//! 3. The conservation identity holds with two capture threads running.
//! 4. A failed interface retires without ending the run, and the report names
//!    it.
//!
//! The fourth is the one that would otherwise have shipped untested; the
//! analyze gate caught that it was the only success criterion with an
//! implementation task and no test.

mod common;

use std::time::Duration;

use common::{corpus_addrs, fixtures_dir, SharedBuf};
use fragcap::{
    AttributionScript, InterfaceAddrs, InterfaceDeclaration, JsonLinesWriter, LinkType,
    PayloadMode, PcapngWriter, Pipeline, PipelineConfig, ReplaySource, ScriptedAttributor,
};
use fragcap_core::error::SourceError;
use fragcap_core::filter::FilterProgram;
use fragcap_core::interface::{InterfaceId, RetirementReason};
use fragcap_core::packet::RawPacket;
use fragcap_core::pipeline::SourceBinding;
use fragcap_core::stats::SourceStats;
use fragcap_core::traits::PacketSource;

/// The Ethernet side. Every committed fixture is Ethernet, so the second
/// interface is synthesised below rather than taken from the corpus.
const FIRST: &str = "tcp-session";
const SECOND: &str = "synthetic-loopback";

/// A capture file in BSD loopback encapsulation, built here because no
/// committed fixture uses a link type other than Ethernet.
///
/// That matters: with two Ethernet interfaces, a pipeline that used one
/// capture-wide link type would pass every assertion in this file. The
/// per-interface parser is only actually exercised when the two differ, so the
/// difference has to be manufactured.
///
/// The frames are 127.0.0.1 to 127.0.0.1 over UDP, which parses to a flow key
/// with an ambiguous direction. Ambiguity is not a rejection, so a clean parse
/// count still means every frame was understood.
fn loopback_capture(frames: usize) -> Vec<u8> {
    let mut out = Vec::new();
    // Classic pcap header, little endian, link type 0 (NULL).
    out.extend_from_slice(&0xa1b2c3d4u32.to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&4u16.to_le_bytes());
    out.extend_from_slice(&0i32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&65_535u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());

    for i in 0..frames {
        let payload = [0xa5u8; 4];
        let mut udp = Vec::new();
        udp.extend_from_slice(&(40_000u16 + i as u16).to_be_bytes());
        udp.extend_from_slice(&5055u16.to_be_bytes());
        udp.extend_from_slice(&((8 + payload.len()) as u16).to_be_bytes());
        udp.extend_from_slice(&0u16.to_be_bytes());
        udp.extend_from_slice(&payload);

        let mut ip = Vec::new();
        ip.push(0x45);
        ip.push(0);
        ip.extend_from_slice(&((20 + udp.len()) as u16).to_be_bytes());
        ip.extend_from_slice(&0u16.to_be_bytes());
        ip.extend_from_slice(&0u16.to_be_bytes());
        ip.push(64);
        ip.push(17);
        ip.extend_from_slice(&0u16.to_be_bytes());
        ip.extend_from_slice(&[127, 0, 0, 1]);
        ip.extend_from_slice(&[127, 0, 0, 1]);
        ip.extend_from_slice(&udp);

        // BSD loopback: a four byte address family in host byte order.
        let mut frame = Vec::new();
        frame.extend_from_slice(&2u32.to_le_bytes());
        frame.extend_from_slice(&ip);

        out.extend_from_slice(&1_700_000_000u32.to_le_bytes());
        out.extend_from_slice(&(i as u32 * 1000).to_le_bytes());
        out.extend_from_slice(&(frame.len() as u32).to_le_bytes());
        out.extend_from_slice(&(frame.len() as u32).to_le_bytes());
        out.extend_from_slice(&frame);
    }
    out
}

struct Run {
    pcapng: Vec<u8>,
    jsonl: Vec<u8>,
    report: fragcap::PipelineReport,
}

/// The loopback side's only address.
fn loopback_addr() -> InterfaceAddrs {
    InterfaceAddrs::new(["127.0.0.1".parse().expect("a literal address parses")])
}

/// Compose two replay sources as two declared interfaces and run them.
///
/// `swap` gives each interface the *other* one's address set, which is the only
/// way to tell a genuinely per-interface address set from a shared union: a
/// union is unchanged by swapping its halves, so a pipeline that ignored the
/// per-binding set would produce identical counters either way.
fn run_two(extra: Option<Box<dyn PacketSource>>, swap: bool) -> Run {
    let dir = fixtures_dir();

    let a = ReplaySource::open(dir.join(format!("{FIRST}.pcap"))).expect("fixture opens");
    let b = ReplaySource::from_bytes(loopback_capture(4)).expect("the synthetic capture parses");
    let link_a = a.link_type();
    let link_b = b.link_type();
    assert_ne!(
        link_a, link_b,
        "the fixtures must differ in link type or the per-interface parser is untested"
    );

    let script =
        AttributionScript::load(dir.join(format!("{FIRST}.script"))).expect("script loads");

    let pcapng_buf = SharedBuf::default();
    let jsonl_buf = SharedBuf::default();

    // Every interface is declared before any packet is written, which is what
    // lets the annotation `iface` key be decided once rather than per packet.
    let mut pcapng = PcapngWriter::new(pcapng_buf.clone()).expect("in-memory write cannot fail");
    pcapng
        .declare_interface(&InterfaceDeclaration::new(link_a, 65_535, FIRST))
        .expect("the first interface is accepted");
    pcapng
        .declare_interface(&InterfaceDeclaration::new(link_b, 65_535, SECOND))
        .expect("the second interface is accepted");
    if extra.is_some() {
        pcapng
            .declare_interface(&InterfaceDeclaration::new(link_a, 65_535, "failing"))
            .expect("the third interface is accepted");
    }

    let names: Vec<&str> = if extra.is_some() {
        vec![FIRST, SECOND, "failing"]
    } else {
        vec![FIRST, SECOND]
    };
    let jsonl = JsonLinesWriter::new(jsonl_buf.clone(), &names, PayloadMode::MetadataOnly)
        .expect("in-memory write cannot fail");

    // Each interface gets its own addresses, per specification section 12.6.
    let ethernet_addrs = InterfaceAddrs::new(corpus_addrs(FIRST).iter().copied());
    let (addrs_a, addrs_b) = if swap {
        (loopback_addr(), ethernet_addrs)
    } else {
        (ethernet_addrs, loopback_addr())
    };

    let mut sources = vec![
        SourceBinding::new(InterfaceId::new(0), Box::new(a), addrs_a),
        SourceBinding::new(InterfaceId::new(1), Box::new(b), addrs_b),
    ];
    if let Some(extra) = extra {
        sources.push(SourceBinding::new(
            InterfaceId::new(2),
            extra,
            InterfaceAddrs::default(),
        ));
    }

    let mut pipeline = Pipeline::new(
        sources,
        Box::new(ScriptedAttributor::new(script)),
        PipelineConfig {
            ..PipelineConfig::default()
        },
    )
    .expect("a non-zero capacity and a non-empty source list build");
    pipeline.add_sink(Box::new(pcapng));
    pipeline.add_sink(Box::new(jsonl));

    let report = pipeline.run();
    Run {
        pcapng: pcapng_buf.contents(),
        jsonl: jsonl_buf.contents(),
        report,
    }
}

/// A source that yields nothing and then fails the way a removed adapter does.
struct LostDevice {
    yielded: bool,
}

impl PacketSource for LostDevice {
    fn next_packet(&mut self, _timeout: Duration) -> Result<Option<RawPacket>, SourceError> {
        if self.yielded {
            return Err(SourceError::DeviceLost {
                detail: "the adapter was removed".into(),
            });
        }
        self.yielded = true;
        Ok(None)
    }

    fn set_filter(&mut self, _filter: &FilterProgram) -> Result<(), SourceError> {
        Ok(())
    }

    fn stats(&self) -> SourceStats {
        SourceStats::default()
    }

    fn link_type(&self) -> LinkType {
        LinkType::ETHERNET
    }
}

// FR-031, FR-028, SC-004. Both interfaces reach the file, and each packet
// references the one it came from.
#[test]
fn both_interfaces_are_declared_and_every_packet_names_its_own() {
    let run = run_two(None, false);

    let jsonl = String::from_utf8(run.jsonl).expect("the writer emits UTF-8");
    let mut lines = jsonl.lines();
    let header = lines.next().expect("a header line");
    assert!(header.contains(FIRST) && header.contains(SECOND));

    let mut from_first = 0;
    let mut from_second = 0;
    for line in lines {
        if line.contains(r#""type":"trailer""#) {
            continue;
        }
        if line.contains(&format!("\"iface\":\"{FIRST}\"")) {
            from_first += 1;
        } else if line.contains(&format!("\"iface\":\"{SECOND}\"")) {
            from_second += 1;
        } else {
            panic!("a record named no declared interface: {line}");
        }
    }
    assert!(from_first > 0, "the first interface produced nothing");
    assert!(from_second > 0, "the second interface produced nothing");
    assert_eq!(
        (from_first + from_second) as u64,
        run.report.stats.packets_captured,
        "every captured packet must appear exactly once, labelled"
    );

    assert!(
        !run.pcapng.is_empty(),
        "the pcapng writer produced nothing for a two interface capture"
    );
}

// FR-026. Two link types, two parsers. If the pipeline used one capture-wide
// link type, one fixture's frames would parse against the wrong encapsulation
// and produce rejections rather than flow keys.
#[test]
fn each_interface_is_parsed_against_its_own_link_type() {
    let run = run_two(None, false);
    let parse = run.report.stats.parse;
    // The discriminating assertion. A loopback frame parsed as Ethernet eats
    // its first fourteen bytes as a link header and reads a nonsense ether
    // type, so a capture-wide link type would show up here as rejections.
    assert_eq!(
        parse.rejected(),
        0,
        "a frame was parsed against a link type that was not its interface's: {parse:?}"
    );
    assert!(
        parse.direction_ambiguous > 0,
        "the loopback frames should parse to flow keys with an undetermined direction"
    );
    assert!(
        run.report.stats.packets_captured > 0,
        "nothing was captured, so nothing was parsed"
    );
}

// SC-006. The identity S08 established, now with two capture threads.
#[test]
fn the_conservation_identity_holds_with_several_capture_threads() {
    let run = run_two(None, false);
    let s = &run.report.stats;
    assert!(
        s.packets_attributed + s.packets_unattributed <= s.packets_captured,
        "more packets were attributed than were captured"
    );
    assert_eq!(
        s.buffer_dropped, 0,
        "the default capacity is far larger than the corpus"
    );
    assert_eq!(s.sink_dropped, 0, "no sink refused anything");
}

// FR-029, and the reason the deviation was taken. Each interface's report is
// readable on its own, and the capture-wide view is their sum.
#[test]
fn each_interface_reports_its_own_backend_counters() {
    let run = run_two(None, false);
    let s = &run.report.stats;
    assert_eq!(s.sources.len(), 2, "one report per interface");
    let summed: u64 = s.sources.iter().map(|(_, st)| st.received).sum();
    assert_eq!(s.source().received, summed, "the total is the sum of parts");
    assert!(s.source_for(InterfaceId::new(0)).is_some());
    assert!(s.source_for(InterfaceId::new(1)).is_some());
}

// T033a. SC-012, FR-027, FR-028. One interface fails; the others finish.
#[test]
fn a_failed_interface_retires_without_ending_the_run() {
    let baseline = run_two(None, false).report.stats.packets_captured;
    let run = run_two(Some(Box::new(LostDevice { yielded: false })), false);

    assert_eq!(
        run.report.stats.packets_captured, baseline,
        "the surviving interfaces must deliver everything they would have alone"
    );

    let lost = run
        .report
        .retirements
        .iter()
        .find(|r| r.interface == InterfaceId::new(2))
        .expect("the failed interface must appear in the report");
    assert_eq!(
        lost.reason,
        RetirementReason::DeviceLost {
            detail: "the adapter was removed".into()
        },
        "the report must name why, not merely that"
    );

    assert_eq!(
        run.report.retirements.len(),
        3,
        "every interface retires, and every retirement is reported"
    );

    // FR-028. Nothing was observed and then discarded, so no drop counter moved.
    assert_eq!(run.report.stats.buffer_dropped, 0);
    assert_eq!(run.report.stats.sink_dropped, 0);
}

// Review of pull request 12, and the reason the address set moved onto the
// binding. Specification section 12.6 matches a packet's source against the
// address set of the *capturing interface*, and one run-wide set cannot say
// that on a multi-homed machine.
//
// Swapping the two sets is what makes the assertion discriminating: a shared
// union is unchanged by swapping its halves, so a pipeline that ignored the
// per-binding set would report the same counters both ways round.
#[test]
fn each_interface_uses_its_own_address_set_and_not_a_shared_union() {
    let correct = run_two(None, false);
    assert_eq!(
        correct.report.stats.parse.rejected(),
        0,
        "with each interface given its own addresses, every frame should parse"
    );

    let swapped = run_two(None, true);
    assert!(
        swapped.report.stats.parse.no_local_endpoint > 0,
        "with the address sets swapped, frames should stop finding a local          endpoint. They did not, which means the per-interface set is not          actually being used: {:?}",
        swapped.report.stats.parse
    );
    assert_eq!(
        swapped.report.stats.packets_captured, correct.report.stats.packets_captured,
        "the same packets are captured either way; only what parses changes"
    );
}

// Review of pull request 12. A capture thread that panics must not leave the
// others running.
//
// Every capture thread holds a clone of the single producer, so resuming the
// unwind immediately would leave a live source pushing into a buffer nobody
// closes: the output thread waits on it forever and the guard's drop joins a
// thread that never returns. A panic would become a hang, which reports
// nothing at all and is strictly worse than a panic that reports a defect.
//
// The assertion that discriminates is the survivor's packet count. Without the
// stop, it runs to its own limit; with it, it is wound down almost immediately.
#[test]
fn a_panicking_capture_thread_stops_the_others_rather_than_hanging() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Yields frames until its own generous limit, so that a missing stop shows
    /// up as a much larger count rather than as a hung test.
    const SURVIVOR_LIMIT: usize = 100_000;

    struct Panicking;
    impl PacketSource for Panicking {
        fn next_packet(&mut self, _t: Duration) -> Result<Option<RawPacket>, SourceError> {
            panic!("a capture thread defect");
        }
        fn set_filter(&mut self, _f: &FilterProgram) -> Result<(), SourceError> {
            Ok(())
        }
        fn stats(&self) -> SourceStats {
            SourceStats::default()
        }
        fn link_type(&self) -> LinkType {
            LinkType::ETHERNET
        }
    }

    struct Survivor {
        produced: Arc<AtomicUsize>,
    }
    impl PacketSource for Survivor {
        fn next_packet(&mut self, _t: Duration) -> Result<Option<RawPacket>, SourceError> {
            if self.produced.fetch_add(1, Ordering::Relaxed) >= SURVIVOR_LIMIT {
                return Err(SourceError::Closed);
            }
            Ok(Some(RawPacket::new(
                fragcap_core::packet::Timestamp::from_nanos(1),
                fragcap_core::packet::Payload::from_static(&[0u8; 4]),
                4,
            )))
        }
        fn set_filter(&mut self, _f: &FilterProgram) -> Result<(), SourceError> {
            Ok(())
        }
        fn stats(&self) -> SourceStats {
            SourceStats::default()
        }
        fn link_type(&self) -> LinkType {
            LinkType::ETHERNET
        }
    }

    let produced = Arc::new(AtomicUsize::new(0));
    let pipeline = Pipeline::new(
        vec![
            SourceBinding::new(
                InterfaceId::new(0),
                Box::new(Survivor {
                    produced: Arc::clone(&produced),
                }),
                InterfaceAddrs::default(),
            ),
            SourceBinding::new(
                InterfaceId::new(1),
                Box::new(Panicking),
                InterfaceAddrs::default(),
            ),
        ],
        Box::new(ScriptedAttributor::new(AttributionScript::default())),
        PipelineConfig::default(),
    )
    .expect("two sources build");

    let outcome = catch_unwind(AssertUnwindSafe(|| pipeline.run()));
    assert!(
        outcome.is_err(),
        "a capture thread defect must reach the caller as a panic, not be filed \
         under an accounting category"
    );

    let count = produced.load(Ordering::Relaxed);
    assert!(
        count < SURVIVOR_LIMIT,
        "the surviving capture thread ran to its own limit ({count}), which \
         means the panic did not wind it down"
    );
}
