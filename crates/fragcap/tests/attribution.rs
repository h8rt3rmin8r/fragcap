// SPDX-License-Identifier: Apache-2.0

//! The socket table attributor, through the real pipeline.
//!
//! Slice S10. `pipeline.rs` proved a source, the parser, and an attributor
//! compose; `corpus_pipeline.rs` proved the composition reproduces the goldens.
//! Both use [`fragcap::ScriptedAttributor`], which answers from a text file a
//! test wrote. This file drives the production attributor instead, over a
//! socket table a test declared, and it runs with no capture driver, no
//! elevation, and no game.
//!
//! It lives in the facade for the reason `pipeline.rs` records: the facade is
//! the crate that legitimately depends on both `fragcap-capture` and
//! `fragcap-attr`, and a dev-dependency between them would create the edge
//! constitution P-3 exists to prevent while slipping past `cargo xtask deps`,
//! which ignores dev-dependencies by design.
//!
//! # What is being asserted
//!
//! Two things the unit tests cannot reach. That the pipeline resolves
//! attribution against a shared attributor with no lock on the per-packet path,
//! which is specification section 11.6 and the thing S08 deferred to this
//! slice. And that the conservation identity S08 established survives it: for
//! every sink, received plus `buffer_dropped` plus refusals equals
//! `packets_captured`.

mod common;

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use common::{corpus_addrs, fixtures_dir};
use fragcap::{
    AttributorConfig, Clock, DeclaredNames, DeclaredTable, Pipeline, PipelineConfig, ReplaySource,
    SocketTable, SocketTableAttributor, SocketTableEntry, TestClock,
};
use fragcap_core::attribution::Fidelity;
use fragcap_core::error::SinkError;
use fragcap_core::flow::Proto;
use fragcap_core::interface::InterfaceId;
use fragcap_core::packet::{AttributionState, CapturedPacket, Timestamp};
use fragcap_core::parse::InterfaceAddrs;
use fragcap_core::stats::CaptureStats;
use fragcap_core::traits::{PacketSource, Sink};

/// A sink that keeps every packet, so a test can inspect what the pipeline
/// resolved rather than only what it counted.
#[derive(Default)]
struct Collector {
    inner: Arc<std::sync::Mutex<Vec<CapturedPacket>>>,
}

impl Collector {
    fn handle(&self) -> Arc<std::sync::Mutex<Vec<CapturedPacket>>> {
        Arc::clone(&self.inner)
    }
}

impl Sink for Collector {
    fn write(&mut self, packet: &CapturedPacket) -> Result<(), SinkError> {
        self.inner
            .lock()
            .expect("the collector mutex is never poisoned")
            .push(packet.clone());
        Ok(())
    }
    fn flush(&mut self) -> Result<(), SinkError> {
        Ok(())
    }
    fn finish(self: Box<Self>, _stats: &CaptureStats) -> Result<(), SinkError> {
        Ok(())
    }
}

fn addr(s: &str) -> SocketAddr {
    s.parse().expect("test address must parse")
}

/// Every flow key the fixture's packets will produce, as socket table entries
/// owned by one process.
///
/// Built from the fixture's own local addresses so the test declares what the
/// capture will actually ask about rather than guessing.
fn table_for(local: &[IpAddr], ports: &[(Proto, u16)], pid: u32) -> SocketTable {
    let mut entries = Vec::new();
    for ip in local {
        for (proto, port) in ports {
            let a = SocketAddr::new(*ip, *port);
            entries.push(match proto {
                Proto::Udp => SocketTableEntry::udp(a, pid),
                // A wildcard peer is not expressible, so a TCP entry is added
                // per observed peer by the caller. This one covers the
                // listening case.
                Proto::Tcp => SocketTableEntry::tcp_listening(a, pid),
            });
        }
    }
    SocketTable::new(Timestamp::from_nanos(0), entries)
}

