<!-- spec-impact: 13.7, 15, 17.2.1, 19, 25, 28.1 -->
Store compatibility facts append-only in schema version 10 with additive route,
address-family, and protocol columns. Legacy-incomplete, stale, or mismatched
rows remain visible but cannot authorize work; the latest exact applicable row
controls reuse. Retests append conflicting evidence instead of rewriting it,
and no elapsed-time threshold fabricates staleness. This slice adds no
dependency, bypass policy, or Deep Capture completion claim.
