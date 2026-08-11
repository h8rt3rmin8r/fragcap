# Quickstart: Extcap analyzer integration

Runnable validation scenarios for slice S18 sub-slice A. Every scenario here runs
at tier 1: no capture driver, no elevation, no analyzer.

## Prerequisite

npcap is a required, separate install for live capture (specification section
20). None of the scenarios below need it: they use the offline substrate.

## 1. The analyzer discovery contract

```bash
fragcap extcap --extcap-interfaces
fragcap extcap --extcap-dlts --extcap-interface fragcap
fragcap extcap --extcap-config --extcap-interface fragcap
```

Expect: the first prints one `interface {value=fragcap}...` line; the second
prints `dlt {number=1}{name=EN10MB}{display=Ethernet}`; the third prints four
`arg` lines (`--profile`, `--roles`, `--direction`, `--loopback`) plus the
direction `value` lines. Each exits 0. Each is accepted by the extcap
control-grammar check (SC-001).

## 2. Stream a capture to a FIFO (offline)

Using the hidden offline substrate so no driver is needed, stream to a regular
file used as the FIFO path:

```bash
fragcap extcap --capture --fifo out.fcapng \
  --profile game --replay-source fixtures/<a-fixture>.pcap \
  --process-script <script> --attr-script <script>
```

Expect: `out.fcapng` is a valid pcapng an unmodified parser reads in full,
reproducing the committed pcapng golden for that fixture (SC-002). Its bytes equal
a plain `run --out` capture of the same input (FR-005, SC-006).

## 3. The dialog options select the capture

```bash
fragcap extcap --capture --fifo out.fcapng \
  --profile game --roles client --direction in --loopback \
  --replay-source fixtures/<a-fixture>.pcap
```

Expect: the stream is scoped exactly as the equivalent
`run --profile game --roles client --direction in --loopback` over the same
input (SC-003), because the extcap capture overlays the options the same way
`run` does.

## 4. Misuse is refused before capture

```bash
fragcap extcap --capture                       # missing --fifo
fragcap extcap --extcap-dlts                    # missing --extcap-interface
fragcap extcap --extcap-dlts --extcap-interface nope   # unknown interface
```

Expect: each exits 2 with a message naming the cause, and no capture is started
(SC-005).

## 5. doctor reports extcap installation

```bash
FRAGCAP_EXTCAP_DIR=<a dir with a fragcap binary> fragcap doctor
FRAGCAP_EXTCAP_DIR=<an empty dir> fragcap doctor
```

Expect: the first reports the analyzer extcap integration as installed and names
the directory; the second reports it as not installed and names the same
directory (SC-004). The probe reads the directory only; it installs nothing.

## 6. The full gate

```bash
cargo xtask ci
cargo xtask neutral
```

Expect: `ci` is green (fmt, clippy, the workspace tests, lint, deps, license,
fixture drift) and the platform-neutral core still builds (SC-007). The extcap
path is covered entirely at tier 1.
