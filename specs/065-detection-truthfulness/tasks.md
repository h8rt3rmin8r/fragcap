# Tasks: Detection truthfulness and the column split

**Slice**: S065 | **Branch**: `065-detection-truthfulness` | **Date**:
2026-08-20

**Input**: [spec.md](./spec.md), [plan.md](./plan.md),
[research.md](./research.md), [data-model.md](./data-model.md),
[quickstart.md](./quickstart.md)

**Closes**: #169, #168, #174

Tests are written before the code they cover, per the constitution's
test-driven discipline. A task that says "assert" writes a failing test; the
implementation task that follows makes it pass.

## Phase 1: Setup

- [x] T001 Read `crates/fragcap-profile/src/pe.rs`,
      `crates/fragcap-profile/src/signature.rs`, and
      `crates/fragcap-targets/src/readiness.rs` end to end, and record in
      `specs/065-detection-truthfulness/plan.md` any place the design in
      data-model.md does not match what is actually there
- [x] T002 Confirm the baseline is green by running `cargo xtask ci` in the
      foreground before any edit, so a failure later is attributable to this
      slice

## Phase 2: Foundational (blocking prerequisites)

These change vocabulary that every later phase reads. Nothing in phases 3 to 5
can start until they land.

- [x] T003 Add `pe::section_names` test cases to
      `crates/fragcap-profile/src/pe.rs`: a non-PE byte string yields nothing, a
      PE with a section table yields its names in table order, a name shorter
      than eight bytes is NUL-trimmed, and a declared section count larger than
      the supplied bytes yields only the names actually present
- [x] T004 Add `pe::tests_support::minimal_pe_with_sections(&[&str])` to
      `crates/fragcap-profile/src/pe.rs`, building a DOS stub, a `PE\0\0`
      signature, a COFF header carrying `NumberOfSections` and
      `SizeOfOptionalHeader`, a stub optional header, and a 40-byte section
      header per name
- [x] T005 Implement `pe::section_names(bytes: &[u8]) -> Vec<String>` in
      `crates/fragcap-profile/src/pe.rs` per research.md R-1, with every offset
      bound-checked and a malformed header yielding an empty vector rather than
      an error, and document the P-1 posture in the module comment
- [x] T006 Add the `Signature::is_matchable` and marker-form tests to
      `crates/fragcap-profile/src/signature.rs`: `section:.bind` is applied,
      `denuvo-anti-tamper-marker` is inert, `section:` alone is skipped, and the
      applied plus inert plus skipped sum still equals the total
- [x] T007 Replace `SignatureKind::is_implemented` with
      `Signature::is_matchable` in `crates/fragcap-profile/src/signature.rs`,
      add the internal match mode for a PE section name, extend
      `compile_pattern` to compile a `section:` glob, and route an unrecognized
      binary-marker pattern to inert rather than skipped
- [x] T008 [P] Add `Engine::ALL` and `Engine::product_name()` to
      `crates/fragcap-profile/src/engine_rule.rs` with a unit test that
      `product_name()` is distinct from `as_str()` for at least one variant, and
      document why the two differ
- [x] T009 Add `DetectionScan` to `crates/fragcap-targets/src/entry.rs` with
      `as_str`, a `parse` that rejects an out-of-set value, and a round-trip
      test over every variant, mirroring `TargetClassification`
- [x] T010 Add the `detection_scan` field to `TargetEntry` in
      `crates/fragcap-targets/src/entry.rs` as `Option<DetectionScan>`
- [x] T011 Move `SCHEMA_VERSION` to 7 and add `MIGRATE_6_TO_7` plus the
      `detection_scan` column with its CHECK to both the fresh DDL and the
      migration in `crates/fragcap-targets/src/schema.rs`
- [x] T012 Apply the version 6 to 7 step in
      `crates/fragcap-targets/src/store.rs`, read and write the new column in
      the target insert, update, and select paths, and add a test that a store
      stamped at version 6 opens, migrates, and reads the column as `None`

## Phase 3: User Story 1 - The evidence names what is actually there (P1)

**Goal**: a product is named only when its marker was observed. The false DRM
label goes; a real one arrives.

**Independent test**: two generated PE fixtures in otherwise identical trees,
one carrying a `.bind` section and one not, both shipping `steam_api64.dll`.
The first reports Steam DRM; the second reports no DRM. No game, no platform
install.

- [x] T013 [US1] Add the bounded-scan tests to
      `crates/fragcap-profile/src/signature.rs`: a tree whose launch executable
      carries `.bind` reports the DRM product at verified fidelity; an otherwise
      identical tree without it reports none; an executable below the depth
      bound is not read; a file named `.exe` that is not a PE image produces no
      finding and no error
