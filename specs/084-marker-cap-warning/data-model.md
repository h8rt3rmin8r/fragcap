# Data Model: Marker Cap Warning Subject

## Scanned root

Represents the directory passed to technology detection for one scan.

Fields:

- `path`: Displayable filesystem path of the scanned directory.

Validation rules:

- Must be captured when a scan succeeds and a `ScanOutcome` is produced.
- May be relative or absolute, matching the caller's scan input.
- Must not be synthesized from a target handle, row number, or catalog identity.

## Marker-cap warning

Represents the human warning emitted when binary-marker detection skipped executable candidates because the candidate count cap was reached.

Fields:

- `root`: Scanned root that owns the reduced coverage.
- `max_read`: Number of executable candidates read before the cap.
- `skipped`: Number of executable candidates not examined.
- `consequence`: Statement that technology detection for this root may be incomplete.

Validation rules:

- `skipped` must be exact.
- `root` must be present in every marker-cap warning.
- The warning must remain one line.

## Coverage warning list

Represents every reduced-coverage diagnostic for one scan outcome.

Fields:

- `warnings`: Ordered human-readable warning strings.

Validation rules:

- Unreadable subtree warnings and marker-cap warnings must both be emitted when both causes exist.
- The set is returned by the shared scan outcome helper so callers receive the same cause list.
