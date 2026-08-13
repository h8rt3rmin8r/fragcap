# Phase 1 Data Model: Technology-Detection Surface

This describes the entities the slice introduces and the shapes they take in
code and in the target artifact. Field names in the artifact use the schema's
snake_case; Rust type names are illustrative and finalized in implementation.

## Category (enum)

The technology category vocabulary, shared by the runtime types and the schema.

- Values: `engine`, `anti_cheat`, `sdk`, `framework`, `emulator`, `container`,
  `runtime`, `launcher`.
- The vendored ruleset populates: `engine` (from `[Engine]`), `anti_cheat` (from
  `[AntiCheat]`), `sdk` (from `[SDK]`), `emulator` (from `[Emulator]`),
  `container` (from `[Container]`), `launcher` (from `[Launcher]`).
- Defined but unpopulated by this source: `framework`, `runtime` (reserved for a
  future hint source or the deferred `Evidence` deduction).
- The schema serialization is exactly these strings.

## Ruleset (vendored asset)

The committed `rules.ini` and its integrity record.

- **Bytes**: the verbatim file, stored LF / UTF-8 / no BOM.
- **Lock record** (`rules.lock.json`): `source` (repository URL), `commit`
  (pinned 40-char SHA), `license` (SPDX identifier `MIT`), `sha256` (lowercase
  hex over the committed bytes), and a `note` documenting the normalization used
  for the hash so it is reproducible.
- **Attribution** (`THIRD_PARTY_NOTICES.md`): MIT license text and
  `Copyright (c) 2021 SteamDB`.

## Rule (parsed, pre-compile)

One line of the applied ruleset before regex compilation.

- `category`: the `Category` the section maps to.
- `technology`: the rule key with any trailing `[]` array marker removed (several
  `Key[] = ...` lines share one technology name).
- `pattern`: the raw regex text (the ruleset's path regex).

## CompiledRuleset (load-time product)

The result of parsing and compiling the applied ruleset.

- `rules`: the successfully compiled matchers, each carrying its `category`,
  `technology`, compiled `Regex`, and the raw `pattern` it came from.
- `compiled_count`: number of patterns that compiled.
- `skipped`: the patterns that failed to compile, each recording its `category`,
  `technology`, `pattern`, and the compile error, so the affected technology is
  identifiable (FR-005).
- `total_count`: number of patterns in the applied sections.
- **Invariant** (asserted): `compiled_count + skipped.len() == total_count`
  (FR-006).

## ScanOutcome (per install directory)

The result of scanning one install directory against a `CompiledRuleset`.

- `findings`: a list of `TechnologyFinding`, deduplicated per (category,
  technology), grouped/ordered by category then technology name.
- `unreadable`: the paths (if any) that could not be read during the walk, a
  surfaced condition distinct from "scanned and found nothing" (FR-010).
- **Distinctness rule**: an empty `findings` with an empty `unreadable` means a
  complete scan with no technologies; a non-empty `unreadable` means coverage was
  reduced and the result is partial.

## TechnologyFinding (one detected technology)

The unit of the report and of the schema structure.

- `category`: `Category`.
- `name` / `technology`: the technology's name (the ruleset key, sanitized to a
  human-readable label; underscores in keys are kept as written by upstream).
- `marker_path`: the relative install-directory path (forward-slash separated)
  of a representative file that matched (FR-011, the auditable evidence).
- `fidelity`: always `heuristic-unverified` (FR-011, P-9).

### Schema serialization (`$defs/technology`)

```json
{
  "type": "object",
  "required": ["category", "name", "fidelity"],
  "properties": {
    "category": { "enum": ["engine", "anti_cheat", "sdk", "framework",
                            "emulator", "container", "runtime", "launcher"] },
    "name": { "type": "string", "minLength": 1 },
    "marker_path": { "type": "string" },
    "fidelity": { "$ref": "#/$defs/fidelity" }
  },
  "additionalProperties": false
}
```

The top-level schema gains:

```json
"technologies": {
  "description": "Technologies detected in the target's install directory, each a heuristic-unverified finding with the marker path that revealed it.",
  "type": "array",
  "items": { "$ref": "#/$defs/technology" }
}
```

`technologies` is optional at the top level (a target may carry none). When
present and empty it is a valid empty array; the scaffold may omit it when no
technologies were detected (the artifact stays conformant either way, FR-013 /
US2 acceptance 3).

## Relationships and boundaries

- A `ScanOutcome` is produced from a `CompiledRuleset` plus an install directory;
  the `CompiledRuleset` is built once from the embedded `Ruleset` bytes.
- The Steam scaffold consumes a `ScanOutcome` for the install directory it is
  already classifying and serializes `findings` into the target artifact's
  `technologies` array (reusing the schema's `fidelity` vocabulary).
- The CLI subcommand consumes a `ScanOutcome` and prints it grouped by category.
- Nothing here is a `PacketSource` or `FlowAttributor`; the model does not touch
  the capture pipeline, the resolver's `Target`, or the packet-stream writers.
