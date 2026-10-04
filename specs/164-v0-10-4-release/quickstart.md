# Quickstart: S164 candidate validation

Use source and controlled fixtures only. Do not install or run a real title, production Doctor, real trust mutation, or sensitive live capture on the local machine.

1. Preview `scripts/New-Release.ps1 patch -DryRun` on clean main before mutation. Its non-dry-run path cannot run after this specification gate on the release branch.
2. Run the documented version-only cargo-release command on `release/0.10.4`, then the wrapper's embedded-version, golden, conformance, and changelog steps directly. Inspect every generated diff and all three lockfiles.
3. Run `cargo xtask notes 0.10.4`, `cargo xtask spec`, `cargo xtask supply-chain`, `cargo xtask ci`, `cargo xtask msrv`, `cargo xtask neutral`, and `cargo xtask docs build`, with headless noninteractive child execution. Run locked offline metadata for isolated harnesses.
4. Push the reviewed candidate and open the PR. Require no unresolved review threads and green required checks on the final head. At most one follow-up `@Codex review` request is allowed.
5. Handoff the PR for operator merge. A later publication pass verifies exact merged source, tag, release jobs, public files, certification, and registry before any records-only PR advances current published-state markers.
