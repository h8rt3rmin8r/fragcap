# Feature Specification: S160 QUIC observation headroom

**Feature Branch**: `codex/s160-quic-observation-headroom`

**Created**: 2026-09-21

**Status**: Draft

**Input**: User description: "The latest PR failed CI checks upon hitting main. Un-fuck this situation."

## Clarifications

### Session 2026-09-21

- No operator clarification is required. Main run 35567343253, the retained
  artifact, prior S150 and S158 decisions, and issue #435 define the defect and
  the non-negotiable loss, memory, forwarding, cleanup, and local-execution
  boundaries.
- S158's conclusion that writer readiness alone prevents canonical QUIC queue
  saturation is disproven. The correction may revise the bounded queue
  authority, but it may not block forwarding, discard evidence silently, or
  weaken any loss gate.

## User Scenarios & Testing

### User Story 1 - Trust the first Windows result (Priority: P1)

A maintainer can merge an unrelated reviewed change without the Windows native
performance gate changing from green to red because of how the operating system
scheduled an already-ready evidence writer.

**Why this priority**: A gate that changes outcome across identical product
behavior cannot protect main or support reliable releases.

**Independent Test**: Run the canonical finite QUIC workload while withholding
consumer progress after readiness, then confirm the complete burst remains
bounded and lossless and the first fresh Windows campaign passes.

**Acceptance Scenarios**:

1. **Given** the canonical QUIC workload and a ready consumer that receives no
   further scheduling during the finite burst, **When** the workload completes,
   **Then** every observation remains admitted within the declared bound.
2. **Given** a fresh Windows runner, **When** the complete short performance
   campaign runs once, **Then** all fourteen cases pass with zero queue or
   storage loss and no rerun is needed.
3. **Given** traffic beyond the supported bound, **When** the queue saturates,
   **Then** forwarding remains independent and every discarded event and byte
   is reported by its existing named counter.

---

### User Story 2 - Read truthful conservation evidence (Priority: P1)

A maintainer can read one performance sample and determine exactly how accepted,
omitted, queue-dropped, and storage-dropped payload bytes reconcile without a
loss being omitted or counted twice.

**Why this priority**: The failed report exposed real loss, but its observed-byte
field described only written observations while its displayed equation treated
that field as all produced observations.

**Independent Test**: Feed controlled retained, omitted, queue-dropped, and
storage-dropped totals through the report projection and mutate each input in
turn; only the exact conservation result may pass.

**Acceptance Scenarios**:

1. **Given** accepted and dropped payload observations, **When** the report is
   produced, **Then** total observed bytes include each disposition exactly
   once and satisfy the published equation.
2. **Given** any retained, omitted, queue-dropped, or storage-dropped count is
   changed independently, **When** the sample is evaluated, **Then** conservation
   fails with the affected predicate identifiable.
3. **Given** a legacy loss-free report, **When** it is validated, **Then** its
   meaning and pass result remain unchanged.

---

### User Story 3 - Keep the correction bounded (Priority: P2)

An operator retains nonblocking forwarding, finite memory, exact cleanup, and
the same sensitive-operation boundaries while the ordinary evidence queue gains
enough headroom for the canonical workload.

**Why this priority**: Eliminating CI nondeterminism cannot trade away runtime
safety or conceal overload behavior.

**Independent Test**: Exercise the queue at, below, and above its declared
capacity and verify finite ownership, zero loss inside the supported workload,
exact loss beyond it, and complete terminal cleanup.

**Acceptance Scenarios**:

1. **Given** the declared queue bound, **When** the canonical workload runs,
   **Then** peak memory and artifact size remain within their existing ceilings.
2. **Given** one event beyond capacity while the consumer is stalled, **When**
   admission is attempted, **Then** the producer is not blocked and the refusal
   is counted exactly.
3. **Given** accepted work and any overload loss, **When** cleanup completes,
   **Then** queue ownership is zero and all accepted tasks are terminal.

### Edge Cases

- The consumer reports readiness and is descheduled immediately afterward.
- The consumer resumes while the finite burst is still being admitted.
- The final admitted event exactly reaches the declared capacity.
- One additional event arrives after the queue reaches capacity.
- An event contains retained payload while another represents the same traffic
  at a different observation layer.
- Storage retires after accepting events but before writing them.
- Counter arithmetic reaches its saturation boundary.
- A campaign is interrupted before its terminal reconciliation record.

## Requirements

### Functional Requirements

- **FR-001**: The system MUST define one shared bounded application-observation
  queue authority for ordinary product sessions and the canonical performance
  harness.
