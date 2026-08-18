# Implementation Plan: doctor gains an action layer (--fix)

**Branch**: `056-doctor-action-layer` | **Date**: 2026-08-18 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from
`/specs/056-doctor-action-layer/spec.md`

## Summary

Add an action layer above the existing pure `doctor` classifier. `fragcap doctor`
is unchanged. A new `fragcap doctor --fix` runs the same classifier, prints the
same report, then offers to perform the remediations the report named, one at a
time, under the operator's confirmation. Each actionable `Check` gains an optional
structured action bound to it so the printed remediation and the offered action
cannot drift; `--fix` may act only on actions carried by a check present in the
current report. `--fix` is refused with `--json` and when stdout is not a terminal;
`--yes` pre-confirms for unattended interactive use. Two findings the classifier
does not yet name (catalog store missing, no target entries) are surfaced as new
additive pure checks. Network-dependent actions (npcap installer fetch, catalog
fetch) are gated on the `net` capability and degrade to naming the manual step in a
default build.

The whole change is confined to `fragcap-cli`. It reuses existing capabilities
(`extcap install`, the S055 discovery composition, `catalog update`) rather than
reimplementing them. Two governance items are settled in research: the operator
authorized amending constitution Licensing rule 2 to permit a user-confirmed fetch
of the vendor's own installer (recorded there and applied as a task), and the
npcap license was read and determined to permit fetching and launching the
vendor's own installer without redistribution.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82 (default build), pinned toolchain
for `net`/`live` builds.

**Primary Dependencies**: existing only. The action layer adds no new crate. The
npcap installer fetch and the catalog fetch reuse `http_req` behind the existing
`net` feature (already in `fragcap-targets`; the CLI reaches network fetch through
existing command paths). No new `Cargo.lock` delta is expected.

**Storage**: SQLite via existing stores (`catalog.db`, `local.db`); read only for
the target-entry-count probe. No schema change.

**Testing**: `cargo test` (Tier 1, no capture driver, no elevation, no network);
platform side effects (launch installer, relaunch elevated) are Tier 2, stated not
hidden, per the S010 precedent. GNU toolchain locally
(`cargo +1.96.0-x86_64-pc-windows-gnu ...`), MSVC `cargo xtask ci` in CI.

**Target Platform**: Windows (native). The classifier and action-selection logic
are platform-neutral and run on any target; the side effects are Windows.

**Project Type**: CLI (single Rust workspace, the `fragcap-cli` crate).

**Performance Goals**: not performance sensitive; `doctor` is an interactive
one-shot.

**Constraints**: classifier purity preserved (FR-002); `--fix` acts only on
report-named actions (FR-003); P-1 and P-9 bind; house text hygiene (UTF-8 no BOM,
LF, no em/en dashes).

**Scale/Scope**: one new CLI flag pair (`--fix`, `--yes`), one action-layer module,
two new additive checks plus their probe inputs, six wired actions, a confirmation
seam with a console impl and a scripted double, glossary entries, a changelog
decisions fragment, and a constitution amendment.

## Constitution Check

*GATE: evaluated before Phase 0 and re-evaluated after Phase 1 design.*

- **P-1 Passive Observation (NON-NEGOTIABLE)**: PASS. `--fix` modifies the local
  environment only through named, confirmed actions (install a driver via the
  vendor's installer, register the extcap source, fetch fragcap's own catalog,
  relaunch fragcap elevated, run discovery). It never touches traffic and never
  opens a target process. No packet interception, injection, hooking, handle, LSP,
  or image modification is introduced.
- **P-2 Core Stays Platform-Neutral**: PASS. All changes are in `fragcap-cli`.
  `fragcap-core` is untouched. The action-selection logic is platform-neutral and
  tested without the platform.
- **P-3 Capture and Attribution Stay Separate**: PASS. Not touched.
- **P-4 No Silent Loss**: PASS. Every action outcome is reported
  (performed/skipped/degraded/failed); a failed action is never reported as success.
- **P-5 Compatibility Outranks Richness**: PASS. Output format of `doctor` itself is
  unchanged; `--fix` adds an interactive phase after the existing report.
- **P-6 Glossary First**: PASS with task. New terms (action layer, structured
  action, action outcome) get glossary entries in the same change (FR-018, a task).
- **P-7 Wrappers Stay Thin**: PASS. No wrapper parses output; the action layer is
  native Rust in the CLI.
