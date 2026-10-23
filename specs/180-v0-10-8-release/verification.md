# S180 Verification

## 2026-10-23: candidate preparation

The release branch starts from accepted S179 main `fe0bc6096ff103389f8fe5400a8968c31a495ebc`. The published baseline remains v0.10.7. Spec-Kit specify, clarify, requirements and release checklists, plan, tasks and blocking analysis completed before version mutation; specification commit is `43e0f79`. The patch dry run selected 0.10.8 and previewed all ten existing fragments.

`cargo release 0.10.8 --workspace --execute --no-confirm` made only the version commit `542ed8e` and did not tag, push or publish. Offline metadata refreshed the fuzz and native-performance locks. Each isolated lock changed nine first-party versions from 0.10.7 to 0.10.8; no third-party package or feature changed. Explicit sink, Windows integration, native conformance, review template and master Applies-To identities moved to 0.10.8. Existing golden regeneration passed 7 facade, 31 CLI Capture and 20 CLI extcap tests. The generated fixture and CLI files retain their established formats.

The three measured supply-chain views retain package and edge counts: linux-all `a0668d8106017e22e6041155ae069b6b650cf851eb59b142f55606b82d94dabd` (188/451), windows-all `d1f9aedf9d49736189e7ea63943de19152c0d89028fc7719f27f652f100e5f23` (187/427), and windows-release `85cfc1eef5b3c99948af9874ae0f1b2aa03ee8b43b144d96b4e1250c398fd5df` (149/320). The policy and unsafe-review reference rebind only these measured first-party identities; dates and exceptions remain unchanged. `cargo xtask supply-chain` passed.

`cargo xtask changelog --release 0.10.8 2026-10-23` assembled eleven fragments, including the dated S180 release decision, exactly once and reset Unreleased. Its internal `git rm` staging attempts were denied by the restricted command sandbox, but it physically removed the fragments and wrote the correct changelog. The later authorized Git staging must record those deletions. `cargo xtask notes 0.10.8` passed the short-highlight gate.

The first restricted `cargo xtask ci` attempt failed 20 Windows tests on local socket and temporary-file permission denials. A serialized rerun with normal headless filesystem and network access completed with exit 0 and `ci: all checks passed`, including format, Clippy, workspace tests, docs, spec, supply chain, package certification, conformance, performance and Windows integration authorities. `cargo xtask msrv` passed the Rust 1.88 workspace build; `cargo xtask neutral` passed core, capture and attribution Linux builds. Fresh `cargo deny check advisories licenses bans sources` exited 0 with all four checks okay and only governed duplicate warnings.

`cargo xtask docs build` generated 95 static pages with `.nojekyll` and `CNAME`. Site unit tests passed 4/4 after rerunning with normal pnpm cache access; the restricted attempt could not open pnpm's SQLite store. Production Playwright accessibility passed all 14 tests in 4.2 minutes. These are local candidate checks, not proof of public deployment.

After adding this receipt and the maintainer handoff, `cargo xtask lint` and `cargo xtask docs check` passed, including both CLI reference configurations. A strict pass over 25 changed text files found valid UTF-8 without BOM, LF-only endings, one final newline and no mojibake markers; `git diff --check` passed. The site checks used the existing installed dependency tree; hosted docs CI remains the authoritative pinned pnpm installation check.

No candidate PR merge, tag, public release, registry publication, new current marker, independent whole-product review or real-game compatibility is claimed by this preparation record. Exact-head hosted and public-byte evidence must be recorded after those operations actually complete.
