# Implementation Plan: Technology-Detection Surface

**Branch**: `feat/technology-detection` | **Date**: 2026-08-13 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from
`specs/031-technology-detection/spec.md`

## Summary

Vendor SteamDB's `FileDetectionRuleSets` `rules.ini` (MIT, (c) 2021 SteamDB) as a
committed, integrity-locked data asset, and add a technology-detection surface in
`fragcap-profile` that compiles the ruleset's path regexes and matches them
against a target's install-directory file paths to report the technologies
present, grouped by category (engine, anti-cheat, SDK, emulator, container,
launcher). Findings surface through a new on-demand CLI subcommand and are
materialized into a target artifact as a new `technologies` structure in the
master target schema. Detection reads paths only (P-1), skips-and-counts any
ruleset pattern the RE2-style engine cannot compile (P-4), stamps every finding
`heuristic-unverified` (P-9), adds no new dependency and no lockfile crate, and
keeps the live capture path and the packet-stream writers unchanged (P-5).

## Technical Context

**Language/Version**: Rust, workspace edition 2021, MSRV 1.82 (must stay green
via `cargo xtask msrv`).

**Primary Dependencies**: `regex` (already a `fragcap-profile` dependency since
S05, RE2-style), `serde_json` (already a `fragcap-profile` runtime dependency
since S026). No new dependency and no new `Cargo.lock` crate.

**Storage**: One committed data asset (`rules.ini`) embedded at compile time via
`include_str!`, beside the existing embedded `target-schema.v1.json`. A lock
record (JSON) and a third-party attribution notice travel with it.

**Testing**: `cargo test --workspace --locked`. Unit tests in the new
`fragcap-profile` module (ruleset parse, compile-skip-count conservation, path
scan, dedup, unreadable surfacing, hand-rolled SHA-256 against NIST vectors and
against the committed asset), schema-conformance fixtures for the `technologies`
structure, and a CLI-level test for the subcommand output shape.

**Target Platform**: Windows primary; detection is pure `std::fs`/`std::path` and
builds on any target (it stays out of `fragcap-core`, so P-2 is not implicated,
but it is platform-neutral regardless).

**Project Type**: CLI tool over a layered crate workspace (compiler/CLI-like).

**Performance Goals**: Detection is an on-demand, one-shot filesystem walk over a
single install directory. The scan is depth-bounded (see research) so it stays
affordable on a large install; there is no per-packet or capture-loop cost
because detection never runs inside the capture loop.

**Constraints**: P-1 path-only, no file-content reads, no process handle/memory,
no network. P-4 counted skips and surfaced unreadable paths. P-5 packet-stream
writers unchanged. P-9 honest fidelity. No new dependency; MSRV 1.82 green;
UTF-8-no-BOM, LF, no em/en dashes.

**Scale/Scope**: One vendored ~700-line ruleset; roughly a few hundred compiled
patterns across six applied categories; one install directory per invocation.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive Observation Only (NON-NEGOTIABLE)**: PASS. Detection reads
  directory entries and file paths only (`std::fs::read_dir`), never file
  contents, never a process handle, never process memory, never the network. It
  is strictly narrower than the permitted set and touches nothing on the
  denylist. Anti-cheat is detected by marker path and never interacted with; the
  surface is a safety signal, not an evasion aid.
- **P-2 Core Stays Platform-Neutral**: PASS. The new module lives in
  `fragcap-profile`, not `fragcap-core`; the core allowlist (`["bytes"]`) is
  untouched. `fragcap-profile` gains no new dependency.
- **P-3 Capture And Attribution Stay Separate**: PASS. Detection is neither a
  `PacketSource` nor a `FlowAttributor`; it does not enter the capture pipeline
  and does not run during capture.
- **P-4 No Silent Loss**: PASS. A ruleset pattern the engine cannot compile is
  skipped, counted, and the affected technology is surfaced; compiled + skipped =
  total is asserted. An unreadable install directory or subtree is surfaced as a
  named condition distinct from a clean empty scan. A missing or hash-mismatched
  asset is a surfaced error, not a silent zero-rule fallback.
- **P-5 Compatibility Outranks Richness**: PASS. The pcapng and JSON Lines
  writers are not touched; a `.fcapng` stays byte-compatible with unmodified
  analyzers. The technologies metadata lives only in the target artifact governed
  by the master schema.
- **P-6 Glossary First**: PASS (planned). New terms ("technology detection", the
  vendored ruleset, "marker path") get glossary entries in this change, and the
  specification gains a technology-detection section reference.
- **P-7 Wrappers Stay Thin**: PASS. All logic is in Rust; no wrapper parses
  output. The CLI subcommand emits the report; wrappers are untouched.
- **P-8 House Standards Apply**: PASS (planned). UTF-8-no-BOM, LF, no em/en
  dashes, SPDX headers on new source. The vendored `rules.ini` is third-party
  content stored verbatim; see the licensing note below for how the text-hygiene
  linter treats it.
- **P-9 The Instrument Does Not Lie (NON-NEGOTIABLE)**: PASS. Every finding is
  `heuristic-unverified` and names the marker path that produced it; nothing is
  inflated to a fact, nothing detected is masked or withheld. Anti-cheat findings
  are reported, not suppressed, precisely because withholding them would be the
  lie this principle forbids.

