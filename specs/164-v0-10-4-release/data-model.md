# Data Model: S164 release states

## Candidate

Fields: version `0.10.4`, source branch, base commit, final PR head, first-party package identities, generated evidence, changelog, notes, local checks, hosted checks, review rounds, unresolved review threads. Candidate-ready means exact final head has required green checks and no unresolved actionable review threads. It does not imply a tag or public files.

## Public release

Fields: annotated tag object, peeled merged source, release workflow run and four job states, protected registry approval, six asset names, byte counts and SHA-256 values, three checksum sidecars, certification report, ten crate version records and checksums. Published means all independent checks pass on exact merged source.

## Current published-state record

Fields: `docs/published-release.json` and current baseline projections. It remains at v0.10.3 during candidate review and advances only through a separate records-only PR after public v0.10.4 verification. Historical evidence is immutable.

## Transitions

Clean merged main -> specified candidate -> reviewed green PR -> owner merge -> exact-source tag -> verified public release -> records-only PR. Failed or pending checks keep the corresponding state incomplete.
