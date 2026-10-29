# S183 candidate validation

1. Confirm branch `release/0.10.9`, `.specify/feature.json` pointing to `specs/183-v0-10-9-release`, and current public v0.10.8 markers.
2. Preview `scripts/New-Release.ps1 patch -DryRun`. Prepare its version, golden and changelog operations on the existing worktree branch without tag or push.
3. Run `cargo xtask ci`, `cargo xtask msrv`, `cargo xtask neutral`, `cargo xtask docs build`, release notes, graph and current-advisory checks. Read every terminal result.
4. Run controlled Windows package certification and attempt the physical current-user CA trust row. Record exact binary identity and verify cleanup after any refusal or failure.
5. Push the candidate branch and open its official PR. Reconcile exact-head checks, actual review threads and at most one requested second Codex round.
6. Hand the reviewed SHA to the owner for final review and merge. After that separate approval, the annotated tag, protected registry approval, actual public-file reconciliation and current-record PR remain.
