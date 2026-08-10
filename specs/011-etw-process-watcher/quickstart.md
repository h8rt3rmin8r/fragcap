# Quickstart: Validating S11

**Slice**: S11 | **Date**: 2026-08-09 |
**Spec**: [spec.md](spec.md)

How to check that this slice does what it says, in the order that gets a
disagreement soonest. Everything in the first two sections runs on any machine
with a Rust toolchain. Nothing before the tier 2 section needs Windows,
elevation, or a game.

## Tier 1: any machine, no elevation

The whole of specification section 10.2 is here, which is the point of putting
the tree in `fragcap-core`.

```bash
cargo test -p fragcap-core process
```

That covers ancestry, retention, identifier recycling, the resolution rule for
an unknown start time, out-of-order delivery, and the reconciliation of a
process reported by both the snapshot and the event stream.

The two launcher chains reconnaissance actually observed are replayed through
the scripted watcher:

```bash
cargo test -p fragcap-attr chains
```

The Division 2 chain is the one to read first. Three of its seven processes
share the image name `TheDivision2.exe` and only the last holds sockets, so the
test asserts three distinct nodes and that ancestry tells them apart. If that
test passes and section 10.3's `descends_from` still cannot distinguish them in
S12, the fault is in S12 rather than here.

## The whole gate

```bash
cargo xtask ci
```

Runs on a machine with no elevation, no capture driver, and no game, because the
ETW watcher is behind the `etw` feature and that feature is off. If this needs
elevation to pass, the feature gating is wrong.

## Platform neutrality

```bash
cargo xtask neutral
```

Extended by this slice to build `fragcap-attr` as well as `fragcap-core` and
`fragcap-capture`. It exits 2 rather than 0 when the neutral target is not
installed, so a check that did not run does not look like one that passed.

What it proves: `fragcap-core` carries the process tree and still has no
platform dependency, and `fragcap-attr` builds with its backend absent rather
than stubbed.

## The P-1 check, which is mechanical

```bash
cargo xtask lint
```

Fails if any fragcap source names a process access right that carries memory
rights. This is the same shape as the transmit-call check S09 added, and it
exists for the same reason: the argument that fragcap never reaches inside a
process should be checkable rather than remembered.

To see it work, add `PROCESS_VM_READ` to any source file and run it again. It
should fail. Remove it.

## Tier 2: Windows, elevated

These do not run in the ordinary check set and do not run in continuous
integration today, because no runner is elevated.

```bash
cargo test -p fragcap-attr --features etw -- --ignored
```

From an elevated terminal. The test spawns a short-lived child of its own and
asserts that the start event names the test process as its parent, carries the
child's image path and command line, and is followed by an exit event.

The child lives for a few milliseconds deliberately. Any implementation that
polled would miss it, which is the property section 10.1 is built on and the one
worth having a test fail over.

## Without elevation, on Windows

```bash
cargo test -p fragcap-attr --features etw not_elevated
```

From an ordinary terminal. Asserts that starting the watcher fails with
`WatcherError::NotElevated` rather than with a generic failure, and that no
polling path exists to fall back to.

The second half of that is checked by reading the code, not by the test. There
is no timer, no interval, and no loop over an enumeration anywhere in
`fragcap-attr::etw`.

## What a reviewer should look at first

1. **`ProcessTree::resolve`.** Every ancestry claim the project will ever make
   goes through it. The case worth staring at is two nodes sharing a `pid`,
   because getting it wrong produces a tree that is plausible and wrong rather
   than one that fails.
2. **The `FILETIME` conversion.** A magic number here yields timestamps that
   look reasonable and are decades off, and no synthetic test catches it. The
   epoch offset should be a named constant with the two epochs spelled out.
3. **`CommandLine`, and every place it is matched.** If any call site turns
   `Unavailable` into an empty string, FR-036 is broken and P-9 with it.
4. **The `unsafe` blocks in `fragcap-attr::etw`.** There should be few, each
   with the invariant it relies on stated, and every call checked against its
   documented failure return rather than assumed to succeed.
5. **`ProcessEvent::Started`'s new field.** It is a breaking change to a variant
   of a `#[non_exhaustive]` enum, which is a deviation this slice records rather
   than a change it makes quietly.
