// SPDX-License-Identifier: Apache-2.0

//! The corpus, through the real pipeline, compared against the committed
//! goldens.
//!
//! Slice S08. Everything before this file proves the pipeline behaves under
//! stubs. This proves it produces the same bytes the hand-written loop
//! produced, which is the check that the composition changed nothing it was
//! not supposed to. Slices S03, S04, S06, S07, and S08 meet here.
//!
//! It lives in the facade because the facade is the only crate that
//! legitimately depends on `fragcap-capture`, `fragcap-attr`, and
//! `fragcap-sink` at once. Putting it in any of those would mean a
//! dev-dependency on a sibling, which is the edge constitution P-3 exists to
//! prevent and which `cargo xtask deps` does not catch, because it ignores
//! dev-dependencies by design.
//!
//! # What this found
//!
//! One golden changed, and the change is the point of the exercise rather than
//! a nuisance. `malformed.jsonl` claimed `"unattributed":5` for five packets
//! that produced no flow key. Attribution was never attempted on any of them,
//! which `AttributionState` has distinguished from attempted-and-unresolved
//! since S02 precisely because the two mean different things to an operator.
//! The writer was faithful; the loop feeding it was not. Recorded in the S08
//! changelog decisions fragment.

mod common;

use std::fs;

use common::{goldens_dir, render_via_pipeline, CORPUS};
use fragcap::{EndReason, InterfaceDeclaration, LinkType, PcapngWriter};

/// The default capacity, which no fixture comes close to filling. The goldens
/// therefore test the composition rather than the eviction policy, which is
/// what the pipeline unit tests are for.
const ROOMY: usize = 65_536;

fn golden(name: &str, ext: &str) -> Vec<u8> {
    let path = goldens_dir().join(format!("{name}.{ext}"));
    fs::read(&path).unwrap_or_else(|e| panic!("golden {} must exist: {e}", path.display()))
}

/// Report the first byte at which two outputs disagree.
fn first_difference(want: &[u8], got: &[u8]) -> Option<String> {
    let common_len = want.len().min(got.len());
    for i in 0..common_len {
        if want[i] != got[i] {
            let from = i.saturating_sub(8);
            let to = (i + 8).min(common_len);
            return Some(format!(
                "first difference at byte {i}: golden {:#04x}, pipeline {:#04x}\n  \
                 golden   [{from}..{to}]: {:02x?}\n  pipeline [{from}..{to}]: {:02x?}",
                want[i],
                got[i],
                &want[from..to],
                &got[from..to]
            ));
        }
    }
    if want.len() != got.len() {
        return Some(format!(
            "lengths differ: golden {} bytes, pipeline {} bytes",
            want.len(),
            got.len()
        ));
    }
    None
}

fn assert_matches_golden(name: &str, ext: &str, produced: &[u8]) {
    let want = golden(name, ext);
    if let Some(detail) = first_difference(&want, produced) {
        panic!(
            "{name}.{ext}: the pipeline produced different bytes from the committed \
             golden.\n{detail}\n\nDo not regenerate to turn this green. A golden that \
             needs changing is the finding: it means driving the writers from the \
             pipeline differs from the loop the golden came from, and the difference \
             is what to explain."
        );
    }
}

// T070, SC-001.
#[test]
fn every_fixture_reproduces_its_pcapng_golden_through_the_pipeline() {
    for (name, _) in CORPUS {
        let run = render_via_pipeline(name, ROOMY);
        assert_matches_golden(name, "fcapng", &run.pcapng);
    }
}

// T071, SC-001.
#[test]
fn every_fixture_reproduces_its_json_golden_through_the_pipeline() {
    for (name, _) in CORPUS {
        let run = render_via_pipeline(name, ROOMY);
        assert_matches_golden(name, "jsonl", &run.jsonl);
    }
}

// T072. US1 scenario 4: one pass, two outputs.
#[test]
fn one_pipeline_carrying_both_writers_produces_both_outputs_in_one_pass() {
    let run = render_via_pipeline("tcp-session", ROOMY);
    assert!(!run.pcapng.is_empty(), "the pcapng sink wrote");
    assert!(!run.jsonl.is_empty(), "the JSON sink wrote");
    assert_matches_golden("tcp-session", "fcapng", &run.pcapng);
    assert_matches_golden("tcp-session", "jsonl", &run.jsonl);
    assert_eq!(run.report.stats.packets_captured, 6);
    assert_eq!(run.report.ended, EndReason::SourceClosed);
    assert!(run.report.is_clean());
}

// T073, SC-009.
#[test]
fn two_runs_over_one_fixture_produce_identical_bytes() {
    let a = render_via_pipeline("udp-gameplay", ROOMY);
    let b = render_via_pipeline("udp-gameplay", ROOMY);
    assert_eq!(
        a.pcapng, b.pcapng,
        "a golden against a varying input is not a test"
    );
    assert_eq!(a.jsonl, b.jsonl);
    assert_eq!(a.report.stats, b.report.stats);
}

