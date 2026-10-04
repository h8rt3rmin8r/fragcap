# Verification Record: S164 v0.10.4 candidate

**Date**: 2026-10-03. This records candidate-source evidence only; owner merge and public release remain pending.

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

Official [PR #449](https://github.com/h8rt3rmin8r/fragcap/pull/449) is open. First-round Codex review found the Unix and `release.toml` instruction drift; all three printed/configuration surfaces now agree, and full `cargo xtask ci` and `cargo xtask docs build` passed again. Review-thread disposition, second-round review, and final-head hosted checks remain pending at this record. Public v0.10.3 remains the independently verified baseline. S164's post-merge tag, four release jobs, six public files, certification, ten registry packages, and records-only PR have not occurred.
