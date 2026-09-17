# S155 Research

## R-1: Preserve readiness and acceptance as separate contracts

**Decision**: Keep `cargo xtask review-handoff` and `docs/security/native-product-review-record.v1.json` unchanged in meaning. Add `cargo xtask review-record <path>` for a separately supplied completed reviewer-owned record.

**Rationale**: The current record is intentionally `not-started` and proves only that the repository is ready to hand to a reviewer. Mutating it into synthetic success would erase the distinction required by #333 and P-9.

**Rejected alternatives**: Reusing `review-handoff` for completed records conflates readiness with acceptance. Committing an implementation-authored completed record fabricates independence.

## R-2: Validate mechanics without certifying human independence

**Decision**: The completed-record validator enforces closed schema, exact candidate identity, complete area coverage, finding disposition and bounded public-safe evidence. A mechanically blocker-free result states that structural blockers were not found but explicitly leaves reviewer authenticity, independence and final #333 acceptance to maintainers.

**Rationale**: JSON can carry an independence declaration but cannot prove the relationship or authorship behind it. Automation can reject impossible or incomplete records without becoming the reviewer.

**Rejected alternatives**: Returning an `approved` state from automation exceeds mechanically knowable facts. Treating pull-request bots as the whole-product reviewer does not satisfy the installed-build scope.

## R-3: Replay immutable public bytes through the existing package authority

**Decision**: Add a candidate registry containing exact v0.10.1 release identities and a dedicated hosted workflow that downloads all six public files, verifies them, and invokes `scripts/Test-PackageCertification.ps1` with the pinned predecessor package.

**Rationale**: This tests what users download. Reusing package certification preserves one installer lifecycle and cleanup authority.

**Rejected alternatives**: Rebuilding v0.10.1 from source would produce different bytes. Adding another installer script would duplicate security-sensitive lifecycle logic.

## R-4: Distinguish portable and installed controlled smoke

**Decision**: Refactor the script's controlled smoke into one helper and invoke it once for the ZIP executable and once immediately after clean MSI installation. Report the two outcomes as unique surfaces in report schema version 3.

**Rationale**: A portable smoke cannot prove installed layout or installed parent identity. Separate rows prevent one success from satisfying both requirements.

**Rejected alternatives**: A Boolean `installed` flag on the existing row is ambiguous and cannot prove both paths ran. A second ad hoc script would drift from the existing containment checks.

## R-5: Keep the QUIC independent retest open

**Decision**: Document that the installed controlled target exercises the shipped HTTP and HTTPS path but does not expose the S128 QUIC performance harness. Keep #413 open pending independent installed QUIC reproduction.

**Rationale**: The published v0.10.1 CLI has no packaged command that executes the source-only performance harness. Adding one now would create later product bytes and could not prove the immutable candidate.

**Rejected alternatives**: Calling source-built performance tooling an installed-product retest would misstate the evidence. Expanding production solely for this campaign is outside S155.

## R-6: Use hosted Windows only for product execution

**Decision**: Run the public-candidate workflow on disposable GitHub-hosted Windows and perform only static validation locally.

**Rationale**: This follows the operator's explicit safety boundary and provides a clean environment with automatic teardown.

**Rejected alternatives**: Running the installed product on the owner's workstation is prohibited. A self-hosted runner would not establish disposable-host containment.
