# Data Model: Exact Game Inspection Ownership

## Inspection scope

`requested`: `auto`, `exact-game`, or `launch-chain`. `effective`: `exact-game` or `launch-chain`, fixed before session effects. `policy_id`: stable code for the chosen scope policy. Routing strategy and destination admission are independent fields.

## Live owner proof

`connection_id`, client-side peer and proxy-local endpoints, accepted instant, socket creation instant, PID, process creation instant, canonical executable identity, account/session verification, launch receipt, ancestry, bound role, and evidence completeness. Exact proof requires every required element; absence or conflict produces a named uncertainty.

## Inspection decision

`action`: `inspect` or `uninspected`. `reason`: stable exact-game, launch-chain, known-middleware, pending-timeout, unavailable-telemetry, queue-saturated, stale-socket, changed-generation, ambiguous, or unproven-role code. `decided_at`, elapsed lookup duration, and policy identity are retained. The decision is immutable for the connection or association.

## Lifecycle relationship

An uninspected decision changes no process watcher, route-owner inventory, shutdown authority, or session cleanup record. Later process evidence may enrich reporting but cannot rewrite the frozen inspection action.
