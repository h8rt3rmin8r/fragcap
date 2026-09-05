# Feature Specification: Native Deep Capture Performance Envelope

**Feature Branch**: `codex/128-native-performance-envelope`

**Created**: 2026-09-04

**Status**: Draft

**Input**: User description: "S128: establish bounded throughput, latency, memory, disk, queue, certificate-cache, task, shutdown, soak, and churn behavior for every shipped native Deep Capture protocol under issue #326."

## User Scenarios & Testing

### User Story 1 - Gate the complete native protocol matrix (Priority: P1)

A maintainer can run one bounded performance campaign that exercises HTTP/1.1, HTTP/2, WebSocket, gRPC, generic TCP, generic UDP, and QUIC with payload retention both enabled and disabled, then receive a complete result for every required case.

**Why this priority**: Deep Capture cannot claim an acceptable performance envelope while a shipped protocol or retention mode remains unmeasured.

**Independent Test**: Run the short campaign against the canonical case registry and confirm that every required case reports throughput, latency, resource, loss, and shutdown measurements against its predeclared budgets.

**Acceptance Scenarios**:

1. **Given** the canonical performance registry, **When** the short campaign runs, **Then** every combination of seven protocol families and two retention modes produces exactly one terminal result.
2. **Given** an omitted, duplicated, renamed, or unbudgeted required case, **When** the performance gate validates the campaign, **Then** it fails with the exact incomplete or drifting identity.
3. **Given** a measurement exceeding any declared pass threshold, **When** the gate evaluates the report, **Then** that case fails without averaging the breach away through another case.

---

### User Story 2 - Prove bounded degradation and cleanup (Priority: P2)

A maintainer can overload the proxy with controlled traffic and determine independently whether memory, task ownership, disk growth, queue pressure, certificate caching, loss accounting, and shutdown remain within their supported bounds.

**Why this priority**: A fast proxy that leaks resources, loses evidence silently, or fails to stop remains unsuitable for a game session.

**Independent Test**: Run the overload campaign with finite synthetic traffic and verify exact conservation, declared retention, bounded peak resources, and clean shutdown for every case.

**Acceptance Scenarios**:

1. **Given** pressure beyond an evidence queue or retention bound, **When** the proxy continues forwarding, **Then** every omitted, dropped, or truncated item reconciles through named counters.
2. **Given** concurrency and certificate-name churn at the declared supported limit, **When** the campaign completes, **Then** peak task and certificate-cache ownership remain within their declared ceilings.
3. **Given** a completed or interrupted workload, **When** cleanup begins, **Then** the listener and every accepted task reach a terminal outcome within the shutdown deadline and no proxy residue remains.
4. **Given** payload retention disabled, **When** the same traffic is replayed, **Then** forwarding remains complete while retained payload and disk growth satisfy the disabled-retention contract.

---

### User Story 3 - Reproduce short and multi-hour evidence (Priority: P3)

A maintainer can reproduce a quick pull-request gate and a multi-hour soak campaign from the same versioned workload definitions, preserving enough environment and result detail to distinguish product regression from machine variation.

**Why this priority**: A one-off benchmark number cannot protect later releases or demonstrate long-session stability.

**Independent Test**: Execute both campaign profiles, validate their reports against the same registry, and compare two short runs plus the soak terminal report for stable identities, bounded variance, and complete provenance.

**Acceptance Scenarios**:

1. **Given** two short campaigns on the same supported environment, **When** their reports are compared, **Then** case identities and budgets match and each measured gate result is reproducible within its declared tolerance.
2. **Given** the soak profile, **When** it runs with repeated connection, stream, datagram, and certificate churn, **Then** either a complete two-hour terminal or an explicit project-owner approval of preserved zero-failure evidence establishes the slice authority.
3. **Given** an interrupted soak, **When** its incomplete report is inspected, **Then** completed samples remain readable but cannot be mistaken for a passing terminal campaign.
4. **Given** an environment outside the report's declared comparability class, **When** results are compared, **Then** the tool reports them as non-comparable rather than as a regression or pass.

### Edge Cases

- A machine pauses, sleeps, or experiences scheduler stalls during a latency sample.
- Timer precision is too coarse for a meaningful individual operation.
- A direct-loopback baseline is itself unstable between warmup and measured traffic.
- A protocol case completes forwarding but its evidence sink saturates.
- Payload retention reaches its byte cap before the workload ends.
- Certificate-name churn exceeds the entry or byte ceiling.
- The proxy refuses excess concurrent work at its configured limit.
- Cleanup reaches its deadline with accepted work still non-terminal.
- A report is truncated, duplicated, edited, or produced from a different registry version.
- A soak is interrupted before its minimum duration or terminal reconciliation.

## Requirements

### Functional Requirements

