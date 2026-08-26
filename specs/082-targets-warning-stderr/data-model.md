# Data Model: Targets Warning Stream Contract

## Command Result

Represents bytes that are the requested result of a targets command.

- Listing table and footer.
- Empty-listing guidance.
- Discovery candidate rows and account summary.
- Registration counts.
- Target detail, ambiguity, import, export, remove, and no-match output.

Validation rules:

- Remains on standard output.
- Is not affected by warning presence.
- Is not suppressed by warning verbosity gates.

## Diagnostic Warning

Represents a non-fatal condition the operator may need to see, but which is not part of the command result.

- Catalog bootstrap warning.
- Discovery skipped warning.
- Discovery coverage warning.
- Steam enumeration warning.
- Detection read or coverage warning.

Validation rules:

- Routes through the shared emitter.
- Appears on standard error in normal and quiet human mode.
- Is suppressed in silent mode.
- Appears as a structured warning diagnostic in JSON mode.

## Stream Contract

Represents the separation between result output and diagnostics.

- `stdout`: command result.
- `stderr`: warning and error diagnostics.

Validation rules:

- Warning-producing and warning-free runs over the same result state have byte-identical standard output.
- No warning line is written directly to standard output.
