# Data Model: v0.10.14 Release Candidate

## Timing Defect Receipt

- **Issue**: #566.
- **Source observation**: exact merged-main hosted run and failed attempt identity.
- **Failure**: controlled startup exceeded an unrealistically tight test-only one-second launch allowance.
- **Correction**: test-only finite launch allowance aligned with adjacent controlled calibration cases.
- **Non-change**: production defaults, explicit operator deadlines, duration, parsing, and named late-launch failures.
- **Acceptance evidence**: repeated focused Windows outcomes plus unchanged deadline-boundary checks.

## Candidate Source Identity

- **Version**: 0.10.14.
- **Branch**: `release/0.10.14`.
- **Base**: merged S194 main commit `1e44519a8d34406fcd3178365c1dde68b689439d`.
- **Mutable until handoff**: yes, but every source change invalidates prior final-head evidence.
- **Terminal state**: exact PR head with complete version, notes, generated-output, dependency, package, and CI reconciliation.

## Public Release Identity

- **Version**: 0.10.13.
- **Authority**: published release registry, frozen review candidate, public assets, crates, and current markers.
- **Mutation in S195**: none.
- **Successor state**: only a later owner-authorized publication and independent readback may advance it.

## Release Fragment

- **Fields**: slice or issue identity, category, optional specification impact, bold ISO date lead, concise decision or user-visible outcome.
- **Transition**: unreleased file -> validated source -> consumed once into v0.10.14 history.
- **Invalid state**: heading lead, missing date, duplicate consumption, or retained consumed fragment.

## Dependency Graph Receipt

- **Graph kind**: product, fuzz, performance, packaging, or other isolated lockfile.
- **Fields**: manifest, lockfile, first-party version, package identities, features, sources, provider selection, policy result.
- **Invariant**: candidate preparation may update first-party versions but cannot accept unexplained third-party drift.

## Qualification Receipt

- **Fields**: exact source head, command or hosted run, platform, terminal outcome, artifact identity, and evidence limit.
- **Kinds**: local source, hosted source, exact-head package, conformance, notes, documentation, dependency, and review.
- **Invalidation**: any source change after the receipt.

## Review Receipt

- **Fields**: PR, head, reviewer surface, round number, reaction or comment identity, response, correction commit, thread resolution, and final check state.
- **Round bound**: automatic round one plus at most one explicit round two.
- **Terminal state**: every actual comment answered, every thread resolved, no third request.
