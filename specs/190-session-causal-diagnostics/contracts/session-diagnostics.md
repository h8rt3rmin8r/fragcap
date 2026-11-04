# Contract: Session Causal Diagnostics

## Connection record

The additive `proxy.connection-diagnostic` record keeps `connection_id`, `terminal`, `category`, and `code`. New fields name `schema_version`, `stages` (finite ordered `{stage, reached_ms, completed_ms}` entries), `last_reached`, `last_completed`, `boundary`, `tls_alert`, `transport_category`, `os_error_code`, `header_timeout_phase`, and `completed_exchanges`. A separate `proxy.diagnostic-accounting` record reconciles retained, lost, and unavailable detail. Exact process ownership remains in `application.jsonl` correlation records, joined by the same connection ID; that authority reports exact, unavailable, or ambiguous without inference. Null means a fact was not observed; it must not mean success.

## Phase record

The terminal and manifest contain the same `phase_timing` map keyed by startup, capture preparation, acquisition, observation, capture return, owner release, proxy shutdown, and cleanup. Each entry has `status`, `budget_ms`, nullable `elapsed_ms`, and nullable `remaining_ms`. The authorization plan continues to state intended budgets and origins; measured values are attached only after their phase actually starts.

## Compatibility and privacy

New fields are additive. Existing manifest v2 readers can ignore them. The compact record excludes raw authority, URL, payload, credential, certificate identity, private key, capability, and arbitrary error text. Existing sensitive artifacts retain their own exact observations and access rules. Unknown owner and failure details remain explicit.
