# Feature Specification: TLS Evidence and Sensitive Artifact Lifecycle

**Feature Branch**: `codex/107-tls-sensitive-artifacts`
**Created**: 2026-09-01
**Status**: Complete
**Input**: Implement GitHub issues #300, #304, and #322 as one native Deep Capture slice, and correct the Windows platform workflow coverage exposed during S106 housekeeping.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Export Authorized TLS Session Secrets (Priority: P1)

An operator who deliberately requests a TLS key log receives a live, session-scoped file containing only secrets produced by fragcap's client-facing native proxy TLS endpoint. The final bundle path exists before proxy traffic starts, is announced to the operator, is flushed while the session runs, and is described honestly when it is empty, incomplete, retained, or removed.

**Why this priority**: The key log is the missing native evidence needed to decrypt controlled client-to-proxy TLS in standard analyzers. It is also highly sensitive, so its authorization and lifecycle must be correct before it is exposed.

**Independent Test**: Run controlled TLS 1.2 and TLS 1.3 clients through a key-log-enabled session, use the resulting file to decrypt the client-facing traffic in an independent analyzer, and verify that the same session creates no key log when the option is absent.

**Acceptance Scenarios**:

1. **Given** an explicitly authorized key-log request, **when** a native Deep Capture session starts, **then** the final bundle file is protected and announced before the first proxy TLS handshake.
2. **Given** concurrent successful client-facing TLS 1.2 and TLS 1.3 handshakes, **when** secrets are emitted, **then** complete standard key-log records are appended without interleaving and flushed during the live session.
3. **Given** upstream TLS handshakes, **when** key logging is enabled, **then** no upstream-only secret is written or described as client-facing evidence.
4. **Given** no explicit key-log request, **when** a session runs, **then** no key-log file is created and no manifest entry claims one.
5. **Given** an empty, partial, failed, retained, or removed key log, **when** the session report and manifest are finalized, **then** both report the exact observed state without upgrading it to success.

---

### User Story 2 - Use Explicit Client Credentials and Explain TLS Refusals (Priority: P1)

An operator may explicitly provide a client certificate chain and matching private key for an upstream that requires mutual TLS. fragcap uses only those supplied credentials, never searches for or extracts a target process's key, and records a stable refusal category when a TLS connection cannot be inspected.

**Why this priority**: Mutual TLS is a common boundary that otherwise appears as an unexplained failure. Supporting operator-owned credentials while refusing target-key extraction preserves the project's authorization boundary.

**Independent Test**: Exercise a controlled upstream that accepts the supplied identity, one that rejects it, one that requires a missing identity, and controlled validation and protocol failures; verify traffic success or the exact refusal category and confirm no credential bytes appear in logs or artifacts.

**Acceptance Scenarios**:

1. **Given** an explicit readable certificate chain and matching private key, **when** the upstream requests an acceptable client identity, **then** fragcap authenticates with that identity and continues the inspected connection.
2. **Given** an upstream that requires a client certificate and no operator credential was supplied, **when** the upstream refuses the handshake, **then** fragcap records `client-certificate-required` and does not search the target or operating-system stores for private keys.
3. **Given** expired, invalid, mismatched, or rejected operator credentials, **when** they are loaded or used, **then** the session refuses safely and reports a non-secret diagnostic.
4. **Given** an explicit certificate-validation or protocol-version failure, **when** the handshake fails, **then** fragcap records the corresponding stable category.
5. **Given** behavior that could be certificate pinning but is not conclusively observable, **when** the client-facing handshake is refused, **then** fragcap records `unknown` rather than claiming pinning or attempting a bypass.

---

### User Story 3 - Protect, Retain, Delete, and Share Sensitive Evidence (Priority: P1)

An operator can see which bundle artifacts are sensitive, choose whether completed sensitive artifacts are retained or deleted, and create a separate shareable copy with sensitive material omitted. Original evidence is never changed by sharing. Permission, copy, journal, and deletion failures remain visible and recoverable.

**Why this priority**: Key logs and private CA material turn an ordinary capture bundle into sensitive evidence. Shipping the producer without a complete protection and cleanup lifecycle would create avoidable local exposure.

**Independent Test**: Create a bundle containing ordinary evidence, a CA key, and a key log; inspect Windows access controls, retain it once, delete sensitive files once, interrupt and recover a cleanup, and produce a share copy whose transformation manifest proves which sensitive artifacts were omitted while the source remains byte-identical.

**Acceptance Scenarios**:

1. **Given** a bundle directory or sensitive file is created, **when** creation completes, **then** restrictive current-user access controls are applied atomically before sensitive content is exposed to unintended local principals.
2. **Given** a completed session whose plan says retain, **when** cleanup runs, **then** sensitive evidence remains and the result is reported and audited.
3. **Given** a completed session retained by its confirmed plan, **when** the operator runs the targeted cleanup command with explicit confirmation one or more times, **then** exactly the declared sensitive artifacts are removed, ordinary evidence remains, and repeated execution is idempotent.
4. **Given** cleanup is interrupted after an action is journaled, **when** recovery runs, **then** the pending sensitive-artifact action is completed or reported without expanding into general session recovery.
5. **Given** a request to prepare evidence for sharing, **when** the copy is produced, **then** the source is unchanged, sensitive roles are omitted from the copy, and a transformation manifest identifies every included and omitted artifact.
6. **Given** a permission, journal, copy, or deletion failure, **when** the operation ends, **then** the failure names the affected artifact and no success claim conceals it.

