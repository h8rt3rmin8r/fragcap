# Research: v0.11.0 Direct Route Release

## Source and version

**Observation**: Public v0.10.14 is the latest verified release. Merged S197 through S201 are on `main` at `f48d3525ad5317bc1d76c54b20ce5524d791242d`, with successful post-merge checks. The source workspace still declares 0.10.14, and the five slices' fragments remain under `changelog.d/`.

**Decision**: Prepare v0.11.0 from that accepted source. The new minor version clearly distinguishes the first public direct final-client route from the prior patch series. Keep the v0.10.14 published record until the tagged public files and registry archives have been verified.

**Alternatives considered**: v0.10.15 would also be mechanically valid, but it would present the new route as another repair in the 0.10 line. A new 1.0 release would overstate the version change and the existing support contract.

## Release authority

**Observation**: `release.toml` permits version preparation only on `release/*` branches and disables tag, push, and publication. The tagged workflow checks version identity, certifies Windows files, creates a GitHub release, and waits for protected registry approval. `CONTRIBUTING.md` reserves final PR merge and tag push for the owner.

**Decision**: Use `release/0.11.0` and the repository's dry-run-first preparation path. The current authorization covers candidate push and PR only. After owner merge, the same S202 release record can continue through owner-tagged publication and verified current-state reconciliation.

**Alternatives considered**: Publishing directly from the S201 merge would retain version 0.10.14 and omit assembled release notes. Combining version bump with a tag or push would violate the repository's explicit separation of owner actions.

## Documentation and verification

**Observation**: Site source includes S201 command and installation guidance but labels the direct route as a source candidate while v0.10.14 is public. The main-branch docs workflow deployed after S201. Candidate package certification and local checks already exercised the merged feature, while S202 must validate the versioned release tree.

**Decision**: In the candidate, identify v0.11.0 as upcoming and preserve v0.10.14 as public. After release readback, change candidate wording and current markers together. Record actual Codex findings and check results; a missing security-bot response is an evidence limit, not a new release prerequisite.

**Alternatives considered**: Marking the new route as already public during candidate preparation would be false. Requiring a private game or a combined hosted Npcap run would add an unapproved standing gate despite controlled and local product evidence.
