# Research: S172

**Date**: 2026-10-09

## Release Inventory

**Decision**: Reuse S164/S166's inventory and version-only cargo-release/xtask commands, on the established release branch.

**Rationale**: Prior candidate diffs and `scripts/New-Release.ps1` identify first-party lockfiles, sink and Windows integration assertions, conformance, specification applicability, not-started review template, goldens and supply-chain digests. The full helper requires clean main and cannot own this already specified branch.

**Alternatives considered**: Full wrapper after specification fails its preflight. Broad replacement corrupts historical/public identities. Explicit inventory preserves both authorities.

**Sources**: `release.toml`, `scripts/New-Release.ps1`, `scripts/cut-release.sh`, `specs/166-v0-10-5-release/verification.md`, `xtask/src/spec.rs`, `xtask/src/review_record.rs`, `xtask/src/review_handoff.rs`.

## Dependency and Stale Failure

**Decision**: Carry only `site/package.json` and `site/pnpm-lock.yaml` from #470 head `ed5a094ee93a48670de5ae1844c8f5e0288cdabc` and run current-source acceptance.

**Rationale**: The old HTTPS fixture formats request fragments after handshake, permitting the 25 ms no-ALPN classifier to select generic relay. S169 already disables Nagle, queues the complete request before handshake and asserts zero generic fallback. No new correction is warranted without reproduction on current source.

**Alternatives considered**: Revalidating old source excludes merged fixes; changing product classification from the stale failure is unsupported.

**Sources**: Live #470 diff/checks and failed Ubuntu log, exact head fixture, current `crates/fragcap-proxy/tests/https_proxy.rs`, `crates/fragcap-proxy/src/runtime.rs`, S169 commit `edc8afd`.

## Publication Truth

**Decision**: Preserve verified public v0.10.5 records and finish at the local committed candidate boundary.

**Rationale**: This kickoff supplies local autopilot authorization. Prior push authorizations named earlier slices. Local certification contract validation does not certify final produced packages.

**Alternatives considered**: Advancing publication records from candidate evidence would misstate delivery. Tagging/publishing without new explicit authorization exceeds the requested local protocol.

**Sources**: `CONTRIBUTING.md`, `AGENTS.md`, autopilot protocol, published release and frozen review candidate records.
