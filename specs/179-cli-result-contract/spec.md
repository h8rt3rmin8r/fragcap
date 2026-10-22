# Feature Specification: Consistent CLI Result Output

**Feature Branch**: `codex/s179-cli-result-contract`

**Created**: 2026-10-22

**Status**: Draft

**Input**: Owner-approved S179 for GitHub issue #488, aligning global JSON, quiet, and silent flags with actual command results.

## Clarifications

### Session 2026-10-22

- Q: Which flag wins when `--json --silent` is selected? A: The explicitly selected machine result or event stream remains, while optional human text stays suppressed; hiding the machine stream would turn an automation request into an unexplained empty success.
- Q: Do JSON-export and protocol commands gain a new result envelope? A: No. Their existing JSON document or extcap protocol is the authoritative machine artifact and remains unchanged.
- Q: Can silent hide consent, cleanup obligations, or errors? A: No. Those are required action and failure facts, separate from optional status or ordinary human success output.

## User Scenarios & Testing

### User Story 1 - Structured command results (Priority: P1)

A script invokes a non-capture command with `--json` and receives machine-readable success or failure records with stable identities and typed fields instead of human prose. Existing commands that intentionally output JSON documents retain those documents as their stdout artifact.

**Why this priority**: Issue #488 reports that advertised structured output is absent from common target, catalog, and schema workflows, so automation cannot consume results reliably.

**Independent Test**: Run controlled target list/show, catalog import/seed, and schema validation commands with `--json`; parse each stdout line as JSON and assert typed fields and the original exit class.

**Acceptance Scenarios**:

1. **Given** an empty or populated temporary target store, **When** the operator requests list or show with `--json`, **Then** stdout contains structured target results and no human table, including explicit empty and missing-selection outcomes.
2. **Given** a valid offline catalog document, **When** import or seed runs with `--json`, **Then** stdout reports actual imported or filled counts, skipped tiers and reasons, and the destination store in structured records.
3. **Given** a profile with several independent defects, **When** schema validate runs with `--json`, **Then** each diagnostic has its own code, path, line, column, and message fields, followed by a summary, and the command retains usage exit 2.
4. **Given** a JSON-producing export or schema-print command, **When** `--json` is also present, **Then** the exported document remains a single valid JSON document without a result envelope or human line mixed into it.

### User Story 2 - Predictable verbosity (Priority: P2)

An operator can choose quiet or silent output without losing required authorization text, error diagnostics, or the selected machine protocol stream. The global help accurately states the precedence when flags are combined.

**Why this priority**: `--silent` currently allows ordinary human command results through, while Capture emits structured events even when silent is set.

**Independent Test**: Compare stdout and stderr for the same controlled commands under default, quiet, silent, JSON, and JSON plus silent; assert exactly which results, events, warnings, prompts, and errors remain.

**Acceptance Scenarios**:

1. **Given** a successful non-interactive command, **When** `--quiet` is selected, **Then** its final result remains while optional progress is suppressed; **When** `--silent` is selected without JSON, **Then** ordinary human success output and optional warnings are suppressed.
2. **Given** `--json --silent`, **When** a command completes or fails, **Then** its selected structured result or error stream remains parseable and no human prose is mixed into it.
3. **Given** Capture, Deep Capture, or calibration with `--json`, **When** `--silent` is also present, **Then** the existing stderr lifecycle events remain the selected machine stream, including terminal truth and required actions, and no capture sink bytes are written by the CLI result layer.
4. **Given** an interactive authorization or mandatory cleanup action, **When** `--silent` is selected, **Then** required consent and action text remains visible; errors remain visible for every verbosity.

### User Story 3 - Complete command matrix (Priority: P3)

The global flags have a documented result contract for every command family that accepts them, including Doctor's existing structured stdout, targets, catalog, schema, technologies, native capture commands, and protocol or document exporters.

**Why this priority**: A partial repair would leave the same promise false on a less common command.

**Independent Test**: A command/flag matrix enumerates every top-level command family and representative subcommands, checks output channel and JSON validity, and compares human/default behavior with quiet and silent behavior.

**Acceptance Scenarios**:

1. **Given** Doctor's read-only report, **When** JSON or JSON plus silent is selected, **Then** its existing structured stdout check records and readiness facts remain intact.
2. **Given** a command that emits a protocol or JSON document by definition, **When** global JSON is selected, **Then** that authoritative artifact is preserved exactly rather than wrapped or partially duplicated.
3. **Given** an accepted global flag combination, **When** the command is invoked, **Then** it follows its declared result policy; any genuinely unsupported combination fails as a usage error before side effects and names the command and unsupported flag.