**Licensing (constitution "Licensing And Third-Party Obligations")**: The vendored
ruleset is MIT, which is on the permitted license list, so the vendoring is
allowed and the obligation is attribution. Two placement rules follow from the
`cargo xtask license` check, which mirrors each publishable crate's root
`LICENSE`/`NOTICE`/`README.md` byte-for-byte against the repository root: the
SteamDB attribution must NOT be a bare crate-root `NOTICE` (that name is reserved
for the Apache-2.0 crate notice the check mirrors), so it is a distinctly named,
nested third-party notice beside the asset. The crate's own `license` field stays
`Apache-2.0`; a vendored MIT data file does not change it. This resolves
checklist item CHK021. Adding the asset, its notice, and its lock is recorded as
a dated changelog decision (the asset is a pinned artifact in spirit; no
`.github/workflows/**`, `rust-toolchain.toml`, `release.toml`, or `scripts/**`
file changes are required because the hash check runs inside `cargo test`).

**Result**: All gates pass. No Complexity Tracking entries required.

## Project Structure

### Documentation (this feature)

```text
specs/031-technology-detection/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   ├── cli-technologies.md   # The new subcommand's contract
│   └── technologies-schema.md # The schema structure contract
├── checklists/
│   ├── requirements.md
│   └── constitution-and-licensing.md
└── tasks.md             # /speckit-tasks output (not created here)
```

### Source Code (repository root)

```text
crates/fragcap-profile/
├── assets/
│   ├── target-schema.v1.json          # extended: + technologies structure
│   └── steamdb/
│       ├── rules.ini                  # NEW: vendored verbatim (MIT, LF)
│       ├── rules.lock.json            # NEW: source, commit, SPDX, sha256
│       └── THIRD_PARTY_NOTICES.md      # NEW: MIT text + (c) 2021 SteamDB
├── src/
│   ├── technologies.rs                # NEW: ruleset parse/compile/scan/report
│   ├── sha256.rs                      # NEW: hand-rolled SHA-256 (no dep)
│   ├── jsonschema/
│   │   ├── document.rs                # unchanged embed path
│   │   └── variants.rs                # extended: validate technologies
│   └── lib.rs                         # re-export the new public surface
└── tests/
    └── schema_conformance.rs          # extended: technologies fixtures

crates/fragcap-cli/src/
├── commands/
│   ├── mod.rs                         # register the new command
│   └── technologies.rs               # NEW: on-demand detection subcommand
├── args.rs / cli.rs                   # extended: parse the subcommand
└── ...

crates/fragcap-steam/src/
└── scaffold.rs                        # extended: populate technologies on the
                                       #   materialized target artifact

docs/schema/target-schema.v1.json      # extended: byte-identical to embedded
docs/glossary/*.md                     # NEW entries; index updated
docs/fragcap-specification.md          # + technology-detection section (or note)
changelog.d/031-technology-detection.* # added + decisions fragments
```

**Structure Decision**: The detection engine lives in `fragcap-profile` beside
`engine_rule.rs`, because it is the same shape of work (path-and-layout reasoning
over an install directory, no platform dependency) and `fragcap-profile` already
carries the regex engine and the master schema it extends. The CLI surface lives
in `fragcap-cli` (which already depends on `fragcap-profile`); the scaffold
enrichment lives in `fragcap-steam` (which already depends on `fragcap-profile`),
so no dependency direction is inverted (P-2, P-3). The hand-rolled SHA-256 keeps
the "no new lockfile crate" constraint absolute, in the same idiom as the
hand-rolled glob matcher, writers, and schema validator.

## Key Design Decisions

1. **Vendor the whole `rules.ini` verbatim; apply the direct category sections.**
   The lock hash is meaningful only over the complete, unmodified upstream file,
   and the NOTICE covers the whole work. The detection engine applies the six
   direct category sections (`Engine`, `AntiCheat`, `SDK`, `Emulator`,
   `Container`, `Launcher`). The `Evidence` two-pass deduction is parsed-past
   (its section is recognized so its lines are not miscounted) but not executed;
   deferring the inference engine is recorded and does not weaken the first-pass
   findings.

2. **RE2 incompatibility is handled at load, never by editing the asset.** Some
   upstream patterns use PCRE constructs the `regex` crate rejects (possessive
   quantifiers such as `\w++`, and any others). Each pattern is compiled
   independently; a compile error skips that one pattern, increments a skipped
   count, and records the affected technology. The vendored bytes are never
   altered to force compilation (that would break the lock and the "faithful
   copy" property). `compiled + skipped == total` is asserted (FR-006).

3. **Hand-rolled SHA-256, checked in `cargo test`.** A small `sha256.rs`
   (validated against known NIST test vectors) hashes the embedded `rules.ini`
   bytes and a test asserts equality with the lock's recorded hash. This puts the
   integrity check inside the existing gate with zero workflow changes and zero
   new crates. The hash is over the committed bytes as stored (LF, UTF-8, no BOM);
   the lock's `note` documents that normalization so a human `sha256sum` on the
   committed file reproduces it.

4. **`technologies` is an additive, backward-compatible extension of schema v1.**
   A new optional top-level `technologies` array is added to
   `target-schema.v1.json` (both the embedded copy and the `docs/schema` copy,
   byte-identical), plus a `$defs/technology` object and a category enum
   (`engine`, `anti_cheat`, `sdk`, `framework`, `emulator`, `container`,
   `runtime`, `launcher`). It reuses the existing `fidelity` `$def`. Because the
   schema was `additionalProperties: false`, adding a known optional property is
   backward compatible: prior artifacts still validate, and artifacts carrying
   `technologies` now validate where before they would have been rejected. No
   schema-version bump; recorded as a decision. The hand-rolled variant validator
   in `variants.rs` is extended to accept and shape-check the new property.

5. **On-demand CLI plus scaffold enrichment; capture path untouched.** A new
   `fragcap technologies --path <dir>` (exact name finalized in the CLI contract)
   prints the grouped report. The Steam scaffold, which already materializes a
   target artifact, gains the `technologies` structure for the install directory
   it classifies. `run`, the pipeline, and the pcapng/JSONL writers are not
   touched.

## Complexity Tracking

No constitution violations. No entries.
