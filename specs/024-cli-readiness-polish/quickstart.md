# Quickstart / Validation Guide: CLI readiness, help, and output-contract polish

How to prove the slice works end to end. Details of shapes and codes live in
`contracts/` and `data-model.md`; this is the run guide.

## Prerequisites

- Windows for the capability, elevation, and npcap-version paths; the pure
  `doctor` classifier tests and the help/exit/JSON tests run on any platform.
- Toolchain per `rust-toolchain.toml` (MSRV 1.82).
- For the featured smoke test: npcap installed.

## Full gate (authoritative)

Run in the foreground and read the output; never background it:

```bash
cargo xtask ci
```

This runs fmt, `clippy --all-targets --all-features -D warnings`,
`cargo test --workspace --locked`, `cargo xtask lint`, `deps`, and `license`.
The `lint` step also asserts no `OpenProcess`/`ReadProcessMemory`/
`WriteProcessMemory` appears in fragcap sources - the P-1 guard the elevation
gate must not trip.

## Per-story validation

### Story 1 - honest doctor (#63, #69, #70.2)

- Extend the pure-function unit tests in `crates/fragcap-cli/src/doctor/checks.rs`
  (they build an `Inputs` and assert `Check`/`Report` outcomes):
  - `live_available: None` yields a `Fail` and `Report::exit() == Exit::FAILURE`.
  - `socket_table_available: None` yields a `Warn` and does not block.
  - loopback absent yields `Warn` and `report.ready()` stays true.
  - live absent reworders the empty-interfaces message (assert on the detail).
- Manual, featured build on a machine with npcap:

```bash
cargo run -p fragcap-cli --features live,socket-table,etw -- doctor
```

  Expect capability lines reporting the backends present and an npcap line
  showing a real version. Build **without** features to see the live-backend
  `fail` and the "not ready" verdict:

```bash
cargo run -p fragcap-cli -- doctor
```

### Story 2 - featured release binary (#62)

- Inspect `.github/workflows/release.yml`: the build step passes
  `--features live,socket-table,etw` and the npcap SDK-acquisition step is
  present in the release job.
- Confirm a dated decision fragment exists under `changelog.d/` for the workflow
  change.

### Story 3 - profile `--json` (#65)

```bash
cargo run -p fragcap-cli -- --json profile list
cargo run -p fragcap-cli -- --json profile validate <a-broken-profile.toml>
```

- `list` emits structured `profiles` counts, not human text.
- `validate` emits one `diagnostic` event per problem plus a terminal `summary`.
- Test asserts by parsing each NDJSON line with the dev-only `serde_json` and
  checking the per-diagnostic fields (`code`/`path`/`line`/`col`/`message`).

### Story 4 - exit codes (#68)

```bash
cargo run -p fragcap-cli -- profile show   <missing-ref>; echo $?
cargo run -p fragcap-cli -- profile validate <missing-ref>; echo $?
```

- Both print `1` for a reference that resolves to nothing.
- A genuinely malformed reference (neither a valid id nor a readable path) may
  still exit `2`. Covered by an integration test asserting both codes.

### Story 5 - help text (#66, #67)

```bash
cargo run -p fragcap-cli -- run --help
cargo run -p fragcap-cli -- extcap --help
```

- No `value_parser`/`Vec<String>` note; no `S15`/`S16`/`S17`; `--launch`
  describes real behavior. A test scans rendered help for these substrings.

### Story 6 - elevation gate (#56)

- On a **non-elevated** Windows terminal, with a featured build:

```bash
cargo run -p fragcap-cli --features live,socket-table,etw -- run --profile <valid> --out out.pcapng --duration 1s; echo $?
```

  Expect an elevation-required message, exit `1`, and no driver access. Offline
  commands (`profile`, `doctor`, `steam profile`, `replay`) still run.
- Because the gate is Windows-only, add a platform-neutral test at the predicate
  seam (elevated=false + live-capture command → refusal) so CI covers the logic
  off-Windows.

### Story 7 - validate output polish (#70.1)

```bash
cargo run -p fragcap-cli -- profile validate <a-valid-profile.toml>
```

- The success line names the path once.

## Spec/governance checks

- `docs/fragcap-specification.md` sections 17 and 26.3 reflect the new behavior
  (exit alignment note, structured `diagnostic`/`summary` events, doctor
  capability lines, loopback severity, npcap version).
- Any newly introduced term has a glossary entry (section 4.3) in this slice.
