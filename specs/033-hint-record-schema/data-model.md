# Phase 1 Data Model: Target-Hint-Record Schema Revision

Additions to the master target schema. All are optional; every pre-existing shape
is unchanged.

## `$defs/launch_entry` (new)

One Steam launch configuration.

```json
{
  "type": "object",
  "required": ["executable"],
  "properties": {
    "os":          { "type": "string" },
    "osarch":      { "type": "string" },
    "launch_type": { "type": "string" },
    "beta_branch": { "type": "string" },
    "executable":  { "type": "string", "minLength": 1 },
    "arguments":   { "type": "string" },
    "description": { "type": "string" }
  },
  "additionalProperties": false
}
```

- Filters (`os`, `osarch`, `launch_type`, `beta_branch`) are optional free strings.
- `executable` is required and non-empty (an entry names a binary).
- `arguments`, `description` are optional labels.

## `$defs/engine` (new)

Engine attribution with its provenance and confidence.

```json
{
  "type": "object",
  "required": ["source", "confidence"],
  "properties": {
    "name":       { "type": "string" },
    "source":     { "enum": ["pcgamingwiki", "exe_heuristic", "depot_filename_rules"] },
    "confidence": { "enum": ["confirmed", "high", "medium", "low", "unknown"] }
  },
  "additionalProperties": false
}
```

- `name` optional (a failed lookup can record source/confidence with no name).
- `source`, `confidence` required from their enums.
- Independent of the record `fidelity` and the record's provenance `source`.

## New top-level / record properties

Added to top-level `properties` and to `$defs/record.properties`:

```json
"launch":            { "type": "array", "items": { "$ref": "#/$defs/launch_entry" } },
"launcher_mediated": { "type": "boolean" },
"engine":            { "$ref": "#/$defs/engine" }
```

- `launch` has no `minItems` (an empty array is valid: looked up, found none).
- All three are optional.

## Variant gating (new `allOf` branch)

```json
{
  "if": { "required": ["kind"],
          "properties": { "kind": { "enum": ["profile", "package", "export"] } } },
  "then": { "properties": { "launch": false, "launcher_mediated": false, "engine": false } }
}
```

- Valid at the top level only on `hint`; valid inside each `export` record (via
  `$defs/record`); rejected on strict `profile`/`package` and the `export` top
  level.

## Validator (hand-rolled, `variants.rs`)

- `allowed_top_keys`: `Hint` arm gains `launch`, `launcher_mediated`, `engine`;
  `Strict` and `Export` arms do not.
- `check_records`: allowed-key set gains the three; each record shape-checked.
- New `check_launch` / `check_launch_entry` / `check_engine`; inline
  `launcher_mediated` boolean check.
- New diagnostic codes `InvalidEngineSource`, `InvalidEngineConfidence`; mapped in
  `parse.rs` to `DiagnosticCode::WrongType`.

## Unchanged

- `$defs/fidelity` and its enum (authored/verified/heuristic-unverified/observed).
- `$defs/provenance`, `game`, `capture`, `match`, `stage`, `record` core, and
  `technology`. No schema version change.
