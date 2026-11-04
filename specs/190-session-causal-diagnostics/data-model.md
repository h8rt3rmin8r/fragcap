# Data Model: Session Causal Diagnostics

## Connection diagnostic

One accepted connection ID owns one bounded terminal diagnostic. Its existing terminal, code, and category remain. Add schema version, ordered stage observations, last reached and last completed stage, boundary, alert or transport category when observed, first-request and completed-exchange facts. Stage observations have monotonic milliseconds from accept. A stage can be reached without completion. Connection detail lost and unavailable counters remain authoritative for records not retained. Exact process ownership is a separate application correlation record joined by connection ID; missing or ambiguous ownership is reported there without inference.

## Stage vocabulary

`accept`, `authentication`, `authority`, `upstream-tcp`, `tunnel-response`, `client-tls`, `upstream-tls`, `protocol`, `first-request`, and `terminal` are a finite vocabulary. Each connection records milestones in observed order; a protocol path records only milestones it actually encounters. Repeated exchanges increment bounded counters; they do not append an unbounded timeline.

## Phase measurement

Each authorized phase has a name, budget in milliseconds, terminal state (`not-started`, `completed`, `interrupted`, or `failed`), optional elapsed milliseconds, and optional remaining milliseconds. The phase owner measures elapsed from its actual monotonic start to end; these instants are not exported as separate timestamps. `remaining = max(budget - elapsed, 0)` only when the phase started. The value is historical unused budget, not permission to resume.

## Relations

Connection IDs join proxy lifecycle, application observations, calibration diagnostics, and exact process-instance correlation when independently proven. Phase measurements join the pre-effect authorization plan and final manifest by the session ID. Missing joins remain explicit and do not change a protocol fact into an ownership fact.
