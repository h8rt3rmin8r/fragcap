# Data Model: Native Deep Capture Failure Injection

## Failure Boundary Registry

- `schema_version`: exact supported schema version.
- `reviewed_on`: date of the complete boundary review.
- `effect_boundaries`: closed journaled-effect inventory.
- `lifecycle_transitions`: closed checked-state transition inventory.
- `failure_families`: mandatory concrete failure categories.
- `outcome_dimensions`: authorities every applicable case must assert independently.
- `tests`: executable evidence references for matrix execution and validator rejection behavior.

## Boundary

- `id`: stable kebab-case identity.
- `kind`: `effect` or `lifecycle`.
- `source`: owning production source and concept.
- `resource_kind`: journal resource kind for effects.
- `from` and `to`: lifecycle states for checked transitions.
- `before_driver`: deterministic injection driver before the boundary.
- `after_driver`: deterministic injection driver after possible acquisition or transition.
- `cleanup_resources`: ordered resources whose safe cleanup remains expected.
- `recovery`: `exact-action`, `no-action`, or `refusal`.
- `lifecycle_trace`: ordered endpoint pairs actually traversed by the production coordinator.

Each boundary expands into exactly two matrix cells keyed by `(boundary_id, side)`.

## Failure Scenario

- `id`: generated as `<boundary-id>:<side>`.
- `boundary_id`: parent boundary.
- `side`: `before` or `after`.
- `driver`: controlled failure behavior.
- `failure_family`: one of the registry's mandatory categories.
- `expected`: outcome vector.

## Outcome Vector

- `terminal`: `complete`, `partial`, `failed`, or `interrupted`.
- `artifact`: `written`, `partial`, `failed`, `not-attempted`, or `unaffected`.
- `fact`: `appended`, `failed`, `not-written`, or `unaffected`.
- `event`: `delivered`, `failed-recorded`, or `unaffected`.
- `cleanup`: ordered resource attempt expectations.
- `journal`: `complete`, `crash-prefix`, `unavailable`, or `unchanged`.
- `recovery`: `exact-action`, `no-action`, or `refusal`.

## Invariants

1. Boundary identity is unique and stable.
2. Generation yields exactly two cells for every boundary.
3. Every mandatory failure family appears in at least one executable cell.
4. Every cell declares all seven outcome dimensions, using `unaffected` only where the authority genuinely does not apply.
5. Before-side injection never calls the owned effect.
6. After-side injection assumes possible acquisition and demands cleanup or exact recovery.
7. A failed or incomplete artifact never maps to `written`.
8. A fact never maps to `appended` without retained qualifying observation evidence.
9. Cleanup failure never suppresses later safe cleanup attempts.
10. Recovery mutation requires exact ownership evidence from the production journal planner.
11. Every transition cell retains its named endpoint pair in terminal truth.
