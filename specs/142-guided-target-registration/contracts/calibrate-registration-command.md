# CLI Contract: Guided Target Registration

## Synopsis

```text
fragcap calibrate <SELECTOR> [OPTIONS]
fragcap calibrate --target <SELECTOR> [OPTIONS]
fragcap calibrate --id <STABLE_ID> [OPTIONS]
```

The visible argument surface is unchanged. `--id` remains stored-only. A positional or `--target` stored miss triggers current installed-game discovery. A numeric stored miss is interpreted as a Steam application identifier; other tokens match candidate display names with Unicode-aware case folding and exact equality.

## Selection

1. Resolve the stored target.
2. Return a stored ambiguity unchanged.
3. Only on no match, run the existing bounded discovery composition.
4. Select zero, one, or several exact candidates.
5. Zero is a truthful no-match. Several are an explicit ambiguity. One produces a registration plan.

No fuzzy, substring, folder, executable-hint, predicted-handle, or path-prefix selection exists.

## Registration Plan Event

`calibration.registration_plan` contains:

- `plan_id`
- `canonical_json`
- every `discovery_*` account field
- `discovery_warning_count`

The canonical object contains schema, operation, effective local store, predicted stable id, the complete selected candidate projection, every conserved discovery count, and the discovery warnings. Human output renders the same authority before input. The machine event also exposes every count and the warning count as top-level scalar fields.

## Confirmation

Interactive mode asks whether to register the displayed exact plan and defaults to no. JSON mode requires `--authorize-stdin`; the response must equal the current plan identifier followed by one newline. Input is flushed after the plan is emitted and before the response is read.

If calibration later selects an effectful attempt, its existing S134 plan is emitted separately and consumes another input line. A registration identifier never authorizes a Deep Capture plan and vice versa.

## Revalidation and Persistence

After affirmative confirmation, discovery is run again. The selected candidate must reproduce the same canonical plan. Drift emits `calibration.registration` with `status=drifted` and performs no target insert. A changed selector result retains its exact no-match or complete ambiguity diagnostic.

An unchanged candidate is passed alone to `fragcap::targets::register_candidate`. The command then resolves the resulting stored entry by canonical Steam anchor or exact path install root. `registered` and `already-present` are both idempotent success outcomes only when that exact target can be recovered.

## Outcome Event

`calibration.registration` contains:

- `plan_id`
- `status`
- `reason`
- optional `target_id`
- `continued`

No outcome field claims launch topology, routing, inspectability, calibration readiness, or compatibility.

## Continuation

After durable resolution, the command clears the original discovery selector and enters every S139-S141 resolution and delegated attempt by stable id. Existing `calibration.guidance` and low-level events retain their meanings. Because the shared registration operation intentionally stores discovery candidates without launch entries, a new row reaches the existing `missing-launch-declaration` limitation rather than receiving fabricated launch authority. A later authoritative target remains subject to its own current Deep Capture plan authorization.

## Compatibility

- Existing stored-target behavior and visible flags remain unchanged.
- Existing low-level calibration and plan authorization remain unchanged.
- The two registration events are additive.
- Parent #380 remains open.
