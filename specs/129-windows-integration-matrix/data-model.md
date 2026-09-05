# Data Model: Native Windows Integration Matrix

## Windows Integration Registry

Immutable versioned review authority.

Fields:

- `schema_version`: Exact supported schema, initially `1`.
- `reviewed_on`: Date of the registry decision.
- `physical_evidence_max_age_days`: Maximum release-authorizing evidence age.
- `authorities`: Closed set of row authority classes.
- `capabilities`: Closed host capability vocabulary.
- `effects`: Closed owned-effect and prohibited-effect vocabulary.
- `completion_domains`: Source-derived release-domain inventory.
- `rows`: Ordered required integration rows.

Validation:

- All collections are non-empty, sorted where order is not semantic, and duplicate-free.
- The row identity set is exact and stable for schema version 1.
- Every completion domain has at least one row and every row maps to a known domain.
- Unknown authority, capability, effect, outcome, or publication values fail validation.

## Integration Row

Fields:

- `id`: Stable lowercase hyphenated identity.
- `domain`: One completion-domain identity.
- `tier`: `hosted` or `physical`.
- `authority`: Production authority exercised by the row.
- `required_capabilities`: Exact predicates over the immutable host snapshot.
- `evidence`: Executable source reference or staged-binary probe identity.
- `expected`: Exact success, refusal, unavailable, or recovery terminal.
- `owned_effects`: Effects the row may create.
- `prohibited_effects`: Effects that must remain unchanged.
- `cleanup`: Exact terminal inventory assertion.
- `publication`: `summary-only` or `none`.
- `timeout_seconds`: Finite row deadline.

State transitions:

```text
declared -> eligible -> running -> passed | failed | incomplete
```

`ineligible` is not a successful terminal. A capability-specific refusal row remains eligible because its predicate requires the absent or denied state.

## Host Capability Snapshot

Fields:

- `platform`: Exact Windows product family.
- `architecture`: Process architecture.
- `elevated`: Observed token elevation.
- `npcap_runtime`: `present` or `absent`.
- `npcap_compatibility_mode`: Observed WinPcap-compatible DLL state.
- `analyzer`: `present` or `absent`, with version when present.
- `ipv4_loopback`: Bind result.
- `ipv6_loopback`: Bind result.
- `interactive_desktop`: `available` or `unavailable`.
- `binary_sha256`: Digest of the staged executable under test.
- `product_version`: Executable-reported version.
- `source_revision`: Commit identity supplied by the runner.

The raw snapshot may include local diagnostic detail, but the public-safe projection includes only closed capability values and version/digest identities.

## Row Result

Fields:

- `sequence`: Monotonic report sequence.
- `row_id`: Registry identity.
- `started_unix_ms` and `finished_unix_ms`: Bounded timing.
- `observed`: Closed terminal outcome.
- `evidence_codes`: Bounded typed facts.
- `effects_before` and `effects_after`: Digests of normalized inventories.
- `cleanup`: `reconciled`, `retained-as-authorized`, or `failed`.
- `failure`: Bounded code and detail class, never raw output.

Validation:

- One result per required row.
- Result order follows registry order.
- The observed outcome equals the row expectation.
- Failed or timed-out rows cannot be converted into omissions.

## Run Report

Newline-framed, append-safe local evidence.

Record kinds:

- `header`: Registry digest, snapshot digest, binary identity, source revision, and run tier.
- `row`: One `Row Result`.
- `terminal`: Expected/observed identity sets, effect reconciliation, capability stability, and complete/incomplete result.

Only a report with one valid terminal is complete. A prefix remains truthful incomplete evidence.

## Public-Safe Summary

Fields:

- Schema and registry digest.
- Source revision, product version, and staged binary digest.
- Closed host capability projection.
- Row identities, outcomes, durations, evidence codes, and cleanup status.
- Aggregate completeness and residue counts.
- Evidence date and expiry date for physical authority.
- Explicit omitted raw evidence classes.

The summary contains no free-form child output or machine identity field.
