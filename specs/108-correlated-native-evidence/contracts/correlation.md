# Contract: Native correlation

- Every observation names a connection and starts as `deferred`.
- One final correlation record per accepted connection appears in id order before the trailer.
- Final states are matched, flow-only, ambiguous, or unavailable, each with a closed reason.
- Joins require endpoint and time overlap and never choose a nearest or latest owner.
- Connection and application accounting equations reconcile.
- UDP and QUIC keys are representable but remain non-exports until later handlers land.
