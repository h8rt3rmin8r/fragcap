# Data Model: Guided Reachability Calibration Front Door

S140 adds transient CLI decision values only. It does not change SQLite, bundle, manifest, compatibility-fact, or public Rust API schemas.

## GuidedCalibrationRequest

| Field | Meaning | Rule |
| --- | --- | --- |
| target selector | Positional or explicit selector | Exactly one selector or durable identifier |
| stores | Optional existing catalog and local overrides | Local store must already exist |
| bundle and bounds | Existing reachability destination and finite limits | Validated only for a runnable proposal |
| authorization | Interactive terminal or exact standard-input plan identifier | Existing S134 rules apply |
| restart warm | Explicit normal close-and-retry request | False by default |
| controlled target | Hidden offline harness selector | Existing test authority only |

## GuidedCalibrationDecision

| Field | Meaning | Rule |
| --- | --- | --- |
| target id and handle | Durable identity and display label | Present after resolution |
| topology | Steam, direct, publisher, or unavailable | Copied from S139 |
| action | `run-reachability`, `operator-action`, `ready`, or `refused` | Exactly one |
| status | Stable decision or terminal status | Never derived from display prose |
| observed case | Current warm case | Operator action only |
| selected case | Exact cold case | Run, operator action, and ready |
| reason | Retest or stable decision reason | Exact machine value |
| images | Declared process image set | Warm guidance only |
| limitations | Every S139 limitation | Preserved in proposal order |
| process control | Whether fragcap controlled an existing process | Always `none` |
| next command | Durable-identifier invocation bound to the effective local store | Ready, operator action, or completed run |

## State Transitions

```text
parsed -> target-resolved -> proposed
  -> refused
  -> operator-action
  -> ready
  -> reachability-selected -> delegated-session -> reassess-recommended
```

No transition before `reachability-selected` allocates a bundle or prepares certificate, proxy, routing, launch, Capture, artifact, or fact effects.

## Validation Rules

- Exactly one target selector namespace is active.
- The resolved target has local and durable identifiers.
- S139 receives a complete process list or unavailable snapshot, never a fabricated empty list.
- `run-reachability` requires one first reachability step with routing protocol.
- `ready` requires zero steps plus current exact positive routing evidence.
- Any TLS step, protocol candidate, non-child routing, or non-IPv4 family is refused.
- Generated commands use the decimal durable identifier and a PowerShell-quoted effective local-store path.