---

### User Story 4 - Run the Windows Platform Gate for Workspace Changes (Priority: P2)

A contributor who changes code exercised by the Windows platform workflow receives that workflow automatically, including changes under the native proxy and facade crates.

**Why this priority**: S106 exposed a platform-only test failure that the follow-up fix did not automatically retest because the workflow's path filter covered only part of the workspace it executes.

**Independent Test**: Verify the workflow has no path filter and its lint contract rejects a filtered trigger, a missing pull-request event, or a missing default-branch push event.

**Acceptance Scenarios**:

1. **Given** a pull request changes any workspace crate tested by the platform workflow, **when** GitHub evaluates workflow paths, **then** the platform workflow is selected.
2. **Given** a pull request changes the workspace manifest, lockfile, task runner, or platform workflow, **when** GitHub evaluates workflow paths, **then** the platform workflow is selected.
3. **Given** a pull request changes any repository path, including a future workspace crate, **when** GitHub evaluates the workflow trigger, **then** the platform workflow remains selected without a path-list maintenance dependency.

### Edge Cases

- The key-log file cannot be created or protected before the listener starts.
- A TLS handshake emits several labels concurrently or terminates after only some secrets are written.
- A provided private key is encrypted, malformed, unsupported, or does not match the certificate.
- An upstream requests a certificate issuer or signature scheme incompatible with the supplied identity.
- The peer closes without a TLS alert, so pinning, client authentication, and generic rejection cannot be distinguished.
- A bundle inherits permissive access from its parent, or permission hardening succeeds for the directory but fails for a newly created sensitive file.
- The process stops between journaling and performing a delete, or between copying an artifact and finalizing the share manifest.
- A share destination already exists, aliases the source, or lacks sufficient space.
- A sensitive artifact is already absent when confirmed cleanup or recovery runs.
- A workflow change touches a newly added workspace crate not named individually in the trigger.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST require an explicit, immutable session-plan request before creating a TLS key log.
- **FR-002**: The system MUST create and protect the final session-bundle key-log path before accepting proxy traffic, then announce its absolute path.
- **FR-003**: The native proxy MUST record only client-facing proxy TLS secrets in a standard analyzer-compatible format and MUST flush complete records during the session.
- **FR-004**: Concurrent key-log writes MUST be serialized so records cannot be torn or interleaved.
- **FR-005**: The system MUST distinguish requested, ready, nonempty, empty, partial, failed, retained, and removed key-log outcomes in session events and bundle metadata.
- **FR-006**: The system MUST omit the key-log artifact and manifest claim when key logging was not explicitly requested.
- **FR-007**: The system MUST accept an operator-supplied client certificate chain and private key only through explicit Deep Capture configuration, and both inputs MUST be supplied together.
- **FR-008**: The system MUST validate supplied client credentials before proxy traffic starts and MUST reject unreadable, malformed, unsupported, mismatched, or invalid credentials without emitting secret material.
- **FR-009**: The native proxy MUST present an accepted operator-supplied identity only to the intended upstream TLS connection.
- **FR-010**: The system MUST NOT discover, copy, read, or extract a target process's private keys and MUST NOT bypass certificate pinning.
- **FR-011**: TLS refusals MUST use stable categories for client-certificate-required, supplied-client-certificate-rejected, certificate-validation, protocol-mismatch, client-trust-rejection, and unknown outcomes.
- **FR-012**: The system MUST claim certificate pinning only when conclusive evidence exists; ambiguous connection termination MUST be `unknown`.
- **FR-013**: Diagnostics, events, manifests, and debug representations MUST exclude private-key bytes and TLS session secrets.
- **FR-014**: Bundle directories and sensitive artifacts MUST receive restrictive current-user access controls before sensitive bytes are made available, with failure preventing unsafe continuation.
- **FR-015**: Private CA keys and TLS key logs MUST use the strictest sensitive-artifact class; ordinary evidence MUST remain separately classified.
- **FR-016**: The immutable session plan MUST state that sensitive artifacts are retained until explicit cleanup, and deletion MUST require a targeted operator command and confirmation.
- **FR-017**: Completed-session cleanup MUST remove exactly the declared sensitive artifacts, preserve ordinary evidence, be idempotent, and report each retained, removed, already absent, or failed action.
- **FR-018**: Sensitive cleanup MUST use a durable, bounded action journal sufficient to finish or report interrupted sensitive-artifact operations; this MUST NOT claim completion of general session crash recovery.
- **FR-019**: The system MUST produce shareable evidence only as a separate destination copy, MUST never alter the source bundle, and MUST omit sensitive artifacts from that copy.
- **FR-020**: Every share copy MUST contain a transformation manifest listing the source identity, included artifacts, omitted sensitive artifacts, and any failure.
- **FR-021**: Share-copy publication MUST be atomic: an incomplete copy MUST NOT be presented as a completed share bundle.
- **FR-022**: Permission, journal, retention, deletion, and sharing actions MUST be auditable through structured non-secret records.
- **FR-023**: The Windows platform workflow MUST run for every pull request and for pushes to the default branch, without a path filter that can omit workspace code.
- **FR-024**: The workflow trigger MUST remain future-compatible without an enumerated subset of current crates or repository paths.
- **FR-025**: Existing passive Capture behavior and Deep Capture behavior without key logs or client credentials MUST remain compatible.

