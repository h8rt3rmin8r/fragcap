# S155 Data Model

## PublishedReviewCandidateV1

- `schema_version`: exact integer `1`.
- `product_version`: exact release version.
- `tag`: exact Git tag.
- `source_revision`: forty lowercase hexadecimal characters.
- `lockfile_sha256`: sixty-four lowercase hexadecimal characters.
- `release_url`: canonical HTTPS release URL.
- `release_run_id`: exact positive GitHub Actions run identifier.
- `target`: exact package target triple.
- `features`: closed shipped feature list.
- `assets`: exactly six unique asset records covering ZIP, MSI, catalog and three checksum sidecars.
- `predecessor`: exact prior release ZIP used for upgrade certification.

Each asset has a stable role, filename, HTTPS URL, byte size and SHA-256 digest. No asset can be inferred by filename alone.

## CompletedReviewRecordV1

- `schema_version`: exact integer `1`.
- `state`: exact value `performed`.
- `candidate`: exact projection of the published candidate registry.
- `reviewer`: stable public identity, organization or affiliation, contact and review timestamps.
- `independence`: contribution declaration, conflicts, relationship statement and HTTPS attestation reference.
- `environment`: disposable Windows identity, installed binary digest, privilege class, network containment, synthetic-data declaration, tools and limitations.
- `checks`: exactly one result for each review-scope area plus the required installed execution cases.
- `findings`: bounded unique finding records.
- `summary`: reviewer conclusion, derived severity counts, P-1 result, no-open-proxy result, limitations and public-safe evidence identity.

## AreaResult

- `area`: one exact identifier from `native-product-review-scope.v1.json`.
- `outcome`: `passed`, `failed` or `indeterminate`.
- `method`: bounded public-safe method description.
- `evidence`: one or more stable evidence references.
- `findings`: zero or more finding identifiers.
- `limitations`: bounded public-safe list.

There are exactly twelve area results. Duplicate, missing or unknown areas are invalid. `failed` and `indeterminate` remain acceptance blockers.

## Finding

- `id`: unique stable public identifier.
- `area`: exact covered area.
- `severity`: `critical`, `high`, `medium`, `low` or `informational`.
- `candidate_revision`: exact reviewed source revision.
- `reproduction`, `impact`, `evidence`, `owner`, `linked_issue`, `disposition`, `remediation`, `retest`: bounded typed facts.

Critical and high findings require a fixed candidate distinct from the reviewed candidate and a separately attributed passed retest with a nonempty different reviewer identity. Disposition is exactly `remediated`, `accepted-by-owner`, `rejected` or `not-reproducible`; critical and high findings require `remediated`. Medium findings require an owner and explicit disposition. No severity supports implicit risk acceptance.

## EvidenceReference

- `id`: stable opaque identifier.
- `sha256`: exact lowercase SHA-256.
- `visibility`: `public` or `private`.
- `url`: optional HTTPS URL.

Paths, payloads, credentials, host identifiers and key material are prohibited from the public record.

## PackageCertificationReportV3

- Retains all version 2 package, identity, lifecycle, residue and payload fields.
- Replaces singular `smoke` with `smokes` containing exactly two rows.
- Row `surface` is exactly `portable` or `installed` and unique.
- Each row records executable identity, backend identity, reached-client state, complete sample count, unexpected process paths, non-loopback observations, loopback observations and cleanup state.

Both rows must pass independently. Missing, duplicate or incomplete rows fail validation.
