# Schema Contract: hint-record subschema revision

## Change to `target-schema.v1.json`

Applied identically to both copies (compared by the drift check):
`crates/fragcap-profile/assets/target-schema.v1.json` (embedded) and
`docs/schema/target-schema.v1.json` (published).

### New top-level optional properties

```json
"launch": {
  "description": "Steam launch configurations for this title, one entry per config.launch entry, carried whole and never reduced to a single binary at seeding time (reduction to the socket holder is the resolver's runtime job).",
  "type": "array",
  "items": { "$ref": "#/$defs/launch_entry" }
},
"launcher_mediated": {
  "description": "True when Steam starts a publisher launcher that then starts the real client, so the invoked launch entry is not the socket holder.",
  "type": "boolean"
},
"engine": { "$ref": "#/$defs/engine" }
```

### New `$defs`

See [data-model.md](../data-model.md) for `launch_entry` and `engine`. Both use
`additionalProperties: false`; `launch_entry` requires `executable` (non-empty);
`engine` requires `source` and `confidence` from their enums.

### New `allOf` gate

Forbids the three on `profile`, `package`, and the `export` top level (valid only
on a `hint` top level and inside each `export` record):

```json
{
  "if": { "required": ["kind"], "properties": { "kind": { "enum": ["profile", "package", "export"] } } },
  "then": { "properties": { "launch": false, "launcher_mediated": false, "engine": false } }
}
```

### `$defs/record` gains

`launch`, `launcher_mediated`, `engine` (so export records carry them).

## Compatibility

- Additive, backward compatible: every pre-existing artifact still validates; no
  schema version bump. Both copies stay byte-identical.
- The record `fidelity` enum and the provenance object are unchanged.

## Hand-rolled validator (`variants.rs`)

- `allowed_top_keys`: `Hint` gains the three; `Strict`/`Export` do not.
- `check_records`: record allowed keys gain the three; records shape-checked.
- `check_launch` / `check_launch_entry` / `check_engine`; `launcher_mediated`
  boolean check.
- Diagnostics: `InvalidEngineSource`, `InvalidEngineConfidence` (out-of-enum);
  `MissingField` (no `executable`), `EmptyString` (empty `executable`),
  `UnknownKey`, `WrongType` reused.

## Conformance fixtures

Added to `crates/fragcap-profile/tests/schema_conformance.rs`:

- `hint-loose-valid.json` - hint with a multi-entry launch array, `launcher_mediated`,
  and a valid engine object (accepted).
- `engine-bad-source.json` - engine `source` out of enum (rejected,
  `InvalidEngineSource`).
- `engine-bad-confidence.json` - engine `confidence` out of enum (rejected,
  `InvalidEngineConfidence`).
- `launch-no-executable.json` - a launch entry missing `executable` (rejected,
  `MissingField`).
- `profile-with-launch.json` - a strict profile carrying `launch` (rejected,
  `UnknownKey`).

Pre-existing fixtures keep their outcomes.
