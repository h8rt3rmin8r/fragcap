# Data Model: Native Protocol Conformance

## ConformanceMatrix

- `schema_version`: exactly `1`
- `product_version`: exact workspace version
- `generated_by`: stable generator identity
- `required_protocols`: closed ordered set
- `implementations`: unique `ImplementationIdentity` values
- `rows`: unique `ConformanceRow` values
- `coverage_rules`: minimum clients, origins, failure cases, artifacts, and tiers

## ImplementationIdentity

- `id`: stable unique identifier
- `role`: `client`, `origin`, `analyzer`, or `proxy`
- `name`: human-readable implementation
- `version`: exact version or exact tool-version source
- `driver_lineage`: identity used for independence counting
- `transport`: sync wire, async wire, HTTP library, HTTP/2 library, or external analyzer

Two identities with the same `driver_lineage` count once even when their ids differ.

## ConformanceRow

- `id`: stable unique row id
- `required`: whether absence or non-pass fails the gate
- `protocol`: closed protocol family
- `case`: positive or exact failure class
- `standards`: one or more stable standard references
- `client_id`: implementation reference
- `origin_id`: implementation reference
- `tls`: version and chain expectation when applicable
- `expected`: exact semantic outcome
- `observed`: normalized semantic outcome
- `status`: `pass`, `fail`, `skip`, `not-run`, or `informational`
- `evidence`: executable test ids and artifact assertion ids
- `tier`: portable, Windows, or analyzer

Only `pass` satisfies a required row. `informational` is permitted only when `required` is false.

## ArtifactAssertion

- `id`: stable identifier
- `role`: application JSON Lines, HAR, TLS key log, pcapng, correlation, proxy lifecycle, cleanup lifecycle, cleanup summary, resource journal, or manifest
- `authority`: production reader, schema, or external analyzer
- `expected`: normalized fact
- `observed`: normalized fact
- `status`: pass or fail

## ConformanceReport

- `schema_version`: exactly `1`
- `matrix_digest`: digest of canonical matrix input
- `product_version`: exact workspace version
- `tool_versions`: exact executing tool identities
- `rows`: ordered row outcomes
- `coverage`: derived distinct implementation and case counts
- `artifacts`: ordered artifact outcomes
- `tiers`: portable, Windows, and analyzer outcomes
- `summary`: exact passed, failed, skipped, missing, duplicate, and not-run totals
- `sanitization`: prohibited-material scan result

## Invariants

1. Every required protocol appears in passing required rows.
2. Every required protocol has at least two distinct passing client lineages and two distinct passing origin lineages.
3. Row ids and implementation ids are unique and every reference resolves.
4. Required rows are never skip, not-run, informational, or missing.
5. Expected and observed results match for pass rows.
6. Every integrated row references all required artifact roles.
7. Derived totals equal the actual row and assertion sets.
8. The report is deterministic after normalization and contains no prohibited material.
