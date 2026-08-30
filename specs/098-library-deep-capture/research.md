# Research: Library-First Deep Capture Sessions

## Decision 1: Place orchestration in the existing facade

**Decision**: Add a split public `fragcap::deep_capture` module. Do not add a ninth workspace crate and do not place orchestration in `fragcap-core` or a concrete sibling.

**Rationale**: The facade already composes Capture, attribution, sinks, Steam launch, profiles, and targets. Deep Capture coordinates those capabilities and therefore belongs at that assembly boundary. A new crate would duplicate the facade role while requiring workspace topology, publication, dependency-gate, license, and release changes. A concrete sibling would need forbidden sibling dependencies or absorb responsibilities it does not own.

**Alternatives rejected**:

- `fragcap-core`: rejected because proxy, trust, target-store, filesystem, and launch I/O would violate P-2.
- `fragcap-cli`: rejected because issue #252 and FR-18 require a public reusable product API.
- New `fragcap-deep-capture` crate: rejected as disproportionate isolation with no cleaner graph.

## Decision 2: Use a prepared-plan authorization boundary

**Decision**: Side-effect-free preflight returns a `PreparedSession` and immutable `SessionPlan`. Authorization names the plan identifier. A mismatched or stale authorization is refused before any external effect.

**Rationale**: The plan contains the target, launch case, backend, bundle destination, trust intent, and effective deadlines that the operator or integration is approving. Binding authorization to the resolved plan prevents changes between review and execution. Interactive prompting remains the caller's responsibility because the library does not own a terminal.

**Alternatives rejected**:

- Boolean confirmation: rejected because it does not identify what was approved.
- Library-owned stdin prompting: rejected because it makes the API CLI-dependent.
- Resolving again after confirmation: rejected because the approved and executed plans could differ.

## Decision 3: Model one explicit, at-most-once coordinator

**Decision**: `DeepCaptureSession` exposes checked lifecycle operations and a convenience end-to-end runner. It owns a state machine and rejects invalid or repeated transitions before calling adapters.

**Rationale**: The current command function mixes state, effects, and presentation. An explicit coordinator makes operation order observable, keeps leases under one owner, and proves repeated stop or cleanup calls cannot duplicate effects. Preflight refusal and invalid transition are returned as typed errors before a terminal run exists. Once effects begin, operational failures are accumulated into a `TerminalReport`.

**Alternatives rejected**:

- One opaque `run` function only: rejected because integrations need controlled start, observation, stop, and cleanup boundaries.
- Typestate-only public API: rejected because it complicates dynamic integration and cannot by itself preserve a report after adapter failure.
- First-error return: rejected because it masks later cleanup and artifact truth.

## Decision 4: Keep adapter seams narrow and effect-only

**Decision**: Define public traits for proxy leasing, trust leasing, launch preparation/execution, ordinary Capture execution, compatibility facts, artifact persistence, clock and identifiers, and event delivery. The coordinator owns classification, ordering, fact selection, artifact policy, and terminal outcome.

**Rationale**: The next native proxy and managed direct-executable slices need replacement points without new command branches. Effect-only adapters prevent policy from leaking into backends. Controlled adapters can exercise all behavior without a driver, trust mutation, game, elevation, or remote service.

**Alternatives rejected**:

- One large environment trait: rejected because implementations could silently absorb policy and tests could not isolate failures.
- CLI types in trait signatures: rejected because that recreates a facade-to-CLI dependency.
- Raw child processes as public values: rejected because ownership and cleanup become ambiguous.

## Decision 5: Reuse ordinary Capture through a facade-owned seam

**Decision**: Lift the reusable ordinary Capture preparation/execution boundary out of CLI-private types, then have both the Capture command and the Deep Capture production adapter consume it. Do not add a Deep Capture packet pipeline.

**Rationale**: FR-9 requires the same packet acquisition and attribution composition. Injecting the CLI's private `PreparedCapture` back into the facade would preserve the architectural defect. A facade-owned runner keeps capture and attribution separate while letting the CLI remain a presentation adapter.

**Alternatives rejected**:

- Duplicate pipeline assembly in Deep Capture: rejected because it creates a second packet path.
- CLI-implemented library adapter: rejected because the public capability would still depend on command-owned business machinery.

## Decision 6: Finalize bundles from a post-cleanup snapshot

**Decision**: Evidence may be staged while the run proceeds, but compatibility fact attempts and every resource cleanup attempt occur before the coordinator creates one immutable terminal snapshot. Bundle artifacts are rendered from that snapshot, with the manifest written last. The in-memory terminal report remains authoritative if a late artifact write fails.

**Rationale**: The v0.7 bundle contains cleanup truth. Finalizing it before cleanup would make the manifest lie. Artifact writes are individually reported, so a missing or partial manifest is an explicit storage failure rather than contradictory success.

**Alternatives rejected**:

- Bundle finalization before cleanup: rejected because cleanup status would be predicted rather than observed.
- Rewrite every earlier artifact after each later failure: rejected because a final write can itself fail and cannot retroactively update already failed output.
- Treat the manifest as authoritative after a write failure: rejected because only the returned report sees the complete result.

## Decision 7: Preserve event compatibility through pure presentation

**Decision**: The library owns typed `DeepCaptureEvent` variants and stable reason codes. The CLI converts them to the existing human and JSON formats without reclassification. Event delivery failures after effects begin are recorded and do not interrupt cleanup; the terminal report remains authoritative if terminal event delivery fails.

**Rationale**: Structured events are a public operational contract, but a reporting failure must not leak proxy or trust resources. Keeping rendering pure preserves existing names and fields while removing lifecycle decisions from CLI code.

**Alternatives rejected**:

- Let adapters emit CLI events: rejected because storage and backend code would own presentation policy.
- Abort on every event failure: rejected because it can prevent cleanup.
- Ignore delivery failure: rejected because the audit stream would silently have gaps.

## Decision 8: Use existing dependencies and feature gates

**Decision**: Add facade feature `deep-capture = ["targets"]`; keep `live`, `socket-table`, and `etw` independent. Promote the facade's existing `serde_json` use to runtime and expand its already-pinned Windows declarations only where production adapters require them.

**Rationale**: Controlled library use must compile without capture-driver features. `serde_json` and `windows-sys` already exist in the lockfile and satisfy the license and MSRV rules. Existing facade edges cover every composed crate.

**Alternatives rejected**:

- Make `deep-capture` imply live capture: rejected because controlled and offline consumers do not require a driver.
- Add an async runtime or channel framework: rejected because the existing lifecycle is synchronous and bounded adapters need no such graph.

## Decision 9: Test the public contract before CLI compatibility

**Decision**: Build controlled direct-library tests for the transition table, fault matrix, evidence rules, deadlines, resource ownership, event gaps, and artifact/fact disagreement. Retain CLI tests as outer compatibility coverage.

**Rationale**: A CLI-only test suite would permit business logic to remain in the command. Direct tests prove the facade is independently usable and that failures at every effect boundary retain evidence and cleanup attempts.

**Alternatives rejected**:

- Move existing tests without changing their entry point: rejected because it would not establish a public library contract.
- Test only the success path: rejected because partial-result truth and cleanup are the load-bearing behavior.
