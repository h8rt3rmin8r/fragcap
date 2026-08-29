# Data Model: Library-First Deep Capture Sessions

## SessionConfig

Immutable caller intent for one target and one launch case.

Fields include target selector, mode or calibration phase, launch case, trust intent, capture options, bundle destination, requested artifacts, backend selection, configured deadlines, and noninteractive policy. Construction performs local shape validation only.

## PreparedSession

Side-effect-free resolution result consumed by one coordinator.

Contains the resolved target identity, validated launch case, effective capped deadlines, prepared ordinary Capture request, backend descriptor, bundle destination, compatibility prerequisites, immutable `SessionPlan`, and `PlanId`. It cannot be cloned into two executable owners.

## SessionPlan and Authorization

`SessionPlan` is the complete reviewable action set, including target, launch, proxy endpoint class, trust action, artifacts, bundle destination, and deadlines. `PlanId` is a stable digest or identifier over the prepared plan. `Authorization::Approved` names that identifier; `Declined` terminates without effects.

## LifecycleState

Public coarse states:

1. `Prepared`
2. `Running`
3. `Observing`
4. `Stopped`
5. `Finalizing`
6. `Terminal`

Internal stages distinguish proxy startup, trust acquisition, launch, Capture, observation correlation, proxy stop, fact attempts, cleanup, snapshot creation, bundle writes, and terminal delivery. An operation validates its allowed current states before effects. Terminal sessions cannot be reused.

## Owned leases

`ProxyLease`, `TrustLease`, and managed launch or Capture ownership values represent effects that require stop or cleanup. Each caches its result and permits at most one explicit release attempt. `Drop` is only a bounded best-effort backstop and is never authoritative reporting.

## CompatibilityObservation

An observed proxy or application event plus optional packet-flow and process correlation anchors. Typed fields distinguish routing, environment propagation, trust behavior, protocol, inspectability, owner executable, handoff, and evidence source. Missing evidence remains absent or inconclusive. Conflicting observations remain separate entries.

## FactCandidate and FactWriteResult

`FactCandidate` is produced only by pure policy from observations. It carries target identity, launch case, fact kind/value, provenance, observation time, backend, and session identity. `FactWriteResult` records appended, skipped with reason, or failed with stable code. Facts are attempted independently and remain append-only.

## CleanupReport

One result per owned or expected resource: proxy process, trust entry, managed target, listener or port, material directory, and any other session resource. Results distinguish released, already absent, failed, timed out, and not attempted with reason. Every applicable resource appears exactly once.

## ArtifactResult

One result per expected artifact role, including `.fcapng`, HAR, key log, compatibility record, fact-write record, cleanup record, and manifest. It records path, sensitivity, written status, omission reason, or write failure. Required JSON files use prepare-and-replace where practical; manifest is written last.

## TerminalSnapshot and TerminalReport

After fact and cleanup attempts, the coordinator freezes a `TerminalSnapshot` containing operation outcome, observations, chronological stage failures, fact-write results, cleanup results, effective deadlines, interruption point, and event-delivery status. Bundle rendering consumes only this immutable value.

`TerminalReport` is the authoritative in-memory result and adds artifact-write outcomes and terminal-event delivery status. Outcomes are `Complete`, `Partial`, `Failed`, or `Interrupted`. A cleanup, persistence, artifact, or delivery gap prevents `Complete` without erasing the underlying operation result.

## DeepCaptureEvent

Typed, ordered lifecycle records derived from coordinator state: plan, confirmation, resource action, launch, Capture, observation, calibration, compatibility write, artifact, cleanup, failure, and terminal. Events carry stable reason codes and typed values. Presentation is external to the model.

## Validation rules

- Target and launch resolution occur once during preflight.
- Authorization must identify the exact prepared plan.
- All proxy endpoints are loopback-scoped.
- Reachability calibration never acquires trust.
- TLS calibration requires explicit trust intent and current same-case final-client routing evidence.
- Routing never implies environment propagation or CA acceptance.
- Only correlated final-client HTTPS evidence can support affirmative inspectability or trust facts.
- No missing observation becomes an affirmative fact.
- Every acquired resource receives at most one authoritative cleanup attempt.
- Every terminal authority is derived from the same frozen snapshot within its scope.
