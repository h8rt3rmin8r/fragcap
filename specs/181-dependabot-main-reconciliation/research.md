# Research: S181

## Locked preparation

**Decision**: Fetch locked dependencies before identical PR/main root contracts; preserve offline compiled-harness discovery.

**Rationale**: The #508 merge failed fetching Tokio 1.53.3 under offline mode while the PR skipped the root step. `docs_coverage` uses `review_handoff::discover_references` to inspect actual compiled tests.

**Alternatives considered**: Removing offline mode weakens discovery; warm caches repeat the defect; duplicate full cold compilation on every PR adds unnecessary work. A manual isolated-cache dispatch proves the real path.

**Source**: [Cargo fetch](https://doc.rust-lang.org/cargo/commands/cargo-fetch.html), `xtask/src/review_handoff.rs`, and [failed main docs job](https://github.com/h8rt3rmin8r/fragcap/actions/runs/40218342047/job/121244259185).

## Complete site audit

**Decision**: Frozen install and `pnpm audit --audit-level low` with pnpm 10.26.0, without dependency-class filters, automatic fix, advisory ignores, or registry-error suppression.

**Rationale**: Rust audits do not inspect the pnpm graph. The default pnpm audit covers production, development, and optional dependencies. The DOMPurify override itself blocks its update.

**Alternatives considered**: Production-only audits omit build tools; audit-fix mutates policy implicitly; new bot configuration is unnecessary to resolve the existing block.

**Source**: [pnpm 10 audit](https://pnpm.io/10.x/cli/audit), `site/pnpm-workspace.yaml`, and [blocked bot job](https://github.com/h8rt3rmin8r/fragcap/actions/runs/40218355283/job/121244302388).

## Isolated Rust graphs

**Decision**: Audit all seven committed locks, retain full root policy checks, and add fresh per-manifest advisory checks for six isolated workspaces.

**Rationale**: Six isolated locks retain rustls-webpki 0.103.15; four also retain Tokio 1.53.1. Repository security records name patched 0.103.16 and 1.53.3. Root workspace audits omit isolated graphs.

**Alternatives considered**: Zero current Rust bot alerts omit these stale patches; deleting research projects alters unrelated history; broad upgrades obscure the security patch.

**Source**: [cargo-deny common options](https://embarkstudios.github.io/cargo-deny/cli/common.html), `CHANGELOG.md`, `changelog.d/pr509-ci.decisions.md`, and committed locks.

## Completion

Two read-only research agents inspected audit coverage and cold-cache preparation under the installed planning workflow. No design unknowns remain. Fresh audits and package-manager output remain implementation evidence to obtain.

## Additional compression findings

Fresh audits found fourteen Zstd advisories in both native spike graphs. The native spike accepted compression-codecs 0.4.40 through existing async-compression 0.4.43; its audit project required a compatible update from async-compression 0.4.12 to 0.4.43 before selecting codecs 0.4.40. Both now resolve zstd 0.14.3, zstd-safe 8.2.0, and zstd-sys 2.1.2. Existing decoder features remain enabled. Fresh audits of all seven graphs report no advisories.

Both native-spike manifests claimed Rust 1.82 although their unchanged time dependencies already require 1.88. Correct those two declarations to 1.88 and qualify them at that compiler. Compression-codecs 0.4.40 declares Rust 1.83, so this patch does not increase the existing graph floor. Preserve historical measurements and the root product floor.
