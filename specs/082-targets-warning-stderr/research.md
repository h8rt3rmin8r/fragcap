# Research: Targets Warning Stream Contract

## Decision: Reuse the shared emitter for targets warnings

**Decision**: Add an `Emitter` parameter to targets command entry points and call `Emitter::warn` for every targets warning.

**Rationale**: The emitter already implements the project-wide diagnostic contract: warnings go to stderr, quiet keeps them, silent suppresses them, and JSON mode renders them as NDJSON diagnostic records. Reusing it fixes all modes at once and keeps wrapper logic thin.

**Alternatives considered**:

- Add `err: &mut dyn Write` to targets and write human warning lines manually. Rejected because it would bypass `--json`, `--quiet`, and `--silent`.
- Keep warnings on stdout for commands whose output is already human-oriented. Rejected because the library-level contract says all warnings use stderr, and piped listings are the failure mode.

## Decision: Treat only diagnostic warnings as warnings

**Decision**: Move lines whose purpose is diagnostic warning output. Keep result facts such as discovery rows, technology evidence, accounts, registration summaries, and target records on stdout.

**Rationale**: The defect is stream contamination by diagnostics, not the presence of discovery results. Moving result facts to stderr would make `--quiet` and `--silent` change command results and would make the tool under-report what it observed.

**Alternatives considered**:

- Move all discovery text to stderr. Rejected because `targets discover` is an inspection command whose discovery rows and account are its result.
- Suppress warning causes entirely when a row already renders `incomplete`. Rejected by P-4 because the counted incomplete state must be recoverable to a cause.

## Decision: Include doctor-triggered targets discovery in the fix

**Decision**: Pass the emitter into `run_discovery_default` so target discovery warnings emitted during `doctor --fix` follow the same diagnostic stream contract.

**Rationale**: The stream contract is library-wide, and `doctor --fix` calling a targets helper should not create a second exception.

**Alternatives considered**:

- Scope the slice only to direct `fragcap targets` invocations. Rejected because it leaves a known target warning leak in a command path that already has an emitter.
