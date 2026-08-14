# Implementation Plan: doctor truthfulness and presentation

**Branch**: `040-doctor-truthfulness-presentation` | **Date**: 2026-08-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/040-doctor-truthfulness-presentation/spec.md`

## Summary

Make `fragcap doctor` report the truth and present it legibly. The Windows probe
currently substitutes a hardcoded empty interface list and an unrelated
filesystem marker for the enumeration the binary is already linked against; this
slice wires the probe to the existing `fragcap::enumerate()` and
`fragcap::detect_driver()` (behind the same `live`+`windows` gate the other
backends use), adds a leading identity section, and moves colorization into a
TTY-gated presentation layer so the plain, golden-tested report is unchanged in
bytes. No capture, enumeration, or driver-detection logic is modified; this slice
only changes whether and how the report consumes and presents it.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82 (edition 2021)

**Primary Dependencies**: existing only. `fragcap` facade
(`enumerate`, `detect_driver`, and a new `virtual_verdict`/`VirtualVerdict`
re-export), `std::io::IsTerminal` (stable 1.70). No new crate; ANSI is
hand-rolled (four status colors, bold heading), consistent with a CLI that
carries zero color code today.

**Storage**: N/A (doctor is read-only; it reads the machine and the filesystem
for path existence only where a check already does).

**Testing**: `cargo test` (workspace). Doctor is tested by hand-built `Inputs`
through the pure `checks::run` classifier plus two golden files compared by
`render_human`/`render_json`; a tolerant end-to-end test drives the real command.

**Target Platform**: Windows (the capture target). The command also runs and
classifies on non-Windows via the existing minimal `Inputs` path.

**Project Type**: CLI (single Rust workspace, the `fragcap-cli` crate over the
`fragcap` facade).

**Performance Goals**: N/A (a one-shot diagnostic; enumeration opens no handle).

**Constraints**: The default `cargo test --workspace` builds with no features and
the Linux `fragcap-core` neutrality build has no capture backend; the new
enumeration call MUST be `#[cfg(all(feature="live", windows))]`-gated with an
empty fallback so both still compile (risk R-1). Human default output must fit 80
columns; color only on an interactive terminal with `NO_COLOR` unset and never in
the JSON form.

**Scale/Scope**: One command, three source modules under `crates/fragcap-cli/src/doctor/`
plus `commands/doctor.rs`, one facade re-export line, two golden files, two
duplicated test fixtures.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive observation**: unaffected. Enumeration opens no handle and no
  transmit call is added. PASS.
- **P-2 Core stays platform-neutral**: the enumeration call lives in
  `fragcap-cli` (not core) and is `cfg`-gated so `fragcap-core` still builds with
  no backend; dependency direction (cli -> facade) is unchanged. PASS, and R-1
  is the standing check.
- **P-3 Capture and attribution separate**: untouched; no source or attributor
  is modified. PASS.
- **P-4 No silent loss**: doctor is not a capture path; no counter is involved.
  The identity/paths additions never suppress an observation. PASS.
- **P-5 Compatibility outranks richness**: the machine-readable form stays one
  record per check; color is confined to the human TTY layer. PASS.
- **P-6 Glossary first**: no new domain term is introduced (interface, loopback,
  extcap, npcap all have entries). If any user-facing string introduces a new
  term, a glossary entry is added in this change. PASS with watch.
- **P-9 The instrument does not lie**: this slice is a direct expression of P-9:
  the report stops asserting an empty interface list and a guessed loopback
  state, and reports enumeration/driver truth or an explicit "undetermined".
  PASS (the motivating principle).
- **Licensing / npcap**: detection only; nothing bundled, downloaded, or
  installed. The Wireshark-bundles-Npcap wording is a link, not an install. PASS.
- **Text hygiene**: UTF-8, LF, no em/en dashes, SPDX headers preserved. PASS.
- **Pinned artifacts**: none touched (no `.github/workflows/**`,
  `rust-toolchain.toml`, `release.toml`, `scripts/**`). PASS.

No violations; Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/040-doctor-truthfulness-presentation/
|-- plan.md              # This file
|-- research.md          # Phase 0 output
|-- data-model.md        # Phase 1 output
|-- quickstart.md        # Phase 1 output
|-- contracts/
|   `-- doctor-output.md  # human + JSON output contract
|-- checklists/
|   |-- requirements.md  # spec quality (from specify)
|   `-- output.md        # domain checklist (from checklist)
`-- tasks.md             # Phase 2 output (speckit-tasks)
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/doctor/
|-- probe.rs     # enumerate + detect_driver behind cfg(all(feature=live, windows)); identity fields
|-- checks.rs    # Identity section classifiers; loopback from DriverReport; guidance strings
`-- mod.rs       # Inputs identity fields; render_human plain (section spacing + 80-col wrap)

crates/fragcap-cli/src/commands/doctor.rs   # TTY-gated ANSI styling around plain render_human

crates/fragcap/src/lib.rs                    # add virtual_verdict + VirtualVerdict to fragcap::core re-export

crates/fragcap-cli/tests/
|-- cli_doctor.rs                 # update ready() fixture; presentation/identity assertions
`-- goldens/doctor-ready.{txt,ndjson}   # regenerated

changelog.d/
|-- 102-doctor-interfaces.fixed.md
|-- 103-doctor-loopback.fixed.md
|-- 105-106-doctor-output.changed.md
`-- dependency-taxonomy.decisions.md     # required/recommended/optional model, referenced by slice 042
```

**Structure Decision**: single-crate change inside `fragcap-cli` plus one facade
re-export. No new module or crate. The correctness work (probe + checks), the
presentation work (mod + commands), and the facade re-export are the three edit
clusters; tests and goldens follow.

## Complexity Tracking

No constitution violations; no entries.
