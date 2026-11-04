# Feature Specification: Session Causal Diagnostics

**Slice**: S190
**Branch**: `codex/s190-session-causal-diagnostics`
**Created**: 2026-11-04
**Status**: In progress
**Issues**: #551 and the remaining reporting requirement in #553
**Architecture of record**: `docs/fragcap-specification.md` sections 12, 20, 22, 25, and 29

## User Scenarios

### User Story 1: Explain a failed proxy connection (P1)

An operator opens a retained session bundle after a short-lived launcher connection fails. The connection record identifies each reached and completed protocol stage, the client-facing or upstream failure boundary, a stable cause and available alert or transport detail, and its relation to the request and process instance. Unknown and ambiguous facts remain explicit. The terminal summary and calibration result project the same connection identity and cause.

**Independent check**: Controlled client TLS refusal, reset, upstream TCP failure, upstream TLS verification failure after client TLS success, and post-response idle timeout each produce distinct retained outcomes with no uncounted observation loss.

### User Story 2: See consumed and available phase time (P2)

An operator can compare authorized finite budgets with actual elapsed and remaining time for acquisition, verified observation, owner release, proxy shutdown, and cleanup in terminal output and the manifest. A phase that never started is identified as such and is never shown as zero elapsed completion.

**Independent check**: Controlled late acquisition, no-client timeout, full observation, and early terminal outcomes produce consistent phase timing in human and JSON reports.

## Edge Cases

- A client TLS alert, transport reset without an alert, timeout, and upstream verification failure have different causes even when all end before the first HTTP request.
- A successful HTTP exchange remains recorded when its keep-alive connection later times out awaiting another request.
- A process ownership join with missing, conflicting, or stale evidence is unavailable or ambiguous, never inferred from time proximity.
- Bounded diagnostic retention reports exact dropped and unavailable counts; a missing detail cannot be presented as an observed negative.
- A phase that starts but is interrupted reports actual consumed time and the authorized time that remained at interruption.

## Functional Requirements

- **FR-001 (#551)**: Each accepted native proxy connection MUST retain a bounded, versioned stage timeline with accept, authentication, requested authority or CONNECT, upstream TCP, tunnel response, client TLS, upstream TLS, protocol selection, first request, and terminal transitions as applicable. The record MUST distinguish last reached from last completed stage.
- **FR-002 (#551)**: Every failed connection MUST identify the client-facing or upstream boundary when observed, a stable failure category, any observed safe TLS alert, and a sanitized transport or OS error category. Missing facts MUST be marked unavailable. A reset without an alert MUST remain distinct from a certificate alert and a timeout.
- **FR-003 (#551)**: A completed client TLS leg MUST remain visible when a later upstream TLS leg fails. A successful exchange MUST remain visible when a later keep-alive read times out, with the timeout marked before-first-request or after-completed-exchange.
- **FR-004 (#551)**: Connection identity MUST join native proxy records, request evidence, calibration result, and any exact process-instance evidence. An unsupported, missing, stale, or ambiguous process join MUST report unavailable or ambiguous without guessing.
- **FR-005 (#551)**: Retained local bundle inspection and terminal reports MUST expose the bounded diagnostic timeline with version and named loss counters. Captured payloads, capabilities, credentials, private certificate material, and unsanitized OS error text MUST NOT enter the diagnostic projection.
- **FR-006 (#553)**: Terminal output and manifest MUST report actual elapsed and remaining time for every authorized lifecycle phase, including startup, capture preparation, acquisition, observation, capture return, owner release, proxy shutdown, and cleanup. Each terminal phase MUST distinguish not-started, completed, interrupted, and failed outcomes.
- **FR-007 (#553)**: Phase time MUST use the existing finite authorized budgets and real phase origins. Reports MUST preserve subsecond precision and MUST NOT imply that unspent time is available after a session ends.
- **FR-008**: Capture mode, routing scope, installation CA lifecycle, proxy forwarding, application observation, and session cleanup semantics MUST remain unchanged except for the added diagnostic facts.

## Success Criteria

- **SC-001**: Each controlled connection failure case has a retained connection ID, distinct boundary and category, truthful last-reached and last-completed stages, and exact diagnostic loss accounting.
- **SC-002**: A client TLS success followed by upstream TLS failure retains the client success fact; a keep-alive timeout after a completed response retains the response and labels the timeout phase.
- **SC-003**: Every controlled lifecycle outcome yields phase elapsed and remaining milliseconds consistent with its authorized budget and observed start/end instants; absent phases are explicitly not started.
- **SC-004**: The session bundle and calibration projection agree on connection IDs and phase facts, and no prohibited secret or raw payload is introduced by these fields.

## Clarifications (2026-11-04)

- Stage facts use the existing accepted connection identity and a finite ordered vocabulary. A stage may be reached without completion; an absent stage is unavailable, not failed. Repeated request activity is represented by bounded counters and first-request or last-completed-exchange facts rather than an unbounded event list.
- Elapsed and remaining time are measured at the phase's actual terminal observation. Remaining means unused authorized budget at that instant, not permission to resume a completed session. An unstarted phase has no elapsed or remaining measurement even though its authorized budget remains visible in the plan.
- Process ownership uses the existing exact socket and process-instance evidence when available. The proxy's session credential alone does not prove process identity. The bundle may report unavailable or ambiguous ownership with no inferred PID.
- Diagnostic fields use stable codes, numeric transport/OS categories where observable, and bounded TLS alert descriptions. Raw error strings, authority values, credentials, payloads, and certificate identities do not enter summary diagnostics; existing raw private artifacts retain their own authority.

## Assumptions and Scope

- The existing bounded native proxy observation stream, process-instance collector, manifest, and terminal projections are extended rather than replaced.
- #542 remains a separate launcher incompatibility investigation. This slice supplies causal evidence but does not claim a cause or field resolution without a post-release observation.
- #552 remains a separate managed handoff and cleanup issue. Its S189 source change is not re-scoped here.
- Controlled automated checks and repository CI verify this implementation. Installed-product field observations occur after release; a newly observed release defect is recorded as a GitHub issue.
