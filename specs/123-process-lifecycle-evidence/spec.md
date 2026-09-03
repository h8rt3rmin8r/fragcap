# Feature Specification: Complete Process Lifecycle Evidence

**Feature Branch**: `codex/123-process-lifecycle-evidence`

**Created**: 2026-09-03

**Status**: Complete

**Input**: User description: "Spec out and implement S123 under autopilot, closing the complete native process lifecycle evidence boundary described by issue #319."

## User Scenarios & Testing

### User Story 1 - Audit the Managed Process Chronology (Priority: P1)

An authorized operator can inspect one Deep Capture process trace and follow the selected managed launch from its launch receipt through relevant process creation, declared stage binding, socket ownership, stage exit, and terminal session state.

**Why this priority**: Process chronology is the missing authority between managed launch intent and packet or application ownership. Without it, a complete-looking bundle can still contain an unexplained process-trace placeholder.

**Independent Test**: Drive a controlled cold managed launch containing a root, intermediate descendant, terminal client, socket-owner changes, and orderly exits. Confirm that every observed transition has one ordered record and the trace reconciles to a complete trailer.

**Acceptance Scenarios**:

1. **Given** a cold direct, platform, or publisher launch whose exact root is created by fragcap, **when** relevant descendants bind and exit, **then** the process trace records the launch receipt, creation-time ancestry, stage transitions, socket-owner transitions, exits, and terminal state in evidence order.
2. **Given** an application observation correlates to a captured packet flow, **when** the flow has an observed process owner, **then** the process trace names the same flow, process, role, and stage anchors without a second attribution decision.
3. **Given** a supported session has a complete observed chronology, **when** the bundle is finalized, **then** `process-trace.jsonl` contains a header, records, and one reconciling trailer rather than `process-trace.unavailable`.

---

### User Story 2 - Distinguish Missing Evidence from Process Truth (Priority: P2)

An operator can tell exactly where process evidence is incomplete. Kernel event loss, trace-buffer loss, unparseable events, watcher termination, late or out-of-order events, missing launch identifiers, unobserved exits, and bounded retention loss remain separate from observed process facts.

**Why this priority**: A guessed process is worse than an explicit gap. Completeness and loss must remain independently auditable under the no-silent-loss and instrument-truth principles.

**Independent Test**: Replay PID reuse, child-before-parent delivery, exit-before-start delivery, watcher loss, kernel loss, missing exit, ambiguous stage, and bounded evidence overflow. Confirm that no identity transfers and every unavailable interval or discarded record is counted with a stable reason.

**Acceptance Scenarios**:

1. **Given** one PID is reused by two process creations, **when** packet and socket evidence spans both lifetimes, **then** each process instance remains distinct and no owner observation crosses its valid interval.
2. **Given** process events arrive out of timestamp order, **when** the trace is reconciled, **then** deterministic event-time ordering and creation identity produce the same result for every permitted delivery order.
3. **Given** the watcher or evidence writer loses information, **when** the session completes, **then** the exact observed loss counters and affected completeness state are present, and no missing process is invented.
4. **Given** only a query snapshot can describe a process, **when** that evidence is retained, **then** its ancestry and lifetime limitations are explicit and it never claims creation-time certainty.

---

### User Story 3 - Reconcile Process, Packet, and Application Anchors (Priority: P3)

An operator or downstream reader can join process lifecycle evidence to packet truth and native application evidence using the same session-local anchors and can identify every unresolved join.

**Why this priority**: The process trace becomes useful only when its claims agree with the packet and application authorities already shipped by S108.

**Independent Test**: Finalize controlled bundles containing matched, unavailable, ambiguous, retained, and lost packet-to-process correlations. Confirm that trace, packet, application, compatibility, and manifest summaries agree on their shared anchors and limitations.

**Acceptance Scenarios**:

1. **Given** packet and application records share a flow identifier, **when** process ownership changes, **then** each process trace ownership record names that same flow identifier and the observation interval supporting it.
2. **Given** packet attribution is absent, ambiguous, retained, or lost, **when** the process trace is finalized, **then** it preserves that exact state without substituting a launch or stage match as socket ownership.
3. **Given** the trace is partial, failed, or unavailable, **when** the manifest is finalized, **then** process-trace authority, finalization, completeness, loss, and correlation claims match the trace trailer or missing-trailer state.

### Edge Cases

- A managed root exits before its child start event is delivered.
- A child start arrives before its creating parent or after the parent exit.
- A PID is reused within one capture and the later process has the same image name.
- A startup snapshot names a stale parent identifier and has no creation instant.
- The platform launch receipt has no process identifier.
- A declared stage is ambiguous, escaped from owned ancestry, or never observed.
- One flow changes from live to retained fidelity or between observed process owners.
- Multiple application streams share one flow and process instance.
- The process watcher ends while packet and proxy observations continue.
- Kernel event loss, buffer loss, parser loss, trace retention overflow, and writer failure occur independently.
- A session stops before the terminal client exits.
- A crash leaves complete newline-framed process records but no trailer.

## Requirements

### Functional Requirements

