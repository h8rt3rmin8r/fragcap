# Data Model: Guided Protocol Calibration Attempt

S141 adds transient CLI and facade policy values only. It does not change SQLite, bundle, manifest, compatibility-fact, or public artifact schemas.

## ProtocolCandidateSet

| Field | Meaning | Rule |
| --- | --- | --- |
| protocols | Concrete compatibility protocol dimensions | Bounded, sorted, deduplicated, and never contains routing or not-applicable |
| origin | `requested` or `observed` | A requested value is not evidence |
| target scope | Stable target and exact launch authority | Never transferred across targets |

## GuidedCalibrationDecision

| Field | Meaning | Rule |
| --- | --- | --- |
| target id and handle | Durable identity and display label | Present after resolution |
| topology | Steam, direct, publisher, or unavailable | Copied from S139 |
| action | `run-reachability`, `run-protocol`, `operator-action`, `ready`, or `refused` | Exactly one |
| status and reason | Stable decision or terminal state | Never derived from display prose |
| selected case | Exact cold launch case | Present for runnable and ready actions |
| requested protocols | Normalized measurement requests | May be empty |
| observed protocols | Eligible final-client candidates from the just-completed session | May be empty and does not itself imply positive facts |
| completed protocols | Requested protocols with current exact positive facts | Derived through fresh S139 evaluation |
| remaining protocols | Requested or newly observed protocols still requiring a step | Carried into the continuation command |
| limitations | Every S139 limitation | Preserved before effects |
| next command | Durable target, effective local store, and remaining candidates | Optional, parseable, no hidden state |

## DeepCaptureRunOutcome

| Field | Meaning | Rule |
| --- | --- | --- |
| exit | Existing CLI exit status | Preserved exactly |
| observations | Typed terminal retained observations | Available only after a session reached terminal reporting |

The existing public low-level command still projects this value to `Exit`. The guided caller uses observations only for automatic candidate derivation and relies on a fresh fact read for completion.

## State Transitions

```text
parsed -> target-resolved -> proposed
  -> refused
  -> operator-action
  -> ready-coverage-unknown
  -> reachability-selected -> authorized-session -> observations-derived -> facts-reassessed -> continuation
  -> protocol-selected -> authorized-session -> observations-derived -> facts-reassessed -> continuation-or-requested-complete
```

No transition before a selected authorized session allocates a bundle or prepares certificate, proxy, routing, launch, Capture, artifact, or fact effects. No transition begins a second session.

## Validation Rules

- Candidate input uses the existing closed enum and normalizes before proposal construction.
- Routing and not-applicable cannot be guided protocol candidates.
- Reachability remains first whenever the exact route is not currently positive.
- `run-protocol` requires one first S139 TLS-phase step and current exact routing evidence.
- Automatic candidates require concrete final-client correlated observations.
- Reassessment uses the same target, launch case, route, address family, backend, versions, and target-version applicability dimensions.
- Generated commands use the decimal durable identifier and a PowerShell-quoted effective local-store path.
- Remaining candidates are explicit command arguments, not durable workflow records.
