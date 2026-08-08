// SPDX-License-Identifier: Apache-2.0

//! Byte stability, checked against committed goldens.
//!
//! Everything else in this slice is asserted by tests written by whoever wrote
//! the code. The goldens are different: they are bytes a human read once and a
//! machine compares afterward, so a change in output is visible to somebody who
//! was not present when it happened.
//!
//! Regenerate with `FRAGCAP_UPDATE_GOLDENS=1 cargo test -p fragcap --test
//! goldens`, then read the diff. Regenerating to turn a red test green destroys
//! the only evidence that the format moved, which is the failure mode this
//! exists to prevent. See `fixtures/goldens/README.md`.

mod common;

use std::fs;

use common::{goldens_dir, render, render_jsonl, CORPUS};
use fragcap::PayloadMode;

/// Report the first byte at which two files disagree.
///
/// A diff that says only "the files differ" makes a format regression a
/// bisecting exercise. The offset plus its surroundings usually names the field
/// outright.
fn first_difference(want: &[u8], got: &[u8]) -> Option<String> {
    let common_len = want.len().min(got.len());
    for i in 0..common_len {
        if want[i] != got[i] {
            let from = i.saturating_sub(8);
            let to = (i + 8).min(common_len);
            return Some(format!(
                "first difference at byte {i}: golden has {:#04x}, writer produced {:#04x}\n  \
                 golden [{from}..{to}]: {:02x?}\n  writer [{from}..{to}]: {:02x?}",
                want[i],
                got[i],
                &want[from..to],
                &got[from..to]
            ));
        }
    }
    if want.len() != got.len() {
        return Some(format!(
            "identical for {common_len} bytes, then lengths differ: \
             golden is {} bytes, writer produced {}",
            want.len(),
            got.len()
        ));
    }
    None
}

fn updating() -> bool {
    std::env::var_os("FRAGCAP_UPDATE_GOLDENS").is_some()
}

#[test]
fn writing_the_same_fixture_twice_produces_the_same_bytes() {
    // The property the goldens rest on. If this fails, the goldens are
    // unmaintainable regardless of whether they currently match.
    //
    // SC-007 also claims byte-identical output across architectures. That half
    // cannot be asserted from one machine: it rests on the writer emitting
    // little-endian unconditionally rather than in host order, which is a
    // property of the code and is tested where the encoding happens. This test
    // covers the run-to-run half only, and says so rather than implying more.
    for (name, _) in CORPUS {
        assert_eq!(
            render(name),
            render(name),
            "{name} is not deterministic across runs"
        );
    }
}

#[test]
fn every_fixture_matches_its_committed_golden() {
    let dir = goldens_dir();
    let mut regenerated = Vec::new();

    for (name, _) in CORPUS {
        let produced = render(name);
        let path = dir.join(format!("{name}.fcapng"));

        if updating() {
            fs::create_dir_all(&dir).expect("goldens directory must be creatable");
            fs::write(&path, &produced)
                .unwrap_or_else(|e| panic!("could not write {}: {e}", path.display()));
            regenerated.push(*name);
            continue;
        }

        let golden = fs::read(&path).unwrap_or_else(|e| {
            panic!(
                "{}: {e}\nRegenerate with \
                 FRAGCAP_UPDATE_GOLDENS=1 cargo test -p fragcap --test goldens",
                path.display()
            )
        });

        if let Some(detail) = first_difference(&golden, &produced) {
            panic!(
                "{name} no longer matches its golden.\n{detail}\n\n\
                 If the output format changed on purpose, regenerate with \
                 FRAGCAP_UPDATE_GOLDENS=1 and read the diff before committing it. \
                 If it did not, this is the regression the goldens exist to catch."
            );
        }
    }

    if !regenerated.is_empty() {
        eprintln!(
            "regenerated {} goldens: {}",
            regenerated.len(),
            regenerated.join(", ")
        );
    }
}

