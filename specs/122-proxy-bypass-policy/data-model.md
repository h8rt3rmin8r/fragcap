# Data Model: Proxy Bypass and Local-Destination Policy

## BypassRule

Immutable normalized predicate.

Fields:

- kind: exact DNS, DNS suffix, IP, or CIDR
- canonical host or network
- prefix length for CIDR
- optional exact port for DNS and IP rules
- canonical text used for ordering, projection, and evidence

Invariants:

- DNS is lowercase ASCII with valid labels and no trailing root dot.
- Suffix matching occurs only at DNS label boundaries and includes the apex.
- IP aliases use one canonical identity.
- CIDR contains no host bits and has no port.
- Port is nonzero.
- `*`, schemes, paths, user-info, empty tokens, and ambiguous IPv6 authority are invalid.

## BypassPolicy

Immutable session policy assembled before effects.

Fields:

- canonical unique operator rules
- exact session infrastructure exclusions derived from the selected listener
- fixed matching-semantics version
- fixed no-fallback flag

Invariants:

- Input order cannot change identity or output.
- Empty operator rules mean no target bypasses.
- Infrastructure exclusions are displayed separately and cannot masquerade as operator rules.
- Controlled origins never enter infrastructure or operator rules automatically.

## RequestedDestination

Canonical requested authority evaluated before DNS.

Fields:

- host kind: DNS or IP
- canonical host
- port

Invariants:

- DNS resolution is not part of bypass matching.
- IPv4-mapped IPv6 becomes IPv4.
- Invalid or absent ports cannot produce an allow decision.

## RoutingDecision

One localized policy outcome.

Fields:

- outcome: proxied, bypassed, infrastructure, refused, or undetermined
- requested destination when available
- matching rule when applicable
- authority: operator policy, session infrastructure, proxy destination policy, or evidence reconciliation
- stable reason

Invariants:

- Exactly one outcome applies to one localized decision.
- Bypassed is scope, not proxy loss or refusal.
- Infrastructure can never be treated as an upstream target.
- Undetermined never becomes bypassed or proxied by inference.

## RoutingDecisionSummary

Conserved count projection.

Fields:

- proxied
- bypassed
- infrastructure
- refused
- undetermined

Invariant:

The sum equals all localized decisions plus explicitly unlocalized observations in the summary authority.
