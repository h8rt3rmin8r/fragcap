# Feature Specification: Deep Capture doctor readiness and cleanup

**Feature Branch**: `074-deep-capture-doctor`

**Created**: 2026-08-25

**Status**: Draft

**Input**: User description: "Issue #218. Add Deep Capture readiness and cleanup checks to doctor without implementing the proxy backend or CA creation lifecycle."

## Clarifications

### Session 2026-08-25

- Q: Should `doctor` silently clean Deep Capture state at startup? A: No. Ordinary `doctor` is read-only. Cleanup remains confirmation-gated through `doctor --fix`.
- Q: Should Deep Capture warnings make Capture mode not ready? A: No. Deep Capture readiness is reported separately and remains non-blocking for ordinary Capture unless a future Deep Capture command asks for a blocking preflight.
- Q: What can be cleaned in this slice? A: Only known fragcap-owned session files under the configured Deep Capture session directory.
- Q: Does this slice create a CA or proxy backend? A: No. It reports readiness and residue only.

## User Scenarios & Testing

### User Story 1 - See Deep Capture readiness in doctor (Priority: P1)

An operator runs `fragcap doctor` before attempting Deep Capture. The report names proxy backend availability, local CA trust state, analyzer key-log readiness, session storage, and known stale residue using the same human and machine-readable surfaces as the rest of doctor.

**Independent Test**: Build injected doctor inputs with a ready Deep Capture state and verify the human and JSON outputs contain a `Deep Capture` section with all expected rows.

**Acceptance Scenarios**:

1. **Given** a supported proxy backend is found, **When** doctor runs, **Then** it reports the backend name and version.
2. **Given** no fragcap Deep Capture CA trust is present, **When** doctor runs, **Then** it reports that state as clean rather than requiring trust before use.
3. **Given** analyzer key-log configuration is visible, **When** doctor runs, **Then** it reports analyzer key-log readiness.

### User Story 2 - Surface stale Deep Capture residue (Priority: P1)

An operator has leftover session manifests, TLS key logs, or sensitive sidecars under fragcap-owned session storage. Doctor reports the residue with the paths needed to understand it and offers cleanup only through the existing `--fix` confirmation flow.

**Independent Test**: Build injected doctor inputs with stale manifests, TLS key logs, and sensitive sidecars and verify each row warns, appears in JSON, and carries the Deep Capture cleanup action.

**Acceptance Scenarios**:

1. **Given** a manifest reports unfinished cleanup, **When** doctor scans the session directory, **Then** the manifest appears as stale residue.
2. **Given** a TLS key log exists under session storage, **When** doctor scans the session directory, **Then** the key-log path appears as sensitive stale residue.
3. **Given** sensitive application, proxy, HAR, or process sidecars exist under session storage, **When** doctor scans the session directory, **Then** those paths appear as sensitive artifacts.

### User Story 3 - Clean only explicit fragcap-owned residue (Priority: P2)

An operator runs `fragcap doctor --fix` and confirms Deep Capture cleanup. The action removes known Deep Capture session files under fragcap-owned session storage and refuses anything outside that root.

**Independent Test**: Exercise the cleanup candidate selection and action loop against a scratch directory containing known Deep Capture files and unrelated files.

**Acceptance Scenarios**:

1. **Given** `doctor --fix` is not confirmed, **When** Deep Capture residue is present, **Then** no cleanup action runs.
2. **Given** cleanup is confirmed, **When** unfinished manifests or known sensitive session sidecars exist under session storage, **Then** those files are removed and the outcome reports success.
3. **Given** unrelated files exist under session storage, **When** cleanup runs, **Then** unrelated files remain.

## Requirements

### Functional Requirements

- **FR-001**: Doctor MUST include a Deep Capture section in human and JSON output.
- **FR-002**: Ordinary `doctor` MUST remain read-only and MUST NOT mutate trust, proxy state, or output artifacts.
- **FR-003**: Deep Capture checks MUST report proxy backend availability and version when available.
- **FR-004**: Deep Capture checks MUST report local CA trust state as absent, current-user trusted, wrong-store, mismatched, or unknown.
- **FR-005**: Deep Capture checks MUST report analyzer key-log readiness.
- **FR-006**: Deep Capture checks MUST report occupied proxy ports and orphaned proxy process facts when known.
- **FR-007**: Deep Capture checks MUST report stale manifests, stale TLS key logs, sensitive sidecars, and session storage path.
- **FR-008**: Stale Deep Capture residue MUST carry a structured cleanup action bound to the printed finding.
- **FR-009**: `doctor --fix` MUST clean only unfinished manifests and known sensitive Deep Capture session sidecars under the configured fragcap session directory and only after the existing confirmation gate.
- **FR-010**: This slice MUST NOT implement proxy orchestration, CA creation, trust installation, system-wide proxy settings, or deletion of arbitrary user-selected output directories.

## Key Entities

- **DeepCaptureInputs**: The injected fact model for Deep Capture readiness and residue.
- **DeepCaptureCa**: The local CA trust classification used by doctor.
- **CleanupDeepCapture**: The structured fix action for known fragcap-owned Deep Capture residue.
- **Session storage**: The configured Deep Capture bundle root, defaulting to `%APPDATA%\fragcap\sessions` on Windows and overrideable with `FRAGCAP_SESSION_DIR`.

## Success Criteria

- **SC-001**: A ready doctor fixture includes Deep Capture rows in both human and JSON output.
- **SC-002**: Injected stale manifest, key-log, and sensitive sidecar facts produce warnings and cleanup actions.
- **SC-003**: The real probe scans session storage read-only and uses bounded traversal.
- **SC-004**: The cleanup action removes only unfinished manifests and known sensitive Deep Capture sidecars under session storage.
- **SC-005**: Existing doctor `--fix`, `--json`, terminal, and confirmation rules continue to apply.

## Assumptions

- No shipped Deep Capture proxy or CA lifecycle exists yet, so this slice reports their state without creating either one.
- Future Deep Capture preflight can reuse the same fact model and decide which warnings should block Deep Capture startup.
