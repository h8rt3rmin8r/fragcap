# Contract: Session Causal Diagnostics

## Connection record

The additive `proxy.connection-diagnostic` record keeps `connection_id`, `terminal`, `category`, and `code`. New fields name `schema_version`, `stages` (finite ordered `{stage, reached_ms, completed_ms}` entries), `last_reached`, `last_completed`, `boundary`, `tls_alert`, `transport_category`, `os_error_code`, `header_timeout_phase`, and `completed_exchanges`. A separate `proxy.diagnostic-accounting` record reconciles retained, lost, and unavailable detail. Exact process ownership remains in `application.jsonl` correlation records, joined by the same connection ID; that authority reports exact, unavailable, or ambiguous without inference. Null means a fact was not observed; it must not mean success.

HTTP read and write boundaries come from the operation that failed. Shared error codes do not determine whether the client or origin failed. The upstream TCP stage begins after allowed address resolution and before the first connection attempt.

SOCKS authentication, authority, protocol, upstream TCP, and reply stages are stamped when their negotiation steps occur. Relay or UDP association teardown does not extend an already completed negotiation stage.

## Phase record

The terminal and manifest contain the same `phase_timing` map keyed by startup, capture preparation, acquisition, observation, capture return, owner release, proxy shutdown, and cleanup. Each entry has `status`, `budget_ms`, nullable `elapsed_ms`, and nullable `remaining_ms`. The authorization plan continues to state intended budgets and origins; measured values are attached only after their phase actually starts.

Measured preparation, acquisition, and return intervals survive a Capture error when those operations began before the error. A failed phase cannot be projected as not started merely because Capture returned no successful outcome.

Each phase keeps its own terminal status. An interruption during observation leaves a previously completed acquisition marked completed.

## Compatibility and privacy

New fields are additive. Existing manifest v2 readers can ignore them. The compact record excludes raw authority, URL, payload, credential, certificate identity, private key, capability, and arbitrary error text. Existing sensitive artifacts retain their own exact observations and access rules. Unknown owner and failure details remain explicit.