### Key Entities

- **TLS Key-Log Artifact**: A session-scoped, proxy-produced, highly sensitive stream of standard TLS secret records with explicit request and lifecycle state.
- **Operator Client Identity**: An explicitly supplied certificate chain and matching private key used only for a selected upstream TLS connection; private material is non-displayable and short-lived.
- **TLS Refusal**: A stable, evidence-backed reason that a TLS connection could not be inspected, including an honest unknown category.
- **Sensitive Artifact Policy**: The immutable retain-until-explicit-cleanup choice and artifact classification attached to a confirmed session plan.
- **Sensitive Action Journal**: A bounded durable record of pending and completed protection, retention, and deletion actions, scoped only to sensitive artifacts.
- **Share Transformation Manifest**: A record proving how a separate share copy differs from its unchanged source bundle.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Controlled TLS 1.2 and TLS 1.3 client-to-proxy sessions can be decrypted by an independent standard analyzer using only the emitted session key log and capture evidence.
- **SC-002**: One hundred concurrent controlled key-log emissions produce only complete, parseable records, while client-facing TLS 1.2 and TLS 1.3 handshakes produce no upstream-only secrets.
- **SC-003**: All no-authorization tests create zero key-log files and zero key-log manifest claims.
- **SC-004**: Controlled mutual-TLS tests cover accepted, rejected, missing, expired, and mismatched identities, and every refusal produces one stable non-secret category.
- **SC-005**: Windows access-control tests demonstrate that an unrelated local principal cannot read newly created private CA keys or TLS key logs.
- **SC-006**: Repeating confirmed sensitive cleanup produces the same final filesystem state and never removes an ordinary evidence artifact.
- **SC-007**: Tests force a pending delete, a deletion failure, malformed input, and the journal record bound; each case recovers the exact declared action or reports its unresolved state without broad deletion.
- **SC-008**: A share-copy test proves the source bundle is byte-identical before and after export and that the completed copy contains no strict sensitive artifacts.
- **SC-009**: Trigger tests reject path filters and require pull-request plus default-branch push coverage, preventing the platform-only coverage gap seen after S106.
- **SC-010**: The repository's complete formatting, lint, dependency, test, MSRV, documentation, security, and platform-relevant local gates pass.

## Clarifications

### Session 2026-09-01

- Q: When may a TLS key log be created? A: Only after an explicit key-log request is captured in the immutable, confirmed session plan; the default remains disabled.
- Q: Where may client credentials come from? A: Only from an operator-supplied certificate-chain path and private-key path provided together. fragcap never searches the target process or local certificate stores for target-owned private keys.
- Q: What is the sensitive-artifact retention behavior? A: Preserve completed evidence until a targeted cleanup command receives explicit confirmation.
- Q: Does #322 require completing general crash recovery issue #320? A: No. S107 adds the narrow durable journal needed to protect and finish sensitive-artifact actions; #320 remains open for full session-resource recovery.
- Q: How does sharing work? A: Sharing always creates a separate atomic copy with sensitive roles omitted and a transformation manifest; original evidence is immutable.
- Q: How broad should the platform trigger be? A: It covers workspace directories as a class because the job tests the workspace as a class, while unrelated documentation stays excluded.

## Assumptions

- Deep Capture remains explicitly selected, visible, reversible, and auditable.
- The existing per-session CA and native HTTP/TLS proxy are available from S103 through S106.
- Standard TLS key-log consumers accept the NSS key-log line format emitted by the selected TLS implementation.
- Operator-supplied client credentials use documented certificate and private-key encodings; encrypted private keys are rejected unless separately specified in a future slice.
- Revocation can only be classified when the peer or validation layer exposes conclusive evidence; otherwise the outcome remains rejected or unknown.
- General session crash recovery, HAR completion, generic transports, and certificate-pinning bypass are outside this slice.
- Broader session recovery issue #320 remains open after S107.

## Scope Traceability

- GitHub #300: FR-001 through FR-006, FR-013 through FR-018, SC-001 through SC-003.
- GitHub #304: FR-007 through FR-013, SC-004.
- GitHub #322: FR-014 through FR-022, SC-005 through SC-008.
- S106 platform follow-up: FR-023 through FR-024, SC-009.
