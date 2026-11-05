# Data Model: Scope-Aware Calibration and Operator Journey

## Requested Inspection Scope

- Values: `auto`, `exact-game`, `launch-chain`.
- Mutable: never after workflow creation.
- Resolution: `auto` becomes an effective value before effects; explicit `exact-game` refuses when exact ownership plus blind relay are unavailable.

## Effective Inspection Scope

- Values: `exact-game`, `launch-chain`.
- Policy identity: `exact-game-v1` or `launch-chain-v1`.
- Middleware handling: `blind-relay` for exact game scope, `inspect-launch-chain` for launch-chain scope.

## Scoped Compatibility Case

- Existing fields: launch case, proxy backend and version, routing strategy, address family, protocol, Fragcap version, optional target version.
- New fields: effective inspection scope, middleware handling, inspection policy identity.
- Applicability: every field must match. A fact with any absent new field is `legacy-incomplete`.

## Compatibility Fact

- New nullable fields: requested scope, effective scope, middleware handling, inspection policy identity.
- New key: scope recommendation.
- Migration: existing rows keep null new fields and remain visible history.
- Write rule: a positive scoped fact requires an exact game-owned observation for exact game scope; topology and relay success cannot create it.

## Calibration Workflow Intent

- Existing immutable fields: target authority snapshot, selected launch case, routing strategy, address family, requested protocols.
- New immutable fields: requested scope, effective scope, middleware handling, policy identity, allow-game-retry, allow-capture-fallback.
- New monotonic checkpoint field: game-scope-successor-active. Once set after an eligible predecessor reaches a terminal state, resume continues the exact-game successor and cannot clear the transition.
- Migration: existing rows become launch-chain/inspect-launch-chain with both fallback authorities false.
- Resume: every immutable field must be reused exactly.

## Calibration Attempt Lineage

- Fields: attempt identity, ordinal, exact scoped case key, optional predecessor identity, fallback kind, effective mode, state, phase, protocol, bundle identity, evidence window, terminal reason, cleanup obligations.
- Bound: one game-scope successor and the existing finite case bound.
- Transition: `Ready -> InFlight -> Completed|Paused|Refused`; a linked retry starts only after the predecessor reaches a terminal state, obligations reconcile, and fresh resolution reproduces the predecessor target and launch authority apart from the reviewed scope transition.
- Fallback success: retained packet accounting is present, the packet-capture artifact and every required artifact are written, event delivery has no failure, and every cleanup result is released or not needed. Any unmet condition preserves the attempt's terminal failure.

## Scope Readiness

- Inputs: runtime readiness, exact current case, applicable facts, measured recommendation, relay transport support, owner-evidence capability, unresolved obligations.
- Outputs: requested/effective scope, readiness dimensions, limitations, and one supported next command.
- Mutation: none.
