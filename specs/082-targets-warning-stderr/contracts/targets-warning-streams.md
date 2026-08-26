# Contract: Targets Warning Streams

## Human Mode

Applies to `fragcap targets`, `fragcap targets list`, `fragcap targets discover`, `fragcap targets scan`, `fragcap targets add --steam`, and any caller that reuses targets discovery helpers.

```text
stdout = command result only
stderr = warning: <message>
```

Warnings are retained under `--quiet` and suppressed under `--silent`, matching the shared emitter contract.

## JSON Mode

When the top-level CLI is invoked with `--json`, targets warnings are emitted on standard error as structured warning diagnostics:

```json
{"ts":"<rfc3339-utc>","event":"warning","message":"<message>"}
```

The command result stream remains standard output and is not mixed with human warning prefixes.

## Non-Warning Result Lines

The following are not diagnostics and remain on standard output:

- Target table rows and headings.
- Empty-listing guidance.
- `registered N newly discovered target(s).`
- `discovery registered N target(s).`
- `targets discover` candidate rows and account summary.
- Technology evidence lines.
- Target detail, ambiguity, no-match, import, export, and remove results.
