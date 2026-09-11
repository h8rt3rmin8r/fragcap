# CLI Contract: Guided Steam Client Setup

## Synopsis

```text
fragcap calibrate <SELECTOR> [OPTIONS]
fragcap calibrate --target <SELECTOR> [OPTIONS]
fragcap calibrate --id <STABLE_ID> [OPTIONS]
```

The visible argument surface is unchanged. Setup is an internal guided state reached only after one exact target is resolved and before the S139 proposal is evaluated.

## Eligibility

1. Resolve or confirmation-gate registration through S142.
2. If any launch declaration is present, skip setup and preserve existing behavior.
3. Parse one positive canonical Steam application identifier from the target anchor.
4. Run the existing bounded discovery composition.
5. Require exactly one current Steam candidate with that application identifier.
6. Require exact non-empty install-root agreement and one suitable executable proposal.
7. Emit one complete setup plan, or report a condition-specific setup limitation without mutation. No candidate, multiple candidates, absent stored install authority, install-root disagreement, absent executable metadata, and unsafe executable syntax remain distinct.

No display-name, folder, predicted-handle, substring, or path-prefix join exists at this boundary.

## Setup Plan Event

`calibration.steam_client_plan` contains:

- `plan_id`
- `canonical_json`
- `target_id`
- `steam_app_id`
- `executable`
- every `discovery_*` account field
- `discovery_warning_count`

The canonical object contains schema, operation, effective local store, complete stored target, complete current candidate, complete discovery authority, proposed executable, exact resulting client declaration, and the explicit effects this plan does not authorize. Human output renders the same canonical object before input.

## Confirmation

Interactive mode asks whether the exact displayed executable is the socket-holding client and defaults to no. JSON mode requires `--authorize-stdin`; the response must equal the current setup-plan identifier followed by one newline. The plan is flushed before input.

A preceding target registration plan and a later Deep Capture plan each require their own response. No response is transferable across the three domains.

## Revalidation and Conditional Persistence

After affirmative confirmation, the command re-resolves the target by stable identifier and repeats the bounded discovery join. It rebuilds the complete plan and requires byte-equivalent canonical authority. Changed selection, candidate data, discovery accounting, warning text, store identity, target field, executable, or result is drift.

The shared target store then starts an immediate transaction and compares its current complete row to the planned row. A match updates only `launch_entries` to one client entry and `fidelity` to `authored`. A changed or missing row remains untouched and produces a non-success outcome. A target re-read, transaction, store, or post-update verification failure emits one terminal failed setup outcome before the operational error is returned.

## Outcome Event

`calibration.steam_client` contains:

- `plan_id`
- `status`
- `reason`
- `target_id`
- `continued`

No outcome field claims routing, propagation, inspectability, protocol coverage, session authorization, or directly observed socket ownership.

Every emitted setup plan has exactly one terminal setup outcome, including persistence failure.

## Continuation

After `applied`, the command re-resolves the target by stable identifier and confirms the exact authored result. It then takes the normal process snapshot and enters the existing S139 proposal. Existing warm, ready, reachability, protocol, continuation, and refusal paths remain authoritative. Any effectful attempt emits and consumes a separate current Deep Capture authorization plan.

## Compatibility

- Existing visible flags and low-level calibration commands are unchanged.
- Existing registration, guidance, and Deep Capture events retain their meanings.
- Existing present launch declarations are never changed.
- The two setup events and one facade store outcome type are additive.
- Parent #380 remains open.
