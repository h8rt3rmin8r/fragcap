// SPDX-License-Identifier: Apache-2.0

//! File rotation (specification 14.2, user story US2): numbered segments at
//! clean section boundaries whose union reproduces the capture.

mod common;

use common::{assert_valid_pcapng_stream, epb_payloads, expected_payloads, packets, walk};

use fragcap_core::stats::CaptureStats;
use fragcap_core::traits::Sink;
use fragcap_core::LinkType;
use fragcap_sink::{Format, InterfaceSpec, RotatingFileSink, RotationPolicy, SinkFactory};

const SNAP: u32 = 262_144;

fn factory() -> SinkFactory {
    SinkFactory::new(
        Format::Pcapng,
        vec![InterfaceSpec::new("eth0", LinkType::ETHERNET, SNAP)],
    )
}

/// Read the numbered `.fcapng` segments under `dir`, in capture order.
fn segments(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut segs: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().map(|e| e == "fcapng").unwrap_or(false))
        .collect();
    segs.sort();
    segs
}

#[test]
fn size_rotation_produces_independent_segments_reproducing_the_capture() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("session.fcapng");

    // A small size threshold forces several segments over 20 packets.
    let mut sink =
        RotatingFileSink::create(&base, RotationPolicy::Size(300), factory()).expect("create");
    let pkts = packets(20, 48);
    for p in &pkts {
        sink.write(p).expect("write");
    }
    Box::new(sink)
        .finish(&CaptureStats::default())
        .expect("finish");

    let segs = segments(dir.path());
    assert!(
        segs.len() >= 2,
        "expected multiple segments, got {}",
        segs.len()
    );

    // Every segment opens on its own, and their union is the capture in order.
    let mut all = Vec::new();
    for seg in &segs {
        let bytes = std::fs::read(seg).unwrap();
        assert_valid_pcapng_stream(&bytes, 1);
        all.extend(epb_payloads(&walk(&bytes)));
    }
    assert_eq!(
        all,
        expected_payloads(&pkts),
        "segments must reproduce the capture with no loss, duplication, or reorder"
    );
}

#[test]
fn no_policy_writes_a_single_segment_at_the_base_path() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("single.fcapng");

    let mut sink =
        RotatingFileSink::create(&base, RotationPolicy::None, factory()).expect("create");
    let pkts = packets(5, 32);
    for p in &pkts {
        sink.write(p).expect("write");
    }
    Box::new(sink)
        .finish(&CaptureStats::default())
        .expect("finish");

    // The base path exists unchanged; no numbered segment was produced.
    assert!(base.exists(), "the base file must exist");
    let bytes = std::fs::read(&base).unwrap();
    assert_valid_pcapng_stream(&bytes, 1);
    assert_eq!(epb_payloads(&walk(&bytes)), expected_payloads(&pkts));
}

#[test]
fn a_threshold_smaller_than_the_header_still_yields_valid_segments() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("tiny.fcapng");

    // A one-byte threshold is smaller than a segment's mandatory header. Each
    // segment must still hold exactly one packet and open on its own; the sink
    // must not spin out empty, unreadable segments.
    let mut sink =
        RotatingFileSink::create(&base, RotationPolicy::Size(1), factory()).expect("create");
    let pkts = packets(4, 16);
    for p in &pkts {
        sink.write(p).expect("write");
    }
    Box::new(sink)
        .finish(&CaptureStats::default())
        .expect("finish");

    let segs = segments(dir.path());
    assert_eq!(
        segs.len(),
        pkts.len(),
        "one packet per segment at the tiniest threshold"
    );
    let mut all = Vec::new();
    for seg in &segs {
        let bytes = std::fs::read(seg).unwrap();
        assert_valid_pcapng_stream(&bytes, 1);
        let payloads = epb_payloads(&walk(&bytes));
        assert_eq!(payloads.len(), 1, "each segment holds exactly one packet");
        all.extend(payloads);
    }
    assert_eq!(all, expected_payloads(&pkts));
}