- [x] T014 [US1] Add the cap-accounting test to
      `crates/fragcap-profile/src/signature.rs`: a tree with more executables
      than `MARKER_SCAN_MAX_CANDIDATES` advances
      `ScanOutcome::marker_candidates_skipped` and makes
      `ScanOutcome::is_complete()` false
- [x] T015 [US1] Add the unreadable-candidate test to
      `crates/fragcap-profile/src/signature.rs`: a candidate executable that
      cannot be read is recorded in `ScanOutcome::unreadable` and does not
      produce a finding
- [x] T016 [US1] Add `MARKER_SCAN_MAX_DEPTH`, `MARKER_SCAN_MAX_CANDIDATES`,
      `ScanOutcome::marker_candidates_skipped`, and `ScanOutcome::is_complete`
      to `crates/fragcap-profile/src/signature.rs`, carrying the depth on the
      walk's `Entry` so the candidate set can be selected
- [x] T017 [US1] Implement the section-name match arm in
      `SignatureSet::detect` in `crates/fragcap-profile/src/signature.rs`,
      reading a bounded prefix of each candidate rather than the whole file,
      behind a named `MARKER_SCAN_PREFIX_BYTES` constant (64 KiB) alongside the
      other two bounds, and record the R-6 candidate rule in the doc comment
- [x] T018 [US1] Remove the two `steam_api*.dll` rows and add
      `{"category":"drm","kind":"binary-marker","pattern":"section:.bind","product":"Steam DRM","confidence":"definitive"}`
      to `crates/fragcap-targets/assets/signatures.json`
- [x] T019 [US1] Re-point the tests that assert on the dropped rows onto a
      signature that still exists, without weakening what they cover, in
      `crates/fragcap-cli/src/commands/technologies.rs`,
      `crates/fragcap-targets/tests/signatures.rs`,
      `crates/fragcap-profile/src/signature.rs`, and
      `crates/fragcap-steam/src/scaffold.rs`, whose hand-built finding models a
      product the detector can no longer produce from that evidence
- [x] T020 [US1] Update the shipped-seed accounting assertion in
      `crates/fragcap-targets/tests/signatures.rs` so it derives the inert count
      from the seed's own rows rather than asserting a literal, and assert the
      seed carries no `steam_api` pattern

## Phase 4: User Story 2 - An installed engine is named (P1)

**Goal**: Ren'Py and GameMaker become nameable, and the two engine detectors
can no longer drift apart unnoticed.

**Independent test**: fixture trees with the canonical Ren'Py and GameMaker
layouts, asserting each engine is named, plus a check that fails when a
launch-resolution engine has no signature.

- [x] T021 [P] [US2] Add engine detection tests to
      `crates/fragcap-profile/src/signature.rs`: a tree with a `renpy/`
      directory reports Ren'Py at verified fidelity, a tree with `data.win`
      reports GameMaker at verified fidelity, and a tree with only `*.rpa`
      reports Ren'Py at heuristic fidelity
- [x] T022 [US2] Add the five engine rows to
      `crates/fragcap-targets/assets/signatures.json`: `data.win` and
      `Steamworks_x64.dll` for GameMaker, and `renpy/`, `librenpython.dll`, and
      `*.rpa` for Ren'Py, with the confidences from research.md R-4
- [x] T023 [US2] Add the directed subset check to
      `crates/fragcap-targets/tests/signatures.rs`: for every `Engine` in
      `Engine::ALL`, the bundled seed carries at least one engine-category
      signature whose product equals `product_name()`. It must iterate the enum
      and assert no count and no second list
- [x] T024 [US2] Add a fixture test to
      `crates/fragcap-targets/tests/signatures.rs` covering the full observed
      Ren'Py tree (`renpy/`, `game/archive.rpa`,
      `lib/py3-windows-x86_64/librenpython.dll`) and the full observed GameMaker
      tree (`data.win`, `Steamworks_x64.dll`), asserting one engine finding each
      after deduplication

## Phase 5: User Story 3 - Each column reports one kind of fact (P2)

**Goal**: two columns with one job each, three visibly distinct coverage
states, and the same answer on both surfaces.

**Independent test**: registered rows carrying known evidence and known
coverage states render the expected columns and markers, and the export of the
same rows carries the same partition and state.

