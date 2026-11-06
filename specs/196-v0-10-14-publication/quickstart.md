# Quickstart: v0.10.14 Publication Validation

## 1. Freeze Source

```text
git rev-parse main
git show --no-patch --format=fuller 5efb38ee93daff140e1ba79f57eea0a93d5b572f
git tag --list v0.10.14
```

Expected: main names the accepted merge before records work, and no divergent v0.10.14 tag exists.

## 2. Publish

Create annotated tag `v0.10.14` at the accepted merge, push that single tag, watch the `release` workflow, and approve only its configured crates.io environment deployment through normal GitHub review.

Expected: identity, package certification, GitHub release, and registry publication all succeed.

## 3. Verify Public GitHub Bytes

Use the GitHub API to enumerate the release and download all six assets into a transient directory. Compare API digests, computed SHA-256 values, sidecars, release metadata, embedded package identity, and the workflow certification report.

Expected: six files, three valid sidecars, official version 0.10.14, tagged source revision, complete package lifecycle, and zero findings.

## 4. Verify Registry Archives

For each of the ten workspace crates, query crates.io version 0.10.14, require non-yanked state, download the public archive, and compare computed SHA-256 with the registry checksum.

Expected: ten matching, non-yanked public archives.

## 5. Exercise Installed Public Product

Install the verified public MSI normally. Do not supply target-store, session, bundle, application-data, or profile-root overrides. Record embedded identity, effective canonical locations, native retained artifacts, supported cleanup, and unavailable behavior. File privacy-scrubbed issues for concrete v0.10.14 defects.

Expected: a truthful bounded installed observation derived from public bytes, without claiming unavailable private-title or independent security evidence.

## 6. Reconcile And Merge Records

Update repository current markers, the v0.10.14 maintainer handoff, published release JSON, frozen review candidate, site state, and S196 verification. Run local repository checks, push the records branch, open the official PR, complete up to two hosted reviews, require green applicable CI, and merge.

Expected: main carries verified v0.10.14 public records.

## 7. Verify Public Completion

Confirm post-merge workflows and Pages deployment succeed, the public site presents v0.10.14, #567 and its milestone are complete, the project item is Done, and local main is clean and synchronized.
