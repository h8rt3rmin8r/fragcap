# Feature Specification: S180 v0.10.8 publication

**Feature Branch**: `release/0.10.8` | **Created**: 2026-10-23 | **Status**: Public files and crates verified; records and site acceptance pending

**Input**: Publish every merged update since v0.10.7 as a fully live formal release under the established autopilot protocol, without another human handoff.

## Clarifications

### Session 2026-10-23

- Q: Which version and source? A: v0.10.8 from current main `fe0bc6096ff103389f8fe5400a8968c31a495ebc`, plus only release preparation and review corrections. The release includes all ten unreleased fragments from S174 through S179. IGDB issue #155 is separate.
- Q: Does this request authorize candidate and records PR integration, tag push, and normal protected publication? A: Yes. The owner explicitly requests a fully public release without human intervention. This release-specific authorization does not change standing repository rules or protection settings.
- Q: What if the external registry approval cannot be submitted with the authorized account? A: Preserve the pending protected deployment and report the exact blocked operation. Do not bypass or weaken the protection.
- Q: When can public markers advance? A: Only after exact tagged assets and all ten crate archives are independently reconciled.

## User Scenarios and Testing

### User Story 1 - Obtain the complete current release (Priority: P1)

An operator downloads certified v0.10.8 Windows packages that contain all merged changes since v0.10.7, including Capture contract fixes, managed Deep Capture shutdown, and CLI guidance and result corrections.

**Independent Test**: The reviewed candidate, merged source, annotated tag, certified packages and public checksums agree on one immutable version and source identity.

### User Story 2 - Consume published crates and documentation (Priority: P1)

A developer obtains all ten non-yanked v0.10.8 crates and reads current documentation that refers to the release actually published.

**Independent Test**: Registry records and downloaded archive checksums agree with the tagged source; current markers and site deployment advance only after publication evidence exists.

### User Story 3 - Complete autonomous release operations (Priority: P2)

The owner receives a final report after reviewed PR integration, exact tagged workflow completion, normal protected registry approval, public reconciliation, records integration, and site deployment.

**Independent Test**: The release record names every exact identity and terminal gate, with unresolved external blockers reported as incomplete rather than called a successful release.

## Requirements

- **FR-001**: Prepare v0.10.8 from the exact accepted main source and assemble every unreleased fragment exactly once in chronological order, without including #155.
- **FR-002**: Update workspace and generated first-party version identities, golden evidence, governed graph digests, the master specification Applies-To value, and short release highlights together; do not rewrite historical release evidence.
- **FR-003**: Keep v0.10.7 current publication markers until v0.10.8 public assets and crates are independently verified.
- **FR-004**: Pass local release gates and exact-head hosted CI, package certification, supply-chain, documentation, and review checks before candidate integration. Resolve every actual review finding. At most two candidate Codex review rounds are allowed.
- **FR-005**: Merge only the reviewed candidate through its PR, then create one annotated v0.10.8 tag on that exact merged source. Never move an existing tag or push directly to main.
- **FR-006**: Allow the tag workflow to certify final Windows bytes before GitHub release creation; preserve checksum, unsigned package, Npcap non-redistribution, and existing package policy.
- **FR-007**: Use the normal protected crates.io approval path. Do not alter environment protection or use an administrative bypass to satisfy unattended publication.
- **FR-008**: Independently reconcile all six public files, three checksum sidecars, final certification authority, and ten non-yanked crate records and archives.
- **FR-009**: Reconcile actual publication via a separate reviewed records PR, verify current public site deployment, and leave historical release records intact.
- **FR-010**: Keep the owner-specific authorization limited to S180. Report exact incomplete states and never claim live game testing or an independent whole-product security audit.

## Success Criteria

- **SC-001**: Candidate version, changelog, notes, generated evidence, supply-chain graph, and source identity agree; all required checks and actual reviews are satisfied.
- **SC-002**: The annotated tag selects the reviewed merged source, the release workflow succeeds, and six public files plus ten crate archives independently reconcile.
- **SC-003**: Current markers and production site identify only the verified v0.10.8 release, with the records PR checks green.

## Out of Scope

IGDB enrichment, new product behavior, new external dependencies, release-policy changes, installed-product or real-game execution, and claims of independent security certification.