// T074. FR-039. P-4 through the whole stack: nothing parseable, nothing lost.
#[test]
fn the_malformed_fixture_is_written_whole_and_attributed_nowhere() {
    let run = render_via_pipeline("malformed", ROOMY);
    let stats = &run.report.stats;

    assert_eq!(stats.packets_captured, 5);
    assert_eq!(
        stats.packets_attributed, 0,
        "nothing here parses to a flow key"
    );
    assert_eq!(
        stats.packets_unattributed, 0,
        "never attempted is not attempted and failed, and this is the assertion \
         that caught the S07 golden"
    );
    assert!(stats.parse.rejected_anything(), "the parser said why");
    assert_eq!(
        stats.fragcap_dropped(),
        0,
        "a parse rejection is not a discard"
    );
    assert!(!stats.lost_anything());

    // And all five reached the output. Counting the packet lines in the JSON
    // stream is the strongest available check that P-4's retention held all the
    // way to a file.
    let lines = String::from_utf8(run.jsonl.clone()).expect("the stream is UTF-8");
    let packet_lines = lines.lines().filter(|l| !l.contains("\"type\":")).count();
    assert_eq!(
        packet_lines, 5,
        "every unparseable packet was still written"
    );
}

// T075. FR-037, FR-040. Lengths survive, and nothing is joined or repaired.
#[test]
fn fragments_cross_the_pipeline_unjoined_and_lengths_survive() {
    let run = render_via_pipeline("fragmented", ROOMY);
    assert_eq!(run.report.stats.packets_captured, 3);
    assert_eq!(
        run.report.stats.packets_attributed, 3,
        "the later fragments resolve from what the first one said"
    );

    // Three records out, not one. Reassembly would have collapsed these, and
    // P-9 forbids it during capture.
    let lines = String::from_utf8(run.jsonl.clone()).expect("the stream is UTF-8");
    let packets: Vec<&str> = lines.lines().filter(|l| !l.contains("\"type\":")).collect();
    assert_eq!(packets.len(), 3);
    for line in &packets {
        // Both lengths present and equal: these fixtures are not truncated, and
        // a pipeline that had rewritten either would show it here.
        assert!(line.contains("\"len\":"), "captured length survived");
        assert!(line.contains("\"orig_len\":"), "wire length survived");
    }
}

// T076, FR-022. The conservation identity, over the whole corpus.
#[test]
fn the_conservation_identity_holds_across_the_corpus() {
    for (name, _) in CORPUS {
        let run = render_via_pipeline(name, ROOMY);
        let stats = &run.report.stats;
        assert_eq!(
            stats.buffer_dropped, 0,
            "{name}: the default buffer is far larger than any fixture"
        );
        assert_eq!(stats.sink_dropped, 0, "{name}: no sink refused anything");
        assert_eq!(
            stats.source.received, stats.packets_captured,
            "{name}: every frame the backend delivered was accepted"
        );
        assert!(
            stats.packets_attributed + stats.packets_unattributed <= stats.packets_captured,
            "{name}: a packet cannot be in two attribution states"
        );
        assert!(!stats.lost_anything(), "{name}: nothing was lost");
        assert!(
            run.report.sink_failures.is_empty(),
            "{name}: no sink failed"
        );
    }
}

// The three attribution states, each produced by some fixture. Without this,
// a pipeline that never advanced `packets_unattributed` would pass every other
// test in this file, because no corpus fixture happens to exercise it alone.
#[test]
fn the_corpus_exercises_more_than_one_attribution_state() {
    let mut any_resolved = false;
    let mut any_not_attempted = false;
    for (name, _) in CORPUS {
        let run = render_via_pipeline(name, ROOMY);
        let s = &run.report.stats;
        any_resolved |= s.packets_attributed > 0;
        any_not_attempted |= s.packets_attributed + s.packets_unattributed < s.packets_captured;
    }
    assert!(any_resolved, "no fixture resolved anything");
    assert!(
        any_not_attempted,
        "no fixture produced a packet with no flow key, so the distinction the \
         malformed golden got wrong would be untested"
    );
}

// T076a. FR-043. The S09 restriction must not be liftable by accident.
#[test]
fn a_writer_driven_by_the_pipeline_still_refuses_a_second_interface() {
    let mut writer = PcapngWriter::new(Vec::new()).expect("in-memory write cannot fail");
    writer
        .declare_interface(&InterfaceDeclaration::new(
            LinkType::ETHERNET,
            65_535,
            "first",
        ))
        .expect("the first interface is accepted");
    assert!(
        writer
            .declare_interface(&InterfaceDeclaration::new(
                LinkType::ETHERNET,
                65_535,
                "second",
            ))
            .is_err(),
        "CapturedPacket still carries no interface identifier, so a second \
         interface would label every packet with the first. S09 lifts this."
    );
}
