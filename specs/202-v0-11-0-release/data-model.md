# Data Model: v0.11.0 Release Identity

## Candidate source

- **Identity**: version 0.11.0, review branch, final PR head, accepted baseline merge, complete change set since v0.10.14.
- **Authorities**: workspace version, generated CLI and package identities, dependency locks, conformance reports, changelog, release notes, specification Applies-To, and candidate documentation.
- **Rule**: All candidate authorities agree. Public pointers continue to identify v0.10.14 before publication.

## Accepted merge

- **Identity**: owner-merged candidate commit and exact tree.
- **Relationship**: The immutable tag must select this source. A source change after review requires renewed candidate checks before tagging.

## Public release

- **Identity**: annotated tag, peeled source, GitHub publication time, six certified Windows files, tagged package report, ten registry package records, and protected deployment result.
- **State**: candidate -> owner-accepted -> tagged -> GitHub-published -> registry-published -> independently verified.
- **Rule**: A missing or mismatched file, checksum, registry archive, or required job prevents transition to independently verified.

## Current-version records

- **Identity**: machine-readable published version and source, website current baseline, release handoff, and frozen public review candidate.
- **Relationship**: These records may move to v0.11.0 only after the public release is independently verified. Historical v0.10.14 records remain intact.

## Verification receipt

- **Fields**: exact source or artifact, test or workflow, result, time, evidence pointer, and limit of the observation.
- **Rule**: Candidate checks, public package readback, registry readback, installed observation, and external reviews remain separately named.
