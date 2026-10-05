# Verification Record: S164 v0.10.4 candidate

**Date**: 2026-10-03; public release verified 2026-10-04.

## Candidate preparation

The S164 specification gate was committed as `75a3ed9` before release mutation. The Windows patch wrapper dry run selected 0.10.4 and previewed five unreleased fragments. Its non-dry-run preflight requires clean main, so the established version-only `cargo release 0.10.4 --workspace --execute --no-confirm` command ran on the existing release branch and created commit `6d8b9c9`. It did not tag, push, or publish.

The source and isolated first-party lockfiles, embedded output, generated goldens, native conformance identity, specification Applies-To, review-handoff template, supply-chain graph digests, and bounded release notes now identify 0.10.4 where they describe candidate source. The changelog assembler consumed all six fragments, including the S164 pinned-script decision, into a chronological v0.10.4 section. Its `git rm` call printed a harmless pathspec error for the new untracked S164 fragment, which the assembler still consumed and removed; the command exited zero and the resulting staged deletions and release section were inspected. First-round review found the equivalent Unix wrapper and `release.toml` instructions still stale; all three pinned surfaces now show an annotated tag and ten product crates.

The three dependency graphs changed only with first-party version identity. Their package and edge counts remain 187/448, 186/424, and 148/317. The unsafe-review digest was rebound to the corresponding windows-all graph without changing reviewed date, expiry, exceptions, or third-party packages.

## Local source checks

The following passed on candidate source:

- Golden regeneration: `fragcap` goldens (7), CLI capture goldens (24), and CLI extcap goldens (19).
- `cargo xtask notes 0.10.4`, `cargo xtask spec`, `cargo xtask supply-chain snapshot`, and `cargo xtask supply-chain`.
- Locked offline `cargo metadata` for `fuzz/Cargo.toml` and `performance/native-proxy/Cargo.toml`.
- Focused review-handoff template test and `cargo xtask review-handoff` (twelve-area readiness; independent review not performed).
- Full `cargo xtask ci` with `CARGO_BUILD_JOBS=1` and `CARGO_PROFILE_TEST_DEBUG=0`; conformance, native performance, and Windows integration authorities passed.
- `cargo xtask msrv` at Rust 1.88, `cargo xtask neutral` for three Linux crates, and `cargo xtask docs build` (84 static pages).
- `git diff --check`, staged diff check, and strict changed-text UTF-8, no BOM, LF, final newline, replacement-character/mojibake, and long-dash scan (24 text files at the recorded check).

The first full CI attempt found the current-source independent-review template still bound to 0.10.3. The template now follows the 0.10.4 workspace version; the frozen public v0.10.3 review candidate remains unchanged. The focused test and the complete rerun passed. No installed real title, production Doctor, real host trust change, sensitive live capture, independent whole-product security review, or owner field trial was performed.

## PR and publication state

The operator merged [PR #449](https://github.com/h8rt3rmin8r/fragcap/pull/449) after all final-head hosted checks passed. First-round Codex review found the Unix and `release.toml` instruction drift; all three printed/configuration surfaces now agree, the comment was answered and its thread resolved, and full `cargo xtask ci` and `cargo xtask docs build` passed again. The one authorized second Codex round found no issue. Reviewed head `d5519a864c325d7436097f0ff1ec1256508790c1` and owner merge `f8bf4914e310a54ae83b41d8afbbe66d149168b8` share tree `6188a7482acd38e38fbd2d9ae2c1d934240c641b`.

The operator separately authorized publication. Annotated tag object `91b508be5e8c6e6ced849776c96a53c536302219` peels to the exact merged source. [Release run 37240338416](https://github.com/h8rt3rmin8r/fragcap/actions/runs/37240338416) passed identity job 111547620820, final Windows package certification job 111547737015, GitHub release creation job 111549691558, and crates.io publication job 111549842230. GitHub records the required `crates-io` deployment state `approved` under the owner's authenticated account. The agent submitted that normal deployment review under the owner's explicit publication authorization and did not bypass the gate.

The public [v0.10.4 release](https://github.com/h8rt3rmin8r/fragcap/releases/tag/v0.10.4) is non-draft and non-prerelease, published `2026-10-04T22:43:35Z`. All six public assets were independently downloaded and matched by exact name, size, and SHA-256 against the schema-version-4 certification report. Three checksum sidecars match their primary files. The report identifies official 0.10.4, exact source, native backend, six complete artifacts, two controlled smokes, and zero findings; `cargo xtask package-certification validate-report` passed. All ten public crates.io version records independently report exact 0.10.4, `yanked=false`, and nonempty checksums. The [publication handoff](../../docs/maintainers/v0.10.4-release-handoff.md) retains every public file and crate checksum. Current published-state markers move only in the separate records-only PR.

## Records boundary

The separate records-only branch advances `docs/published-release.json`, fourteen current baseline markers, frozen review-candidate identity and this evidence after complete public verification. `cargo xtask review-record`, `cargo xtask spec`, `cargo xtask docs check`, full `cargo xtask ci`, and `cargo xtask docs build` passed on the records branch. `git diff --check` and strict changed-text UTF-8 without BOM, LF and final-newline, and mojibake-marker checks passed. Historical v0.10.3 evidence, the unsigned policy, release workflow, dependencies and published bytes remain unchanged. The records PR still requires operator review and merge. Agent verification did not install or execute released fragcap, a real game, production Doctor, real trust mutation, or sensitive live capture on the operator machine. Independent whole-product security review and universal live-title compatibility are not claimed.
