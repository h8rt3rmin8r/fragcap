# Feature Specification: Signed Direct Route Foundation

**Feature Branch**: `codex/s199-signed-direct-route-foundation`
**Created**: 2026-11-09
**Status**: Implementation complete; owner final review and merge pending
**Input**: Owner-authorized S199 autopilot for issues #577 and #579, including automatic push, official PR, CI, and up to two Codex review rounds before owner merge.

## Scope

**Scope in**: Reconcile governing and public policy for an explicitly selected, target-scoped, signed direct Deep Capture route; select one official signed WinDivert x64 release; package its exact DLL, driver, license and corresponding-source material in both Windows distributions; certify provenance, signatures, architecture, contents, lifecycle and read-only readiness.

**Scope out**: Opening a WinDivert route handle, diverting traffic, claiming process-scoped interception, creating a route-to-proxy bridge, changing global proxy settings, reading target memory, modifying game files, or claiming game or anti-cheat compatibility. Issues #578 and #581 through #584 own those later outcomes. #580 is independent research.

**Done when**: Every policy and audit surface accepts only the owner-approved route under its stated limits, exact official bytes and corresponding obligations are certified in MSI and ZIP, Doctor reports their read-only state, ordinary Capture and offline use remain independent of WinDivert, and all applicable repository and hosted package gates pass. The PR remains for owner final review and merge.

## User Scenarios & Testing

### User Story 1 - Accurate Deep Capture policy (Priority: P1)

A contributor or operator can see that passive Capture copies traffic, while deliberately selected Deep Capture may later divert only the chosen client's eligible connections through a signed component. The rules retain the no-memory, no-game-file-edit, explicit-session, bounded-effect and unrelated-workstation continuity boundaries.

**Independent Test**: Check governing, public, template and audit text for contradictions; validate specification and documentation gates.

**Acceptance Scenarios**:

1. **Given** an approved direct-route proposal, **when** a contributor uses the feature template, **then** it does not require falsely affirming a blanket interception-driver ban.
2. **Given** an ordinary Capture request, **when** policy and user guidance are read, **then** Capture remains passive and Npcap remains separate.
3. **Given** an anti-cheat product is detected, **when** guidance is read, **then** detection alone is evidence rather than an automatic eligibility refusal.

### User Story 2 - Reproducible Windows package (Priority: P1)

A Windows user receives the same official signed WinDivert component in the MSI and portable ZIP, with a verifiable component inventory and the material needed to exercise its LGPLv3 rights. Fragcap's own Apache-2.0 identity stays clear.

**Independent Test**: Build both distributions on a fresh Windows runner, compare extracted component hashes and inspect the exact signed driver and license/source inventory.

**Acceptance Scenarios**:

1. **Given** exact pinned upstream bytes, **when** the packages are assembled, **then** both contain identical DLL and driver bytes plus the required component license, notice and corresponding source.
2. **Given** missing, altered, wrong-architecture or unsigned component bytes, **when** package certification runs, **then** it rejects the candidate before publication.
3. **Given** install, repair, reinstall, upgrade or uninstall, **when** the package lifecycle runs, **then** ownership of the component files and any pending driver-service state is reported accurately.

### User Story 3 - Read-only component readiness (Priority: P2)

An operator can inspect whether the packaged component is present and matches the selected release before a future direct-route session. This check has no route or trust effect and distinguishes missing, mismatch, blocked and insufficient-access evidence when observable.

**Independent Test**: Run Doctor against a correct, missing and altered package in controlled locations, and verify no driver handle is opened.

**Acceptance Scenarios**:

1. **Given** a valid package, **when** Doctor runs, **then** it reports exact component readiness without loading the driver.
2. **Given** absent or modified files, **when** Doctor runs, **then** it names the exact failed component and a usable next action.
3. **Given** offline replay or ordinary Capture, **when** it starts, **then** WinDivert files are not required or opened.

### Edge Cases

