# Feature Specification: Doctor Progress And Timing

**Feature Branch**: `codex/079-doctor-progress-timing`

**Created**: 2026-08-26

**Status**: Draft

**Input**: User description: "Implement issue #202: `fragcap doctor` is silent for about 30 seconds on first run; measure probe costs before optimizing and report progress as probes run, while keeping existing report output and JSON surfaces unchanged."

## Clarifications

### Session 2026-08-26

- Q: Should this slice optimize the suspected npcap and ETW costs from #203 and #204? A: No. This slice measures each probe and reports progress. #203 and #204 remain separate follow-up fixes that use the recorded evidence.
- Q: Where does progress appear? A: Progress is an interactive standard-error surface only when the human report is going to a terminal. Redirected human reports and `--json` remain byte-identical to the current machine/report surfaces.
- Q: How is timing exposed? A: A hidden `--timings` option includes per-probe elapsed times on the interactive progress surface and leaves the final human and JSON reports unchanged.

## User Scenarios & Testing

### User Story 1 - See Doctor Working Immediately (Priority: P1)

An operator runs `fragcap doctor` as the first command after installation and sees visible progress promptly instead of a silent terminal that looks hung.

**Why this priority**: `doctor` is the readiness authority and the first command a new user runs. A long silent probe phase is a P-9 truthfulness problem because the operator cannot distinguish work from deadlock.

**Independent Test**: Run `fragcap doctor` in a terminal-like test harness with injected slow probes. The command writes visible progress before the slow probe completes and continues updating as named probes run.

**Acceptance Scenarios**:

1. **Given** the human `doctor` report is directed to a terminal, **When** probe gathering begins, **Then** visible progress appears before any slow probe can complete.
2. **Given** multiple probes run in sequence, **When** each probe begins, **Then** progress names the current check in operator vocabulary.
3. **Given** a probe is slow, **When** the operator observes the terminal, **Then** the last visible progress item identifies the work currently in progress.

---

### User Story 2 - Preserve Stable Report Surfaces (Priority: P1)

A script or analyzer automation that consumes `doctor --json` or redirected human output receives exactly the same report bytes as before this slice.

**Why this priority**: The report renderer and JSON records are stable command results. Progress is a separate stream concern and must not pollute data surfaces.

**Independent Test**: Compare `doctor --json`, redirected human output, and golden human reports before and after adding progress. The command-result bytes are unchanged.

**Acceptance Scenarios**:

1. **Given** `fragcap doctor --json`, **When** doctor runs, **Then** standard output contains only the existing JSON records and no progress or timing records.
2. **Given** `fragcap doctor > file`, **When** doctor runs, **Then** the file contains the same human report bytes as before this slice.
3. **Given** the existing doctor golden tests, **When** they run, **Then** their expected report bodies remain unchanged.

---

### User Story 3 - Attribute Probe Cost With Evidence (Priority: P2)

A maintainer can obtain per-probe timings and use the slice record to decide whether #203 or #204 should be optimized next.

**Why this priority**: The suspected dominant costs are plausible but unmeasured. Fixing the wrong probe would waste effort and could weaken doctor truthfulness.

**Independent Test**: Run doctor with timings enabled against injected probes and on the local Windows environment. The output attributes elapsed time to named probes, and the slice records the observed dominant cost or states why it could not be measured.

**Acceptance Scenarios**:

1. **Given** timings are explicitly requested on an interactive human run, **When** each probe completes, **Then** its elapsed time is visible beside the progress item.
2. **Given** timings are not requested, **When** progress is shown, **Then** the operator sees check names and outcomes without timing noise.
3. **Given** local measurement is unavailable or not representative, **When** the slice is completed, **Then** the slice record states the limitation rather than claiming an unobserved dominant cost.

### Edge Cases

- `--quiet` suppresses progress while preserving warnings and errors.
- `--silent` suppresses progress and all non-error output.
- `--json` suppresses progress and timings entirely so JSON remains a machine surface.
- Standard output redirected to a file suppresses progress and timings even when standard error is a terminal, preserving the redirected human report.
- A probe that fails still records the failure in the final report and resolves its progress item honestly.
- A probe that cannot determine a fact remains unknown or failed as today; the feature must not replace a slow honest answer with a fast fabricated one.
- Timing measurement overhead must not materially change readiness results.