- [x] T025 [US3] Add the summary tests to
      `crates/fragcap-targets/src/readiness.rs`: an entry with an engine and a
      DRM product puts each in its own summary and neither in both; the three
      coverage markers render for the three states; and the two retired
      sentences appear nowhere
- [x] T026 [US3] Replace `known_summary` with `engine_summary` and
      `sensitivities_summary` in `crates/fragcap-targets/src/readiness.rs`,
      partitioning on the finding's `category` and ordering sensitivities
      anti-cheat before DRM per the declared category order
- [x] T027 [US3] Add `detection_scan` to `CandidateTarget` in
      `crates/fragcap-targets/src/source.rs` and to `ClassifierVerdict::Hit` in
      `crates/fragcap-targets/src/classifier.rs`, and set it in
      `SignatureClassifier` from the scan outcome. The `Err` branch, where the
      root itself could not be read and there is no `ScanOutcome` to derive
      from, must record `Incomplete`, not `None`: a scan was attempted and
      failed, which is a different fact from no scan. Cover that branch with a
      test, because it is the unplumbed-source defect FR-015 names
- [x] T028 [US3] Plumb `detection_scan` through
      `crates/fragcap-targets/src/sources/directory.rs` (set from the outcome
      with signatures, `None` without) and
      `crates/fragcap-targets/src/sources/known_roots.rs`
- [x] T028a [US3] Surface the cap truncation as a named warning wherever
      `ScanOutcome::unreadable` is already surfaced (`detect_evidence` in
      `crates/fragcap/src/discovery.rs`, `detect` in
      `crates/fragcap-targets/src/sources/directory.rs`), so a counted loss is
      also nameable rather than only tallied (P-4, FR-005)
- [x] T029 [US3] Plumb `detection_scan` from `detect_evidence` through
      `SteamSource` in `crates/fragcap/src/discovery.rs`, recording `Incomplete`
      when the install root itself could not be read
- [x] T030 [US3] Store `detection_scan` on the registered entry in
      `crates/fragcap-targets/src/register.rs` and add a test that a candidate
      carrying a coverage state produces an entry carrying it
- [x] T031 [US3] Add the `detection_scan` export key to
      `crates/fragcap-targets/src/targets_export.rs` in both directions,
      emitting it only when present and rejecting an out-of-set value at import,
      with a round-trip test and a rejection test. The round-trip test must
      assert both halves FR-016 names: the per-finding `category` and the
      `detection_scan` key, so the already-true half is guarded too
- [x] T032 [US3] Add the CLI listing tests to
      `crates/fragcap-cli/tests/cli_targets.rs`: both new headers are present,
      `KNOWN` is gone, an engine and a sensitivity land in different columns,
      the three coverage markers are distinguishable, and the representative row
      renders within 80 columns measured from the rendered line
- [x] T033 [US3] Render `ENGINE` and `SENSITIVITIES` in `render_table` in
      `crates/fragcap-cli/src/commands/targets.rs`, sizing the engine column to
      content and leaving sensitivities last and free-running, and record the
      width rule in the function's doc comment
- [x] T034 [US3] Plumb the coverage state through `scan_exe_evidence` and the
      `targets add` path in `crates/fragcap-cli/src/commands/targets.rs`, so the
      fourth producing source is not left unplumbed
- [x] T035 [US3] Update `print_target` in
      `crates/fragcap-cli/src/commands/targets.rs` to report the coverage state
      on `targets show`, so the detail view and the listing agree

## Phase 6: Polish and cross-cutting

- [x] T036 [P] Add glossary entries for *binary marker*, *coverage state*, and
      *sensitivities* to the topic files under `docs/glossary/` that already own
      the neighbouring terms (`anti-cheat-and-security.md` for the marker,
      `command-line-and-diagnostics.md` for the two column terms), and link them
      from `docs/glossary/index.md` (P-6)
- [x] T037 [P] Update `docs/fragcap-specification.md` to match what ships (P-11):
      the Appendix B signature rows, the store schema version, and the hero
      listing section (around line 2678), whose worked example and prose both
      still describe a single `KNOWN` column
- [x] T038 [P] Write `changelog.d/065-detection-truthfulness.md` describing the
      user-visible changes: the DRM label correction, the two new engines, and
      the column split as a listing output change
- [x] T039 [P] Write `changelog.d/065-detection-truthfulness.decisions.md`
      carrying D-2 (the dual-detector invariant, with option (a) and why it was
      rejected) and D-3 (the coverage column and the schema bump), each dated
- [x] T040 Re-read every FR in [spec.md](./spec.md) against what was actually
      built, checking the artifact rather than the source: the rendered table
      width from a real run, the export JSON from a real command, and the
      compiled string constants, and record any gap found
