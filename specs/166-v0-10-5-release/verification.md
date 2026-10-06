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

## Candidate PR and exact tag

Candidate [PR #454](https://github.com/h8rt3rmin8r/fragcap/pull/454) opened from head `0312baf6d47f6159589cb334e765c05c71bca630`. Its automatic Codex review completed with no findings. Some first-attempt hosted jobs failed before acquiring a runner; GitHub annotated them "The job was not acquired by Runner of type hosted even after multiple attempts" during its [October 5 Actions incident](https://www.githubstatus.com/incidents/3q1yb5m7ltvb). That is an external scheduling failure, not a passing check or a demonstrated product failure. The final PR head `82b4cb9959ac74cc9df9adb8c457ca973685160c` passed 23 hosted checks with two intended skips. Owner `h8rt3rmin8r` merged it as `4e2869a972abfee11e5c0eae3101c1b1e16f6404`; both heads have tree `cb618ac7b904fb29869344e84ec03f651beac44e`. GitHub records no formal human review object, so a separate approval review cannot be claimed.

After the owner's explicit instruction to push the tag, the agent created and pushed annotated tag object `42f5997986b308dc0cae8e03f6782fb7cdebf615`. It peels to the exact owner-merged source. The usual `CONTRIBUTING.md` tag handoff was superseded for this tag push by that direct owner instruction; no main commit or other release gate was bypassed.

## Public release verification

[Release run 37394913045](https://github.com/h8rt3rmin8r/fragcap/actions/runs/37394913045) completed successfully. Its identity job 112048459638, final Windows package certification job 112048634205, GitHub release job 112051621587, and crates.io job 112051879547 all passed. GitHub records a normal `crates-io` environment approval under authenticated account `h8rt3rmin8r` after the agent submitted the owner's authorized pending-deployment review; deployment 6872864059 finished successfully.

The public [v0.10.5 release](https://github.com/h8rt3rmin8r/fragcap/releases/tag/v0.10.5) is non-draft and non-prerelease, published `2026-10-06T00:49:08Z`. All six public assets were independently downloaded and matched by exact filename, byte count, and SHA-256 against the downloaded schema-version-4 certification report. The three checksum sidecars match their primary files. `cargo xtask package-certification validate-report` passed on the public files and report; it identifies official 0.10.5, exact source, native backend, six complete artifacts, two controlled smokes, and zero findings. All ten crates.io 0.10.5 version records were independently read from the public registry and reported `yanked=false` with nonempty checksums. The [publication handoff](../../docs/maintainers/v0.10.5-release-handoff.md) retains each exact file and crate checksum.

## Records boundary

Current published-state markers move on a separate publication-records branch after this verification. Its first complete local gate found four elapsed 30-day critical reviews; all tests passed, then the supply-chain authority rejected the expired dates. The exact `cargo-deny 0.20.2` all-feature advisory, license, ban and source audit passed against current data, and `cargo xtask supply-chain snapshot` matched all three locked graph digests and counts. Only the `last_reviewed` dates for `ring 0.17.14`, `rustls-native-certs 0.8.4`, `tokio-rustls 0.26.4` and `cargo-deny 0.20.2` advance to October 6; package versions, features, exception dates, unsafe-review expiry, release bytes and product behavior stay fixed. The final complete `cargo xtask ci` run exited zero and reported `ci: all checks passed`. `cargo xtask docs build` exported the static site with exit code zero. Strict UTF-8 without BOM, LF, single final newline and mojibake checks passed for all 31 changed text files; `git diff --check` passed. The records PR still requires owner review and merge. No released fragcap installer, real game, production Doctor, real trust mutation, or sensitive live capture ran on the operator machine. Independent whole-product security review and universal real-title compatibility remain unclaimed.