- **FR-001**: Every supported cold managed Deep Capture session MUST produce a versioned process lifecycle stream beginning with one header and ending with one reconciling trailer on orderly completion.
- **FR-002**: The stream MUST retain the managed launch case and the exact launch receipt process identifier when one was observed; an unavailable identifier MUST remain explicit.
- **FR-003**: Relevant observed process starts MUST retain process identifier, creating parent identifier, image, command-line availability, event time, ancestry authority, and one process-instance identity derived from observed creation evidence.
- **FR-004**: Query-only startup snapshot records MUST remain distinguishable from creation events and MUST NOT claim creation-time ancestry or a known start instant when neither was observed.
- **FR-005**: Declared stage matches and exits MUST retain role, stage, process instance, event time, and the evidence reason that authorized the transition.
- **FR-006**: Socket-owner transitions MUST be derived from retained packet-flow observations and MUST preserve flow identifier, process, role, stage, fidelity, and supporting observation interval without running another attribution mechanism.
- **FR-007**: Process identifiers reused by multiple creation events MUST produce distinct process instances, and no stage, exit, socket owner, packet, or application identity MAY cross an instance lifetime.
- **FR-008**: Event-time reconciliation MUST be deterministic for every permitted input delivery order, including child-before-parent and exit-before-start delivery.
- **FR-009**: Watcher event loss, watcher buffer loss, unparseable event loss, ignored rundown records, watcher termination, evidence retention overflow, writer failure, and unresolved joins MUST remain separately counted or explicitly unavailable.
- **FR-010**: Missing events or intervals MUST reduce trace completeness and MUST NOT create a placeholder process, parent, stage, owner, start, exit, or terminal outcome.
- **FR-011**: Process evidence collection and serialization MUST be bounded and MUST NOT block packet acquisition, proxy forwarding, or application evidence retention.
- **FR-012**: Complete newline-framed process records MUST remain a readable prefix after interruption or crash; only one valid trailer MAY claim orderly completion.
- **FR-013**: The process trace MUST retain session, target, launch, process-instance, flow, role, stage, and terminal anchors needed to reconcile packet and application evidence whenever those anchors were observed.
- **FR-014**: Every process-side flow anchor MUST agree with the existing packet `flow_id` authority and every application-side flow anchor that references the same flow.
- **FR-015**: Unsupported, warm, unowned, ambiguous, watcher-unavailable, and no-process-observed cases MUST use stable typed limitation reasons rather than a generic success or fabricated chronology.
- **FR-016**: The manifest and compatibility artifact MUST derive process-trace finalization, completeness, loss, and correlation claims from the trace outcome.
- **FR-017**: The implementation MUST preserve query-only observation and MUST NOT add target process handles, memory rights, injection, hooks, executable modification, target key extraction, or a second process-attribution path.
- **FR-018**: Tests MUST cover direct, platform, and publisher launch chronology, startup snapshots, PID reuse, out-of-order delivery, missing exits, watcher and writer loss, socket-owner changes, shared flow anchors, interruption, and crash prefixes without a game, account, Internet access, elevation, or capture driver.
- **FR-019**: Documentation and operator-visible evidence MUST describe the exact shipped lifecycle authority and MUST NOT call Deep Capture feature-complete before issue #334.

### Key Entities

- **Process Trace Stream**: The versioned process lifecycle sidecar, including its header, ordered evidence records, loss state, and reconciling trailer.
- **Process Instance**: One operating-system process lifetime identified by a PID plus observed creation identity, or explicitly limited snapshot evidence.
- **Launch Receipt Evidence**: The observed outcome of issuing one immutable managed launch, including an exact created PID when available.
- **Stage Transition**: A declared role and stage binding or exit tied to one process instance and its authorizing ancestry evidence.
- **Socket Owner Transition**: A packet-flow ownership observation tied to a process instance, fidelity, interval, and shared flow identifier.
- **Trace Limitation**: A stable reason and count describing missing, lost, ambiguous, unsupported, or unavailable lifecycle evidence.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One hundred percent of supported controlled cold launch cases produce launch, relevant start, stage, socket-owner, exit, and terminal records or an exact typed limitation for each missing class.
- **SC-002**: Every PID-reuse and permitted event-order permutation produces byte-equivalent reconciled process identities, transitions, completeness, and counts.
- **SC-003**: Zero tested stage, socket-owner, packet, or application anchors transfer across process-instance lifetimes.
- **SC-004**: One hundred percent of watcher, retention, and writer loss injected by the controlled matrix appears in a named count or explicit unavailable state.
- **SC-005**: One hundred percent of process-side flow identifiers agree with the packet registry and any application record carrying that flow identifier.
- **SC-006**: Every orderly process trace contains exactly one valid header and trailer; every crash-prefix case contains no completion claim.
- **SC-007**: Process evidence retention stays within its declared finite bounds while packet acquisition and proxy forwarding remain independent.
- **SC-008**: The complete repository verification gate passes with no prohibited capability and no new dependency package.

## Assumptions

- S123 closes issue #319 only. Doctor completeness, fuzzing, performance qualification, packaging, UX completion, API stabilization, final documentation, independent audit, and the #334 completion gate remain later work.
- Existing managed launch receipts, ETW events, query-only snapshots, role bindings, flow registry observations, application correlations, and manifest version 2 are the sole evidence authorities reused by this slice.
- Cold direct, cold platform, and cold publisher launches can provide complete creation-time evidence. Warm or externally initiated launches may remain explicitly limited because fragcap did not create their root.
- Command lines already supplied by the permitted ETW provider may be retained in the sensitive process trace; unavailable snapshot command lines remain unavailable.
- No new third-party dependency is expected. Any proposed dependency requires a separate license, MSRV, and capability review.
