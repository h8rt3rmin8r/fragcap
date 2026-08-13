# Schema Contract: the `technologies` structure

## Change to `target-schema.v1.json`

Applied identically to both copies (they are compared by the drift check):

- `crates/fragcap-profile/assets/target-schema.v1.json` (embedded)
- `docs/schema/target-schema.v1.json` (published)

### New top-level optional property

```json
"technologies": {
  "description": "Technologies detected in the target's install directory, each a heuristic-unverified finding with the marker path that revealed it.",
  "type": "array",
  "items": { "$ref": "#/$defs/technology" }
}
```

### New `$defs/technology`

```json
"technology": {
  "description": "One detected technology: a category, a name, the marker path that matched, and a heuristic fidelity.",
  "type": "object",
  "required": ["category", "name", "fidelity"],
  "properties": {
    "category": {
      "description": "Technology category.",
      "enum": ["engine", "anti_cheat", "sdk", "framework", "emulator",
               "container", "runtime", "launcher"]
    },
    "name": {
      "description": "Technology name (the ruleset key as a readable label).",
      "type": "string",
      "minLength": 1
    },
    "marker_path": {
      "description": "Relative install-directory path of a representative file that matched.",
      "type": "string"
    },
    "fidelity": { "$ref": "#/$defs/fidelity" }
  },
  "additionalProperties": false
}
```

## Compatibility

- `technologies` is optional; artifacts without it (every existing artifact) stay
  valid. This is an additive, backward-compatible extension of schema version 1;
  no version bump.
- The top-level schema keeps `additionalProperties: false`; the new property is a
  known key, so a target that carries `technologies` now validates where before
  it would have been rejected.
- `fidelity` reuses the existing `$defs/fidelity` enum
  (`authored`/`verified`/`heuristic-unverified`/`observed`). Detection findings
  always use `heuristic-unverified`. This is the targeting fidelity, deliberately
  separate from the attribution `Fidelity` (Live/Retained/None) in `fragcap-core`.

## Hand-rolled validator

The variant validator in `crates/fragcap-profile/src/jsonschema/variants.rs`
enforces the closed property set itself, so it is extended to:

- accept `technologies` as a known optional top-level array, and
- shape-check each item: required `category` (from the enum), required non-empty
  `name`, required `fidelity` (from the fidelity enum), optional `marker_path`
  string, and no additional properties.

## Conformance fixtures

`crates/fragcap-profile/tests/schema_conformance.rs` gains fixtures under
`tests/fixtures/schema/`:

- a valid target carrying a `technologies` array with one finding per a few
  categories (accepted),
- a target with a `technologies` item missing `category` (rejected),
- a target with a `technologies` item whose `category` is not in the enum
  (rejected),
- a target with an empty `technologies` array (accepted).
