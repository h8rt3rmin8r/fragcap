# CLI Contract: Guided Protocol Calibration

## Synopsis

```text
fragcap calibrate <SELECTOR> [--protocol <PROTOCOL>]... [OPTIONS]
fragcap calibrate --target <SELECTOR> [--protocol <PROTOCOL>]... [OPTIONS]
fragcap calibrate --id <STABLE_ID> [--protocol <PROTOCOL>]... [OPTIONS]
```

`--protocol` is repeatable and uses the existing concrete calibration vocabulary except `routing`. It requests measurement and never asserts that the target uses, accepts, or exposes the selected protocol.

## Selection

The command normalizes candidates and passes them to S139 with current target facts. Missing or non-positive exact routing evidence selects the first reachability step. Current positive routing evidence permits the first proposed protocol step. One invocation starts at most one session.

## Observed Candidate Contract

After a terminal session, the facade derives candidates only from observations that have a concrete S120 family and direct final-client correlation. Unknown, unrouted, launcher, intermediate, ambiguous, unavailable, uncorrelated, and packet-only observations do not become candidates. The result is deterministic and deduplicated.

## Decision Event

Every resolved invocation emits `calibration.guidance` with the existing stable fields plus `requested_protocols`, `observed_protocols`, `completed_protocols`, and `remaining_protocols`. All four fields are always present as arrays in JSON. Human mode renders the same sets.

## Actions

### `run-reachability`

The selected S139 routing step delegates to the existing reachability calibration path. On a terminal successful run, eligible observed candidates are placed in the next guided command. No protocol session begins in the same invocation.

### `run-protocol`

The selected S139 TLS step delegates in-process to the existing equivalent:

```text
fragcap deep-capture --id <STABLE_ID> --launch --calibrate tls --calibration-protocol <PROTOCOL> --launch-case <EXACT_COLD_CASE>
```

The candidate protocol controls only the planned measurement. Trust, proxy, launch, collection, fact derivation, cleanup, and terminal outcome remain owned by the low-level session. HAR and key logging remain false unless a later separately specified surface adds them.

### `ready`

No session starts. With no candidate, the existing `ready` status and `current-routing-evidence` reason remain stable, the empty coverage arrays state that protocol coverage is unknown, and the ordinary Deep Capture command remains available. When every supplied candidate has current exact positive evidence, status is `requested-coverage-complete`. Neither status claims universal protocol coverage.

### `operator-action`

Warm behavior remains S140's effect-free guidance or explicit `--restart-warm` handoff. The generated retry preserves the normalized candidate list.

### `refused`

No execution command is invented. Every typed limitation is emitted before a usage or operational error returns.

## Continuation

A generated guided command uses `--id`, the effective `--local-db` path, and one `--protocol` per unresolved candidate. It must parse to the same target and candidate set. It carries no fact, success claim, authorization, or persisted workflow state.

## Compatibility

- Existing low-level calibration options and behavior remain unchanged.
- Existing global JSON, quiet, silent, and exact-plan authorization behavior applies.
- The guided event change is additive.
- Parent #380 remains open.
