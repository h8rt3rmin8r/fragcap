# Feature Specification: S185 Clean-break installation CA

**Feature Branch**: `codex/s185-installation-ca-continuity`
**Created**: 2026-10-31
**Status**: Approved for implementation
**Input**: Owner-approved issues #528 and #529, plus the owner's clean-break instruction: no users exist, remove legacy Fragcap state when installing the new version.

## User Scenarios & Testing

### User Story 1 - Install once and retain trust (Priority: P1)

An operator installs Fragcap and sees that it will trust one machine-specific Fragcap CA for every Windows user. The installer removes the identified pre-S185 Fragcap state, provisions the root, and verifies that it can issue a valid leaf. Later sessions, repair, upgrade, and ordinary uninstall leave the same CA and signer intact.

**Independent Test**: A clean controlled install and a repeated install produce the same public fingerprint and a successful signing proof; an upgrade removes identified prior state and leaves only the new scheme.

**Acceptance Scenarios**:

1. **Given** no installation identity, **when** the elevated installer runs, **then** exactly one machine-unique CA is trusted and its signing key is retained in protected machine-local storage.
2. **Given** a healthy S185 identity, **when** repair or upgrade runs, **then** it reuses the exact certificate and signing key without a second root or renewed trust approval.
3. **Given** identified pre-S185 Fragcap state, **when** the new installer performs cutover, **then** it removes that state and confirms no identified residue remains before enabling Deep Capture.
4. **Given** a cutover or provisioning failure, **when** installation ends, **then** the outcome and exact reason are visible and Deep Capture remains unavailable; passive commands still run with a warning.

---

### User Story 2 - Run two sessions without trust loss (Priority: P1)

An operator runs Deep Capture twice. Both sessions use the same installation root for every supported TLS inspection path. Ending either session, including a failed release of a still-running managed client, does not revoke that root.

**Independent Test**: Two controlled sessions inspect the exact root in plans, issued chains, and the Windows store before and after shutdown, including an owner-release failure.

**Acceptance Scenarios**:

1. **Given** a healthy installation CA, **when** an HTTP CONNECT or QUIC/HTTP/3 session starts, **then** its leaf chains to the installed root and no session trust add/remove operation occurs.
2. **Given** a routed managed client still alive at teardown, **when** route-owner release fails, **then** the installed CA remains trusted, and proxy and route continuity are reported independently.

---

### User Story 3 - Diagnose and repair the installation (Priority: P2)

Every invocation checks installation CA health without modifying it. An unhealthy state gives a specific warning. Deep Capture refuses before session effects and directs the operator to an explicit repair command. Portable users can run the one-time elevated setup command; administrators can explicitly remove or rotate only the exact installed identity.

**Independent Test**: Controlled missing, wrong-store, mismatched, expired, unreadable-key, and missing-key cases yield distinct warnings and pre-effect refusals; ordinary commands retain their output contract.

**Acceptance Scenarios**:

1. **Given** an unhealthy installation, **when** help, version, Capture, or Doctor starts, **then** one read-only warning appears on the diagnostic stream and machine-readable stdout remains intact.
2. **Given** an unhealthy installation, **when** any Deep Capture entry path begins, **then** it refuses before bundle, listener, launch, route, or trust effects, naming the state, LocalMachine/Root, and repair command.
3. **Given** a portable copy, **when** an administrator runs `fragcap ca install`, **then** it uses the same exact provisioning and health rules as MSI.
4. **Given** an explicit administrator command to remove or repair, **when** its preview and exact identity are confirmed, **then** only the installation-owned resources change and the resulting health is verified.

### Edge Cases

- A machine-scope encrypted key is readable by a nonadministrator only if its ACL is wrong. Read-only startup must not attempt decryption as an ordinary user or claim signing was verified.
- A certificate in CurrentUser/Root alone is wrong-store, not healthy installation trust.
- Missing or ambiguous pre-S185 trust identity must not cause deletion of a third-party certificate; incomplete cutover is a named failure.
- A failed installer transaction removes only resources created by that transaction and preserves a previously healthy S185 identity.
- The CA has finite validity. Expiry requires explicit repair/rotation; silent replacement is forbidden.
- A stopped proxy or unresolved child route is not cured by retained CA trust and must remain a separate partial outcome.

