# Data Model: S159 v0.10.2 release

## Candidate identity

`CandidateIdentity` contains version 0.10.2, the release branch head, ten package versions, embedded product versions, generated output versions, specification Applies-To, complete changelog section and release highlights.

Candidate identity is reviewable but is not publication evidence.

## Published identity

`PublishedIdentity` contains the annotated tag object, peeled merged source revision, public release URL and timestamp, release workflow run and four job conclusions.

Published identity exists only after the public workflow completes.

## Asset evidence

`AssetEvidence` contains one expected public filename, byte count and SHA-256 digest. The release has exactly six rows: ZIP, MSI, catalog database and one sidecar for each.

Each sidecar must validate its corresponding file. Certification summary evidence must bind the same version, source, target, backend and feature set.

## Registry evidence

`RegistryEvidence` contains crate name, exact version, yanked state and registry checksum for each of the ten public packages.

Every row must report 0.10.2, `yanked=false` and a nonempty checksum.

## Publication record

`PublicationRecord` projects verified published identity into current repository documentation. It cannot precede complete release and registry evidence and never rewrites historical release handoffs.

## State transitions

The allowed sequence is `specified -> candidate prepared -> PR reviewed -> human merged -> tag pushed -> assets published -> owner approval if requested -> crates published -> publicly verified -> records PR reviewed -> records merged`.

No later state may be inferred from an earlier one.
