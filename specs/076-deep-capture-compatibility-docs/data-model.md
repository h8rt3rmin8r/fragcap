# Phase 1 Data Model: Deep Capture Compatibility Matrix

## `CompatibilityMatrix`

A read-only projection of all compatibility facts for one selected target.

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `rows` | ordered compatibility rows | yes | Every source fact, with no aggregation or winner selection |

Derived matrix state:

- `unknown` when `rows` is empty;
- `known` when one or more rows exist, even if individual values say `unknown`.

## `CompatibilityMatrixRow`

One display-safe projection of one stored fact.

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `id` | optional integer | no | Durable chronology for a stored row |
| `key` | compatibility fact key | yes | Existing closed fact family |
| `value` | string | yes | Existing key-specific closed value |
| `launch_case` | optional launch case | no | Context in which the fact was observed |
| `evidence_source` | evidence source | yes | Existing provenance category |
| `freshness` | freshness state | yes | `current` or `stale` for a row |

The projection intentionally omits note text, final executable names, local
paths, and target display names. The surrounding target detail view already
identifies the selected target locally.

## `CompatibilityFreshness`

Presentation states:

- `current`: the row is not marked stale and its source is not
  `stale-observation`;
- `stale`: the row is marked stale or its source is `stale-observation`;
- `unknown`: no compatibility rows exist for the target.

`unknown` is a matrix-level empty state. It is not inserted as a synthetic fact.

## Ordering

Rows use a total order:

1. stored row id when both rows have one;
2. rows with stored ids before unsaved rows;
3. fact key token;
4. fact value;
5. launch-case token, with absent launch case first;
6. evidence-source token;
7. freshness token.

The fallback fields exist for deterministic pure tests and in-memory callers.
They do not rank evidence quality.

## State Transitions

```text
no stored facts
    -> matrix unknown

fact added by explicit measurement or import
    -> row current unless explicitly stale

fact marked stale or sourced as stale-observation
    -> row stale

new fact for the same key
    -> additional row; prior row remains visible
```

Viewing the matrix performs no transition and writes no data.
