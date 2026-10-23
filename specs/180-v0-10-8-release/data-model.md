# Data Model: S180 publication identities

## Candidate

The candidate binds version 0.10.8, release branch head, complete changelog fragments, generated first-party evidence, review findings, and exact-head CI. Candidate source does not assert public availability.

## Published release

The annotated `v0.10.8` tag points to the accepted merged source. Its tag object and peeled commit are immutable. Final package certification binds the six public files and three sidecar checksums to that source.

## Registry publication

Each of ten product crates has a non-yanked 0.10.8 record and independently downloaded archive checksum. A pending protected deployment is not publication.

## Publication record

`docs/published-release.json` and current public markers identify only a release whose files and crates have already been independently reconciled. Historical records retain their original source and evidence.
