# Research: v0.10.14 Publication

## Decision 1: One Work Slice, Distinct Evidence Phases

**Decision**: S196 owns tag, protected publication, public readback, installed observation, records PR, review, merge, Pages, and issue closeout.

**Rationale**: The owner explicitly rejected a second publication slice. Keeping phases distinct inside one slice preserves truth without adding another stop.

**Alternatives considered**: A tag-only slice followed by a records slice was rejected as unnecessary delivery overhead. Updating records before publication was rejected because it would state an unverified public release.

## Decision 2: Tag The Exact Accepted Merge

**Decision**: Annotated tag `v0.10.14` must peel to `5efb38ee93daff140e1ba79f57eea0a93d5b572f`.

**Rationale**: That squash merge carries the reviewed candidate tree and all merge-triggered workflows passed. Later Spec-Kit and record commits are evidence about publication, not release source.

**Alternatives considered**: Tagging the records branch would include post-publication claims in the released source and break the candidate authority. A lightweight tag would discard the repository's annotated release convention.

## Decision 3: Preserve Normal Registry Protection

**Decision**: Approve the configured crates.io environment deployment through GitHub's authenticated pending-deployment API when requested.

**Rationale**: The owner authorized end-to-end publication. The normal approval is an intended release step and retains the configured protection receipt.

**Alternatives considered**: Disabling reviewers, changing deployment policies, or using administrator bypass was rejected because each would weaken the existing protection.

## Decision 4: Public Readback Is Independent

**Decision**: Download release assets and crate archives from their public endpoints after publication, compute hashes independently, validate sidecars and certification content, and retain only bounded receipts in repository records.

**Rationale**: Candidate, tagged workflow, and public bytes are distinct authorities. Public success cannot be established by reusing runner artifacts.

**Alternatives considered**: Trusting workflow success alone and copying expected candidate hashes were rejected as insufficient public evidence.

## Decision 5: Installed Observation Uses Public Defaults

**Decision**: Acquire the verified public MSI, install it normally, observe actual effective default target/session locations, exercise bounded installed commands, and use supported cleanup without storage overrides.

**Rationale**: Constitution 1.8.0 requires default-path evidence. The owner also directed testing after release and concrete defects as GitHub issues.

**Alternatives considered**: A locally built binary, unpacked ZIP, temporary target database, custom bundle path, or redirected application data cannot establish the installed default journey.

## Decision 6: Records Merge Is Authorized

**Decision**: After public verification, create the official records PR, process no more than two hosted Codex rounds, require terminal applicable CI, merge it, and verify Pages without another owner halt.

**Rationale**: The owner's current instruction expressly says not to halt again and to report only when everything is live and public. This supersedes the ordinary owner-reserved merge ritual for S196.

**Alternatives considered**: Stopping at the PR would directly contradict the requested outcome.
