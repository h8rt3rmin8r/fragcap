# Feature Specification: S186 Installed Deep Capture truth

**Feature Branch**: `codex/s186-installed-deep-capture-truth`
**Created**: 2026-10-31
**Status**: Approved for implementation
**Input**: Owner-approved v0.10.10 defects #534 and #535 and expedited autopilot delivery.

## User Scenarios & Testing

### User Story 1 - Ordinary-user CA health (Priority: P1)

An ordinary account runs help, version, discovery, CA status, or Doctor after a healthy machine installation. The signer is restricted to SYSTEM and Administrators, but the user sees the public CA identity and exact trusted root without a false repair warning. Signing ability remains explicitly unverified until an elevated proof.

**Independent Test**: Exercise each named command against an installer-created restricted signer as an ordinary account, then prove an elevated signing success and failure separately.

**Acceptance Scenarios**:

1. **Given** a healthy public identity and exact machine root with a restricted signer, **when** an ordinary account starts a command, **then** no repair advice appears and signing is reported as unverified.
2. **Given** a missing or invalid public resource, **when** status runs, **then** its specific observed failure is reported.
3. **Given** a signer that cannot decrypt or sign, **when** elevated Deep Capture starts, **then** it refuses before session effects with an exact failure.

---

### User Story 2 - HTTPS CONNECT authority (Priority: P1)

An authorized target sends a request inside HTTPS CONNECT. Equivalent authorities use the same destination and correct HTTPS scheme even when the inner Host omits the default port. A valid supported request is forwarded over separately verified upstream TLS. A genuinely unsupported or unsafe distinct authority produces a precise compatibility reason.

**Independent Test**: Controlled HTTP/1.1 and HTTP/2 tunnel peers cover implicit and explicit ports, name normalization, valid distinct-authority behavior if supported, unsafe cross-authority requests, and a stalled header.

**Acceptance Scenarios**:

1. **Given** CONNECT to port 443 and an equivalent inner Host without a port, **when** the request arrives, **then** it forwards under verified TLS and records an HTTPS exchange.
2. **Given** a distinct inner authority, **when** target scope and verified upstream policy permit handling, **then** it is handled with complete accounting; otherwise it is refused with an exact reason.
3. **Given** an unsafe cross-authority request, **when** it is evaluated, **then** no unverified upstream forwarding occurs.
4. **Given** a stalled header, **when** its bounded deadline expires, **then** the actual read stage is reported independently from authority failures.

---

### User Story 3 - Truthful session evidence (Priority: P2)

The operator reviews a partial managed session. Packet, proxy, application, HAR, compatibility, and cleanup records reconcile independently. Plain HTTP responses and clean process exit cannot establish full HTTPS inspection.

**Independent Test**: Reproduce the reported five authority failures and one header timeout in a controlled or privately retained case and reconcile terminal counts and artifacts.

**Acceptance Scenarios**:

1. **Given** five authority failures and one independent timeout, **when** the session ends, **then** all six terminal causes remain distinct and no full HTTPS fact appears.
2. **Given** a complete supported HTTPS exchange, **when** evidence finalizes, **then** its application, HAR, compatibility, proxy, and packet facts agree.

### Edge Cases

- Ordinary-user denial of signer metadata can coexist with readable public identity and root.
- Elevated metadata access can coexist with a decryption or signing failure.
- Bare Host, explicit default port, absolute-form target, case folding, IPv6 literal, and HTTP/2 authority retain protocol-specific meaning.
- CONNECT authority, SNI, inner authority, and upstream certificate can disagree for distinct reasons.
- A timeout before the inner request has no observed inner authority.
- Public evidence omits private title, account, path, endpoint, process, fingerprint, and raw traffic.

## Requirements

### Functional Requirements

