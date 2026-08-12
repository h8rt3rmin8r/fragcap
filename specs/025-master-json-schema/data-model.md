# Phase 1 Data Model: Master JSON Schema

All documents are JSON objects. Every document has two required top-level keys
regardless of variant: `schema` (version) and `kind` (discriminator). Unknown
keys are refused at every level.

## Top-level envelope (all variants)

| Key | Type | Required | Meaning |
| --- | --- | --- | --- |
| `schema` | integer | Yes | Schema version. Currently `1`. An unsupported value is refused, naming the supported version. |
| `kind` | string enum | Yes | One of `profile`, `hint`, `package`, `export`. Selects the variant. |
| `fidelity` | string enum | Yes | One of `authored`, `verified`, `heuristic-unverified`, `observed`. Ordered trust tier the resolver reads. |
| `notes` | string | No | Human-readable context carried as data, never as a comment. |
| `provenance` | object | Conditional | Where the document came from. Required for `hint` and `export`; optional elsewhere. See below. |

## Shared core: game identity, capture defaults, stages

The core is the process-recognition vocabulary carried forward from master
specification section 15, unchanged in meaning and re-expressed in JSON.

### `game` (object)

| Key | Type | Required (profile/package) | Meaning |
| --- | --- | --- | --- |
| `id` | string slug `[a-z0-9_-]+` | Yes | Unique identifier, used for resolution and as a filename component. |
| `name` | string | Yes | Display name. |
| `platform` | string | No | Platform for managed launch (for example `steam`). |
| `app_id` | string | No | Platform application identifier, kept as a string. |

### `capture` (object, all keys optional)

| Key | Type | Meaning |
| --- | --- | --- |
| `mode` | enum `file`\|`stream`\|`ring` | Output mode default. |
| `duration` | string | Duration literal (existing grammar). |
| `roles` | array of string | Roles to keep. |
| `loopback` | boolean | Include loopback. |
| `payload` | boolean | Include payload. |

### `stage` (array of objects)

Each stage names a role and how to recognize the processes that fill it.

| Key | Type | Required | Meaning |
| --- | --- | --- | --- |
| `role` | string | Yes | Role name, unique within the document. |
| `lifecycle` | enum `transient`\|`session`\|`service` | Yes | Lifecycle class. |
| `terminal` | boolean | No | Exit ends capture. At most one stage may set it (semantic check, not structural). |
| `match` | object (MatchPredicates) | Yes | At least one predicate present. |

### `match` (MatchPredicates object)

At least one key is required (an empty predicate set would match every process
and is refused structurally).

| Key | Type | Meaning |
| --- | --- | --- |
| `exe` | string glob | Image-name glob. |
| `path_contains` | string | Case-insensitive substring of the full path. |
| `path_regex` | string | Regular expression over the full path. |
| `cmdline_contains` | string | Substring of the command line. |
| `descends_from` | string | Role of a synthetic-tree ancestor. |

## Variants (discriminated by `kind`)

### `profile` (strict)

The authoritative description the capture pipeline runs against. Requires the
full core: `game.id`, `game.name`, at least one `stage`, each stage complete
(`role`, `lifecycle`, `match` with at least one predicate). `fidelity` is
typically `verified` or `observed`. `provenance` optional.

### `package` (strict, authoritative, highest precedence)

Structurally identical to `profile`. Distinguished by intent and precedence: a
hand-authored or community-submitted artifact, `fidelity` typically `authored`.
Modeled as the same strict shape as `profile` so a core change reaches both.

### `hint` (loose, partial)

A heuristic guess from a provider or the hint database. May omit fields a
`profile` requires (for example it may carry only a candidate `exe` and a
`game.name`, with no complete stage set). MUST carry `fidelity`
(`heuristic-unverified` in the common case) and MUST carry `provenance`. A hint
missing `fidelity` or `provenance` is refused: a guess that does not declare its
trust level is exactly the guess-worn-as-fact the schema exists to prevent.

### `export` (loose, partial, from the hint database)

The JSON projection of a hint database row (or a set of rows). Structurally a
`hint` (single) or an envelope carrying an array of `hint`-shaped records. MUST
validate against this schema with no manual adjustment (round-trip conformance).
`fidelity` and `provenance` required as for `hint`.

## `provenance` (object)

| Key | Type | Required | Meaning |
| --- | --- | --- | --- |
| `source` | string | Yes (hint/export) | Origin, for example `steam-appinfo`, `engine-rule`, `user`. |
| `seeded_at` | string (date-time) | No | When the record was generated or last refreshed. |

## Fidelity tiers (ordered)

`authored` > `verified` > `heuristic-unverified` > `observed` as a precedence
input for the resolver (#77). The ordering is data the resolver reads; this slice
defines and validates the field, it does not implement the ordering behavior.

## Validation rules owned by this slice (structural)

- `schema` present and a supported version; else refuse naming the supported one.
- `kind` present and a known variant; else refuse.
- `fidelity` present and within the closed enum; else refuse.
- Per variant: required keys present, types correct, enums in range, string
  patterns (slug, glob shape where checkable) satisfied.
- `hint`/`export`: `provenance.source` present.
- `match`: at least one predicate.
- Unknown keys at any level: refused.
- Every violation found is reported together, each located by JSON pointer.

## Validation rules NOT owned by this slice (semantic, deferred to #76 profile-load)

- `descends_from` acyclic and referencing a declared role.
- At most one `terminal` stage.
- At least one non-service stage.
- No ambiguous image match.
- Regex/glob/duration compile (these run at profile load).

These are named here to mark the seam; they remain in the profile-load path and
are rewired onto JSON by #76.