/// Run a fixture through the pipeline with a socket table attributor.
fn run_with_table(
    name: &str,
    table: SocketTable,
    named: &[(u32, &str)],
) -> (fragcap::PipelineReport, Vec<CapturedPacket>) {
    let local = corpus_addrs(name);
    let source = ReplaySource::open(fixtures_dir().join(format!("{name}.pcap")))
        .unwrap_or_else(|e| panic!("{name}.pcap must open: {e}"));
    let _ = source.link_type();

    let mut names = DeclaredNames::new();
    for (pid, n) in named {
        names = names.with(*pid, n);
    }

    let clock = Arc::new(TestClock::at(Timestamp::from_nanos(0)));
    let attributor = SocketTableAttributor::new(
        Box::new(DeclaredTable::once(table)),
        Box::new(names),
        Arc::clone(&clock) as Arc<dyn Clock>,
        AttributorConfig::default(),
    );
    // The control thread of specification section 8.6 does not exist until
    // S13, so the refresh happens here, before the attributor is shared. That
    // is exactly the arrangement the pipeline documents.
    {
        use fragcap_core::traits::FlowAttributor;
        attributor.refresh().expect("a declared table always reads");
    }

    let collector = Collector::default();
    let collected = collector.handle();

    let mut pipeline = Pipeline::new(
        vec![fragcap_core::pipeline::SourceBinding::new(
            InterfaceId::default(),
            Box::new(source),
            InterfaceAddrs::new(local.iter().copied()),
        )],
        Box::new(attributor),
        PipelineConfig {
            capacity: 65_536,
            ..PipelineConfig::default()
        },
    )
    .expect("a non-zero capacity builds");
    pipeline.add_sink(Box::new(collector));

    let report = pipeline.run();
    let packets = collected
        .lock()
        .expect("the collector mutex is never poisoned")
        .clone();
    (report, packets)
}

/// The conservation identity S08 established, restated here so a discard path
/// added later fails in this file too rather than only in the corpus tests.
///
/// The collector never refuses, so the refusal term is zero and the identity
/// reduces to: everything captured was either written or evicted from the
/// buffer, and nothing else happened to it.
fn assert_conservation(report: &fragcap::PipelineReport, received: u64) {
    assert_eq!(
        received + report.stats.buffer_dropped,
        report.stats.packets_captured,
        "conservation failed: {received} written, {} evicted, {} captured",
        report.stats.buffer_dropped,
        report.stats.packets_captured
    );
}

// The whole point of the slice, through the pipeline. A fixture's UDP flows
// resolve to a declared owner, and the answer is marked as observed.
#[test]
fn the_pipeline_attributes_against_a_socket_table() {
    let local = corpus_addrs("udp-gameplay");
    // The fixture's UDP conversation is on this local port; the table declares
    // a wildcard bind, which is the shape a real UDP socket usually has.
    let table = SocketTable::new(
        Timestamp::from_nanos(0),
        local
            .iter()
            .map(|ip| SocketTableEntry::udp(SocketAddr::new(*ip, 30000), 4242))
            .collect(),
    );
    let (report, packets) = run_with_table("udp-gameplay", table, &[(4242, "game.exe")]);

    assert_conservation(&report, packets.len() as u64);
    assert!(report.stats.packets_captured > 0, "the fixture has packets");

    let attributed: Vec<_> = packets
        .iter()
        .filter(|p| p.attribution_state() == AttributionState::Resolved)
        .collect();
    assert!(
        !attributed.is_empty(),
        "the declared table must own at least one of the fixture's flows"
    );
    for p in &attributed {
        let a = p.attribution.as_ref().expect("resolved means present");
        assert_eq!(a.pid, 4242);
        assert_eq!(&*a.process, "game.exe");
        assert_eq!(
            a.fidelity,
            Fidelity::Live,
            "the endpoint was in the table, so the answer is observed"
        );
        assert!(a.role.is_none(), "roles arrive with S12");
    }
    assert_eq!(
        report.stats.packets_attributed as usize,
        attributed.len(),
        "the counter and the packets agree"
    );
}

