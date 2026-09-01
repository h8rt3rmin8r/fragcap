# Research: Warm-To-Cold Restart

## Decision 1: Operator-owned normal shutdown

**Decision**: fragcap never sends a stop, signal, window message, protocol exit, or force-kill action. It asks the operator to close the application normally and only observes image absence.

**Rationale**: The allowed snapshot carries image names but not canonical executable paths. Presence therefore cannot prove selected-target identity. A generic close action could affect an unrelated same-named process and violate P-1 and #309 acceptance.

**Rejected alternatives**: `TerminateProcess`, task-kill fallback, window-message shutdown, Steam-wide exit dispatch, and process-handle path inspection.

## Decision 2: Two distinct confirmations

**Decision**: One confirmation authorizes bounded waiting while the operator closes the application. After cold state, a second confirmation authorizes the newly prepared session. `--yes` explicitly pre-confirms both.

**Rationale**: Facts may change during shutdown. Authorization must bind to the plan prepared afterward, not a stale warm observation.

## Decision 3: Absence of the complete declared image set

**Decision**: Cold requires one complete successful snapshot in which every image named by the current direct, Steam, or publisher launch declaration is absent.

**Rationale**: A remaining helper or intermediate can retain warm environment. Absence does not prove prior ownership, but it safely proves there is no observed same-named process to inherit stale environment.

## Decision 4: Existing wait bound, capped at two minutes

**Decision**: The effective restart deadline is `min(--wait, 120 seconds)`, defaulting to 120 seconds, with a bounded inventory interval.

**Rationale**: The operator already understands `--wait` as acquisition time. A fixed maximum prevents indefinite preflight.

## Decision 5: Ordinary Deep Capture only

**Decision**: `--restart-warm` conflicts with calibration and the hidden controlled target.

**Rationale**: #317 owns calibration expansion. Mixing declared warm calibration tokens with a resulting cold case would blur evidence authority.