## Requirements

### Functional Requirements

- **FR-001**: The Windows installer MUST disclose machine-wide trust and attempt one elevated, transactional, idempotent CA provision on install, including unattended installation.
- **FR-002**: The first S185 install MUST remove identified pre-S185 Fragcap user/session state, trust entries, private material, journals, and bundles from canonical locations, verify removal, and refuse the new Deep Capture path if cutover is incomplete. It MUST NOT migrate or preserve old state for compatibility.
- **FR-003**: The installation CA MUST be machine-unique and persist its exact public DER, SHA-256 fingerprint, SHA-1 thumbprint, validity, generation, and provenance separately from its private signing key.
- **FR-004**: The signer MUST live in dedicated machine-local storage with machine-scope encryption and exact non-inherited SYSTEM/Administrators access. It MUST never appear in the install tree, child environment, session bundle, release archive, or logs.
- **FR-005**: Provisioning MUST reuse a healthy S185 identity and refuse an inconsistent one; creation MUST verify exact trust readback and a signing round trip.
- **FR-006**: Every CLI start, including help and version, MUST perform read-only CA health inspection and report `ready`, `not-provisioned`, `missing-cert`, `missing-key`, `mismatch`, `expired`, or `unreadable` without contaminating stdout.
- **FR-007**: Every Deep Capture and guided-calibration execution path MUST perform a fresh signing and exact trust check before any session effect; unhealthy states MUST refuse with the state, store, and `fragcap ca repair` guidance.
- **FR-008**: Session authorization and manifests MUST bind the installation CA fingerprint. HTTP CONNECT and QUIC/HTTP/3 leaf issuance MUST use the same installed root across sessions.
- **FR-009**: Session shutdown, Doctor recovery, current-user fresh-start, MSI repair, upgrade, and ordinary uninstall MUST NOT remove the installation CA or key. Explicit elevated `ca remove` and `ca repair` MUST verify exact ownership and report the changed resources.
- **FR-010**: Startup warnings, Doctor, package evidence, installer UI, help, threat model, and public documentation MUST use the same installation-versus-session distinction.
- **FR-011**: A session ending with a live routed client MUST retain installation trust and report independent proxy/route continuity without claiming the client is disconnected.

### Key Entities

- **Installation CA identity**: One machine-specific public root, exact hashes, validity, generation, and origin.
- **Installation signer**: Separately encrypted private key with exact administrative access.
- **Installation health**: Read-only observed state plus separate privileged signing proof.
- **Cutover report**: Exact inventory and verified removal of pre-S185 Fragcap resources.
- **Session CA reference**: Fingerprint bound to authorization and artifacts; no ownership of root trust.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Clean install, repeat install, repair, and upgrade produce one identical CA fingerprint and one signer, with zero session trust mutations.
- **SC-002**: Two controlled sessions, covering HTTP CONNECT and QUIC/HTTP/3, issue chains under the same root and leave it trusted after normal and failed owner-release teardown.
- **SC-003**: All seven named health outcomes have distinct controlled fixtures; every unhealthy state refuses each Deep Capture entry before effects while a non-Deep command continues.
- **SC-004**: Installation cutover inventories and removes all identified pre-S185 canonical resources and records zero remaining; a refusal fixture proves unrelated certificates are not removed.
- **SC-005**: The official package gate verifies installer action, rollback, disclosures, and absence of private key bytes in package and ordinary artifacts.

## Assumptions

- The owner states the application has no users. S185 is a clean break and makes no compatibility guarantee for pre-S185 state or artifact formats.
- The installer can use its existing per-machine elevation. A ZIP has no install event and requires an explicit elevated setup command.
- Real-title account evidence remains private. Controlled Windows tests provide repository-owned issue-completion evidence; any later active-release failure becomes a separate defect.
- Issue #526 already owns proxy/route-owner continuity. This slice preserves its truthful status but does not claim that a persistent CA keeps a stopped proxy running or fixes `tls-tunnel-authority-mismatch`.