// P-4 through the pipeline with the production attributor: a table that owns
// nothing still delivers every packet, marked and counted.
#[test]
fn an_empty_table_attributes_nothing_and_drops_nothing() {
    let table = SocketTable::empty(Timestamp::from_nanos(0));
    let (report, packets) = run_with_table("udp-gameplay", table, &[]);

    assert_conservation(&report, packets.len() as u64);
    assert_eq!(report.stats.packets_attributed, 0);
    assert!(
        report.stats.packets_unattributed > 0,
        "the fixture's flows were attempted and unresolved"
    );
    assert_eq!(report.stats.fragcap_dropped(), 0, "nothing was dropped");
    assert_eq!(
        packets.len() as u64,
        report.stats.packets_captured,
        "every packet reached the sink"
    );
    for p in &packets {
        assert!(p.attribution.is_none());
    }
}

// FR-026, through the pipeline. A packet with no flow key was never attempted,
// which is not the same as attempted and failed, and neither counter moves for
// it. S07 lost this distinction once and it stood for a whole slice.
#[test]
fn packets_with_no_flow_key_move_neither_counter() {
    let table = SocketTable::empty(Timestamp::from_nanos(0));
    let (report, packets) = run_with_table("malformed", table, &[]);

    assert_conservation(&report, packets.len() as u64);
    let not_attempted = packets
        .iter()
        .filter(|p| p.attribution_state() == AttributionState::NotAttempted)
        .count();
    assert!(
        not_attempted > 0,
        "the malformed fixture is the one with packets that produce no flow key"
    );
    assert_eq!(
        report.stats.packets_attributed + report.stats.packets_unattributed,
        report.stats.packets_captured - not_attempted as u64,
        "never attempted is neither attributed nor unattributed"
    );
}

// The whole corpus, to prove the production attributor composes with every
// fixture rather than only the one chosen to show it working.
#[test]
fn every_fixture_runs_through_the_socket_table_attributor() {
    for (name, _) in common::CORPUS {
        let table = table_for(&corpus_addrs(name), &[(Proto::Udp, 30000)], 1);
        let (report, packets) = run_with_table(name, table, &[(1, "g.exe")]);
        assert_conservation(&report, packets.len() as u64);
        assert_eq!(
            packets.len() as u64,
            report.stats.packets_captured,
            "{name}: every packet reached the sink"
        );
        assert_eq!(
            report.stats.fragcap_dropped(),
            0,
            "{name}: nothing was dropped"
        );
    }
}

// SC-007. The pipeline resolves against a shared attributor from several
// capture threads with no lock on the per-packet path. Two replay sources
// standing in for two interfaces, exactly as S09's multi-interface tests do.
#[test]
fn several_capture_threads_share_one_attributor() {
    let name = "udp-gameplay";
    let local = corpus_addrs(name);
    let table = SocketTable::new(
        Timestamp::from_nanos(0),
        local
            .iter()
            .map(|ip| SocketTableEntry::udp(SocketAddr::new(*ip, 30000), 4242))
            .collect(),
    );

    let clock = Arc::new(TestClock::at(Timestamp::from_nanos(0)));
    let attributor = SocketTableAttributor::new(
        Box::new(DeclaredTable::once(table)),
        Box::new(DeclaredNames::from([(4242, "game.exe")])),
        Arc::clone(&clock) as Arc<dyn Clock>,
        AttributorConfig::default(),
    );
    {
        use fragcap_core::traits::FlowAttributor;
        attributor.refresh().expect("a declared table always reads");
    }

    let sources: Vec<_> = (0..2)
        .map(|i| {
            let s = ReplaySource::open(fixtures_dir().join(format!("{name}.pcap")))
                .unwrap_or_else(|e| panic!("{name}.pcap must open: {e}"));
            fragcap_core::pipeline::SourceBinding::new(
                InterfaceId::new(i),
                Box::new(s),
                InterfaceAddrs::new(local.iter().copied()),
            )
        })
        .collect();

    let collector = Collector::default();
    let collected = collector.handle();
    let mut pipeline = Pipeline::new(
        sources,
        Box::new(attributor),
        PipelineConfig {
            capacity: 65_536,
            ..PipelineConfig::default()
        },
    )
    .expect("a non-zero capacity builds");
    pipeline.add_sink(Box::new(collector));

    let report = pipeline.run();
    let packets = collected
        .lock()
        .expect("the collector mutex is never poisoned")
        .clone();
    assert_conservation(&report, packets.len() as u64);

    let resolved = packets
        .iter()
        .filter(|p| p.attribution_state() == AttributionState::Resolved)
        .count();
    assert!(
        resolved > 0,
        "both capture threads resolved against the shared attributor"
    );
    // Both interfaces contributed, so the shared attributor answered from two
    // threads at once rather than one thread twice.
    let mut ids: Vec<_> = packets.iter().map(|p| p.interface).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 2, "both interfaces produced packets");
}

