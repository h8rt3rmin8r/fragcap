# S156 Research

## R-1: Structured lifecycle events are the positive authority

**Decision**: Parse NDJSON stderr and require exactly one `deep_capture.proxy_started` event plus exactly one terminal `deep_capture.calibration_phase` event with status `reached-client`, both carrying the same nonempty session identifier and the exact controlled case dimensions.

**Rationale**: These records are emitted by the product from durable lifecycle transitions. Socket-table polling is an external sample of a connection that may open and close between 100 ms observations.

**Rejected alternatives**: Increasing polling frequency or request count reduces but does not remove the race. Retrying failed campaigns masks nondeterminism. Free-text substring matching does not bind fields or session identity.

## R-2: Preserve socket sampling as negative evidence and diagnostics

**Decision**: Continue sampling the complete descendant process set and all owned TCP and UDP endpoints. Fail on any unexpected process path or non-loopback address. Record sample count, endpoint count and whether exact loopback was sampled, but allow zero endpoints and `loopback_socket_observed=false`.

**Rationale**: A missed transient endpoint proves nothing. An observed unexpected owner or non-loopback endpoint is concrete contradictory evidence and remains security-relevant.

**Rejected alternatives**: Removing socket observation loses defense-in-depth. Treating wildcard binds as non-loopback would reject ordinary listener state; the existing distinction between allowed wildcard diagnostics and exact loopback positive samples remains useful.

## R-3: Version the report at schema 4

**Decision**: Emit schema 4 with separate `firewall_containment`, `structured_reachability` and `socket_observation` fields in each smoke row. Keep schema 2 and schema 3 as read-only compatibility branches under their original semantics.

**Rationale**: Reusing schema 3 while changing the meaning of `network_observation` would make historical and current evidence indistinguishable.

**Rejected alternatives**: Silently loosening schema 3 breaks auditability. Removing legacy validation would make retained reports unreadable.

## R-4: Use closed predicate diagnostics

**Decision**: Evaluate all smoke predicates into a deterministic ordered list of at most sixteen stable identifiers, render no observation values, and cap the final diagnostic message at 1 KiB.

**Rationale**: The compound S155 error hid the failing subpredicate. Closed identifiers provide actionable CI evidence without leaking paths or endpoint values.

**Rejected alternatives**: Dumping the parsed event or observation object is easier but exposes host-specific values. Throwing at the first predicate loses simultaneous failure information.

## R-5: Keep parsing in the orchestration boundary and contract validation in Rust

**Decision**: PowerShell parses the product's existing NDJSON because it owns the child process and the raw output. Rust validates the bounded report and supplies exhaustive mutation coverage.

**Rationale**: Adding a product command or library solely for package certification would expand shipped surface. Duplicating report rules in ad hoc workflow expressions would fragment authority.

**Rejected alternatives**: Moving all parsing into Rust requires a new interface and does not improve the hosted child boundary. Leaving all validation in PowerShell would weaken type-driven mutation coverage and P-7.

## R-6: Hosted execution is the only product-level verification

**Decision**: Run local unit, static, format and compliance checks only. Require both affected package workflows to execute the final head on disposable hosted Windows.

**Rationale**: This satisfies the operator's explicit safety boundary while testing the real package and installed surfaces that failed.

**Rejected alternatives**: Local installed execution is prohibited. Source-only Rust tests cannot prove Windows firewall, MSI and process observation integration.