- **FR-001**: The system MUST maintain one versioned canonical registry that defines every required protocol and retention case before measurement begins.
- **FR-002**: The required case inventory MUST contain HTTP/1.1, HTTP/2, WebSocket, gRPC, generic TCP, generic UDP, and QUIC, each with payload retention enabled and disabled.
- **FR-003**: Every case MUST declare workload size, warmup, repetitions or duration, concurrency, and pass budgets before producing measured results.
- **FR-004**: Each measured case MUST report useful-byte throughput, added loopback latency relative to a same-run direct baseline, CPU time, peak resident memory growth, artifact disk growth, queue pressure, certificate-cache occupancy, task ownership, and shutdown elapsed time.
- **FR-005**: Throughput and latency MUST be reported per case and MUST NOT be pooled across protocols or retention modes to conceal a failing case.
- **FR-006**: Resource ceilings MUST distinguish configured capacity, observed peak, final ownership, and any refused or discarded work.
- **FR-007**: Every discarded, omitted, or truncated packet, byte, stream item, datagram, event, or artifact write observed by a workload MUST reconcile through an existing named counter or an explicitly versioned performance-harness counter.
- **FR-008**: Forwarded traffic MUST use complete workload payloads independently from whether evidence retention is enabled, capped, saturated, or disabled.
- **FR-009**: Every campaign MUST prove that accepted tasks terminate, the exact listener is released, and cleanup leaves no proxy-owned residue within the declared deadline.
- **FR-010**: The short profile MUST complete within the ordinary continuous-integration time budget and MUST use thresholds broad enough to tolerate shared-runner variance without suppressing large regressions.
- **FR-011**: The manual soak profile MUST run for at least two continuous wall-clock hours by default and MUST exercise repeated connection, stream, datagram, retention, and certificate-name churn. S128 acceptance MAY instead use an explicit project-owner approval of a preserved report after at least 1,875 zero-failure case terminals and one hour of continuous evidence.
- **FR-012**: A soak MUST emit bounded periodic samples plus a distinct terminal reconciliation; an interrupted or short raw report MUST remain explicitly incomplete. A separate sanitized acceptance summary MAY record project-owner approval without altering that raw status.
- **FR-013**: Reports MUST identify the registry version, campaign profile, product version, source revision, operating-system family, architecture, logical CPU count, timer characteristics, start and end times, and declared comparability class without retaining machine-unique secrets.
- **FR-014**: Regression comparison MUST reject mismatched case inventories, budgets, profiles, or comparability classes and MUST never convert a non-comparable result into a pass.
- **FR-015**: The ordinary repository gate MUST validate registry completeness, report schema, source inventory drift, attributed executable coverage, and the short performance profile.
- **FR-016**: The long soak MUST be runnable through an explicit command and manual workflow entry point without adding a shipped runtime performance or fault-control switch or a recurring schedule.
- **FR-017**: Performance instrumentation MUST remain outside target processes and MUST NOT introduce target handles, memory reads, hooks, traffic mutation, ambient routing, trust changes, or external network access.
- **FR-018**: All workload traffic MUST stay on harness-owned loopback endpoints, use synthetic payloads, and perform no real operating-system trust-store mutation.
- **FR-019**: The system MUST publish supported limits, threshold rationale, degradation behavior, reproduction commands, and interpretation rules alongside the registry.
- **FR-020**: S128 MUST add no claim that Deep Capture is complete; issue #334 remains the sole completion gate.

### Key Entities

- **Performance Registry**: The versioned authority for required cases, profiles, workloads, budgets, tolerances, and executable evidence references.
- **Performance Case**: One protocol-family and retention-mode workload with its own immutable identity and thresholds.
- **Campaign Profile**: A short pull-request profile or multi-hour soak profile sharing the same case vocabulary while declaring different duration and sampling requirements.
- **Performance Sample**: One bounded interval measurement covering traffic, latency, CPU, memory, disk, queues, cache, tasks, and lifecycle state.
- **Campaign Report**: A newline-framed, crash-readable record containing metadata, samples, case terminals, and one reconciling campaign terminal.
- **Comparability Class**: The coarse environment identity that determines whether two reports may support a regression conclusion.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All 14 required protocol and retention cases have predeclared budgets and produce exactly one terminal result in a complete short campaign; a soak produces one terminal result per case per completed cycle.
- **SC-002**: The short campaign completes within 15 minutes on each supported continuous-integration operating-system family.
- **SC-003**: Every required case sustains at least 1 MiB/s of useful synthetic loopback payload throughput and stays within its reviewed protocol-specific p95 added-latency ceiling, which ranges from 25 to 750 milliseconds in the short profile.
- **SC-004**: Peak worker memory remains at or below 256 MiB, the maximum same-case fresh-worker private-memory span remains at or below 32 MiB, certificate ownership remains at or below 256 entries and 8 MiB, and final accepted-connection task ownership is zero.
- **SC-005**: Each workload artifact remains at or below 32 MiB, and disabled-retention cases report zero retained payload bytes.
- **SC-006**: For every pressure case, observed work equals forwarded, retained, refused, omitted, truncated, and dropped dispositions according to the declared conservation equation, with no unexplained remainder.
- **SC-007**: Clean shutdown completes within 5 seconds after a short or soak workload, with the listener released and no incomplete accepted tasks.
- **SC-008**: Soak evidence records a sample at least every 60 seconds and shows no more than 32 MiB same-case private-memory growth. It is accepted by either a complete two-hour terminal or explicit project-owner approval after at least 1,875 zero-failure case terminals and one continuous hour.
- **SC-009**: Two successive short campaigns on the same comparability class agree on every pass/fail result and keep median throughput and p95 latency within the declared 75 percent cross-process Windows comparison tolerance.
- **SC-010**: The ordinary repository gate rejects every seeded missing-case, stale-reference, budget, conservation, terminal, comparability, and source-drift defect.

## Assumptions

- The existing native protocol paths and loss counters are the source of runtime truth; S128 measures and validates them rather than creating parallel proxy behavior.
- Absolute short-profile thresholds are deliberately conservative for shared runners. Historical trends remain evidence, but a single noisy comparison does not rewrite the predeclared hard budgets.
- CPU and resident-memory observations concern only the fragcap performance harness process and its owned proxy thread, never a target process.
- The two-hour soak is an explicit, costly profile and is not added to every pull-request run; the short profile and registry validation remain ordinary gates.
- Native Windows integration breadth remains issue #327. S128 may run portable loopback cases on Windows and Linux but does not absorb that issue's complete system matrix.
- No new product dependency is required unless planning proves the standard library and existing platform bindings cannot measure a mandatory field faithfully.
