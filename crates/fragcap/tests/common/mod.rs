// SPDX-License-Identifier: Apache-2.0

//! Turning a fixture into a written capture.
//!
//! Shared by the structural validation and golden comparison targets, which
//! need the same input and ask different questions of it.
//!
//! This lives in the facade because producing it needs a replay source from
//! `fragcap-capture`, a scripted attributor from `fragcap-attr`, and the writer
//! from `fragcap-sink`, and the facade is the only crate that legitimately
//! depends on all three. Reaching them from `fragcap-sink/tests/` would mean a
//! dev-dependency on a sibling, which is the edge constitution P-3 prevents and
//! which `cargo xtask deps` does not catch, because it ignores
//! dev-dependencies by design. See slice S06 plan decision D-7.

#![allow(dead_code)]

use fragcap_core::interface::InterfaceId;
use fragcap_core::pipeline::SourceBinding;
use std::io::Write;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fragcap::{
    AttributionScript, AttributionState, CaptureStats, CapturedPacket, FlowAttributor,
    FlowRegistry, HeaderParser, InterfaceAddrs, InterfaceDeclaration, JsonLinesWriter, LinkType,
    PacketSource, PayloadMode, PcapngWriter, Pipeline, PipelineConfig, PipelineReport,
    ReplaySource, Sink, SourceError, SourceStats, WriteGate,
};

/// Every fixture in the corpus, with the local addresses its script implies.
///
/// The addresses are not decoration: direction is determined by matching a
/// packet's source against the capturing interface's address set, so a wrong
/// entry here silently inverts `dir` on every packet of that fixture.
pub const CORPUS: &[(&str, &[&str])] = &[
    ("burst", &["192.0.2.10"]),
    ("fragmented", &["192.0.2.10"]),
    ("ipv6-mixed", &["2001:db8::10"]),
    ("loopback", &["127.0.0.1"]),
    ("malformed", &["192.0.2.10", "2001:db8::10"]),
    ("port-reuse", &["192.0.2.10"]),
    ("tcp-session", &["192.0.2.10"]),
    ("udp-gameplay", &["192.0.2.10"]),
];

pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
}

pub fn goldens_dir() -> PathBuf {
    fixtures_dir().join("goldens")
}

/// Read a fixture, parse and attribute every packet, and write the result as
/// pcapng.
///
/// Deterministic by construction: the only inputs are the fixture bytes, the
/// script, and constants. Nothing here reads a clock or the environment, which
/// is what lets the output be compared against a committed golden.
pub fn render(name: &str) -> Vec<u8> {
    let local: Vec<IpAddr> = CORPUS
        .iter()
        .find(|(n, _)| *n == name)
        .unwrap_or_else(|| panic!("{name} is not in the corpus table"))
        .1
        .iter()
        .map(|s| s.parse().expect("a corpus address must parse"))
        .collect();

    let dir = fixtures_dir();
    let mut source = ReplaySource::open(dir.join(format!("{name}.pcap")))
        .unwrap_or_else(|e| panic!("{name}.pcap must open: {e}"));
    let script = AttributionScript::load(dir.join(format!("{name}.script")))
        .unwrap_or_else(|e| panic!("{name}.script must load: {e}"));
    let attributor: Box<dyn FlowAttributor> = Box::new(fragcap::ScriptedAttributor::new(script));
    let link = source.link_type();
    let mut parser = HeaderParser::new(InterfaceAddrs::new(local.iter().copied()));
    let flow_registry = FlowRegistry::default();

    let mut buf = Vec::new();
    let mut writer = PcapngWriter::new(&mut buf).expect("in-memory write cannot fail");
    writer
        .declare_interface(&InterfaceDeclaration::new(link, snap_len(link), name))
        .expect("declaring an interface cannot fail in memory");

    let mut received = 0u64;
    loop {
        let raw = match source.next_packet(Duration::from_millis(0)) {
            Ok(Some(raw)) => raw,
            Ok(None) => panic!("a replay source never times out"),
            Err(SourceError::Closed) => break,
            Err(e) => panic!("{name}: unexpected source failure: {e}"),
        };
        received += 1;
        let mut packet = CapturedPacket::from_raw(raw, InterfaceId::default());
        parser.apply(link, &mut packet);
        if let Some(key) = packet.flow {
            packet.attribution = attributor.resolve(&key, packet.ts);
            packet.flow_id = Some(flow_registry.observe(key, packet.attribution.as_ref()));
        }
        writer.write(&packet).expect("every packet is written");
    }

    assert!(
        source.replay_stats().read_whole_file(),
        "{name} did not read whole: {}",
        source.replay_stats()
    );

    // A fixed statistics snapshot derived from the fixture, so the trailing
    // block is stable across runs. Non-zero fragcap counters on purpose: they
    // are the values with no standard pcapng field, and a golden of all zeroes
    // would not prove they were written.
    let stats = CaptureStats {
        packets_captured: received,
        sources: vec![(
            InterfaceId::default(),
            SourceStats {
                received,
                ..Default::default()
            },
        )],
        ..Default::default()
    };
    Box::new(writer)
        .finish(&stats)
        .expect("finishing cannot fail in memory");
    buf
}

