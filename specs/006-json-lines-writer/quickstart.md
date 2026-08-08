# Quickstart: JSON Lines Writer

**Slice**: S07

**Created**: 2026-08-08

**Feature**: [spec.md](spec.md)

Everything here runs with a Rust toolchain and nothing else. No capture driver,
no elevated privilege, no game, per section 25.1.

## Prerequisites

- The pinned toolchain, 1.96.0.
- The S04 fixture corpus at `fixtures/`.
- Optional, for the manual checks below: `jq`.

## Run the gate

```bash
cargo xtask ci
```

```bash
cargo xtask neutral && cargo xtask msrv
```

## Run this slice's tests

Escaping and number formatting, the two places this format goes wrong
invisibly:

```bash
cargo test -p fragcap-sink json
```

Every emitted line parsed by a third-party JSON reader:

```bash
cargo test -p fragcap --test goldens jsonl
```

The check that justifies the slice's design, that both formats still say the
same thing about the same packet:

```bash
cargo test -p fragcap --test agreement
```

## Regenerate the goldens

Only when the format changed on purpose. Read the diff before committing.

```bash
FRAGCAP_UPDATE_GOLDENS=1 cargo test -p fragcap --test goldens
```

This regenerates both the `.fcapng` and `.jsonl` goldens.

## Read a stream by hand

The format exists for this, so it is worth doing once:

```bash
jq -c 'select(.proc == "game.exe") | {ts, dir, dst}' fixtures/goldens/tcp-session.jsonl
```

Expected: one line per attributed packet, alternating `out` and `in`.

Note that `jq` prints `ts` as `1700000000` and `1700000000.005`, not with six
digits. That is `jq` parsing the number into a double and re-printing it, not
what the file contains; the bytes carry `1700000000.000000`. It is a live
demonstration of the caveat in the format contract: a consumer that parses into
a double reintroduces the approximation this writer avoided, so parse as a
decimal if the sixth digit matters.

Confirm the header and trailer are distinguishable without parsing the whole
object:

```bash
head -1 fixtures/goldens/tcp-session.jsonl && tail -1 fixtures/goldens/tcp-session.jsonl
```

Expected: a `"type":"header"` object carrying the version and interface set,
and a `"type":"trailer"` object carrying every counter.

Confirm that unattributed packets are present rather than dropped, which is the
P-4 property a consumer most needs to trust:

```bash
jq -c 'select(.attr == "none") | {ts, dir, len}' fixtures/goldens/malformed.jsonl
```

Expected: five records. `malformed.pcap` parses to no flow at all, and every
packet is still in the stream.

Confirm the endpoint naming under an unknown direction, which is the slice's
one deliberate divergence from the section 13.5 example:

```bash
jq -c 'select(.type == null) | {dir, src, dst, local, remote}' fixtures/goldens/loopback.jsonl | head -1
```

Expected:

```text
{"dir":"unknown","src":null,"dst":null,"local":"127.0.0.1:8080","remote":"127.0.0.1:51000"}
```

`dir` is `unknown`, `src` and `dst` are absent (`jq` renders an absent key as
null), and `local` and `remote` carry the endpoints. Both ends of a loopback
conversation are on this host, so which sent a given packet is not determined,
and the record says so by which keys it uses rather than by guessing.

The `select(.type == null)` skips the header, which carries none of these keys.

## What good looks like

| Check | Passing means |
| --- | --- |
| `cargo xtask ci` | Format, lints, tests, conventions, dependencies, licenses |
| `--test agreement` | The two output formats have not drifted apart |
| `--test goldens` | Output is byte-stable and no golden drifted |
| `jq` on a golden | An off-the-shelf consumer reads it with no special tooling |

## What is not covered here

- **The pipeline.** S08 drives this writer.
- **Transports.** S15. This writer targets a `Write`.
- **The console summary.** S14.
- **The session anchor.** Specified for the header, deferred with S08.
- **Reading JSON Lines.** The parser in the tests is a verification tool.
