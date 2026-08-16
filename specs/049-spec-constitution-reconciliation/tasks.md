---

description: "Task list for S049: specification and constitution reconciliation"
---

# Tasks: Specification and constitution reconciliation

**Input**: Design documents from `specs/049-spec-constitution-reconciliation/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/checks.md](contracts/checks.md)

**Tests**: Included. The spec's acceptance criteria for the two enforcement
stories (US3, US4) are literally unit tests, and the constitution requires
verification, so `SpecImpact` parsing, the version comparison, the fragment
format check, and the pure release-gate decision are all test-first. The two
document stories (US1, US2) are validated by grep and the quickstart, not unit
tests.

**Organization**: By user story. US1 and US2 are the P1 reconciliation (the
defect fix and the durable rules); US3 is the P2 lock-step gate; US4 is the P3
release-time gate. The shared `spec-impact` field convention and its parser are
factored into Foundational because US3 and US4 both build on them.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: US1..US4, or none for Setup/Foundational/Polish
- All paths are repository-relative.

## Path conventions

Single Rust workspace. Tooling lives in `xtask/src/`; documents in `docs/`,
`.specify/`, and `changelog.d/`; the one workflow file under `.github/workflows/`.
Rust tests live inline as `#[cfg(test)] mod tests` in the module under test, the
existing pattern in `xtask/src/changelog.rs`.

---

## Phase 1: Setup (Shared context)

**Purpose**: Confirm the working state before edits.

- [x] T001 Confirm the branch is `049-spec-constitution-reconciliation` and the working tree is clean apart from `specs/049-spec-constitution-reconciliation/` (`git status`).
- [x] T002 Read `Cargo.toml` `[workspace.package] version` (expected `0.4.0`) and record it as the value `Applies-To` and the version check must agree on.

---

## Phase 2: Foundational (Blocking prerequisites for US3 and US4)

**Purpose**: The `spec-impact` field convention and its parser, shared by the
format check (US3) and the release gate (US4); plus the P-6 glossary obligation
so CI's docs check stays green once the new terms appear.

**CRITICAL**: US3 and US4 cannot be completed until this phase is done.

- [x] T003 Add the `SpecImpact` value type and its parser to `xtask/src/spec.rs` (new file): parse `none` and a comma list of section-number tokens matching `[0-9]+(\.[0-9]+)*`; reject an empty value, a missing comment, or a non-numeric token. Include the SPDX header and module doc comment in the house style.
- [x] T004 [P] Add `#[cfg(test)] mod tests` to `xtask/src/spec.rs` covering `SpecImpact` parsing: accepts `none` and `3.3, 27.3`; rejects empty, `abc`, and a bare `<!-- spec-impact: -->`.
- [x] T005 Add a helper to `xtask/src/spec.rs` that extracts a leading `<!-- spec-impact: ... -->` value from a fragment body string (returns the raw value or None), so the assembler-strip (US4) and the format check (US3) share one reader.
- [x] T006 Document the `spec-impact` field in `changelog.d/README.md`: its purpose, the leading-HTML-comment placement, the `none | section-list` grammar, and that it is stripped from `CHANGELOG.md`.
- [x] T007 Retrofit a `spec-impact: none` line as the first line of the existing fragment `changelog.d/doctor-npcap-delay-load.fixed.md` (it touches no specification section).
- [x] T008 [P] Add glossary entries for `Applies-To` (specification field) and `spec-impact` (changelog fragment field) to `docs/glossary/rust-and-tooling.md`, following the section 4.3 entry template with primary-source references (the specification document-control block and `changelog.d/README.md`).
- [x] T009 Regenerate `docs/glossary/index.md` (via the docs tooling / `cargo xtask docs check` reproducibility path) so the two new entries appear in the alphabetical index.

**Checkpoint**: `SpecImpact` parses and is tested; the field is documented and the
one existing fragment carries it; the glossary is complete.

---

## Phase 3: User Story 1 - The specification describes shipped reality (Priority: P1)

**Goal**: `docs/fragcap-specification.md` describes v0.1.0 through v0.4.0 as
shipped, names v0.5.0 as in progress, carries an `Applies-To` field, and contains
no reference presenting an unreleased version as current or first.

**Independent Test**: `grep -n '^\*\*Applies-To:\*\*' docs/fragcap-specification.md`
prints `0.4.0`; a version-currency grep finds no v0.2.0-as-first-release framing;
the title no longer reads "v0.1.0" and section 28's heading no longer reads
"Roadmap Beyond v0.2.0".