- A signed driver can still be blocked at load time. Read-only health must not promise that the future route will work.
- A driver can remain loaded after package file removal until reboot; uninstall reporting must not promise immediate driver-service removal.
- A portable copy has no MSI CA event. Its existing manual installation-CA path remains distinct from bundled WinDivert files.
- A mismatched DLL and driver pair, unexpected signer, stale release hash, or unsigned replacement must fail certification.
- Exact component evidence must not disclose private target identity or traffic.

## Requirements

### Functional Requirements

- **FR-001**: The governing policy MUST distinguish passive Capture from explicitly selected, target-scoped Deep Capture diversion using the selected official signed component, preserving no target memory access, no in-place game-file changes, bounded effects, cleanup and unrelated-workstation continuity.
- **FR-002**: The constitution, master specification, contributor guidance, issue template, audit policy and relevant public pages MUST agree; anti-cheat detection MUST NOT become a blanket eligibility gate.
- **FR-003**: The dependency and license policy MUST allow only the selected reviewed WinDivert distribution path without widening unrelated dependency license or source allowlists.
- **FR-004**: The Windows MSI and portable ZIP MUST contain identical pinned official x64 WinDivert DLL and driver bytes, and complete applicable LGPLv3 license, notice, source availability and relinking material.
- **FR-005**: The acquisition and certification path MUST verify official provenance, archive and member hashes, x64 architecture, exact driver Authenticode validity, package-entry equality and expected file placement before publication.
- **FR-006**: The package lifecycle MUST cover install, repair, same-version reinstall, upgrade, ordinary uninstall and portable extraction, preserving existing installation-CA behavior and reporting driver-service limitations accurately.
- **FR-007**: Doctor MUST inspect component readiness without opening a driver handle or changing machine state, and MUST distinguish absent, mismatch, signature/architecture failure, access-denied and observable block outcomes.
- **FR-008**: Ordinary Capture, offline replay and read-only inspection MUST not require or load WinDivert; selected future direct-route Deep Capture remains Administrator-gated before effects.
- **FR-009**: No package or policy change MUST add a global proxy setting, game instrumentation, unbounded route, or private-title success claim.
- **FR-010**: The complete Spec-Kit analysis and applicable repository, supply-chain, documentation, Windows packaging and exact-head CI gates MUST pass before S199 is offered for owner merge.
- **FR-011**: The hosted audit finding RUSTSEC-2026-0459 MUST be repaired in the existing bundled SQLite graph without an advisory waiver; the resulting Rust minimum, graph identity and documentation MUST agree.

### Key Entities

- **Component release**: Official upstream version, asset variant, immutable archive and member hashes, architecture and signature identity.
- **Package entry**: Exact installed path, role, expected hash, signature requirement and license/source obligation shared by MSI and ZIP.
- **Readiness result**: Read-only observation of component bytes and installation context, distinct from a successful future driver open.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All current governing and public surfaces pass a reviewed contradiction inventory, with zero remaining blanket-driver or automatic anti-cheat-brand bans.
- **SC-002**: Every exact component entry in both packages has the same pinned hash, correct architecture and required signature; each deliberate negative mutation is rejected.
- **SC-003**: All required package lifecycle rows complete on the disposable Windows runner, with exact file ownership and no unreported residue.
- **SC-004**: Read-only Doctor cases report exact valid, absent and altered states, and ordinary Capture/offline checks pass without WinDivert present.
- **SC-005**: Spec-Kit analysis, local CI parity, hosted package certification and all exact-head checks pass; every arriving review comment is addressed before owner merge handoff.
- **SC-006**: The exact-head audit accepts bundled SQLite 3.54.1 through rusqlite 0.41.1, and the minimum-toolchain gate runs on Rust 1.89.

## Assumptions

- S199 is the next free slice after merged S198.
- The approved component is the official WinDivert x64 release under the upstream LGPLv3 option; the exact version, variant and hashes are selected during research and pinned before implementation.
- S199 prepares distribution and readiness. The later route adapter is the first operation permitted to open a driver handle.
- The user's explicit instruction authorizes branch push, PR publication and one optional second Codex review round, while reserving merge.
