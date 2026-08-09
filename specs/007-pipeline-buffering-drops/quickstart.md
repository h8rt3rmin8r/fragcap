# Quickstart: Validating S08

**Slice**: S08 | **Date**: 2026-08-08 | **Plan**: [plan.md](plan.md)

How to convince yourself the pipeline is correct, in the order that finds
problems soonest. Every command runs in the foreground and is watched to
completion; none of it needs a capture driver, elevated privilege, or a game.

## Prerequisites

The repository as checked out. No npcap, no network access, no elevation.

## The one command

```bash
cargo xtask ci
```

This is the whole gate: formatting, Clippy with warnings denied, the workspace
test suite, the conventions linter, the dependency direction check, and the
license check. If it passes, the slice's mechanical obligations are met.

Two checks are outside it because they need a toolchain or a target the runner
may not have. Both exit 2 rather than 0 when they cannot run, so a check that
did not run cannot look like one that passed.

```bash
cargo xtask neutral
```

```bash
cargo xtask msrv
```

`neutral` is the one that matters most for this slice. It is the mechanical
proof that adding threads to `fragcap-core` did not add a platform dependency,
which is constitution P-2.

## The three layers, individually

Run these when something in `ci` fails and you want to know where.

**Layer 1, the buffer alone.** Single-threaded, no source, no sink. Eviction,
ordering, the count, and the terminal item's exemption from the capacity bound.

```bash
cargo test -p fragcap-core pipeline::buffer
```

**Layer 2, the pipeline against stubs.** In-crate stubs that can be told to
refuse a specific packet, fail non-countably at a specific packet, or block
until the test releases them. This is where every drop path and every end
reason is forced.

```bash
cargo test -p fragcap-core pipeline::
```

**Layer 3, the corpus end to end.** The real replay source, the real parser,
the real scripted attributor, and both real writers, over all eight fixtures,
compared against the committed goldens.

```bash
cargo test -p fragcap --test corpus_pipeline
```

## What each layer would catch

| Symptom | Layer that finds it | What it means |
| --- | --- | --- |
| Golden mismatch, one byte | 3 | Driving the writers from the pipeline changed something the hand-written loop was doing. Read the diff before changing the golden. |
| Golden mismatch, whole file | 3 | Ordering, or a packet lost or duplicated crossing the buffer. |
| Conservation identity fails | 2 | A discard path with no counter, which is the defect P-4 names. |
| A drop test hangs | 2 | The producer is waiting on the consumer, which is the one thing section 12.4 forbids. |
| Eviction count off by one | 1 | The terminal item is being counted, or eviction is happening outside the lock that counts it. |
| `cargo xtask neutral` fails | outside | Something platform-specific reached `fragcap-core`. |
| `cargo xtask deps` fails | in `ci` | A dependency edge went the wrong way. Check whether a test was put in a sibling crate rather than the facade. |

## Confirming the accounting by hand

The assertion that matters is not that a counter is non-zero but that nothing
escaped the accounting. In every pipeline test, for every sink:

```text
received + stats.buffer_dropped + refusals == stats.packets_captured
```

If a future change adds a way for a packet to end up somewhere else, this fails
rather than the change passing quietly. That is the property the slice is for,
and it is worth preserving in any test added later.

## Confirming no dependency was added

```bash
git diff main -- Cargo.toml Cargo.lock crates/*/Cargo.toml
```

The expected result for this slice is no change to any manifest. The pipeline
uses the standard library only. A diff here is a decision, and it belongs in
the changelog's decisions fragment before it belongs in the code.

## Regenerating fixtures

Not needed for this slice, and doing it would be a warning sign: S08 changes no
fixture and no golden. If a golden appears to need regenerating, the pipeline
is behaving differently from the loop the goldens were produced by, and the
difference is the finding.

The command exists for completeness and is documented in
[../../fixtures/README.md](../../fixtures/README.md).
