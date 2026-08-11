// SPDX-License-Identifier: Apache-2.0

//! Ring mode (specification section 7.2, FR-8, user story US1): a rolling
//! in-memory window dumped to a valid pcapng file on finish. The window keeps the
//! recent tail; a window larger than the input keeps everything and is
//! byte-comparable to a plain file capture.

mod common;

use common::{assert_valid_pcapng_stream, epb_payloads, expected_payloads, packets, walk};

use fragcap_core::stats::CaptureStats;
use fragcap_core::traits::Sink;
use fragcap_core::LinkType;
use fragcap_sink::{
    Format, InterfaceSpec, RingSink, RingWindow, RotatingFileSink, RotationPolicy, SinkFactory,
};

const SNAP: u32 = 262_144;

fn factory() -> SinkFactory {
    SinkFactory::new(
        Format::Pcapng,
        vec![InterfaceSpec::new("eth0", LinkType::ETHERNET, SNAP)],
    )
}

// SC-001. A size window smaller than the input retains exactly the newest packets
// fitting the window; the dump is valid pcapng containing that tail in capture
// order, with everything older evicted.
#[test]
fn a_size_window_dumps_only_the_recent_tail() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("ring.fcapng");

    // 20 packets of 48 bytes each; a 200-byte window holds the newest four
    // (4 * 48 = 192 <= 200; a fifth would be 240 > 200).
    let pkts = packets(20, 48);
    let mut sink = RingSink::create(out.clone(), RingWindow::Size(200), factory()).expect("create");
    for p in &pkts {
        sink.write(p).expect("write");
    }
    let retained = sink.retained();
    let evicted = sink.evicted();
    assert_eq!(retained, 4, "the window holds the newest four packets");
    assert_eq!(
        evicted + retained as u64,
        pkts.len() as u64,
        "local conservation: evicted + retained == accepted"
    );

    Box::new(sink)
        .finish(&CaptureStats::default())
        .expect("finish");

    let bytes = std::fs::read(&out).unwrap();
    assert_valid_pcapng_stream(&bytes, 1);
    let dumped = epb_payloads(&walk(&bytes));
    let expected = expected_payloads(&pkts);
    assert_eq!(
        dumped,
        expected[expected.len() - retained..].to_vec(),
        "the dump is exactly the retained tail, in capture order"
    );
}

// SC-005. The sink-local conservation identity holds even when every packet but
// the newest is evicted (a window smaller than one packet).
#[test]
fn eviction_conserves_and_never_empties_a_seen_capture() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("tiny.fcapng");

    let pkts = packets(8, 64);
    let mut sink = RingSink::create(out.clone(), RingWindow::Size(1), factory()).expect("create");
    for p in &pkts {
        sink.write(p).expect("write");
    }
    assert_eq!(sink.retained(), 1, "a seen capture never dumps empty");
    assert_eq!(sink.evicted(), 7);

    Box::new(sink)
        .finish(&CaptureStats::default())
        .expect("finish");
    let bytes = std::fs::read(&out).unwrap();
    assert_valid_pcapng_stream(&bytes, 1);
    let dumped = epb_payloads(&walk(&bytes));
    assert_eq!(dumped.len(), 1, "the newest packet is dumped");
    assert_eq!(dumped[0], expected_payloads(&pkts).last().unwrap().clone());
}

// Review of PR #30 (Codex P1). A ring dump target that cannot be created fails at
// construction, before any capture runs, rather than at finish after the whole
// in-memory window has been captured and would then be lost. This matches how the
// ordinary file sink validates its destination during assembly.
#[test]
fn an_unwritable_dump_path_fails_at_creation_not_at_finish() {
    let dir = tempfile::tempdir().unwrap();
    // A path under a directory that does not exist cannot be created.
    let bad = dir.path().join("no-such-dir").join("ring.fcapng");
    let result = RingSink::create(bad, RingWindow::Size(1024), factory());
    assert!(
        result.is_err(),
        "an unwritable dump path is rejected before capture starts"
    );
}

// SC-002, FR-012. A window larger than the whole input evicts nothing, and the
// dumped file is byte-identical to a plain single-segment file capture of the
// same input: same packets, none lost, reordered, or duplicated.
#[test]
fn a_whole_input_window_equals_a_plain_file_capture() {
    let dir = tempfile::tempdir().unwrap();
    let ring_out = dir.path().join("ring.fcapng");
    let file_out = dir.path().join("file.fcapng");

    let pkts = packets(12, 100);

    // A ring with a window far larger than the whole input.
    let mut ring =
        RingSink::create(ring_out.clone(), RingWindow::Size(1_000_000), factory()).expect("create");
    for p in &pkts {
        ring.write(p).expect("write");
    }
    assert_eq!(ring.evicted(), 0, "nothing is evicted");
    assert_eq!(ring.retained(), pkts.len());
    Box::new(ring)
        .finish(&CaptureStats::default())
        .expect("finish");

    // The same packets through a plain (no-rotation) file sink.
    let mut file =
        RotatingFileSink::create(&file_out, RotationPolicy::None, factory()).expect("create");
    for p in &pkts {
        file.write(p).expect("write");
    }
    Box::new(file)
        .finish(&CaptureStats::default())
        .expect("finish");

    let ring_bytes = std::fs::read(&ring_out).unwrap();
    let file_bytes = std::fs::read(&file_out).unwrap();
    assert_eq!(
        ring_bytes, file_bytes,
        "a whole-input ring dump is byte-identical to a plain file capture"
    );
}

// FR-002 (duration). A duration window dumps exactly the packets whose instant is
// within the window measured back from the newest. `packets` stamps instant
// 1000 + i, so a 3 ns window over 10 packets keeps the newest four: eviction is
// `newest - instant > 3`, so instants below 1006 go and 1006..=1009 stay (the
// boundary at exactly the window edge is retained).
#[test]
fn a_duration_window_dumps_the_recent_tail_by_instant() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("ring.fcapng");

    let pkts = packets(10, 40);
    let mut sink = RingSink::create(
        out.clone(),
        RingWindow::Duration(std::time::Duration::from_nanos(3)),
        factory(),
    )
    .expect("create");
    for p in &pkts {
        sink.write(p).expect("write");
    }
    // newest instant 1009; keep instants >= 1006 -> 1006, 1007, 1008, 1009 (four).
    let retained = sink.retained();
    assert_eq!(
        retained, 4,
        "keep instants within the window, boundary inclusive"
    );

    Box::new(sink)
        .finish(&CaptureStats::default())
        .expect("finish");
    let bytes = std::fs::read(&out).unwrap();
    assert_valid_pcapng_stream(&bytes, 1);
    let dumped = epb_payloads(&walk(&bytes));
    let expected = expected_payloads(&pkts);
    assert_eq!(dumped, expected[expected.len() - retained..].to_vec());
}