- **FR-002**: The declared authority MUST be large enough to admit the complete
  canonical finite QUIC workload when the consumer makes no progress after
  readiness.
- **FR-003**: Queue capacity MUST remain finite and MUST stay within the existing
  worker memory ceiling under the maximum retained-payload policy.
- **FR-004**: Queue admission MUST remain nonblocking and MUST NOT delay or
  refuse traffic forwarding because artifact storage is slow.
- **FR-005**: Events beyond capacity MUST retain the existing exact event and
  applicable byte loss accounting.
- **FR-006**: The performance report MUST distinguish bytes represented by
  accepted artifact records from bytes lost before storage and MUST project one
  total observed-byte value that includes every disposition exactly once.
- **FR-007**: The published payload conservation equation MUST match the report
  field semantics and MUST reject every independent counter mutation.
- **FR-008**: Loss-free historical performance reports at schema version 1
  MUST remain readable and valid without rewriting their evidence. New
  campaigns MUST emit schema version 2, and unknown versions MUST be refused.
- **FR-009**: Queue current ownership MUST be zero at terminal, accepted task
  ownership MUST reconcile, and shutdown MUST remain within its existing finite
  deadline.
- **FR-010**: The short campaign MUST continue to reject any queue or storage
  loss. Capacity headroom MUST NOT convert observed loss into success.
- **FR-011**: Deterministic tests MUST cover a stalled ready consumer at the
  supported burst bound and exact refusal immediately beyond the bound.
- **FR-012**: The correction MUST record why S158's readiness-only diagnosis was
  insufficient and why the prior rejection of added headroom is superseded by
  new evidence.
- **FR-013**: The correction MUST add no dependency, protocol, trust behavior,
  external traffic, target instrumentation, or product application-artifact
  content change. Performance report version 2 MAY add capacity-proof fields
  and correct the meaning of total observed payload bytes.
- **FR-014**: Local verification MUST use source tests and controlled fixtures
  only. The installed product, real games, real trust mutation, and sensitive
  live capture MUST NOT run on the operator workstation.
- **FR-015**: A fresh first-attempt Windows native performance workflow on the
  final pull-request head MUST pass without using a rerun as acceptance.
- **FR-016**: Every performance sample MUST report total attempted application
  events and the queue capacity used, and the campaign MUST fail if the complete
  canonical burst exceeds that capacity even when concurrent draining prevented
  observed loss.

### Key Entities

- **Application observation queue authority**: The single finite capacity used
  by an ordinary product session and its canonical performance measurement.
- **Canonical finite QUIC burst**: The fixed short-profile exchange that
  produces raw transport and derived application observations under the reviewed
  retention limits.
- **Payload disposition**: One mutually exclusive accepted-retained,
  accepted-omitted, queue-dropped, or storage-dropped byte outcome.
- **First-attempt evidence**: The initial workflow conclusion for one exact
  source revision, without a retry substituting for that conclusion.
- **Performance report version**: The schema discriminator that preserves
  version 1 as historical read-only evidence and makes corrected payload and
  capacity semantics authoritative only in version 2.

## Success Criteria

### Measurable Outcomes

- **SC-001**: The complete canonical QUIC burst is admitted with zero event or
  byte loss when consumer progress is withheld after readiness.
- **SC-002**: Exactly one additional event beyond the supported bound is refused
  nonblockingly and advances the exact expected loss counters.
- **SC-003**: Every controlled retained, omitted, queue-dropped, and
  storage-dropped payload combination satisfies one exact conservation equation,
  and every single-field mutation is rejected.
- **SC-004**: All fourteen short-profile cases pass on the first fresh Windows
  workflow attempt with zero hard invariant failures and clean shutdown.
- **SC-005**: Peak worker memory remains at or below 256 MiB, each artifact
  remains at or below 32 MiB, and final queue ownership is zero.
- **SC-006**: All repository-controlled format, lint, test, specification,
  performance-authority, and documentation gates pass without running the
  installed product locally.
- **SC-007**: Every one of the 98 short-profile samples proves total attempted
  application events are at or below the shared queue capacity.

## Assumptions

- The canonical QUIC workload remains the existing fixed synthetic loopback
  exchange and continues to represent an ordinary loss-free performance case,
  not an overload case.
- Arbitrarily long consumer starvation cannot be made lossless with a bounded
  nonblocking queue. Traffic beyond the declared bound remains an explicit,
  counted degradation path.
- The existing 256 MiB worker memory ceiling and payload-retention limits remain
  authoritative.
- Issue #435 owns this correction and the v0.10.2 stabilization milestone owns
  its scheduling.
