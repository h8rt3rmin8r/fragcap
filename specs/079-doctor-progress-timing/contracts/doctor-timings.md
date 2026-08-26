# Contract: Interactive Doctor Timings

## Invocation

```text
fragcap doctor --timings
```

`--timings` is a hidden maintainer flag. It follows the same enablement rules as
interactive progress.

## Stream

- Timing output is written to stderr.
- Each completed progress item includes elapsed milliseconds.
- The final human report remains unchanged.

## Required Timing Labels

The timing stream must include completed measurements for:

- `identity`
- `platform`
- `capture driver and interfaces`
- `process event tracing`
- `analyzer integration`
- `target stores`
- `Deep Capture readiness`
- `report rendering`

## Suppression

No timing output is emitted for:

- `fragcap doctor --json --timings`
- `fragcap doctor --timings > file`
- `fragcap -q doctor --timings`
- `fragcap --silent doctor --timings`
- `fragcap doctor --fix --timings`

## Invariants

- Timings are local diagnostic evidence, not a stable automation API.
- Timings must be measured around real probe work, not fabricated.
- Timings must not replace the final doctor report's readiness decisions.
