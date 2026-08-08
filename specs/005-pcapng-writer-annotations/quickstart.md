# Quickstart: pcapng Writer and Annotation Encoding

**Slice**: S06

**Created**: 2026-08-08

**Feature**: [spec.md](spec.md)

How to run and validate this slice. Everything in the first three sections runs
anywhere with a Rust toolchain, no capture driver, no elevated privilege, and
no game, per specification section 25.1. The tshark section needs Wireshark and
is deliberately not part of any gate.

## Prerequisites

- The pinned toolchain, 1.96.0, installed by `rust-toolchain.toml`.
- The committed fixture corpus at `fixtures/`, from S04. Nothing to fetch.
- Optional, for the manual verification only: Wireshark 4.6.3 or later, for
  `tshark` and `capinfos`.

## Run the gate

The same set the automated checks run, in the same order:

```bash
cargo xtask ci
```

Two further checks that need a toolchain or target the runner may lack, and
which exit 2 rather than 0 when they cannot run:

```bash
cargo xtask neutral && cargo xtask msrv
```

## Run this slice's tests

The annotation grammar, including the round trip that proves the encoder and
decoder agree:

```bash
cargo test -p fragcap-sink annotation
```

The structural validation, which walks written output by its declared block
lengths and never calls the writer's encoding functions:

```bash
cargo test -p fragcap-sink --test structure
```

The golden comparison across the whole S04 corpus:

```bash
cargo test -p fragcap-sink --test goldens
```

Expected: every fixture in `fixtures/` produces a file byte-identical to its
committed golden under `fixtures/goldens/`. A failure names the fixture and the
offset of the first differing byte.

## Regenerate the goldens

Only when the output format changed on purpose. This rewrites committed files,
so read the diff before committing it; that reading is the review the goldens
depend on.

```bash
FRAGCAP_UPDATE_GOLDENS=1 cargo test -p fragcap-sink --test goldens
```

Then confirm the drift check passes on the regenerated set:

```bash
cargo test -p fragcap-sink --test goldens
```

A golden that changed without an intended format change is a defect, not a
stale file. Regenerating to make a red test green is the failure mode this
mechanism exists to prevent.

## Verify against an analyzer that never heard of fragcap

This is the actual claim of specification section 13.1 and constitution P-5,
checked against the population it is about. It is a manual step: the continuous
integration runners are not guaranteed to have Wireshark, and a check that did
not run must never look like one that passed.

Produce a file from a fixture, then read it with unmodified tooling:

```bash
capinfos fixtures/goldens/tcp-session.fcapng
```

Expected, among the output:

```text
File type:           Wireshark/... - pcapng
File encapsulation:  Ethernet
File timestamp precision:  microseconds (6)
Capture application: fragcap/0.1.0
Capture comment:     fragcap:profile=0.1.0
```

Read the per-packet attribution, which is the property that has to hold with no
plugin and no configuration:

```bash
tshark -r fixtures/goldens/tcp-session.fcapng -T fields -e frame.comment
```

Expected: one line per packet, each beginning `fragcap:` and carrying at least
a `dir` and an `attr` key, for example:

```text
fragcap:pid=7412;proc=eso64.exe;dir=out;attr=live
```

Confirm the interface declaration and statistics were understood:

```bash
capinfos fixtures/goldens/tcp-session.fcapng | grep -A8 "Interface #0"
```

Expected: the declared name, `Encapsulation = Ethernet`, `Time precision =
microseconds (6)`, and a non-zero `Number of stat entries`.

## What good looks like

| Check | Passing means |
| --- | --- |
| `cargo xtask ci` | Format, lints, tests, conventions, dependency direction, licenses |
| `--test structure` | The file satisfies pcapng's own structural rules |
| `--test goldens` | Output is byte-stable, and no golden drifted |
| `capinfos` | An unmodified reader accepts the file and names fragcap |
| `tshark -e frame.comment` | Attribution is visible with no plugin |

## What is not covered here

- **Live capture.** Interfaces are declared by the caller in this slice. S09.
- **The pipeline.** Nothing here buffers, counts, or fans out. S08 drives this
  writer.
- **JSON Lines output.** S07, which reuses the annotation derivation this slice
  exposes.
- **Reading pcapng as a capability.** The structural validator is a test, not a
  feature. fragcap writes pcapng and reads classic pcap.
