# Completed Review Record Contract

`cargo xtask review-record <PATH>` validates a reviewer-supplied public-safe JSON record against the immutable candidate and the closed twelve-area scope.

## Exit behavior

- Exit zero only when the JSON is structurally valid and contains no mechanical acceptance blocker.
- Exit nonzero for unreadable or oversized input, schema violations, candidate drift, incomplete scope, blocked findings, unsafe public content or inconsistent summary counts.
- A zero exit states only that no mechanical blocker was found. It does not certify reviewer authenticity or independence and does not close #333.

## Closed top-level shape

The only top-level keys are `schema_version`, `state`, `candidate`, `reviewer`, `independence`, `environment`, `checks`, `findings` and `summary`. Unknown keys fail validation.

## Required coverage

The record contains each area in `docs/security/native-product-review-scope.v1.json` exactly once. It also identifies methods and evidence for listener binding, destination refusal, trust recovery, artifact protection and export, parser abuse, installer lifecycle and the installed QUIC retest owned by #413.

## Findings gate

Critical and high findings cannot be accepted or waived. They require remediation on a distinct fixed candidate and a separate passed independent retest whose reviewer identifier is nonempty and differs from the original reviewer. Finding disposition is closed to `remediated`, `accepted-by-owner`, `rejected` or `not-reproducible`; critical and high findings require `remediated`. Medium findings require an owner and explicit terminal decision. Any unresolved required result blocks completion.

## Public safety

The file is limited to 512 KiB. Strings and collections are bounded. Evidence uses stable opaque identifiers, hashes, visibility and optional HTTPS URLs. The validator rejects obvious local paths, credentials, private-key material, raw payload markers and host-specific identifiers.

## Synthetic fixtures

Tests may construct records under temporary paths. Such fixtures are never committed as a completed review and never described as real reviewer evidence.
