# Data Model: S162 v0.10.3 patch release

## Release candidate

Fields: source commit, ten package versions, lockfile identity, specification applicability, generated output identities, conformance identities, assembled changelog, and highlights. State: prepared, checked, reviewed, operator-merged.

## Published release

Fields: annotated tag object and peeled source, GitHub workflow run and four jobs, GitHub release visibility, three certified files and three checksum sidecars, certification report, and ten crates.io version records. State: tagged, hosted-running, approval-waiting, verified, or failed.

## Publication record

Fields: observed source, tag, run, approval history, public file digests, certification result, crate checksums, and published time. It may move current baseline markers only after the published release reaches verified state. Historical v0.10.2 records are immutable.

## Invariants

- The annotated tag peels to the exact operator-merged candidate source.
- Six public files and certification refer to the same 0.10.3 source.
- All ten registry records are exact 0.10.3, non-yanked, and checksum-bearing.
- Candidate data never masquerades as public publication evidence.
