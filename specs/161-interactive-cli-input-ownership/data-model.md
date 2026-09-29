# Data Model: S161 Interactive CLI Input Ownership

No durable data or artifact schema changes.

## Standard input owner

- **Handle**: One process-global buffered stdin stream.
- **State**: Unlocked between reads; exclusively locked during one authorization response.
- **Rule**: A command may present and read a separate interactive question while the authorization owner object exists.
- **Transition**: `Idle -> AuthorizationRead -> Idle`; a prompt uses the same global stream during Idle.
- **Invariant**: No guard is retained across command dispatch.

## Doctor confirmation

- **Offered action**: Existing report-named action, in existing order.
- **Decision**: Affirmative, negative, or input failure.
- **Transition**: `Offered -> Affirmative -> Performed/Failed`; `Offered -> Negative -> Skipped`; `Offered -> InputFailure -> Stop`.
- **Invariant**: Only an affirmative decision reaches the performer. Input failure never prints `skipped` for that action or proceeds to another action.

## Other prompts

- **Socket-holder answer**: Existing yes, no, or unsure classification. EOF remains unsure and does not author a positive assertion. Write, flush, and read errors stop registration.
- **Warm-restart answer**: Existing affirmative or negative completed response. EOF or I/O error is a failure and cannot request restart.
- **Plan authorization**: Existing exact plan id, finite input bound, complete line, separate plan authority, and no-effect refusal remain unchanged.

## Traceability

- `FR-001..FR-004`: Doctor and confirmation decision.
- `FR-005`: Socket-holder and warm-restart prompts.
- `FR-006`: Plan authorization.
- `FR-007`: Command gates.
- `FR-008`: Print Screen attribution limit.