- [x] T040a Run the quickstart Tier 2 checks on the operator's machine and
      record the result with its date against SC-001 and SC-002: no DRM product
      on arc_raiders, barotrauma, shale_hill_secrets, or
      trapped_with_ivy_piper; Steam DRM still on detroit_become_human, palworld,
      and enshrouded; Ren'Py and GameMaker named. If the machine is not
      available, say so rather than reporting it green
- [x] T041 Run `cargo xtask ci` in the foreground and watch it to completion

## Dependencies

```text
Phase 1 (T001-T002)
   |
Phase 2 (T003-T012)  foundational vocabulary and schema
   |
   +--> Phase 3 (T013-T020)  US1, needs T005 and T007
   |
   +--> Phase 4 (T021-T024)  US2, needs T008
   |
   +--> Phase 5 (T025-T035)  US3, needs T009-T012 and T016
                             T032/T033 also read US1 and US2 output
   |
Phase 6 (T036-T041)
```

Phases 3 and 4 are independent of each other once phase 2 lands. Phase 5's
rendering tasks are best done after both, because the columns are only
demonstrably worth having when the data behind them is correct, which is the
sequencing #174 states for itself.

## Parallel opportunities

- T008 runs alongside T003 to T007 (a different file, no shared type).
- Phase 3 and phase 4 run in parallel after phase 2.
- Within phase 4, T021 runs alongside T022 (test and asset).
- Within phase 6, T036 to T039 are four independent files.

## Independent test criteria

| Story | Criterion | Needs the operator's machine? |
| --- | --- | --- |
| US1 | Two generated PE fixtures produce opposite DRM answers | No |
| US2 | Two fixture trees name Ren'Py and GameMaker; the subset check fails on an unsignatured engine | No |
| US3 | Three registered rows render three distinct markers; export carries the same partition and state | No |

## Implementation strategy

US1 alone is a shippable improvement: it removes a label that is wrong on 28 of
32 rows and replaces it with one that is right on the measured sample. That is
the MVP. US2 makes the engine column worth reading. US3 is the presentation
change that the first two make worthwhile, and is deliberately last for that
reason.

## Audit record (T040)

Every FR re-read against what was built, checking artifacts rather than the
source that produced them. Two real defects were found and fixed before the
halt, both by looking at output rather than at code:

1. **The 80 column claim was false.** research.md R-8 and SC-006 said a
   representative row fit in 74 columns, on the strength of a 22 character
   handle taken from the sketch in #174. Rendering the real listing showed the
   longest handle is 47 characters and rows run to 100. The budget is restated
   over the columns the tool controls, the overflow is now its own test, and the
   correction is recorded in the spec, research, plan, and decisions rather than
   quietly applied.
2. **The truncation warning was written twice and tested zero times.** The
   analyze gate raised it (U1) and it was implemented at both call sites by hand,
   which is a drift surface with no test on either. Folded into one
   `ScanOutcome::coverage_warnings`, and covered.

Evidence checked as artifacts rather than source:

| Claim | Checked by |
| --- | --- |
| The retired readiness sentences are gone | byte scan of the built `fragcap.exe`, with a control string to prove the scan works |
| The new headers and markers ship | same byte scan |
| The subset check bites | removed the Ren'Py rows and watched the test fail with an actionable message, then restored them |
| SC-001 and SC-002 | a real discovery run on the operator machine, 2026-08-20, per the table below |
| Every producing source is plumbed (FR-015) | four real runs: `targets scan` with and without a catalog, `targets add --exe` with and without a marker, and `targets add` with no exe |
| No unplumbed writer remains | every non-test `TargetEntry` writer enumerated; `promote_target_launch` correctly leaves the field alone, because promotion runs no scan |

Tier 2 result, operator machine, 2026-08-20:

| Title | before | after |
| --- | --- | --- |
| Detroit Become Human | Steam DRM | Steam DRM |
| Palworld | Steam DRM | Unreal, Steam DRM |
| Enshrouded | Steam DRM | Steam DRM |
| ARC Raiders | Steam DRM | Unreal, no DRM |
| Barotrauma | Steam DRM | no engine, no DRM |
| Shale Hill Secrets | Steam DRM | GameMaker, no DRM |
| Trapped with Ivy and Piper | Steam DRM | Ren'Py, no DRM |

All three coverage markers were observed in that run: `-` on a scanned-clean
row, `incomplete` on two rows whose install directories could not be read, and
`not scanned` on a row registered without a scan.