### Edge Cases

- A malformed JSON profile and multiple schema violations retain distinct usage errors without flattening known diagnostic fields.
- A missing input file or inaccessible store yields a structured error on stderr in JSON mode, does not print a false success record, and preserves the existing 1 or 2 exit class.
- A no-match target selection and an empty target list are distinct results; neither manufactures a target row.
- JSON result writes are complete newline-delimited records; a failed write must not be reported as a successful result.
- Paths and names may contain spaces, Unicode, terminal controls, or secrets; JSON escaping preserves exact values and diagnostics do not add credential or capture-payload material.
- Source warnings and skipped tiers remain counted or named under JSON mode rather than disappearing with the human renderer.

## Requirements

### Functional Requirements

- **FR-001**: Every accepted `--json` invocation MUST produce only documented structured records or an existing machine artifact on its designated output channel; human result text MUST NOT contaminate those bytes.
- **FR-002**: Non-capture command results MUST use stdout, while errors and warnings use stderr; Capture, Deep Capture, calibration, and extcap MUST retain their existing event or protocol channel ownership.
- **FR-003**: Target list and show MUST expose stable target identity, exact selection or empty status, and available domain fields in structured output; other target actions MUST expose actual result or documented machine artifact without converting a human table into a JSON string.
- **FR-004**: Catalog import and seed MUST expose actual destination, imported count, tier fills, tier counters, and skipped-tier reasons in structured output; catalog export MUST retain its existing export document shape.
- **FR-005**: Schema validation MUST expose one structured diagnostic record per known defect with code, path, line, column, and message, then a terminal summary; schema print MUST retain the embedded JSON Schema document.
- **FR-006**: Default and quiet modes MUST preserve final human results and required warnings; quiet suppresses optional progress. Human silent MUST suppress ordinary success results and optional warnings while preserving errors and required consent or action text.
- **FR-007**: When `--json` is selected, its machine result or lifecycle stream MUST take precedence over human silent suppression; `--json --silent` MUST not emit optional human text.
- **FR-008**: Doctor's JSON stdout check records, Capture-family stderr lifecycle records, and existing document and extcap protocol outputs MUST retain their field meanings, framing, exit classes, and sink isolation.
- **FR-009**: Any command unable to honor an accepted global combination MUST reject it before side effects with exit 2 and a named reason; no existing machine result mode may be silently removed merely to simplify the implementation.
- **FR-010**: Global and command help, the master specification, and the chronological slice plan MUST state the actual output and verbosity contract without claiming the unreleased change is already published.
- **FR-011**: A controlled command/flag matrix MUST cover targets, catalog, schema, Doctor, Capture, and every remaining top-level family, including default, quiet, silent, JSON, and JSON plus silent where applicable.

### Key Entities

- **CLI result record**: A newline-delimited structured success or failure fact with a stable command and result identity, typed fields, and an existing exit class.
- **Schema diagnostic record**: One validation defect with code, key path, optional source line and column, and message; a summary counts all defects.
- **Machine artifact**: A command's authoritative JSON document or protocol output whose existing schema must remain intact under the global JSON flag.
- **Output policy**: The selected human or JSON shape, verbosity, channel, and required-text exceptions for one invocation.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every matrix row produces the specified exit class, stdout/stderr channel, and parseable result shape, with no human text mixed into JSON output.
- **SC-002**: A controlled profile with at least four independent defects yields four distinct diagnostic records and one summary with matching count.
- **SC-003**: Controlled target and catalog results report exact observed identities, fill counts, and omission reasons without inventing rows or completed work.
- **SC-004**: Existing Capture stderr events, Doctor stdout records, export documents, and extcap protocol fixtures remain byte or schema compatible where this slice does not change their declared contract.
- **SC-005**: The repository's local CI parity gate and hosted exact-head checks pass without live capture, game launch, or real operator data.

## Assumptions

- Issue #488 is the only GitHub issue owned by S179; IGDB #155 remains a separate deferred feature.
- Existing `--json` machine protocols are authoritative; this slice adds missing command result shapes and reconciles flag precedence without rewriting capture evidence or storage schemas.
- `--silent` refers to ordinary human status and success output; selected JSON results and lifecycle events remain available because the operator explicitly requested them. Required consent and cleanup action text remain visible.
- Existing commands that export a JSON document or speak extcap retain that artifact or protocol rather than gaining an extra envelope.
- Controlled temporary stores, fixtures, and injected writers establish implementation acceptance; no real game, trust store, or owner evidence is modified.