- [x] T010 [US1] Add `**Applies-To:** 0.4.0 \` to the document-control header block of `docs/fragcap-specification.md` (the run of `**Field:** value \` lines), distinct from the document's own `**Version:**` field.
- [x] T011 [US1] Update the document-control history / section 1 in `docs/fragcap-specification.md` to record v0.1.0 through v0.4.0 as shipped with their scope, and identify v0.5.0 as the work in progress; correct the section 1 framing that calls v0.2.0 "the first public release".
- [x] T012 [US1] Rewrite section 3.3 (Success Criteria) in `docs/fragcap-specification.md` so it reflects the real release history rather than v0.2.0 as the first/only functional release.
- [x] T013 [US1] Rewrite section 27.3 (Feature Slices) in `docs/fragcap-specification.md` to reflect shipped slices through v0.4.0 and place this v0.5.0 work correctly (version-currency only, not a re-plan of the slice list).
- [x] T014 [US1] Update section 28 in `docs/fragcap-specification.md`: rename the heading away from "Roadmap Beyond v0.2.0", and update its table-of-contents entry and any inbound intra-document links so the corrected anchor is not left dangling.
- [x] T015 [US1] Replace section 23.1's landing-page paragraph in `docs/fragcap-specification.md` with the verbatim Appendix D text from the handoff plan (transcribed under house text hygiene: no em-dashes/en-dashes).
- [x] T016 [US1] Reconcile the header version signals in `docs/fragcap-specification.md` so none disagree with `Applies-To`: make the document title version-neutral ("fragcap Technical Specification") rather than "fragcap v0.1.0 Technical Specification", and resolve the standalone `**Version:** 0.1.0` field by either removing it or explicitly relabeling it as the specification document's own revision distinct from `Applies-To` (grep the repo first to confirm nothing parses that field before removing). After this task, `Applies-To` is the sole software-version anchor in the header. Then sweep the remaining document for any other stale version reference presenting an unreleased version as current/first, leaving historically correct references intact.
- [x] T017 [US1] Verify US1: `grep` confirms the `Applies-To` line, the corrected title, the corrected section 28 heading, and zero v0.2.0-as-first-release references; confirm no dangling intra-document anchor remains.

**Checkpoint**: The specification reads as current; the `Applies-To` field exists
so the US3 version check can go green.

---

## Phase 4: User Story 2 - The durable rules survive every future session (Priority: P1)

**Goal**: `.specify/memory/constitution.md` carries P-10 and P-11 verbatim, with
an updated Sync Impact Report and a MINOR version bump.

**Independent Test**: `grep -n 'P-10\|P-11' .specify/memory/constitution.md` finds
both principles; the footer reads `Version: 1.2.0`.

- [x] T018 [US2] Insert principle P-10 (One Path To A Target) and P-11 (The Specification Describes What Shipped) verbatim from the handoff plan Appendix C after P-9 in `.specify/memory/constitution.md`, transcribed under house text hygiene.
- [x] T019 [US2] Prepend a new Sync Impact Report block to `.specify/memory/constitution.md` recording the version change `1.1.0 -> 1.2.0` (MINOR, two principles added), the added principle names, and the reason, following the existing report format.
- [x] T020 [US2] Update the footer line of `.specify/memory/constitution.md` to `**Version**: 1.2.0 | **Ratified**: 2026-08-06 | **Last Amended**: 2026-08-16`.
- [x] T021 [US2] Verify US2: both principles present verbatim, version footer reads 1.2.0, and no template edit is required (the plan-template Constitution Check reads this file live).

**Checkpoint**: The guiding rules the rest of v0.5.0 relies on are in force.

---

## Phase 5: User Story 3 - Drift cannot recur silently (Priority: P2)

**Goal**: `cargo xtask spec` asserts `Applies-To` equals the workspace version and
that every fragment carries a well-formed `spec-impact` line, wired into the `ci`
aggregate and `ci.yml`.

**Independent Test**: `cargo run --package xtask -- spec` exits 0 on the reconciled
tree; flipping `Applies-To` to a different version makes it exit 1 and report both
values; removing a fragment's `spec-impact` line makes it exit 1 and name the
fragment.

**Depends on**: Foundational (T003, T005), and US1 (T010) for the real
`Applies-To` field so the wired check is green.

### Tests for User Story 3 (write first)

- [x] T022 [P] [US3] Add tests to `xtask/src/spec.rs` for `workspace_version()`: it reads `0.4.0` from the root `Cargo.toml` `[workspace.package]` block and does not match `rust-version`.
- [x] T023 [P] [US3] Add tests to `xtask/src/spec.rs` for the version lock-step: equal values pass; unequal fail; an absent/unparseable `Applies-To` is could-not-run (2). Use fixture strings, not the live files.
- [x] T024 [P] [US3] Add tests to `xtask/src/spec.rs` for the fragment-format check: a fragment with a valid `spec-impact` passes; a missing or malformed one fails and is named.

### Implementation for User Story 3

- [x] T025 [US3] Add `workspace_version(root)` to `xtask/src/main.rs` (or `spec.rs`), modeled on the existing `workspace_msrv`: first `version = "..."` line under `[workspace.package]`.
- [x] T026 [US3] Implement the version lock-step in `xtask/src/spec.rs`: read `Applies-To` from `docs/fragcap-specification.md`, compare to `workspace_version`, report both on mismatch; return the 0/1/2 outcome.
- [x] T027 [US3] Implement the fragment-format check in `xtask/src/spec.rs`: every `changelog.d/*.md` except `README.md` begins with a well-formed `spec-impact` comment; name each offender.
- [x] T028 [US3] Add a `run(root) -> io::Result<usize>` entry point to `xtask/src/spec.rs` that aggregates both assertions (count of problems; `Err` for could-not-run), matching the `license::run` shape.
- [x] T029 [US3] Wire `spec` into `xtask/src/main.rs`: add the `mod spec;` declaration, a `"spec"` match arm with the 0/1/2 messages, the `USAGE` line, and a `spec` step in the `"ci"` aggregate after `docs check`.
- [x] T030 [US3] Add the step `- name: Specification version lock-step` running `cargo run --package xtask -- spec` to the `check` job in `.github/workflows/ci.yml`, alongside the other `xtask` steps.
- [x] T031 [US3] Add `changelog.d/S049-ci-spec-check.decisions.md` (a dated `decisions` fragment, leading `<!-- spec-impact: none -->`) recording the ci.yml change per the pinned-artifact rule.
- [x] T032 [US3] Verify US3: `cargo run --package xtask -- spec` exits 0; `cargo test --package xtask spec` passes; a temporary `Applies-To` flip yields exit 1 (then restore).

**Checkpoint**: The alignment is self-enforcing locally and in CI.

---

## Phase 6: User Story 4 - A named section change is backed by a real edit (Priority: P3)

**Goal**: `cargo xtask changelog --release` refuses to assemble when a fragment's
`spec-impact` names a section that the release diff does not back with a
specification edit, and the `spec-impact` comment never reaches `CHANGELOG.md`.

**Independent Test**: `cargo test --package xtask changelog` proves the pure
release-gate decision: a `spec-impact: 23.1` fragment with a changed-file set
lacking `docs/fragcap-specification.md` yields a violation; with it present, none;
`spec-impact: none` never constrains; and the assembler strips the comment.

**Depends on**: Foundational (T003, T005).

### Tests for User Story 4 (write first)

- [x] T033 [P] [US4] Add tests to `xtask/src/spec.rs` for the pure release-gate decision `fn`: over `(fragment_name, SpecImpact)` list plus a changed-file set, a section-naming fragment without the spec path yields a violation naming the fragment; with the spec path present, none; `None` fragments never violate.
- [x] T034 [P] [US4] Add a test to `xtask/src/changelog.rs` proving a leading `<!-- spec-impact: ... -->` line is stripped from an assembled fragment body (it never appears in the assembled output).

### Implementation for User Story 4

- [x] T035 [US4] Implement the pure release-gate decision function in `xtask/src/spec.rs` per data-model.md (inputs: fragments + changed-file set; output: violations), reusing the T005 reader and T003 parser.
- [x] T036 [US4] Extend the fragment-body handling in `xtask/src/changelog.rs` to strip a leading `spec-impact` comment (alongside `strip_leading_section_header`) so it is never emitted to `CHANGELOG.md`.
- [x] T037 [US4] Add the release-gate preflight to the `Mode::Release` path in `xtask/src/changelog.rs`: gather fragments and the release diff (`git diff --name-only <base>..HEAD`, `<base>` = `git describe --tags --abbrev=0 --match 'v*.*.*'`), call the decision function, and fail (exit 1) with named offenders before any rewrite or fragment deletion; exit 2 if git or the base ref is unavailable.
- [x] T038 [US4] Verify US4: `cargo test --package xtask` passes the release-gate and strip tests; a manual `cargo run --package xtask -- changelog --check` shows no `spec-impact` comment in the assembled body.

**Checkpoint**: The reverse drift (a fragment claiming an unbacked spec change) is
blocked at release assembly.

---

## Phase 7: Polish & cross-cutting

**Purpose**: This slice's own changelog fragments, and the whole-gate verification.

- [x] T039 [P] Add `changelog.d/S049-spec-reconciliation.changed.md` (leading `<!-- spec-impact: 1, 3.3, 23.1, 27.3, 28 -->`) describing the specification reconciliation for users.
- [x] T040 [P] Add `changelog.d/S049-spec-lockstep.added.md` (leading `<!-- spec-impact: none -->`) describing the new `Applies-To` field, the `cargo xtask spec` check, and the release gate.
- [x] T041 [P] Add `changelog.d/S049-constitution.changed.md` (leading `<!-- spec-impact: none -->`) noting principles P-10 and P-11 (constitution is not part of the specification, so `none`).
- [~] T042 Run the full gate set: `cargo xtask ci`. PARTIAL in this environment, which has no MSVC linker (no `cl.exe`/`link.exe`), so the workspace-test step and the single `cargo xtask ci` command cannot run here. Every component that does not require the MSVC linker was run and passed: `cargo fmt --all -- --check` (clean), `cargo clippy -p xtask --all-targets` (clean), `cargo check -p xtask --all-targets` (clean), the xtask unit tests (88 passed, via a `x86_64-pc-windows-gnu` build using the on-PATH mingw toolchain, including all 8 new `spec` tests and the changelog strip test), and `cargo xtask spec | lint | deps | license | changelog --check` (all exit 0), plus `scripts/lint-docs.sh check` (pass). Only this slice's crate (`xtask`) has code changes, so the untested non-xtask crates are unaffected. The full `cargo xtask ci` must be run by the operator or CI on an MSVC host.
- [x] T043 Ran the [quickstart.md](quickstart.md) scenarios: 1 (Applies-To/sweep grep), 2 (constitution P-10/P-11, version 1.2.0), 3 (`spec` exit 0; forced divergence exit 1, then restored), 5 (fragment format), and 6 (unit tests) confirmed. Scenarios 4 and 7 (full `cargo xtask ci`) are covered by the component runs in T042; the single-command form needs an MSVC host.
- [x] T044 Final text-hygiene pass over every edited file (spec, constitution, glossary, README, xtask sources, ci.yml, fragments): UTF-8 no BOM, LF, no trailing whitespace, single trailing newline, no em-dashes or en-dashes.

---

## Dependencies & execution order

### Phase dependencies

- **Setup (Phase 1)**: no dependencies.
- **Foundational (Phase 2)**: after Setup. Blocks US3 and US4 (shared parser and field convention). Does NOT block US1 or US2.
- **US1 (Phase 3)** and **US2 (Phase 4)**: both P1, independent of each other and of Foundational; either can start after Setup. US3's wired check depends on US1 (T010).
- **US3 (Phase 5)**: after Foundational and US1 (for a green wired check).
- **US4 (Phase 6)**: after Foundational. Independent of US1/US2/US3 at the code level.
- **Polish (Phase 7)**: after all desired stories; T042/T043 require everything in place.

### Within each story

- Tests (US3, US4) are written before the implementation they cover.
- US1 and US2 are pure document edits with a verify task each.

### Parallel opportunities

- T004 and T008 run in parallel within Foundational (different files).
- US1 (Phase 3) and US2 (Phase 4) can be done in parallel by different people; they share no files.
- Within US3, the test tasks T022/T023/T024 are parallel (same file, but authored together as one test module; treat as a batch).
- Within US4, T033 and T034 touch different files and are parallel.
- Polish fragments T039/T040/T041 are parallel (separate files).

---

## Implementation strategy

### MVP (the two P1 stories)

1. Setup (Phase 1).
2. US1 (spec describes reality) and US2 (constitution P-10/P-11). At this point
   the defect the slice exists for is fixed and the durable rules are in force.
3. STOP and validate: grep checks and a constitution read.

### Incremental delivery

1. MVP (US1 + US2) fixes the drift and adds the principles.
2. Foundational + US3 makes the version drift mechanically impossible and adds the
   fragment-format check to CI.
3. US4 adds the release-time backing check.
4. Polish adds the changelog fragments and runs the whole gate set.

The slice ships all four stories together (they are one PR), but this order lets
each be validated independently as it lands.

---

## Notes

- `[P]` = different files, no dependency on an incomplete task.
- The `Applies-To` value is `0.4.0` for this slice; it becomes `0.5.0` only in the
  future v0.5.0 release commit (not this slice).
- The release gate is file-level (specification changed at all) and does not
  validate section existence, both deliberate per research R-4.
- Only `ci.yml` is a pinned artifact here (T031 decision fragment); `release.yml`
  is intentionally not touched.
- Commit after each logical group; stage only this slice's files, never
  `.specify/feature.json`.
