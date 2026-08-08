# Quickstart: validating S03

**Slice**: S03

**Created**: 2026-08-08

How to check that this slice does what it claims. Every command runs in the
foreground and is watched to completion, per the constitution's verification
discipline.

## Prerequisites

The pinned toolchain from `rust-toolchain.toml`. Nothing else. This slice adds
no dependency, needs no capture driver, no elevation, and no game running,
which is the property specification section 25 exists to preserve and which
this slice must not be the first to break.

## The gate set

```bash
cargo xtask ci
```

Runs, in order: `cargo fmt --all -- --check`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo test --workspace --locked`, `cargo xtask
lint`, `cargo xtask deps`, and `cargo xtask license`.

Two further checks are not in `ci` because they need a target or a toolchain the
runner may not have. Both are relevant here and both exit 2 rather than 0 when
they cannot run:

```bash
cargo xtask neutral
```

```bash
cargo xtask msrv
```

`neutral` is the P-2 proof and matters more in this slice than in any so far,
because this is the first slice with real logic that could have reached for a
platform facility. `msrv` matters because the slice adds no dependency, so a
change in its result would mean the code itself used something newer than 1.82.

## What each success criterion is verified by

| Criterion | Verified by |
| --- | --- |
| SC-001 combination coverage | `parse::tests`, one test per row of the contract table's supported cases |
| SC-002 every rejection reachable | `parse::tests`, one test per rejection variant, each asserting exactly one counter moved |
| SC-003 zero allocations | `tests/no_alloc.rs`, counting allocator over the full corpus |
| SC-004 loopback distinguishable | `parse::direction::tests` |
| SC-005 one key per conversation | `parse::tests`, both halves inserted into a map, one entry asserted |
| SC-006 fragments | `parse::fragment::tests` plus the matched and orphaned cases in `parse::tests` |
| SC-007 adversarial chain terminates | `parse::ip::tests`, nine-header chain |
| SC-008 nothing dropped | `stats::tests`, every parse counter advanced, both drop totals asserted zero |
| SC-009 core still portable | `cargo xtask neutral` and `cargo xtask deps` |
| SC-010 glossary | `docs/glossary.md` read against the change's new public items |
| SC-011 gate set | `cargo xtask ci` |

## Running just this slice's tests

```bash
cargo test -p fragcap-core parse
```

```bash
cargo test -p fragcap-core --test no_alloc
```

The allocation test is a separate binary because it installs a global
allocator, which is per binary. Running it inside the unit test build would
measure the test harness as well as the parser.

## Reading the counters by hand

Nothing in this slice runs a capture, so the counters are only observable from
a test. The shape a later slice will surface is:

```rust
let mut parser = HeaderParser::new(InterfaceAddrs::new([local_ip]));
let outcome = parser.parse(LinkType::ETHERNET, frame);
let stats = parser.stats();
```

`stats.rejected()` is the sum of the twelve rejection counters.
`stats.direction_ambiguous` and `stats.fragment_evicted` are separate and are
not rejections. If `stats.rejected()` is high on a capture that should have been
attributable, the individual counters say which of twelve remedies applies,
which is the whole reason there are twelve.

## What this slice does not let you do yet

Run a capture. Read a fixture. Write a file. Attribute a flow to a process.
S04 adds the replay source that turns this parser into something you can point
at a recorded capture, and S08 adds the pipeline that runs it. Until then the
tests are the only caller, and that is the expected state.