- **FR-001 (#534)**: Read-only startup and status MUST evaluate observable public identity, validity, cutover receipt, and exact machine root independently from private signer access.
- **FR-002 (#534)**: Ordinary-user denial of signer metadata MUST NOT produce a damaged-installation verdict or repair advice. The result MUST distinguish unverified signing from a privileged proof or confirmed failure.
- **FR-003 (#534)**: Missing signer, public identity, wrong-store root, mismatch, expiry, and inaccessible public resources MUST retain truthful distinct results.
- **FR-004 (#534)**: Deep Capture and calibration MUST retain fresh exact-root and sign/verify proof before any session effect, refusing on confirmed failure.
- **FR-005 (#534)**: Help, version, discovery, status, and Doctor MUST preserve human and structured output, including clean machine-readable stdout.
- **FR-006 (#535)**: The proxy MUST interpret inner HTTP authorities in their actual HTTPS tunnel context, including scheme, default port, and normalized host, without altering raw observations.
- **FR-007 (#535)**: Supported requests MUST retain scoped routing and independently verified upstream TLS. Unsafe or unsupported distinct authorities MUST refuse with a precise reason.
- **FR-008 (#535)**: Header timeout diagnostics MUST identify their actual stage independently from authority mismatch without merely widening the deadline.
- **FR-009 (#535)**: Packet, proxy, application, HAR, compatibility, and cleanup evidence MUST reconcile. HTTP-only entries and clean exit MUST NOT imply full HTTPS inspection.
- **FR-010**: Controlled acceptance MUST establish each issue separately, including restricted Windows ACL behavior and the authority-plus-timeout pattern, while keeping private field evidence local.

### Key Entities

- **Public installation health**: Observable CA identity, validity, receipt, and exact root plus separate signer-proof status.
- **Tunnel authority**: Normalized CONNECT destination and independently observed SNI and inner request authority.
- **HTTPS exchange**: Forwarded request and response with verified upstream identity and complete evidence.
- **Compatibility verdict**: Result derived from observed protocol and cause, independent of unrelated HTTP and cleanup success.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every named ordinary-user command on a restricted installer signer yields zero false repair warnings and valid machine-readable stdout; CA status reports signing as unverified.
- **SC-002**: Controlled missing signer, identity, wrong-store, mismatch, expiry, and public-access cases yield exact distinct results; elevated signing success and pre-effect failure both pass.
- **SC-003**: Controlled equivalent HTTPS authority cases forward and record complete exchanges; unsafe cases perform zero unverified upstream forwards.
- **SC-004**: A controlled five-to-one mismatch and timeout pattern retains six exact terminal causes and zero full HTTPS facts; a complete supported exchange yields full HTTPS only after response evidence.
- **SC-005**: Repository and controlled Windows/proxy gates pass on the candidate, with each issue assessed independently.

## Assumptions

- The v0.10.10 reports in #534 and #535 are accurate field observations. Their counts do not prove the underlying authority values.
- Controlled reproduction establishes repository-owned acceptance without running a private real game. Later release failures become concrete new defects.
- Patch release publication is a later owner-controlled action after this implementation slice.

## Clarifications

### Session 2026-10-31

- **CA verdict**: `ready` means the public identity and exact machine root are observable and valid. It does not mean the private signer was proved. Status reports signer proof as `unverified` until a privileged sign/verify operation. Access denial on a restricted signer does not override public health; a confirmed missing or invalid signer remains a distinct failure when that fact is observable.
- **Tunnel identity**: Equality is over normalized host and effective port in the request's actual tunnel context. A bare Host inside HTTPS CONNECT uses the HTTPS default, while the raw header remains unchanged. Genuinely distinct authorities remain refused unless the existing scope and verification contract demonstrably supports independent handling.
- **Timeout**: The single header timeout is an independent observed terminal. The slice will identify the read stage using controlled evidence and retain the configured bound, rather than treating it as a sixth authority mismatch.
- **Field evidence**: The private five-to-one observation remains local. Controlled reproduction and repository gates establish the implementation correction. A public report may give redacted counts and causes but not private values or a claim of successful real-title HTTPS after this source change.
