# fragcap master target schema

This directory holds the published copy of fragcap's master JSON Schema,
`target-schema.v1.json` (JSON Schema Draft 2020-12). It is byte-identical to the
copy embedded in the binary; a drift between the two fails a workspace test.
Emit the embedded copy with `fragcap schema print`, and validate any JSON file
against it with `fragcap schema validate <file>`.

The schema governs every machine-readable targeting and attribution artifact.
Validation is structural only: the semantic invariants of profile loading
(acyclic `descends_from`, at most one terminal stage, role reachability, no
ambiguous image match) are enforced by the profile-load path, not by the schema.

## Top-level envelope

Every document is a JSON object with these keys.

| Key | Type | Required | Meaning |
| --- | --- | --- | --- |
| `schema` | integer | yes | Schema version. Currently `1`. |
| `kind` | enum | yes | `profile`, `package`, `hint`, or `export`. Selects the variant. |
| `fidelity` | enum | yes | `authored`, `verified`, `heuristic-unverified`, `observed`. Ordered trust tier. |
| `notes` | string | no | Human context carried as data, not a comment. |
| `provenance` | object | conditional | Required for `hint` and `export`. |
| `game` | object | conditional | Required for `profile` and `package`. |
| `capture` | object | no | Capture defaults. |
| `stage` | array | conditional | Required (non-empty) for `profile` and `package`. |
| `records` | array | no | Export envelope only: a batch of loose records. |

Unknown keys are refused at every level.

## Variants (by `kind`)

- **`profile` / `package`** (strict): require `game` (with `id` and `name`) and a
  non-empty `stage` array. `package` is a hand-authored or community-submitted
  profile at the highest precedence; structurally identical to `profile`.
- **`hint`** (loose): a heuristic guess. May omit fields a profile requires, but
  must carry `fidelity` and `provenance`.
- **`export`** (loose): the JSON projection of hint-database rows. Requires
  `provenance`; may carry a `records` array of loose records, each with its own
  `fidelity` and `provenance`.

## Objects

### `game`

| Key | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | strict only | Slug `^[a-z0-9_-]+$`. |
| `name` | string | strict only | Non-empty display name. |
| `platform` | string | no | For managed launch, for example `steam`. |
| `app_id` | string | no | Platform application id, kept as a string. |

### `capture`

`mode` (`file`/`stream`/`ring`), `duration` (string), `roles` (array of string),
`loopback` (boolean), `payload` (boolean). All optional.

### `stage`

| Key | Type | Required | Notes |
| --- | --- | --- | --- |
| `role` | string | yes | Non-empty; unique within the document (checked at profile load). |
| `lifecycle` | enum | yes | `transient`, `session`, or `service`. |
| `terminal` | boolean | no | At most one per document (checked at profile load). |
| `match` | object | yes | At least one predicate. |

### `match`

At least one of `exe` (glob), `path_contains`, `path_regex`, `cmdline_contains`,
`descends_from` (all strings). An empty `match` is refused.

### `provenance`

`source` (non-empty string, required) and `seeded_at` (string, optional).

## Fidelity tiers

`authored` > `verified` > `heuristic-unverified` > `observed`. The resolver reads
this ordering; the instrument never fabricates a tier (constitution P-9).
