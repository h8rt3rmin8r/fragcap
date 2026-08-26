# Research: Marker Cap Warning Subject

## Decision: Record the scan root on ScanOutcome

**Rationale**: `coverage_warnings()` is intentionally the shared helper that reports every reduced-coverage cause. The helper currently has the skipped count but not the scan subject. Adding the scanned root to `ScanOutcome` lets every existing caller inherit the improved warning without duplicating wording at each CLI or discovery boundary.

**Alternatives considered**:

- Pass a root argument into `coverage_warnings(root)`. Rejected because callers could pass the wrong root or omit it, recreating the drift the helper was introduced to prevent.
- Format marker-cap warnings in each caller. Rejected because issue #206 exists because callers should not have to remember every reduced-coverage cause.

## Decision: Say Consequence, Not Remedy

**Rationale**: The slice does not add a cap configuration. The warning should therefore say that binary-marker detection skipped executable candidates and technology detection may be incomplete for the named root. It should not tell the operator to raise a cap that no command exposes.

**Alternatives considered**:

- Add a cap flag. Rejected as out of scope and not needed to make the existing warning truthful.
- Keep the current short warning and rely on row `incomplete`. Rejected because it does not recover the loss to a subject.

## Decision: Keep Warning Framing At Callers

**Rationale**: `fragcap technologies` indents warnings under scan output, while targets commands route warnings through the emitter. The shared helper should return the human diagnostic body only; callers retain their current prefix and stream behavior.

**Alternatives considered**:

- Include `warning:` in the shared string. Rejected because it would duplicate prefixes in existing emitters.
