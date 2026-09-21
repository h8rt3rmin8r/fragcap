# Research: S159 v0.10.2 release

## R-1: Patch version

**Decision**: Publish the current delta as v0.10.2.

**Rationale**: S154 through S158 improve deterministic loss testing, independent-review intake, package-certification authority, native documentation and Windows scheduling reliability. They do not introduce a breaking public contract or broaden authority. A patch communicates the actual compatibility boundary.

**Alternatives considered**: v0.11.0 overstates feature expansion; retaining v0.10.1 leaves merged reliability corrections unavailable in official bytes.

## R-2: Existing release machinery remains authoritative

**Decision**: Reuse the version-only cargo-release configuration, release generators, human-reviewed release branch, tag-triggered release workflow, package certification and dependency-ordered publisher.

**Rationale**: These surfaces already produced verified v0.10.1. The release request does not reveal a defect requiring workflow modification. Reusing them minimizes the release delta and preserves exact policy checks.

**Alternatives considered**: Direct main changes violate repository integration policy; manual asset construction bypasses certification; changing release workflows during the cut adds unrelated risk.

## R-3: Two pull requests are required

**Decision**: Merge candidate preparation before tagging, then reconcile actual publication through a separate records-only pull request.

**Rationale**: Exact tag commit, release run, asset hashes, registry checksums and publication time do not exist during candidate preparation. Writing them early would fabricate evidence. The established v0.10.1 process separates these states successfully.

**Alternatives considered**: One pre-publication pull request cannot truthfully contain future evidence; direct post-publication pushes to main violate human review.

## R-4: Owner interaction is bounded

**Decision**: Ask the operator only to merge the prepared release PR and, if GitHub pauses the workflow, approve the protected crates.io deployment. A later records PR also requires human merge.

**Rationale**: Repository and environment policy reserve these actions for the owner. All preparation, review response, CI monitoring, tag push and public verification can proceed without additional choices.

**Alternatives considered**: Agent merge and deployment approval exceed authority; administrator bypass would misstate the normal protected path and is unnecessary.

## R-5: No local installed-product execution

**Decision**: Use source tests locally and hosted disposable Windows certification for packaged behavior.

**Rationale**: The operator explicitly prohibits running this sensitive installed software locally. Existing hosted gates already provide controlled loopback, containment and cleanup evidence without using a real game.

**Alternatives considered**: Local install or production Doctor is outside authority; skipping package evidence would weaken the release gate.
