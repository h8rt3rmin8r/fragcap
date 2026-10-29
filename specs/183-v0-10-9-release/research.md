# Research: S183 release candidate

## Existing release mechanism

**Decision**: Follow `scripts/New-Release.ps1 patch -DryRun` and its constituent steps on the already selected `release/0.10.9` branch.

**Rationale**: The script requires `main` and creates the release branch. The managed worktree already has the release branch and Spec-Kit artifacts. Its remaining steps are `cargo release` without tag/push/publish, embedded-version updates, targeted golden regeneration, changelog assembly and CI.

**Alternative**: Move work into the primary checkout. Rejected because that would mix S183 with the clean primary main and invalidate the attached worktree identity.

## Published-state boundary

**Decision**: Leave v0.10.8 current markers until post-merge asset and registry reconciliation.

**Rationale**: Constitution P-11 and S180's release record distinguish a source candidate from public bytes. The owner requested handoff before merge.

## Advisory and review boundary

**Decision**: Audit every locked product and isolated graph on the candidate, then rely on exact-head CI and actual review comments. Count automatic bot activity as round one and request no more than one explicit second round.

**Rationale**: Issue #519 concerns the released graph while main is already corrected. A passing main audit alone does not certify the candidate or published files.