/// Read a fixture and write the result as JSON Lines.
///
/// Deliberately a separate pass over the same inputs rather than a tee off the
/// pcapng one. The two formats must agree because they read the same derivation,
/// not because they share a loop; sharing the loop would hide a divergence that
/// a real deployment with two sinks would hit.
pub fn render_jsonl(name: &str, mode: PayloadMode) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut writer =
            JsonLinesWriter::new(&mut buf, &[name], mode).expect("in-memory write cannot fail");
        let (packets, stats) = replay(name);
        for p in &packets {
            writer.write(p).expect("every packet is written");
        }
        Box::new(writer)
            .finish(&stats)
            .expect("finishing cannot fail in memory");
    }
    buf
}

/// Read a fixture, parse every packet, and resolve every flow.
///
/// The shared front half of both renders.
pub fn replay(name: &str) -> (Vec<CapturedPacket>, CaptureStats) {
    let local: Vec<IpAddr> = CORPUS
        .iter()
        .find(|(n, _)| *n == name)
        .unwrap_or_else(|| panic!("{name} is not in the corpus table"))
        .1
        .iter()
        .map(|s| s.parse().expect("a corpus address must parse"))
        .collect();

    let dir = fixtures_dir();
    let mut source = ReplaySource::open(dir.join(format!("{name}.pcap")))
        .unwrap_or_else(|e| panic!("{name}.pcap must open: {e}"));
    let script = AttributionScript::load(dir.join(format!("{name}.script")))
        .unwrap_or_else(|e| panic!("{name}.script must load: {e}"));
    let attributor: Box<dyn FlowAttributor> = Box::new(fragcap::ScriptedAttributor::new(script));
    let link = source.link_type();
    let mut parser = HeaderParser::new(InterfaceAddrs::new(local.iter().copied()));
    let flow_registry = FlowRegistry::default();

    let mut packets = Vec::new();
    let mut attributed = 0u64;
    let mut unattributed = 0u64;
    loop {
        let raw = match source.next_packet(Duration::from_millis(0)) {
            Ok(Some(raw)) => raw,
            Ok(None) => panic!("a replay source never times out"),
            Err(SourceError::Closed) => break,
            Err(e) => panic!("{name}: unexpected source failure: {e}"),
        };
        let mut packet = CapturedPacket::from_raw(raw, InterfaceId::default());
        parser.apply(link, &mut packet);
        if let Some(key) = packet.flow {
            packet.attribution = attributor.resolve(&key, packet.ts);
            packet.flow_id = Some(flow_registry.observe(key, packet.attribution.as_ref()));
        }
        // Three states, not two. A packet that produced no flow key was never
        // attempted, which `stats.rs` distinguishes from attempted and
        // unresolved because the remedies differ and because P-4's counter is
        // defined as "retained and marked because attribution did not
        // resolve". S07 wrote `attribution.is_some()` here and folded the
        // never-attempted packets into `unattributed`, which put a wrong count
        // into the `malformed` golden. Slice S08 found it by driving the same
        // writers from the pipeline, which counts the three states apart.
        match packet.attribution_state() {
            AttributionState::Resolved => attributed += 1,
            AttributionState::Unresolved => unattributed += 1,
            AttributionState::NotAttempted => {}
        }
        packets.push(packet);
    }

    assert!(
        source.replay_stats().read_whole_file(),
        "{name} did not read whole: {}",
        source.replay_stats()
    );

    let received = packets.len() as u64;
    let stats = CaptureStats {
        packets_captured: received,
        packets_attributed: attributed,
        packets_unattributed: unattributed,
        sources: vec![(
            InterfaceId::default(),
            SourceStats {
                received,
                ..Default::default()
            },
        )],
        ..Default::default()
    };
    (packets, stats)
}

