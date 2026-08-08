# Quickstart: validating S04

**Slice**: S04

**Created**: 2026-08-08

How to check that this slice does what it claims, and how to work with the
corpus it adds.

## Prerequisites

The pinned toolchain from `rust-toolchain.toml`. Nothing else.

That is the point of the slice. From here on, every pipeline test runs with no
capture driver, no elevated privilege, and no game, which is the claim
specification section 25.1 makes and the return on the trait separation the
project has been paying for since S01.

## The gate set

```bash
cargo xtask ci
```

Format, lint, the whole test suite, repository conventions, dependency
direction, and per-crate licensing. The corpus drift check runs inside the test
step, so a fixture that no longer matches its generator fails here rather than
somewhere later.

The two checks outside `ci` are unaffected by this slice but are worth running,
because it is the first to add file I/O anywhere:

```bash
cargo xtask neutral
```

```bash
cargo xtask msrv
```

## Working with the corpus

The fixtures under `fixtures/` are committed, and they are generated rather
than hand-made. The generator is the readable record of what each one contains;
the `.pcap` is just its output.

To change a fixture, edit the generator in
`crates/fragcap-capture/tests/corpus.rs`, then regenerate:

```bash
FRAGCAP_UPDATE_FIXTURES=1 cargo test -p fragcap-capture --test corpus
```

Then review the resulting diff. A regenerated fixture whose diff nobody read is
the same defect as a golden file updated without looking, which specification
section 25.4 calls out for the goldens and which applies here for the same
reason.

Without the environment variable, that same test checks instead of writing:

```bash
cargo test -p fragcap-capture --test corpus
```

It fails, naming the file, if a committed fixture or script differs from what
the generator produces, if a fixture has no script or a script no fixture, if
anything exceeds its size ceiling, or if a fixture stops exercising the
condition section 25.3 states for it.

## What each success criterion is verified by

| Criterion | Verified by |
| --- | --- |
| SC-001 the pipeline runs with no driver | `tests/corpus.rs`, the end-to-end test reading, parsing, and resolving |
| SC-002 reading twice is identical | `tests/corpus.rs`, over every fixture |
| SC-003 byte order and resolution | `pcap::tests`, the same capture generated four ways |
| SC-004 every skip cause reachable | `pcap::tests`, one per counter, asserting only that one moved |
| SC-005 nothing dropped for being unusual | `pcap::tests`, zero-length, out-of-order, unknown link type |
| SC-006 scripted port reuse | `scripted::tests` |
| SC-006a wildcard bind agreement | `scripted::tests` |
| SC-006b the seam is unwidened | `scripted::tests`, a trait object built from the unchanged trait |
| SC-007 eight pairs | `tests/corpus.rs`, checked both directions |
| SC-008 each condition asserted | `tests/corpus.rs`, one assertion per fixture |
| SC-009 drift detection | `tests/corpus.rs`, plus altering a fixture by hand |
| SC-010 size ceilings | `tests/corpus.rs` |
| SC-011 addresses and payloads | `tests/corpus.rs`, over every packet of every fixture |
| SC-012 core still portable | `cargo xtask neutral` and `cargo xtask deps` |
| SC-013 glossary | `docs/glossary.md` read against the new public items |
| SC-014 gate set | `cargo xtask ci` |

## Reading a fixture by hand

The corpus is classic pcap, so ordinary tooling opens it:

```bash
tshark -r fixtures/tcp-session.pcap
```

That is a convenience rather than part of the gate. Nothing in the project
depends on a packet analyzer being installed, and the condition assertions in
the corpus test are what actually hold the corpus to its description.

## Using the substrate in a later slice

```rust
let mut source = ReplaySource::open("fixtures/tcp-session.pcap")?;
let mut attributor = ScriptedAttributor::new(
    AttributionScript::load("fixtures/tcp-session.script")?,
);
let mut parser = HeaderParser::new(InterfaceAddrs::new([local]));

while let Ok(Some(raw)) = source.next_packet(Duration::from_millis(0)) {
    let mut packet = CapturedPacket::from_raw(raw);
    parser.apply(source.link_type(), &mut packet);
    attributor.set_now(packet.ts);
    if let Some(key) = packet.flow.as_ref() {
        packet.attribution = attributor.resolve(key);
    }
}
```

The loop ends on `Err(Closed)`, which is what exhaustion returns. A loop written
against `Ok(None)` would never end, which is why the source does not report
exhaustion that way.

## What this slice does not let you do yet

Write a capture file, buffer, count drops, or fan out to sinks. S06 adds the
pcapng writer, S07 the JSON Lines writer, and S08 the pipeline that runs the
whole thing over this corpus and compares against goldens. Until then the tests
in this slice are the only caller, and that is the expected state.
