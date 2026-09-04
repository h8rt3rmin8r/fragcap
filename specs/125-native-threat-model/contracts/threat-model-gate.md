# Threat Model Gate Contract

## Invocation

```text
cargo xtask threat-model
```

## Success

- Exit `0`.
- Print the registry schema version and counts for threats, executable tests,
  protocol families, and direct proxy dependencies.

## Validation Failure

- Exit `1`.
- Print one deterministic diagnostic per invalid field, unresolved reference,
  missing or ignored test, protocol drift, or dependency drift.

## Unable to Run

- Exit `2` when the registry, source authority, or manifest cannot be read or
  parsed sufficiently to perform validation.

## CI Integration

`cargo xtask ci` invokes the same validation after the specification check and
before native conformance evidence. No check may be skipped as success.