#[test]
fn every_fixture_matches_its_committed_jsonl_golden() {
    let dir = goldens_dir();
    for (name, _) in CORPUS {
        let produced = render_jsonl(name, PayloadMode::WithPayload);
        let path = dir.join(format!("{name}.jsonl"));

        if updating() {
            fs::create_dir_all(&dir).expect("goldens directory must be creatable");
            fs::write(&path, &produced)
                .unwrap_or_else(|e| panic!("could not write {}: {e}", path.display()));
            continue;
        }

        let golden = fs::read(&path).unwrap_or_else(|e| {
            panic!(
                "{}: {e}
Regenerate with                  FRAGCAP_UPDATE_GOLDENS=1 cargo test -p fragcap --test goldens",
                path.display()
            )
        });

        // Reported by line rather than by byte offset. A JSON stream is
        // line-oriented, so the line number is what a reader can act on.
        let want = String::from_utf8(golden).expect("a golden is UTF-8");
        let got = String::from_utf8(produced).expect("output is UTF-8");
        for (i, (w, g)) in want.lines().zip(got.lines()).enumerate() {
            assert_eq!(
                w,
                g,
                "{name} diverged at line {}.
 golden: {w}
 writer: {g}

                 If the format changed on purpose, regenerate and read the diff.",
                i + 1
            );
        }
        assert_eq!(
            want.lines().count(),
            got.lines().count(),
            "{name}: line counts differ"
        );
    }
}

#[test]
fn writing_the_same_fixture_twice_produces_the_same_jsonl() {
    for (name, _) in CORPUS {
        assert_eq!(
            render_jsonl(name, PayloadMode::WithPayload),
            render_jsonl(name, PayloadMode::WithPayload),
            "{name} is not deterministic across runs"
        );
    }
}

#[test]
fn every_line_of_every_jsonl_golden_parses() {
    // The external oracle over the committed bytes rather than over fresh
    // output, so a golden that was hand-edited into invalidity is caught.
    if updating() {
        return;
    }
    for (name, _) in CORPUS {
        let path = goldens_dir().join(format!("{name}.jsonl"));
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert!(!text.is_empty(), "{name}: empty golden");
        for (i, line) in text.lines().enumerate() {
            let v: serde_json::Value =
                serde_json::from_str(line).unwrap_or_else(|e| panic!("{name} line {}: {e}", i + 1));
            assert!(v.is_object(), "{name} line {}: not an object", i + 1);
        }
    }
}

#[test]
fn a_golden_exists_for_every_fixture_in_the_corpus() {
    // Which fixtures have goldens should never be a question a contributor has
    // to answer, and a missing golden is otherwise invisible: the comparison
    // above would simply not run for it.
    if updating() {
        return;
    }
    let dir = goldens_dir();
    for (name, _) in CORPUS {
        for ext in ["fcapng", "jsonl"] {
            let path = dir.join(format!("{name}.{ext}"));
            assert!(
                path.exists(),
                "{} is missing. Every corpus fixture has a golden in both formats.",
                path.display()
            );
        }
    }
}

#[test]
fn the_annotation_survives_a_round_trip_through_the_written_file() {
    // The goldens prove the bytes are stable. This proves they still mean what
    // section 13.3 says, by decoding what was written rather than comparing it
    // to what the encoder produced.
    use fragcap::Annotation;

    let buf = render("tcp-session");
    let mut comments = Vec::new();
    let mut i = 0;
    while i < buf.len() {
        let block_type = u32::from_le_bytes(buf[i..i + 4].try_into().unwrap());
        let total = u32::from_le_bytes(buf[i + 4..i + 8].try_into().unwrap()) as usize;
        if block_type == 0x0000_0006 {
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
                    comments.push(String::from_utf8(body[o + 4..o + 4 + len].to_vec()).unwrap());
                }
                o += 4 + len + (4 - len % 4) % 4;
            }
        }
        i += total;
    }

    assert_eq!(comments.len(), 6, "one annotation per packet");
    for c in &comments {
        let a = Annotation::decode(c)
            .unwrap_or_else(|e| panic!("a written annotation must decode: {c}: {e}"));
        assert_eq!(a.pid, Some(4242));
        assert_eq!(a.process.as_deref(), Some("game.exe"));
        assert_eq!(a.encode(), *c, "decoding and re-encoding is the identity");
    }
}
