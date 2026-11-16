# Research: Platform Dispatch and Advisory Capture History

## Current source facts

- S207 routes the exact final game directly and leaves ordinary launcher networking native. It removes the prior-positive compatibility gate. This permits a warm platform case without retroactive proxy environment inheritance, provided final-game ownership is proven independently.
- Deep Capture currently classifies Steam as cold or warm by image-name snapshot. The policy refuses warm Steam. Cold Steam is prepared as an exact PlatformClientLaunch, then a one-shot dispatch gate waits for the session-created platform root's creation event.
- The startup Toolhelp snapshot carries process ID, parent ID, image, and possibly start time but does not itself establish creation-time ancestry or exact path. The existing cold platform root uses a launch receipt plus observed event. Warm root proof therefore needs a distinct query-only identity and event bridge; image name alone is insufficient.
- Constitution P-1 forbids memory-read handles and process control of warm or unrelated processes. Query-only process identity is permitted with exact rights, but a warm platform must never enter the session-owned shutdown set.
- The local SQLite store is version 16. Existing deep_capture_facts are typed positive observations with case dimensions. They are unsuitable as a complete per-attempt journal because failure, loss, cleanup, and unknown outcomes may have no positive fact candidate.

## Decisions

### R1: Warm platform proof

Options: trust an image-name snapshot, require cold launch, or use query-only identity plus the existing watcher and exact final-client gate. Choose query-only identity. It must bind one canonical executable, process ID, creation instant, account and session, then require a matching still-live platform root and a new exact final-game generation after dispatch. Refuse ambiguous, exited, changed, or unprovable roots. No warm process receives shutdown rights. This is the narrow path that satisfies #610 without claiming snapshot parentage as a creation event.

### R2: Dispatch and fallback authority

Options: retry after any failed launch, never offer an alternate, or permit one predeclared alternate only after proof that the first method did not start a game. Choose the bounded alternate. Authorization contains both methods and their effects; any uncertain delivery, active game, changed target, or changed process inventory refuses retry. An alternate does not broaden packet or application scope.

### R3: History shape

Options: extend individual compatibility facts, create one per-attempt table, or infer history from native bundles on demand. Choose one versioned per-attempt table in the existing local store. Facts retain their original meaning; a history row stores selected and observed dimensions independently, including unknowns, and links to the native session without copying raw traffic. The migration adds an empty table and leaves earlier rows untouched.

### R4: Retention and comparison

Options: unbounded append, global latest-only, or bounded target-scoped sequence. Choose bounded target-scoped sequence with oldest-row pruning after successful insertion, preserving the current attempt and accounting for pruned rows. Comparison requires the same target, product version, selected launch, scope, and route identity. Known build clues must match. If either build clue is absent, the comparison is explicitly provisional. No history row becomes an authorization input.

### R5: Additional platform adapters

The current concrete adapter is Steam. Retain a platform-neutral dispatch contract and require an exact stored association plus a concrete adapter for any other platform. A missing adapter is a named unsupported association, not a guessed direct path. Controlled Steam adapter tests establish the contract without claiming a shipped integration for a platform the repository cannot yet identify.

## Evidence boundary

Controlled process, store, and route tests plus hosted gates qualify this source change. They do not establish installed ESO launch behavior or a private title's network result.
