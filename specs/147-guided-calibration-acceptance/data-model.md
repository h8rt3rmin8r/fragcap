# Data Model: Guided Calibration Acceptance

S147 adds a repository acceptance authority and one additive product-store migration required by the audited pause vocabulary.

## Acceptance Registry

| Field | Type | Invariant |
| --- | --- | --- |
| `schema_version` | integer | Exactly `1` |
| `reviewed_on` | date string | Present and non-empty |
| `scope` | string | Names guided calibration issue #380 |
| `implementation_evidence` | enum | Exactly `controlled-automated` |
| `live_compatibility` | object | Explicitly unverified in S147, operator-owned, published-release-only |
| `criteria` | array | Exactly thirteen unique required identifiers |

## Acceptance Criterion

| Field | Type | Invariant |
| --- | --- | --- |
| `id` | stable token | One of `AC-01` through `AC-13`, exactly once |
| `statement` | string | Concise parent acceptance proposition |
| `evidence_class` | enum | Exactly `controlled-automated` |
| `tests` | array | Non-empty and unique within the criterion |

## Evidence Reference

| Field | Type | Invariant |
| --- | --- | --- |
| `path` | repository-relative Rust path | Confined, tracked, and readable |
| `function` | Rust identifier | Exact unconditional test function |
| `proves` | string | Non-empty bounded proposition |

## Validation States

- **Valid**: all thirteen criteria are exact, every reference is executable, and the evidence boundary is honest.
- **Invalid registry**: schema, inventory, field, identity, classification, or duplicate-reference failure.
- **Stale evidence**: a path or function no longer resolves to an ordinary non-ignored test.
- **Unacceptable evidence**: an ignored, conditionally disabled, untracked, non-Rust, or documentary reference.

## Product Data

### Workflow Pause Vocabulary Migration

Target-store schema version 13 rebuilds only the `calibration_workflows` table constraint so `pause_reason` also accepts `update` and `anti-cheat`. Every version 12 column and row is copied exactly, and the workflow record version remains unchanged. No plan, response, secret, trust state, endpoint, effect obligation, or compatibility claim enters the checkpoint.

### Stored Client Selection Plan

| Field | Type | Invariant |
| --- | --- | --- |
| `plan_id` | `stored-client-selection-v1:` digest | Binds the complete target, selected executable, effective store, resulting authored launch, and no-effect declaration |
| `target` | complete durable target | Must reproduce exactly after confirmation |
| `selected_executable` | canonical Windows executable | Must identify exactly one eligible client-only launch entry |
| `resulting_launch` | one authored client entry | Never rewrites Steam authority or collapses an ordered publisher chain |
| `no_effects` | closed string set | Declares that the plan itself performs no process, routing, trust, network, capture, or compatibility effect |

The confirmed store operation uses complete-row optimistic authority. A changed or missing row performs no write. Compatibility facts, bundles, manifests, and artifacts remain unchanged and append-only.
