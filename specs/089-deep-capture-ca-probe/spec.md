# Feature Specification: Deep Capture CA Trust-State Probe

**Feature Branch**: `codex/089-deep-capture-ca-probe`

**Created**: 2026-08-28

**Status**: Approved for implementation

**Input**: GitHub issue #250, implement a read-only Windows trust-state probe for
fragcap-owned Deep Capture certificate authorities.

## User Scenarios & Testing

### User Story 1 - See the Actual Trust State (Priority: P1)

As an operator, I want `fragcap doctor` to tell me whether certificate material
created by an earlier Deep Capture session remains trusted, so I can distinguish a
clean machine from supported trust, misplaced trust, or inconsistent evidence.

**Why this priority**: Doctor currently reports a known placeholder. That makes a
shipped trust-sensitive diagnostic untruthful.

**Independent Test**: Inject manifest-backed certificate identities and simulated
Windows certificate-store inventories, then verify each state without changing a
real trust store.

**Acceptance Scenarios**:

1. **Given** no manifest-backed CA identity is present in either Root inventory,
   **When** doctor classifies the inventory, **Then** it reports the CA absent.
2. **Given** an exact manifest thumbprint is in current-user Root, **When** doctor
   classifies the inventory, **Then** it reports supported current-user trust and
   the observed thumbprint.
3. **Given** an exact manifest thumbprint is only in local-machine Root, **When**
   doctor classifies the inventory, **Then** it reports the wrong store and the
   observed thumbprint.
4. **Given** the manifest's recorded thumbprint and bundled CA material differ,
   **When** doctor classifies the evidence, **Then** it reports both values as a
   mismatch instead of choosing one silently.
5. **Given** a required store or certificate cannot be read, **When** doctor runs,
   **Then** it reports an unknown state with the concrete reason.

---

### User Story 2 - Clean Up Only Exact Owned Trust (Priority: P2)

As an operator, I want a wrong-store or mismatched finding to offer cleanup only
when fragcap can name the exact manifest-backed certificate, so confirmation can
never authorize a subject-name or issuer-name wildcard removal.

**Why this priority**: Cleanup is useful only if the ownership boundary is stronger
than metadata that unrelated certificates can share.

**Independent Test**: Feed the classifier exact and inexact evidence and verify the
cleanup action appears only for an exact observed thumbprint backed by a manifest.

**Acceptance Scenarios**:

1. **Given** an exact manifest-backed thumbprint in the wrong Root store, **When**
   doctor renders the finding, **Then** it offers confirmation-gated cleanup.
2. **Given** malformed, incomplete, or merely name-matching certificate evidence,
   **When** doctor renders the finding, **Then** it offers no trust cleanup.
3. **Given** ordinary doctor runs without `--fix`, **When** it probes trust, **Then**
   it creates, installs, removes, and modifies nothing.

---

### User Story 3 - Preserve Capture Readiness and Public Truth (Priority: P3)

As a Capture user, I want Deep Capture trust warnings to remain non-blocking and
documented accurately, so a Deep Capture-only concern never disables passive
Capture.

**Why this priority**: The modes have different authority boundaries. Their
readiness verdicts must remain separate.

**Independent Test**: Render human and JSON reports for all CA states, confirm the
same observed thumbprints appear, and confirm the Capture verdict remains ready.

**Acceptance Scenarios**:

1. **Given** any observable Deep Capture CA state, **When** doctor renders human
   and JSON output, **Then** both forms contain the same uppercase SHA-1 values.
2. **Given** a Deep Capture CA warning and no Capture failures, **When** doctor
   computes readiness, **Then** it reports ready and exits successfully.

### Edge Cases

- Multiple historical manifests can name the same thumbprint; identity is
  deduplicated before classification.
- Multiple distinct owned thumbprints can remain trusted; doctor reports an
  inconsistent state rather than hiding all but one.
- A manifest thumbprint must be exactly 40 hexadecimal digits after harmless
  whitespace and separator normalization; malformed values are evidence errors.
- A manifest can be truncated, invalid JSON, or inaccessible. Such evidence is
  unknown, not absent.
- An unrelated certificate is ignored if no Deep Capture manifest names it.
- Store enumeration failure is unknown even when another store was read, because
  absence cannot then be proved.

## Requirements

### Functional Requirements

- **FR-001**: Doctor MUST derive owned CA identities only from thumbprints recorded
  by fragcap Deep Capture manifests under fragcap-owned session storage.
- **FR-002**: Doctor MUST enumerate current-user Root and local-machine Root using
  read-only Windows certificate-store APIs.
- **FR-003**: Doctor MUST classify absent, supported current-user Root, wrong
  local-machine Root, manifest/material mismatch, and unobservable/error states.
- **FR-004**: Doctor MUST normalize and render actual SHA-1 certificate
  thumbprints as 40 uppercase hexadecimal digits in human and JSON output.
- **FR-005**: Doctor MUST ignore certificates whose thumbprints are not backed by
  an owned Deep Capture manifest.
- **FR-006**: Wrong-store and mismatch findings MUST carry cleanup only when the
  exact observed owned thumbprint and store are available.
- **FR-007**: Ordinary doctor MUST NOT create CA material, change certificate
  stores, start a proxy, or write session state.
- **FR-008**: Deep Capture CA findings MUST remain non-blocking for Capture mode.
- **FR-009**: Classification tests MUST use injected inventories and MUST NOT
  mutate a real certificate store.
- **FR-010**: Specification section 26.3 and public CLI documentation MUST describe
  the implemented identity, stores, states, and cleanup boundary.
- **FR-011**: A Windows manual demonstration MUST record before, trusted, and
  cleaned observations with dates, or the exact reason mutation was not safe.

### Key Entities

- **Owned CA identity**: A normalized SHA-1 thumbprint read from a Deep Capture
  manifest, plus optional bundled-certificate material used to validate it.
- **Certificate observation**: A normalized thumbprint and exact Windows store
  location, read without mutation.
- **Trust-state evidence**: Owned identities, observations, and any errors that
  prevent complete classification.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Tests cover every required state and an unrelated certificate, with
  zero writes to a real trust store.
- **SC-002**: Human and JSON reports render identical observed thumbprints for all
  actionable states.
- **SC-003**: Every cleanup offer names one exact owned thumbprint and store; no
  name-only evidence produces an action.
- **SC-004**: All repository gates pass and ordinary doctor contains no trust-store
  mutation call.

## Assumptions

- The manifest `trust.thumbprint` written by the MVP is the durable fragcap-owned
  identity. Certificate subject and issuer names are not ownership evidence.
- Current-user Root is the only supported location. Local-machine Root is the
  wrong trust scope relevant to this slice.
- SHA-1 is used only as the Windows certificate-store identifier established by
  the MVP, not as a cryptographic integrity claim.
- Remediation remains behind the existing `doctor --fix` confirmation gate.