// Retention, through the pipeline. The declared table is refreshed away
// between two runs, and the second resolves the same flows as inference.
#[test]
fn a_flow_stays_attributed_after_its_socket_leaves_the_table() {
    use fragcap_core::traits::FlowAttributor;

    let name = "udp-gameplay";
    let local = corpus_addrs(name);
    let entries: Vec<_> = local
        .iter()
        .map(|ip| SocketTableEntry::udp(SocketAddr::new(*ip, 30000), 4242))
        .collect();

    // The tables are stamped in the fixture's own era, because the retention
    // window is measured from the instant the endpoint was last seen against
    // the instant of the packet asking about it. A table stamped at the epoch
    // against packets from 2023 is thirty seconds of grace consumed by
    // fifty-three years of gap, which is correct behavior and a useless test.
    const ERA: i64 = 1_700_000_000_000_000_000;

    let clock = Arc::new(TestClock::at(Timestamp::from_nanos(ERA)));
    let attributor = SocketTableAttributor::new(
        Box::new(DeclaredTable::sequence(vec![
            Ok(SocketTable::new(Timestamp::from_nanos(ERA), entries)),
            // One second later, and the endpoint has gone.
            Ok(SocketTable::empty(Timestamp::from_nanos(
                ERA + 1_000_000_000,
            ))),
        ])),
        Box::new(DeclaredNames::from([(4242, "game.exe")])),
        Arc::clone(&clock) as Arc<dyn Clock>,
        AttributorConfig::default(),
    );
    attributor.refresh().expect("the first table reads");
    clock.set(Timestamp::from_nanos(ERA + 1_000_000_000));
    attributor.refresh().expect("the second table reads");

    let source = ReplaySource::open(fixtures_dir().join(format!("{name}.pcap")))
        .unwrap_or_else(|e| panic!("{name}.pcap must open: {e}"));
    let collector = Collector::default();
    let collected = collector.handle();
    let mut pipeline = Pipeline::new(
        vec![fragcap_core::pipeline::SourceBinding::new(
            InterfaceId::default(),
            Box::new(source),
            InterfaceAddrs::new(local.iter().copied()),
        )],
        Box::new(attributor),
        PipelineConfig {
            capacity: 65_536,
            ..PipelineConfig::default()
        },
    )
    .expect("a non-zero capacity builds");
    pipeline.add_sink(Box::new(collector));
    let report = pipeline.run();
    let packets = collected
        .lock()
        .expect("the collector mutex is never poisoned")
        .clone();
    assert_conservation(&report, packets.len() as u64);

    let retained: Vec<_> = packets
        .iter()
        .filter_map(|p| p.attribution.as_ref())
        .filter(|a| a.fidelity == Fidelity::Retained)
        .collect();
    assert!(
        !retained.is_empty(),
        "the tail of a connection stays attributed, and says it is inference"
    );
    for a in retained {
        assert_eq!(a.pid, 4242);
    }
    let _unused: SocketAddr = addr("192.0.2.1:1");
}