- **P-8 House Standards Apply**: PASS with discipline. Text hygiene enforced.
- **P-9 The Instrument Does Not Lie (NON-NEGOTIABLE)**: PASS. The classifier stays
  pure and honest; `--fix` never fabricates an outcome and reports a failed action
  as failed.
- **P-10 One Path To A Target**: PASS. The discovery action reuses the S055
  discovery composition (one registration path); it adds no second path.
- **P-11 The Specification Describes What Shipped**: PASS with task. The master
  specification's `doctor` section (26.3) and the Licensing section are updated to
  describe the action layer and the amended rule 2; `cargo xtask spec` lock-step is
  respected.

- **Licensing (Third-Party Obligations)**: CONDITIONAL, resolved in research and
  tracked in Complexity Tracking. The unamended rule 2 ("never downloads, installs,
  or invokes an installer") forbids the npcap fetch action. The operator authorized
  amending rule 2 (a narrow, user-confirmed, vendor-installer-only carve-out that
  preserves rules 1, 3, 4 and P-1/P-9). The amendment (constitution 1.2.0 -> 1.3.0)
  is a task in this slice; the npcap license was read and permits the fetch (no
  redistribution). Until the amendment lands, the design's default (open the
  download page) is already compliant, so no code depends on the forbidden behavior
  before the rule permits it.

Gate result: PASS, with the Licensing carve-out recorded in Complexity Tracking and
executed as tasks.

## Project Structure

### Documentation (this feature)

```text
specs/056-doctor-action-layer/
|-- plan.md              # This file
|-- spec.md              # Feature spec
|-- research.md          # Phase 0 output
|-- data-model.md        # Phase 1 output
|-- quickstart.md        # Phase 1 output
|-- contracts/           # Phase 1 output
|   |-- doctor-fix-cli.md
|   |-- action-catalog.md
|   `-- confirm-seam.md
|-- checklists/
|   |-- requirements.md
|   `-- action-layer.md
`-- tasks.md             # Phase 2 (/speckit-tasks; not this command)
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/
|-- doctor/
|   |-- mod.rs           # Inputs, Check, Report, Status (EXTENDED: optional action
|   |                    #   on Check; two new Inputs fields; unchanged rendering)
|   |-- checks.rs        # pure classifiers (EXTENDED: two new additive checks;
|   |                    #   existing checks and tests UNMODIFIED)
|   |-- probe.rs         # thin real-input gatherer (EXTENDED: gather the two new
|   |                    #   facts; not unit tested)
|   |-- action.rs        # NEW: Action, ActionKind, ActionOutcome, the pure
|   |                    #   selection of offered actions from a Report
|   `-- fix.rs           # NEW: the --fix driver: refusal rules, the confirm loop,
|                        #   performing actions, the final re-run and verdict
|-- commands/doctor.rs   # thin shell (EXTENDED: route --fix / --yes / --json)
|-- cli.rs               # DoctorArgs (EXTENDED: --fix, --yes)
|-- sources/             # NEW small module or reuse: the ActionConfirm seam
|                        #   (console impl + scripted double)
`-- ...

crates/fragcap-cli/tests/
`-- cli_doctor.rs        # EXTENDED: refusal rules, action selection, confirm loop
                         #   with a scripted double, degradation in default build

docs/glossary/           # EXTENDED: action layer, structured action, action outcome
docs/fragcap-specification.md  # EXTENDED: section 26.3 + Licensing
.specify/memory/constitution.md # EXTENDED: rule 2 carve-out, 1.2.0 -> 1.3.0
changelog.d/             # NEW: S056 added fragment + npcap/rule-2 decisions fragment
```

**Structure Decision**: Single crate (`fragcap-cli`). The classifier stays where it
is and is only extended additively. The action layer is two new modules
(`action.rs` for the pure selection, `fix.rs` for the driver) plus a confirm seam,
keeping the pure decision logic separable from the side effects for Tier 1 testing.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|--------------------------------------|
| Amend constitution Licensing rule 2 (non-negotiable) to permit a user-confirmed fetch of the vendor's own npcap/Wireshark installer | Issue #143's action table lists fetch+launch as the npcap-absent action; the operator chose to enable it rather than ship link-only or defer. Rule 2 as written forbids it. | Keeping rule 2 absolute (open-page only) was offered and declined by the operator; deferring the npcap action entirely was offered and declined. The carve-out is scoped to the vendor's own installer under explicit interactive confirmation, preserves rules 1/3/4 and P-1/P-9, and is recorded with a dated decision and a version bump. |
