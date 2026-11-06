# Research: v0.10.14 Release Stabilization and Candidate

## Decision 1: Correct the controlled test budget, not production timing

**Decision**: Change only `steam_client_setup_reports_completion_then_reaches_fresh_calibration` from a one-second launch allowance to seven seconds, matching adjacent controlled calibration regressions. Retain its one-second observation duration and all production defaults, parsing, finite bounds, and operator-supplied deadline handling.

**Rationale**: Hosted attempt 42174601003 reached controlled startup after about 1.5 seconds, so the test was asserting runner scheduling rather than the setup-to-calibration contract. Adjacent end-to-end controlled calibration tests already use seven seconds. Existing parser and ownership tests prove that explicit shorter deadlines remain exact and cannot exceed the session launch deadline.

**Alternatives considered**: Increasing a production default was rejected because the defect is test orchestration. Removing the deadline was rejected because controlled work must remain finite. Retrying a failed test was rejected because it would hide the same race.

## Decision 2: Prepare one patch candidate on the required release branch

**Decision**: Use `release/0.10.14`. Because Spec-Kit records are created before version mutation, execute the release script's documented constituent operations on the existing branch after its dry-run preview rather than asking the script to create a second branch.

**Rationale**: `release.toml` permits only `release/*`, and the release script normally creates that branch from clean main. Constituent execution preserves the same version, generated-output, changelog, notes, and validation authorities without conflicting with the already active Spec-Kit branch.

**Alternatives considered**: A `codex/*` branch would violate release policy. Running the script's mutating mode from the existing branch would fail its own branch-creation precondition. Skipping repository automation would risk incomplete candidate identity.

## Decision 3: Repair invalid fragments before changelog assembly

**Decision**: Convert the S193 and S194 decision fragments from Markdown headings to the required bold dated lead, add the S195 decision record, then let the changelog command consume every fragment exactly once.

**Rationale**: The release dry-run found a real validator failure before version mutation. Correcting the authoritative fragments preserves validation and chronology.

**Alternatives considered**: Bypassing or weakening the validator was rejected. Hand-copying fragment contents into the changelog while leaving invalid sources was rejected because it creates duplicate authority.

## Decision 4: Advance candidate identities while preserving public identities

**Decision**: Advance workspace packages, embedded test identity, generated goldens, conformance records, specification Applies-To, changelog, notes, and candidate documentation to v0.10.14. Keep `docs/published-release.json`, frozen published-review candidate records, public downloads, registry references, and current-public documentation on verified v0.10.13.

**Rationale**: Candidate source and public bytes are distinct evidence. Advancing public markers before publication would make them false.

**Alternatives considered**: Advancing every occurrence of v0.10.13 was rejected because historical and current-public records must continue to name the bytes they verify.

## Decision 5: Audit every locked dependency authority without waiver

**Decision**: Refresh first-party version identities in the workspace and isolated lockfiles, refresh the supply-chain inventory through its repository command, and run advisory, license, ban, and source checks for every locked graph.

**Rationale**: A workspace-only check can miss fuzzing, performance, or packaging graphs. A patch candidate must not silently resolve a different third-party graph.

**Alternatives considered**: Auditing only the root lockfile was rejected. Blanket advisory or license waivers were rejected.

## Decision 6: Keep review and publication bounded

**Decision**: Automatic hosted review activity is round one. If needed, send exactly one explicit Codex request for round two after first-round findings are reconciled. Answer every actual comment and resolve every thread. Stop when the final PR head has successful applicable checks and exact-head package evidence.

**Rationale**: This matches the owner's review cap and preserves the owner-controlled merge and publication sequence.

**Alternatives considered**: Repeated review requests were rejected by the explicit cap. Treating a reaction or absent comment as a detailed review was rejected because evidence must be literal.
