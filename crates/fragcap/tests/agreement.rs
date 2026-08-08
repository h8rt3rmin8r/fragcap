// SPDX-License-Identifier: Apache-2.0

//! Do the two output formats say the same thing about the same packet?
//!
//! This is the test that justifies the shape of S06 and S07. Section 13.3's
//! annotation and section 13.5's JSON object both answer which process produced
//! a packet, in which direction, and how the answer was reached. S06 split
//! deriving that answer from rendering it so there would be one derivation; if
//! the split were not real, each writer would be internally consistent and the
//! two would disagree, which is the hardest kind of defect to notice.
//!
//! The goldens catch a format that changed. Only this catches two formats that
//! drifted apart.

mod common;

use std::collections::BTreeMap;

use common::{render, render_jsonl, CORPUS};
use fragcap::PayloadMode;
use serde_json::Value;

/// The attribution facts of one packet, as either format reports them.
///
/// Compared as a map so a missing key on one side is a difference rather than
/// a panic, and so the failure names the key.
type Facts = BTreeMap<String, String>;

/// Pull the annotation comments out of a pcapng capture, in packet order.
fn pcapng_facts(buf: &[u8]) -> Vec<Facts> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 8 <= buf.len() {
        let ty = u32::from_le_bytes(buf[i..i + 4].try_into().unwrap());
        let total = u32::from_le_bytes(buf[i + 4..i + 8].try_into().unwrap()) as usize;
        if ty == 0x0000_0006 {
            let body = &buf[i + 8..i + total - 4];
            let captured = u32::from_le_bytes(body[12..16].try_into().unwrap()) as usize;
            let padded = captured + (4 - captured % 4) % 4;
            let mut o = 20 + padded;
            while o + 4 <= body.len() {
                let code = u16::from_le_bytes(body[o..o + 2].try_into().unwrap());
                let len = u16::from_le_bytes(body[o + 2..o + 4].try_into().unwrap()) as usize;
                if code == 0 {
                    break;
                }
                if code == 1 {
                    let text = std::str::from_utf8(&body[o + 4..o + 4 + len]).unwrap();
                    out.push(parse_annotation(text));
                }
                o += 4 + len + (4 - len % 4) % 4;
            }
        }
        i += total;
    }
    out
}

/// Parse a `fragcap:` annotation into its keys, undoing percent-encoding.
fn parse_annotation(text: &str) -> Facts {
    let body = text.strip_prefix("fragcap:").expect("sentinel");
    let mut facts = Facts::new();
    if body.is_empty() {
        return facts;
    }
    for pair in body.split(';') {
        let (k, v) = pair.split_once('=').expect("key=value");
        let mut decoded = Vec::new();
        let bytes = v.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap();
                decoded.push(u8::from_str_radix(hex, 16).unwrap());
                i += 3;
            } else {
                decoded.push(bytes[i]);
                i += 1;
            }
        }
        facts.insert(k.to_string(), String::from_utf8(decoded).unwrap());
    }
    facts
}

/// Pull the same facts out of a JSON Lines stream, in packet order.
fn jsonl_facts(buf: &[u8]) -> Vec<Facts> {
    let text = std::str::from_utf8(buf).expect("the stream is UTF-8");
    let mut out = Vec::new();
    for line in text.lines() {
        let v: Value = serde_json::from_str(line).expect("every line parses");
        if v.get("type").is_some() {
            continue; // header or trailer
        }
        let mut facts = Facts::new();
        for key in ["pid", "proc", "role", "stage", "dir", "attr"] {
            if let Some(found) = v.get(key) {
                let rendered = match found {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                facts.insert(key.to_string(), rendered);
            }
        }
        out.push(facts);
    }
    out
}

#[test]
fn both_formats_report_the_same_attribution_for_every_packet() {
    for (name, _) in CORPUS {
        let from_pcapng = pcapng_facts(&render(name));
        let from_json = jsonl_facts(&render_jsonl(name, PayloadMode::WithPayload));

        assert_eq!(
            from_pcapng.len(),
            from_json.len(),
            "{name}: the two formats disagree about how many packets there were"
        );
        assert!(
            !from_pcapng.is_empty(),
            "{name}: no packets compared, so this test proved nothing"
        );

        for (i, (p, j)) in from_pcapng.iter().zip(from_json.iter()).enumerate() {
            // The pcapng annotation carries `iface` only in a multi-interface
            // capture and the JSON record always carries it, which is the one
            // documented divergence. It is not an attribution fact, so it is
            // not compared here.
            assert_eq!(
                p, j,
                "{name} packet {i}: the two output formats describe it differently.\n\
                 pcapng: {p:?}\n  json: {j:?}"
            );
        }
    }
}

#[test]
fn the_agreement_covers_every_attribution_state() {
    // A test that compared only attributed packets would pass while the
    // formats disagreed about unattributed ones, which is the case where the
    // presence rules actually differ.
    let mut seen_attributed = false;
    let mut seen_unattributed = false;

    for (name, _) in CORPUS {
        for facts in jsonl_facts(&render_jsonl(name, PayloadMode::WithPayload)) {
            match facts.get("attr").map(String::as_str) {
                Some("none") => {
                    seen_unattributed = true;
                    for k in ["pid", "proc", "role", "stage"] {
                        assert!(
                            !facts.contains_key(k),
                            "{name}: {k} on an unattributed packet"
                        );
                    }
                }
                Some("live") => {
                    seen_attributed = true;
                    assert!(facts.contains_key("pid") && facts.contains_key("proc"));
                }
                other => panic!("{name}: unexpected fidelity {other:?}"),
            }
        }
    }

    assert!(
        seen_attributed,
        "the corpus must exercise attributed packets"
    );
    assert!(
        seen_unattributed,
        "the corpus must exercise unattributed packets, or this proves half of what it claims"
    );
}

#[test]
fn the_two_formats_agree_about_direction_including_the_unknown_case() {
    // `loopback` is the fixture where direction is undetermined, which is the
    // case both formats have a special rule for and could differ on.
    let from_pcapng = pcapng_facts(&render("loopback"));
    let from_json = jsonl_facts(&render_jsonl("loopback", PayloadMode::WithPayload));

    assert!(!from_pcapng.is_empty());
    for (p, j) in from_pcapng.iter().zip(from_json.iter()) {
        assert_eq!(p.get("dir").map(String::as_str), Some("unknown"));
        assert_eq!(j.get("dir").map(String::as_str), Some("unknown"));
    }
}
