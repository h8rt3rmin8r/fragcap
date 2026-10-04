# Research: S164 v0.10.4 release

## R1: Existing candidate machinery

**Decision**: Reuse the Windows patch wrapper's dry-run preview and documented version-only cargo-release command, then regenerate goldens, conformance identity, changelog, and repository records directly on `release/0.10.4`.

**Rationale**: `scripts/New-Release.ps1` and `release.toml` separate branch preparation from tagging and publication. Its non-dry-run path requires clean main and creates the release branch, so it cannot run after a committed Spec-Kit gate on that branch. S162 used the direct documented sequence successfully.

**Alternatives considered**: Hand-edit every Cargo version and generated file risks drift. Altering pinned release machinery for an ordinary patch expands the slice without need.

## R2: Patch content

**Decision**: Include merged S163 and every unassembled fragment since public v0.10.3. Keep release notes limited to observed setup, root binding, and failure-reporting changes.

**Rationale**: Main contains S163 and five outstanding S162/S163 fragments. Two S162 publication-state fragments are records about verified v0.10.3 and must be assembled chronologically as well. Public v0.10.3 remains the baseline.

**Alternatives considered**: Selecting only S163 fragments would leave already merged unreleased records unassembled.

## R3: Authority and timing

**Decision**: Push and open the PR under current authorization; wait for two review rounds at most and green final-head CI; stop for operator merge. Do not tag or claim publication in this turn.

**Rationale**: The user explicitly authorized push and PR, and explicitly reserves the merge ritual. Repo governance separates tag and publication from candidate preparation.

**Alternatives considered**: Waiting for push authorization would contradict the current request. Tagging before owner merge would select the wrong source.

## R4: Evidence boundary

**Decision**: Preserve current published markers at v0.10.3 and use a later records-only PR after exact public-file, certification, and ten-crate verification.

**Rationale**: Candidate source and public release are distinct facts under P-9 and P-11. A green PR does not prove user-downloadable bytes exist.
