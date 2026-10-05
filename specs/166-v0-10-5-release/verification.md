# Verification Record: S166 v0.10.5 candidate

**Date**: 2026-10-05

## Candidate source

S166 began from clean merged main `434daa603c3fad06107ec1cf26d3421adf04ae01` after S165. The analyzed specification gate was committed as `02e8848c8753e631b4d959a3e5a64a8ecbb6c8f8`. The version-only cargo-release dry run and execution selected 0.10.5 for ten product crates and xtask. Neither command tagged, pushed, or published.

All six unreleased fragments were assembled into the v0.10.5 changelog section, with dated decisions ordered chronologically. First-party and isolated lockfile identities, embedded output assertions and goldens, native conformance identity, specification Applies-To, the not-started review template, release notes, and supply-chain graph digests now identify the 0.10.5 candidate where appropriate. Reviewed public-state surfaces, including the frozen v0.10.4 review candidate, remain at v0.10.4.

The three dependency graph counts remain 187/448, 186/424, and 148/317. Lockfile diffs change first-party versions only. The unsafe-review digest was rebound to the same reviewed inventory; no third-party version, source, feature, or package count changed.

## Local gates

- Golden regeneration passed: 7 facade, 24 CLI capture, and 19 CLI extcap tests.
- `cargo xtask notes 0.10.5`, `cargo xtask spec`, `cargo xtask supply-chain snapshot`, `cargo xtask supply-chain`, and `cargo xtask review-handoff` passed. Independent security review was not performed.
- Offline metadata refreshed the fuzz and native-performance lockfiles.
- `cargo xtask docs check` passed its thirteen-topic contract and CLI example tests.
- The complete `cargo xtask ci` rerun passed after eight trailing-space violations in new Markdown were corrected. It included formatting, clippy, tests, lint, dependencies, licensing, supply chain, package certification, wrappers, skills, documentation, specification, calibration acceptance, native threat and failure matrices, conformance, performance, and Windows integration.
- `cargo xtask msrv` built at Rust 1.88. `cargo xtask neutral` built all three required Linux crates. `cargo xtask docs build` exported 86 static pages.
- `git diff --check` and strict UTF-8 without BOM, LF, final newline, and mojibake checks passed on 35 changed text files at the recorded check.

No installed real game, production Doctor, real trust mutation, sensitive live capture, independent whole-product review, or owner field trial was performed. Controlled S165 tests do not prove universal title compatibility.

## Publication boundary

Candidate [PR #454](https://github.com/h8rt3rmin8r/fragcap/pull/454) opened from head `0312baf6d47f6159589cb334e765c05c71bca630`. Its automatic Codex review completed with no findings. Some first-attempt hosted jobs failed before acquiring a runner; GitHub annotated them "The job was not acquired by Runner of type hosted even after multiple attempts" during its [October 5 Actions incident](https://www.githubstatus.com/incidents/3q1yb5m7ltvb). That is an external scheduling failure, not a passing check or a demonstrated product failure. Hosted final-head checks remain required.

Human approval, merge, and tag push are required by `CONTRIBUTING.md`. The public v0.10.5 release, protected deployment, six public files, certification, ten registry versions, and separate records-only PR are not yet complete.
