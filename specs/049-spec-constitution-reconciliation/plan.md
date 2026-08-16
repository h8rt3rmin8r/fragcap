# Implementation Plan: Specification and constitution reconciliation

**Branch**: `049-spec-constitution-reconciliation` | **Date**: 2026-08-16 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/049-spec-constitution-reconciliation/spec.md`

**Slice**: S049 (GitHub issue #136, milestone v0.5.0). First slice of the v0.5.0
UX overhaul; every later v0.5.0 slice depends on it.

## Summary

Bring the master specification back in line with shipped reality (it still frames
v0.2.0 as the first functional release while the live release is v0.4.0), record
two durable rules as constitution principles P-10 and P-11, and make the
alignment self-enforcing. The enforcement is three mechanical parts: an
`Applies-To` field in the specification bound by a `cargo xtask spec` check to the
workspace version and wired into `ci`; a `spec-impact` field on changelog
fragments; and a release-time gate that refuses to assemble a release when a
fragment claims a specification change the release diff does not contain. No
capture, attribution, or output behavior changes.

## Technical Context

**Language/Version**: Rust (pinned via `rust-toolchain.toml`; MSRV 1.82). Changes
are confined to the `xtask` task-runner crate plus documentation and governance
files.

**Primary Dependencies**: none added. The new code uses only `std`, `git`
(invoked as a subprocess, as `changelog.rs` already does), and the existing xtask
modules (`main.rs`, `changelog.rs`).

**Storage**: N/A (documents and repository check tooling only).

**Testing**: `cargo test --package xtask` (unit tests for `SpecImpact` parsing,
the version comparison, and the pure release-gate decision function), plus the
existing gate set via `cargo xtask ci`.

**Target Platform**: repository tooling; runs on the CI matrix (ubuntu, windows)
and locally.

**Project Type**: single Rust workspace; this slice touches the `xtask` crate,
`docs/`, `.specify/`, `changelog.d/`, and one workflow file.

**Performance Goals**: N/A. The checks are file reads and one `git diff`.

**Constraints**:
- House text hygiene is absolute (UTF-8 no BOM, LF, no trailing whitespace,
  single trailing newline, no em-dashes or en-dashes), including the Appendix C/D
  transcription.
- `.github/workflows/ci.yml` is a pinned artifact, so its change lands with a
  dated `changelog.d/*.decisions.md` fragment.
- `Applies-To` equals the workspace version (currently `0.4.0`) at all times; the
  bump to `0.5.0` is a future release action, not part of this slice.

**Scale/Scope**: one specification document, one constitution, two glossary
entries plus index, one new xtask module, edits to two existing xtask modules,
one workflow step, and changelog fragments (including the `spec-impact` retrofit
of the one existing fragment).

## Constitution Check

*GATE: evaluated before Phase 0 and re-checked after Phase 1 design.*

| Principle | Assessment |
| --- | --- |
| P-1 Passive Observation | N/A. No capture technique touched. Pass. |
| P-2 Core Platform-Neutral | No change to `fragcap-core`; no new dependency anywhere. Pass. |
| P-3 Capture/Attribution Separate | N/A. Pass. |
| P-4 No Silent Loss | N/A to capture. The new checks honor the 0/1/2 contract, so a check that cannot run exits 2 rather than passing silently, which is the same ethos. Pass. |
| P-5 Compatibility Outranks Richness | N/A. No output format change. Pass. |
| P-6 Glossary First | Two new documentation terms (`Applies-To`, `spec-impact`). Precedent (`xtask`, `msrv` have entries) means they require glossary entries; this slice adds them and regenerates the index (R-5). Pass by inclusion. |
| P-7 Wrappers Stay Thin | The release gate and version check are Rust in `xtask`, not shell. Pass (actively upholds P-7). |
| P-8 House Standards Apply | All edited files obey text hygiene; the Appendix transcription is checked for stray dashes. Pass with care. |
| P-9 Instrument Does Not Lie | N/A to capture. Pass. |
| Pinned artifacts | `ci.yml` changes with a dated decisions fragment (FR-011). `release.yml` is NOT changed (the release gate lives in `changelog --release`), so no second decisions fragment. Other pinned artifacts untouched. Pass. |
| Amendment policy | Constitution bump 1.1.0 -> 1.2.0 (MINOR, two principles added) with an updated Sync Impact Report, per the in-repo 1.0.0 -> 1.1.0 precedent. Pass. |

**Gate result**: PASS. No violations; Complexity Tracking not required.

## Project Structure

### Documentation (this feature)

```text
specs/049-spec-constitution-reconciliation/
├── spec.md              # Feature spec (clarified)
├── plan.md              # This file
├── research.md          # Phase 0 decisions R-1..R-7
├── data-model.md        # Fields and parsed values
├── quickstart.md        # Validation scenarios
├── contracts/
│   └── checks.md        # Check and format contracts
└── checklists/
    └── requirements.md  # Spec quality checklist (passing)
```

### Source and repository files touched

```text
docs/
  fragcap-specification.md        # Applies-To field; doc-control history through v0.4.0;
                                  #   sections 3.3, 27.3, 28; section 23.1 = Appendix D;
                                  #   full version-currency sweep (title, section 1, TOC)
  glossary/
    rust-and-tooling.md           # new entries: Applies-To, spec-impact
    index.md                      # regenerated

.specify/memory/constitution.md   # add P-10, P-11; Sync Impact Report; version 1.2.0

xtask/src/
  spec.rs                         # NEW: version lock-step + fragment format check;
                                  #   SpecImpact parse; pure release-gate decision fn
  main.rs                         # dispatch `spec`; add to `ci` aggregate; workspace_version()
  changelog.rs                    # strip leading spec-impact comment; release-gate preflight

.github/workflows/ci.yml          # PINNED: add "Specification version lock-step" step

changelog.d/
  <existing>.fixed.md             # retrofit a spec-impact line onto the one existing fragment
  S049-spec-reconciliation.*.md   # this slice's fragments (added/changed) with spec-impact
  S049-ci-spec-check.decisions.md # dated decision for the ci.yml change (pinned artifact)
```

**Structure Decision**: No new crate. The reconciliation is documentation plus
governance plus repository tooling; the tooling belongs in the existing `xtask`
crate alongside `lint`, `deps`, `license`, and `changelog`, which is where the
constitution places repository checks (P-7: capability in Rust, not shell).

## Phase sequencing (for `/speckit-tasks`)

Not the task list; a dependency sketch so tasks come out ordered.

1. **Constitution amendment** (P-10, P-11, version bump). Independent; the
   guiding rules the rest of v0.5.0 relies on.
2. **Specification reconciliation**: `Applies-To` field, doc-control history,
   sections 3.3/23.1/27.3/28, and the full currency sweep. Independent of the
   tooling but must precede the version check passing.
3. **Glossary entries + index** for `Applies-To` and `spec-impact`.
4. **`xtask spec` module**: `workspace_version()`, the version lock-step check,
   `SpecImpact` parsing, the fragment-format check, and the pure release-gate
   decision function, all unit tested. Dispatch in `main.rs`; add to `ci`.
5. **`changelog.rs`**: strip the leading `spec-impact` comment during assembly;
   add the release-gate preflight to `--release` using the pure function.
6. **`spec-impact` retrofit** onto the existing fragment, and this slice's own
   fragments (with `spec-impact` values), and the `changelog.d/README.md`
   documentation of the field.
7. **`ci.yml` step** plus its `decisions` fragment.
8. **Verify**: `cargo xtask ci` green end to end; quickstart scenarios pass.

## Deferred / documented limitations (from research)

- The release gate is file-level (specification changed at all), not
  per-section, and does not validate that a named section exists (R-4). Recorded
  as intentional.
- `Applies-To` and the workspace version are compared for string equality, not
  semantic ordering (data-model). The invariant is equality.

## Complexity Tracking

No constitution violations; no entries.
