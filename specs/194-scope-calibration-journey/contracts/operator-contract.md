# Operator Contract: Scope-Aware Calibration

## Public inputs

```text
--inspect-scope <auto|game|launch-chain>
--retry-game-scope
--fallback-capture
```

- `--inspect-scope` is accepted by `fragcap calibrate` and `fragcap deep-capture`.
- `auto` resolves to an effective scope before any session effect.
- `--retry-game-scope` authorizes at most one fresh linked game-scope successor after an eligible launch-chain failure and exact obligation reconciliation. The predecessor authorization is inherited within that invocation only while the complete target and resolved launch authority remain unchanged apart from the reviewed scope transition.
- `--fallback-capture` authorizes packet-only Capture fallback when application inspection cannot proceed. Success requires the written capture artifact, every required output, event delivery, and cleanup to reconcile; otherwise the original failure remains authoritative.
- Resume accepts none of these flags and reuses the stored immutable values.

## Structured fields

```text
requested_inspection_scope
effective_inspection_scope
inspection_scope_source
middleware_handling
inspection_policy_id
game_scope_retry_allowed
capture_fallback_allowed
predecessor_attempt_id
fallback_kind
effective_mode
evidence_window
scope_readiness
```

Existing readers may ignore additive fields. Unknown input enum values fail before effects.

## Human presentation

Normal output leads with effective scope and the game result, followed by independently labeled middleware relay, game routing, game inspection, passive fallback, artifact completeness, and cleanup. Verbose output adds requested scope, recommendation source, policy identity, decision timing, owner uncertainty, transport counters, lineage, phase clocks, and loss.

## Authorization

Before effects, the plan names game roles, observed launch chain, middleware relay behavior, unresolved-owner handling, route effects, installation CA use, exact output, observation and relay lifetimes, retry/fallback authority, owner release, and cleanup.

## Refusal contract

Unknown scope, conflicting resume intent, unavailable exact game ownership, changed target or resolved launch authority, repeated scoped case, exhausted bound, unresolved predecessor obligations, unsupported fallback, missing capture output, failed required output, event-delivery failure, and unresolved cleanup all stop the new effect or prevent fallback success while preserving the original failure.
