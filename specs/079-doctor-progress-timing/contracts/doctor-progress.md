# Contract: Interactive Doctor Progress

## Invocation

```text
fragcap doctor
```

Progress is enabled only when all of these are true:

- The command renders the human doctor report.
- `--json` is not selected.
- `--fix` is not selected.
- Standard output is a terminal.
- The selected verbosity allows progress messages.

## Stream

- Progress is written to stderr.
- The final doctor report is written to stdout by the existing renderer.
- Progress lines are diagnostic and not part of the report contract.

## Required Probe Labels

The progress stream must be able to identify these units of work:

- `identity`
- `platform`
- `capture driver and interfaces`
- `process event tracing`
- `analyzer integration`
- `target stores`
- `Deep Capture readiness`
- `report rendering`

## Suppression

No progress output is emitted for:

- `fragcap doctor --json`
- `fragcap doctor > file`
- `fragcap -q doctor`
- `fragcap --silent doctor`
- `fragcap doctor --fix`

## Invariants

- The final human report bytes are unchanged when stdout is redirected.
- The JSON output bytes are unchanged.
- A failed or unknown probe remains represented in the final doctor report.
- Progress must not declare the overall doctor result before classification.
