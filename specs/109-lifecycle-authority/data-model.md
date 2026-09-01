# Data Model: Crash-Safe Lifecycle Authority

## Routing Strategy

Fields:

- `kind`: child environment, command arguments, target-owned configuration, HTTP proxy, SOCKS, or protocol-specific.
- `availability`: implemented, planned, or refused.
- `effects`: ordered non-secret effect declarations.
- `verification`: evidence required to classify socket-owner reachability.
- `cleanup`: ordered reversal declarations.

Only child environment is implemented in S109. Other kinds are representable and refuse before effects.

## Route Plan

Fields:

- `plan_id` and `target_id` binding.
- `strategy`.
- `effect declarations` with symbolic values for session proxy URL and authorization.
- `ownership evidence` for target-owned configuration.
- `verification rule`.
- `cleanup declarations`.

State transitions:

```text
prepared -> authorized -> applied -> verified -> released
                         -> refused
                         -> retained
```

The plan is immutable after preparation. An authorization naming another plan id cannot apply it.

## Resource Obligation

Fields:

- Session, plan, and resource identifiers.
- Resource kind and exact target.
- Ownership evidence.
- Planned recovery action.
- Sensitivity and retention policy.
- Current transition and sequence.

States:

```text
pending -> applied -> cleanup-pending -> released
                  -> retained
                  -> failed
                  -> timed-out
pending -> not-applied
```

Only complete ordered transitions are accepted. Terminal states cannot return to a mutable state.

## Resource Journal

Fields:

- Versioned header with session and plan identity.
- Ordered transition records.
- Optional compacted terminal summary.
- Optional reconciling trailer.
- Accounting for accepted, written, invalid, incomplete, and unresolved records.

A missing trailer means crash prefix, not corruption. A malformed complete record, duplicate sequence, or contradictory transition is invalid and produces no recovery action.

## Recovery Decision

Variants:

- `execute`: current identity matches and one exact action is safe.
- `already-terminal`: no action is needed.
- `retain`: policy requires evidence or residue retention.
- `refuse`: ownership, format, version, or current identity is insufficient.

Recovery decisions are generated before effect execution and appended back to the journal as attempts and results.

## Proxy Lifecycle Stream

Record classes:

- Header.
- Listener and admission.
- Connection open and terminal.
- DNS/upstream evidence or explicit gap.
- TLS and protocol evidence.
- Error and loss.
- Stop and drain.
- Trailer.

Every retained application connection id resolves to at least one connection record or one counted gap.

## Cleanup Lifecycle Stream

Record classes:

- Header.
- Obligation declaration.
- Attempt and retry.
- Result or retained disposition.
- Recovery linkage.
- Gap.
- Trailer.

`cleanup.json` projects only final resource dispositions from this stream.

## Loss Localization

Fields:

- Bounded localized identity map.
- Exact total dropped records.
- Exact total observed and retained bytes.
- Exact unlocalized overflow records and bytes.

Overflow never changes forwarding and never grows the identity map beyond its configured bound.