## Requirements

### Functional Requirements

- **FR-001**: `fragcap doctor` MUST expose a probe-progress surface for interactive human report runs before the final report is rendered.
- **FR-002**: The progress surface MUST name each long-running readiness probe as it begins using operator-facing check names.
- **FR-003**: The progress surface MUST update each named probe with an honest terminal state when that state is known.
- **FR-004**: The first visible progress output MUST be emitted within 200 milliseconds of command dispatch under an injected slow-probe scenario, excluding process startup time.
- **FR-005**: Progress output MUST be written to standard error and MUST NOT be part of the final human report text.
- **FR-006**: `doctor --json` MUST emit no progress output, timing output, or additional JSON records beyond the existing report records.
- **FR-007**: A human report whose standard output is not a terminal MUST remain byte-identical to the current redirected report output and MUST suppress progress.
- **FR-008**: Existing human and JSON report renderers MUST remain the authority for final doctor results.
- **FR-009**: A hidden `--timings` option MUST make per-probe elapsed times obtainable for interactive human runs without changing final human or JSON report bytes.
- **FR-010**: Timings MUST be associated with the same operator-facing probe names used by progress output.
- **FR-011**: Timing measurement MUST distinguish at least npcap driver/interface enumeration, ETW process-tracing availability, extcap integration lookup, store checks, Deep Capture readiness checks, and final report rendering.
- **FR-012**: The slice record MUST state the measured dominant local probe cost, or state the concrete limitation that prevented a representative measurement.
- **FR-013**: The fix MUST NOT skip, lazily omit, or replace any existing doctor probe result unless the final report explicitly says the value was not observed.
- **FR-014**: `--quiet` and `--silent` MUST suppress doctor progress consistently with their existing output-suppression semantics.
- **FR-015**: Tests MUST prove progress appears on the interactive path, progress is absent from `--json` and redirected human output, doctor goldens stay unchanged, and per-probe timings are obtainable.
- **FR-016**: The master specification section governing diagnostics MUST be reconciled with the new progress and timing contract in the same change.
- **FR-017**: Changelog fragments MUST record the user-visible progress fix and any dated measurement/implementation decision worth preserving.

### Key Entities

- **Probe Progress Item**: A named readiness check shown while doctor gathers inputs, with state `running`, `complete`, `failed`, or `unknown`.
- **Probe Timing**: The elapsed wall-clock duration attributed to one progress item when timings are explicitly requested.
- **Doctor Report**: The final human or JSON readiness report, unchanged by this slice.

## Success Criteria

### Measurable Outcomes

- **SC-001**: In an injected slow-probe test, interactive `doctor` emits a visible progress item within 200 milliseconds of the first probe beginning.
- **SC-002**: `doctor --json` output is byte-for-byte unchanged from its pre-slice expected records in automated coverage.
- **SC-003**: Redirected human `doctor` output and existing doctor golden reports are byte-for-byte unchanged.
- **SC-004**: At least six named probe timings are obtainable from an interactive timing-enabled run.
- **SC-005**: The completed slice records local timing evidence for #202 and clearly identifies whether #203 or #204 appears to be the dominant measured cost, or why local evidence was inconclusive.
- **SC-006**: The full repository CI gate passes, or any unavailable platform/toolchain component is reported with the exact command and observed result.

## Assumptions

- The existing doctor classifier and report renderers are correct enough for this slice; changing readiness semantics is out of scope.
- The progress surface can reuse or align with the live capture status infrastructure, but exact implementation belongs to planning.
- The hidden `--timings` option is intended for maintainers and issue triage, not for a stable public machine contract.
- #203 and #204 remain open until measurement justifies and a separate slice changes their probe implementations.
- No new glossary term is expected; existing entries for diagnostics, Event Tracing for Windows, npcap, and readiness checks cover this slice.