/// A byte sink a writer can own while a test still reads it afterwards.
///
/// The pipeline holds its sinks as `Box<dyn Sink>`, which is `'static`, so the
/// borrowed `&mut Vec<u8>` the hand-written renders use is not expressible
/// there. Sharing the buffer is the smallest change that keeps the test able to
/// compare the bytes.
#[derive(Clone, Default)]
pub struct SharedBuf(Arc<Mutex<Vec<u8>>>);

impl SharedBuf {
    pub fn contents(&self) -> Vec<u8> {
        self.0
            .lock()
            .expect("the buffer mutex is never poisoned")
            .clone()
    }
}

impl Write for SharedBuf {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .expect("the buffer mutex is never poisoned")
            .extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// The local addresses a fixture's script implies.
pub fn corpus_addrs(name: &str) -> Vec<IpAddr> {
    CORPUS
        .iter()
        .find(|(n, _)| *n == name)
        .unwrap_or_else(|| panic!("{name} is not in the corpus table"))
        .1
        .iter()
        .map(|s| s.parse().expect("a corpus address must parse"))
        .collect()
}

/// What one run of the real pipeline over a fixture produced.
pub struct PipelineRun {
    pub pcapng: Vec<u8>,
    pub jsonl: Vec<u8>,
    pub report: PipelineReport,
}

/// Run a fixture through the real pipeline with both writers attached.
///
/// Slice S08. Everything here is a seam: a `ReplaySource`, a
/// `ScriptedAttributor`, and two `Sink` values, composed by a `Pipeline` that
/// names none of their concrete types. One pass over the source produces both
/// outputs, which is the fan-out of specification section 8.6.
pub fn render_via_pipeline(name: &str, capacity: usize) -> PipelineRun {
    let dir = fixtures_dir();
    let script = AttributionScript::load(dir.join(format!("{name}.script")))
        .unwrap_or_else(|e| panic!("{name}.script must load: {e}"));
    render_via_pipeline_with(
        name,
        capacity,
        Box::new(fragcap::ScriptedAttributor::new(script)),
        None,
    )
}

/// Run a fixture through the real pipeline with both writers attached, a
/// caller-supplied attributor, and an optional write gate.
///
/// The variant `render_via_pipeline` delegates to with a bare scripted attributor
/// and no gate. A caller that needs a write gate (to exercise the gate-admitted
/// holder tally of slice S059) passes one.
pub fn render_via_pipeline_with(
    name: &str,
    capacity: usize,
    attributor: Box<dyn FlowAttributor>,
    gate: Option<Arc<dyn WriteGate>>,
) -> PipelineRun {
    let local = corpus_addrs(name);
    let dir = fixtures_dir();
    let source = ReplaySource::open(dir.join(format!("{name}.pcap")))
        .unwrap_or_else(|e| panic!("{name}.pcap must open: {e}"));
    let link = source.link_type();

    let pcapng_buf = SharedBuf::default();
    let jsonl_buf = SharedBuf::default();

    let mut pcapng = PcapngWriter::new(pcapng_buf.clone()).expect("in-memory write cannot fail");
    pcapng
        .declare_interface(&InterfaceDeclaration::new(link, snap_len(link), name))
        .expect("declaring an interface cannot fail in memory");
    let jsonl = JsonLinesWriter::new(jsonl_buf.clone(), &[name], PayloadMode::WithPayload)
        .expect("in-memory write cannot fail");

    let mut pipeline = Pipeline::new(
        vec![SourceBinding::new(
            InterfaceId::default(),
            Box::new(source),
            InterfaceAddrs::new(local.iter().copied()),
        )],
        attributor,
        PipelineConfig {
            capacity,
            ..PipelineConfig::default()
        },
    )
    .expect("a non-zero capacity builds");
    if let Some(gate) = gate {
        pipeline.set_write_gate(gate);
    }
    pipeline.add_sink(Box::new(pcapng));
    pipeline.add_sink(Box::new(jsonl));

    let report = pipeline.run();
    PipelineRun {
        pcapng: pcapng_buf.contents(),
        jsonl: jsonl_buf.contents(),
        report,
    }
}

/// The snap length declared for a fixture's interface.
///
/// A constant rather than the largest packet observed, because deriving it from
/// the data would make the declared value depend on which packets happened to
/// be in the file.
fn snap_len(_link: LinkType) -> u32 {
    65_535
}
