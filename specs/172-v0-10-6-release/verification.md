# Verification Record: S172 v0.10.6 Candidate

**Date**: 2026-10-09

**Status**: Required local candidate gates passed; pre-push handoff. The candidate is not published.

## Source and Preparation

Base is merged S171 `2019ddac844dae891903e1c5c4b5a90a93593947`. Specification gate `90d110a` passed blocking analysis before candidate mutation. The version-only dry run and execution completed with explicit no-publish/no-tag/no-push flags on `release/0.10.6`; local version commit is `6237997`. Ten product crates and xtask identify 0.10.6. No remote or publication action occurred.

Only the two site dependency files from #470 head `ed5a094ee93a48670de5ae1844c8f5e0288cdabc` were integrated. The old failed HTTPS fixture was compared with the corrected S169 fixture on current source; no additional product change was warranted from stale evidence.

Offline metadata refreshed fuzz and native-performance lockfiles; their diffs change only first-party versions. Root lockfile changes are likewise first-party version identity only. Generated golden suites passed: seven facade, 24 CLI capture and 19 CLI extcap tests. Sink/Windows assertions, portable conformance, not-started review template and Applies-To agree on 0.10.6. Public and historical records retain their original version.

## Supply-Chain Snapshot

| Graph | Digest | Packages | Edges |
| --- | --- | --- | --- |
| linux-all | 373178b62109ee9406414414edcee8d8c12abde48b233020e4123f828341f5eb | 188 | 451 |
| windows-all | 4aa8c86ad987ded4e8ae38f7bffec7be4cd7b81a61f2723ddeef551c8f024c96 | 187 | 427 |
| windows-release | 93453b6a78d3d4b101c26af8f72634747cf18993bd4e51c11e75b93126cb66c4 | 149 | 320 |

Counts are unchanged. Policy graph and unsafe-review digests are rebound to candidate first-party versions; third-party Rust identities/features, review dates and expiry remain unchanged.

## Local Gate Results

- Current-source HTTPS integration passed all seven cases, including the corrected no-ALPN fixture and zero generic fallback assertion.
- `cargo xtask notes 0.10.6`, `spec`, `review-handoff`, `review-record`, `supply-chain` and `conformance` passed. The initial notes check rejected a nonstandard closing line; the line was corrected and the gate passed without changing its requirements.
- Release assembly consumed all 14 fragments exactly once, reset Unreleased and retained chronological decisions. Two newly authored untracked fragments generated harmless git-rm pathspec diagnostics during assembly; their bodies are present in the release record and no fragment remains except README.md.
- `cargo xtask ci` exited zero and reported `ci: all checks passed`. It covered format, clippy, complete workspace tests, repository lint, dependency direction, license, supply chain/release guard, package-certification contract, wrappers, skills, documentation and CLI examples, specification, guided calibration, threat/failure evidence, review readiness/immutable published registry, fuzz inventory, conformance, performance and Windows integration authority. Full CLI unit coverage included 377 passing tests. Designated live/manual tests remain outside ordinary acceptance.
- `cargo xtask msrv` built the workspace with Rust 1.88; `cargo xtask neutral` built core, capture and attribution for x86_64-unknown-linux-gnu.
- Both isolated harnesses passed `cargo metadata --locked --offline`; each resolves nine product crates at 0.10.6 while its own harness version stays independent. The supplemental inventory checker was corrected to handle scalar output and exclude harness packages before its final successful validation.
- Frozen site installation and four unit tests passed with project-selected pnpm 10.26.0 and Node 24.19.0. The documentation wrapper's first build selected host pnpm 11.25.0; the explicit site-directory pnpm run-build path was also validated under project-selected 10.26.0. Both exports passed, producing 89 pages and the required .nojekyll/CNAME markers. No toolchain pin or wrapper was changed.
- All 14 production accessibility tests passed in 3.1 minutes, covering every public route at 320/768/1440 pixels, changelog hierarchy, contrast, diagrams, search, release/source claims, internal links and not-found recovery.

## Final Receipt Checks

Receipt-only status and task updates follow the full gate. All 38 changed text files passed strict UTF-8 without BOM, LF, one final newline, dash, trailing-whitespace and mojibake checks. `git diff --check` passed. Final `cargo xtask lint`, `docs check` (both nine-case CLI feature variants), `spec` and `notes 0.10.6` passed after receipt updates. No product, dependency, golden or conformance behavior changed after the successful full gate.

## Later Evidence Boundary

The owner subsequently authorized push and the remaining publication workflow. Official [PR #474](https://github.com/h8rt3rmin8r/fragcap/pull/474) opened at `83172255ff71fdc59aab25cef54815057d9a5120`. Dependency PR #470 was closed as superseded after its exact patch was verified. The automatic first Codex round completed with no findings and a thumbs-up; there were no formal reviews or inline threads on that head.

Two hosted audit attempts failed before running cargo-deny because Docker Hub returned HTTP 429 for the action's pinned Rust image. The correction installs the same governed cargo-deny 0.20.2 from locked source under Rust 1.88.0 and runs every existing all-feature advisory, license, ban and source check. The supply-chain wiring gate requires those exact native commands. A fifteenth dated decision fragment was assembled into the existing candidate release record; the assembled diff adds only that decision and leaves Unreleased empty.

The follow-up full `cargo xtask ci` passed after this correction, including all 222 xtask tests and every repository acceptance gate. Five changed text files passed strict encoding and whitespace checks, and `git diff --check` passed. The final code/security review round and hosted audit must validate the pushed correction.

The second round on `29bbc92` identified one P2 finding: the newly introduced Rust action ref was mutable. Pin the reviewed upstream composite action at `e2a55d2ffb04f378e9626c28d38b36d230d1e12f`, use its explicit `toolchain: 1.88.0` input, and require both in the wiring gate. The native audit passed on the preceding head; final-head hosted evidence remains required after this narrow correction. No third external review round is requested.

After the pin correction, all 222 xtask tests, formatting, supply-chain, lint, specification and release-notes gates passed. A temporary mutable-action and stable-version drift produced both expected wiring findings and exit 1; restoring the exact action and Rust version passed. The second correction's dated fragment was assembled into v0.10.6 without duplicating earlier decisions.

Final-head hosted acceptance, owner merge, v0.10.6 tag, public assets and registry publication remain subsequent states. The current published release remains v0.10.5. No real game, live capture, operator-session collection, real trust mutation, universal title-compatibility or independent whole-product review is claimed.
