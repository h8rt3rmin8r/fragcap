# Research: S162 v0.10.3 patch release

## R1: Release machinery

**Decision**: Reuse the established `scripts/New-Release.ps1 patch` preview and execution on `release/0.10.3`, then the tag-triggered GitHub workflow.

**Evidence**: `release.toml` limits cargo-release to version and commit work on `release/*`; the wrapper assembles changelog fragments, regenerates versioned goldens, and runs checks without pushing, tagging, or publishing. S159 used the same two-stage candidate and tag process.

**Alternatives**: Manual version edits multiply identity drift. Changing the wrapper or workflow during an ordinary patch would widen scope without a demonstrated need.

## R2: Version and content

**Decision**: Patch version 0.10.3 includes merged S160 and S161 plus all other unreleased fragments since v0.10.2.

**Evidence**: v0.10.2 remains the latest public release. Main contains the S160 QUIC queue correction and S161 CLI input correction, and `changelog.d/` contains their unassembled fragments. These are nonbreaking reliability changes.

## R3: Publication authority

**Decision**: Preserve operator merge and protected crates.io approval, while using the operator's current instruction as authorization to push the reviewed branch and later tag.

**Evidence**: `AGENTS.md` forbids direct main pushes and agent PR merges; `release.toml` separates candidate preparation from tag publication. The user explicitly requested the next release and offered to approve crates when needed.

## R4: Honest evidence

**Decision**: Require exact merged-source, four-job, public-file, certification, and registry checks before changing any current published-state marker.

**Evidence**: S159's release contract and handoff distinguish candidate, public release, and records. The Print Screen observation is not a demonstrated fragcap release correction and will be excluded from highlights.
