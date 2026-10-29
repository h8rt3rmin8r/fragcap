# S183 candidate verification

## 2026-10-29: source preparation

The candidate starts from S182 squash merge `4ffa5f566d51aa51e59c8293160e55f4513e0cd0`. The published baseline remains v0.10.8 and no v0.10.9 tag or GitHub release exists. Spec-Kit specify, clarify, requirements and release checklists, plan, tasks and blocking analysis completed before version mutation with no critical finding. The patch dry run selected 0.10.9. `cargo release 0.10.9 --workspace --execute --no-confirm` made version commit `7a878b5` with `release.toml` tag, push and publish disabled.

Offline metadata refreshed `fuzz/Cargo.lock` and `performance/native-proxy/Cargo.lock`. Each changed only nine first-party versions from 0.10.8 to 0.10.9; root product Cargo dependency versions were already corrected on merged main. The targeted golden generators passed 7 facade, 31 CLI Capture and 20 CLI extcap tests. Source assertions, conformance identities, the unperformed independent-review record template and master Applies-To moved to 0.10.9. Frozen public review-candidate records and the fourteen current published markers remain v0.10.8.

Measured supply-chain graph views are linux-all `66cbd71da45e277654daa6367157278368f8918c8f0e26875f7722d104c3ad5c` (188 packages, 451 edges), windows-all `df1c6c69f4eccd42ba3d76faa84da436b9b978f5701f1118beea0fcf3dc5a5b8` (187, 427), and windows-release `b8596a2c4ff0c979de1813d0235f84b05ca6e665f07ae5d7b14a87ae3f1e0602` (149, 320). Only first-party package versions changed these identities. Existing unsafe-review owner, expiry and exception rules remain unchanged; `cargo xtask supply-chain` passed.

`cargo xtask changelog --release 0.10.9 2026-10-29` assembled fourteen fragments once. Its internal `git rm` attempted the untracked S183 decision fragment and printed a pathspec warning, but the fragment was physically consumed and the command exited zero. Final staging records all tracked fragment deletions. The assembled Decisions entries were reordered by date to satisfy chronological planning. `cargo xtask notes 0.10.9` passed after the standard full-changelog guidance was restored.

## 2026-10-29: controlled checks

The first full `cargo xtask ci` run reached a failing committed review-handoff template assertion because its unperformed record still named 0.10.8. Updating only that template's candidate product version to 0.10.9 made the focused regression pass. The full serialized rerun exited zero with `ci: all checks passed`, covering format, Clippy, locked workspace tests, docs, spec, supply chain, package contract, wrappers, native conformance, performance authority and Windows integration registry. `cargo xtask spec` separately confirmed Applies-To 0.10.9 and fourteen current-public v0.10.8 surfaces.

Fresh `cargo deny --locked --all-features check advisories licenses bans sources` passed all four product checks. All six isolated lock graphs separately passed current advisory checks: fuzz, native performance, native proxy spike, its audit, HTTP MITM spike, and its audit. The RustSec RUSTSEC-2026-0386 record lists Tokio 1.53.3 in its patched range and names Unix `tokio::io::unix::AsyncFd` constructors. No Windows runtime exploit was demonstrated.

`cargo xtask msrv` built the workspace at Rust 1.88. `cargo xtask neutral` built core, capture and attribution for x86_64-unknown-linux-gnu. `cargo xtask docs build` generated 99 static pages and verified hosting markers. This local docs build used installed pnpm 11.25.0; hosted docs CI remains the authority for pinned pnpm 10.26.0. `cargo xtask conformance --analyzer` initially could not find the installed TShark on PATH; after adding `C:\Program Files\Wireshark` to that process environment it passed with TShark 4.6.8, all 21 required rows, six protocols and ten artifact authorities.

The pinned Npcap SDK 1.16 download was verified at SHA-256 `f0a8be7778ee3ae1b99bbbecb27a3ff0f6c111a4093f1c78c5c5a099607184db`. Physical Windows trust and exact candidate package tests are pending below; no real-title calibration or Deep Capture success is claimed.

The final preparation pass ran `cargo xtask lint` with a clean result and `git diff --cached --check` with no whitespace error. A strict UTF-8 without BOM, LF-only, final-newline and mojibake pass accepted 24 changed text files; binary goldens were excluded from the text decoder.

## Candidate PR and physical evidence

Pending. Merge, tag, protected registry publication, public-file reconciliation and issue #519 closure remain outside this pre-merge candidate handoff.
