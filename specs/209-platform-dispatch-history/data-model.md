# Data Model: Platform Dispatch and Advisory Capture History

## Dispatch selection

A dispatch selection is immutable after authorization. It carries the target stable identity, known platform association or direct declaration, selected method, at most one named alternate, exact executable and application identifier where applicable, game-only route and scope, acquisition deadline, and cleanup ownership class. The selected and alternate methods share the same final-game route authority; neither method widens payload scope.

## Warm platform identity

A warm platform identity contains a canonical executable path, process ID, creation instant, account and logon session, and observation instant. It is query-only. A candidate must still be live and match the prepared association when dispatch occurs. The generation is never a process-control receipt. The post-dispatch game generation must be new, path-matched, and uniquely linked through observed creation-time ancestry or another exact platform dispatch receipt. Ambiguous links remain unavailable.

## Capture history row

The local store adds a target-scoped ordered history table in schema version 17. Each row has a monotonically increasing local identifier, target foreign key, record version, observed time, optional native session identifier, and a validated JSON payload. The payload retains selected and observed launch/route identities, optional target build clue and Fragcap version, requested and effective scope, process proof state, coarse transfer and inspection states, one failure code, coarse loss state, and cleanup state. Unknown is a first-class value for each unobserved dimension. The native bundle carries the detailed counts and trace. No captured body, credential, raw endpoint, or local bundle path belongs in this table.

Insertion and oldest-row pruning occur in one transaction. The limit is 100 retained rows per target. Pruning never selects the inserted row and reports the number removed. Target deletion cascades to its history. Existing compatibility facts and native bundles are not rewritten.

## Drift assessment

The assessment compares two retained rows only after target identity, requested and effective scope, selected route identity, and product version agree. When both target build clues are known, they must agree. Unknown build clues may yield an explicitly provisional comparison, never a claim of unchanged target software. The assessment reports the fields that changed and the comparison basis. It is read-only and cannot affect admission, plan creation, or route selection.
