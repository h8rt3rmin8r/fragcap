# Contract: `profile` structured output (`--json`)

Covers #65. The stream is the section 17.5 NDJSON form on standard error:
each line is one JSON object with a `ts` (RFC3339 UTC) and an `event` name, plus
event-specific fields. Rendering and string escaping reuse the existing
`events.rs` `render()` / `fragcap::write_json_string`.

## `profile list --json`

Emits the same counts the human formatter prints, as structured events instead
of text. One event carrying the bundled count and the per-user-directory counts.

Illustrative (field names finalized in implementation, but the shape is fixed -counts as numbers, directories named):

```json
{"ts":"2026-08-12T06:56:12Z","event":"profiles","bundled":0,"user_total":0,
 "directories":[{"path":"C:\\Users\\...\\fragcap\\profiles","count":0}]}
```

- MUST be machine-readable without parsing human prose (SC-002 class).
- Human (non-`--json`) output is unchanged.

## `profile validate --json`

One `diagnostic` event per diagnostic, then one terminal `summary` event.

```json
{"ts":"...","event":"diagnostic","code":"invalid-duration","path":"capture.duration","line":9,"col":13,"message":"unknown duration unit `zz`; expected ms, s, m, or h"}
{"ts":"...","event":"diagnostic","code":"undeclared-capture-role","path":"capture.roles","line":10,"col":12,"message":"names role `ghost`, which no stage declares"}
{"ts":"...","event":"diagnostic","code":"unknown-key","path":"capture.payloads","line":11,"col":1,"message":"unknown key `payloads`; accepted here: mode, duration, roles, loopback, payload"}
{"ts":"...","event":"summary","diagnostics":3,"ok":false}
```

- Each `diagnostic` event carries the diagnostic's `code`, config `path`,
  `line`, `col`, and `message` as **distinct fields**. It MUST NOT concatenate
  multiple diagnostics into one newline-joined string (the current defect).
- The terminal `summary` event carries the diagnostic count and an ok/failed
  indicator, so a consumer distinguishes a clean profile (zero `diagnostic`
  events, `summary` `ok:true`) from no output at all.
- On a **valid** profile, `--json` emits zero `diagnostic` events and one
  `summary` with `ok:true`; human mode emits the existing success line
  (see the validate path-dedup, contracts/exit-codes.md and #70.1).
- Fields already exist in the human diagnostic formatter; the change preserves
  them structurally rather than pre-rendering.

## Discoverability (FR-010)

Because `doctor --json` uses its own per-check shape and `profile` now uses the
section 17.5 stream, the `--json` global-flag help states its scope (which
surfaces emit structured events), so an inconsistency is documented rather than
silent. Exact help wording is set in implementation.

## Exit codes with `--json`

`--json` does not change exit codes: a validation failure still exits per the
exit-code contract (a genuinely invalid profile is a usage/configuration error,
exit 2; the resolves-to-nothing reference is exit 1 per contracts/exit-codes.md).
The structured `summary` `ok:false` and the process exit code agree.
