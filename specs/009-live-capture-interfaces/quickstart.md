# Quickstart: Validating S09

**Slice**: S09

**Date**: 2026-08-09

**Phase**: 1

How to convince yourself this slice works, in the order the checks get harder
to run. The first two need nothing but a checkout. The third needs Windows and
npcap.

## Tier 0 and tier 1: any machine, no driver

This is the whole ordinary gate, and it must pass with neither the capture
driver nor the npcap software development kit installed. That is SC-011, and it
is the property that keeps this slice from making the repository harder to
contribute to.

```bash
cargo xtask ci
```

Expect: green, including `cargo xtask deps` reporting no edge from
`fragcap-capture` to a sibling, and `cargo xtask lint` reporting no use of the
transmit API.

The selection matrix is the interesting part of this tier, because it is where
the section 12.1 precedence is actually decided:

```bash
cargo test -p fragcap-core --lib interface
```

Expect: every row of the matrix asserted (explicitly named, default route,
loopback present and absent, virtual, down, no address, broad capture), plus
the accounting invariant that selected plus excluded equals the inventory
length for every case.

Multi-interface identity end to end, driven by two replay sources standing in
for two interfaces:

```bash
cargo test -p fragcap --test multi_interface
```

Expect: the pcapng output declares two interfaces with their own link types,
every packet block references the right one, the JSON Lines output names the
interface on every record, and the conservation identity holds with two capture
threads running.

Regression on the single-interface case, which must cost nothing:

```bash
cargo test -p fragcap --test corpus_pipeline
```

Expect: the committed goldens reproduce byte for byte. If they do not, S09 has
changed single-interface output, which SC-005 forbids. Do not regenerate the
goldens to make this pass; that would be the failure, not the fix.

## Platform neutrality

```bash
cargo xtask neutral
cargo xtask msrv
```

Both exit 2 rather than 0 when they cannot run, so a check that did not run
cannot look like one that passed. `neutral` proves `fragcap-core` still builds
for a target with no capture backend; `msrv` proves the workspace still builds
at 1.82, which is the check `libloading` 0.9 would have broken had it been
taken directly.

## Tier 2: Windows with npcap installed

Prerequisites, and fragcap installs none of them:

- Windows.
- npcap, installed separately, with loopback capture support and WinPcap API
  compatibility mode selected. Both are non-default options.
- The npcap software development kit on the library path, because the binding
  links against it at build time.
- An elevated shell.

```bash
cargo test -p fragcap-capture --features live --test live
```

Expect: the test generates its own loopback traffic, opens a live handle, and
asserts that what it sent comes back with the driver's timestamps and lengths.
No game and no profiled process is involved.

A test in this file that finds no driver prints why and returns rather than
failing, so the feature stays usable on a development machine without npcap.
A run that reports nothing but skips has proved nothing; read the output.

## Driver detection without a driver

The path most operators meet first, and the one worth checking by hand:

```bash
cargo run -p fragcap-cli --features live -- --version
```

Expect on a machine without npcap: a message naming the driver, its absence,
and the official download location. Expect always: no installer runs, nothing
is downloaded, and nothing is written outside the capture output. The `doctor`
command that presents this properly is S14; this slice only guarantees the
capability behind it is honest.

## What a reviewer should look at first

1. The error mapping table in [contracts/capture-api.md](contracts/capture-api.md),
   specifically the two `PcapError` rows. Device loss is determined by
   re-enumerating rather than by matching an error string, and that is the
   decision most likely to be quietly reverted by someone simplifying.
2. `CapturedPacket::from_raw` having no default interface identifier. A default
   would let a real capture ship with the wrong identity.
3. The virtual-interface pattern list, and that its verdict is carried rather
   than folded into a boolean.
4. That `buffer_dropped` did not become per-interface. It is capture-wide
   because the buffer is, and per-interface eviction counts would invite the
   inference that the busy interface is at fault rather than the slow sink.
