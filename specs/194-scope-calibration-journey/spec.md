# Feature Specification: Scope-Aware Calibration and Operator Journey

**Feature Branch**: `codex/s194-scope-calibration-journey`

**Created**: 2026-11-05

**Status**: Implemented and locally verified; pull-request review and exact-head CI determine merge readiness

**Input**: User description: "Deliver #560 and #561 as S194 under autopilot: scope-specific game inspection calibration, bounded retry and passive fallback, then the complete CLI, Doctor, output, documentation, and package-artifact journey."

## Clarifications

### Session 2026-11-05

- Q: Which public inspection-scope vocabulary and default should S194 seal? → A: `auto|game|launch-chain`; `auto` resolves before effects and recommends `game` for launcher-mediated targets when exact-owner observation and blind relay are available.
- Q: How should fallback authority be represented? → A: The initial immutable plan records separately whether one linked game-scope retry and packet-only fallback are allowed; resume cannot add either permission.
- Q: What evidence completes S194? → A: Controlled repository checks, exact-head CI, hosted review, and CI package artifact certification; installed-product behavior is assessed after release and defects return as GitHub issues.

## User Scenarios & Testing

### User Story 1 - Calibrate the Exact Inspection Scope (Priority: P1)

An operator calibrating a launcher-mediated target receives a game-only inspection recommendation before any effect, can retain an explicit launch-chain diagnostic choice, and stores observations only under the exact effective scope and middleware-handling contract that produced them.

**Why this priority**: Scope-specific applicability is the durable truth needed by every later command and output surface. Reusing broad-scope evidence as game-only evidence would make all subsequent guidance unreliable.

**Independent Test**: Migrate a store containing legacy compatibility facts and workflows, propose game-only inspection for a launcher-mediated target, record one exact scoped result, and prove that legacy or mismatched rows remain visible but inapplicable.

**Acceptance Scenarios**:

1. **Given** a launcher-mediated target and no exact current recommendation, **When** guided calibration resolves `auto`, **Then** it proposes game scope with blind middleware relay, names topology as inference, and creates no observed compatibility fact until a qualifying game-owned observation completes.
2. **Given** a legacy or launch-chain fact, **When** game-scope eligibility is evaluated, **Then** the fact remains readable history and cannot qualify the game-scope case.
3. **Given** an explicit launch-chain request, **When** the workflow is planned, **Then** launch-chain inspection remains the immutable effective scope for that attempt.

---

### User Story 2 - Preserve Failure and Execute a Declared Fallback (Priority: P2)

An operator can authorize one bounded game-scope retry after a failed launch-chain attempt and can authorize packet-only Capture fallback when application inspection cannot proceed. Every attempt remains immutable, linked, independently timed, and independently reported.

**Why this priority**: The approved relay design needs a practical recovery path that preserves the original failure and avoids midstream scope changes or repeated effects.

**Independent Test**: Drive a failed launch-chain attempt through an authorized linked game-scope retry, then drive an unavailable inspection path through packet-only fallback, asserting separate identities, unchanged target and launch authority, preserved artifacts, exact windows, finite bounds, reconciled outputs and cleanup, and no positive inspection claim from the fallback.

**Acceptance Scenarios**:

1. **Given** an initial plan that authorizes one game-scope retry, **When** launch-chain middleware inspection fails after cleanup is reconciled and the exact target and launch authority remain unchanged, **Then** a fresh linked attempt uses game scope and preserves the predecessor result and bundle without a second prompt in that invocation.
2. **Given** a plan without retry authority, changed target authority, unresolved route ownership, a repeated exact case, or an exhausted bound, **When** retry is considered, **Then** no successor effect starts and one typed reason identifies the refusal.
3. **Given** declared packet-only fallback and unavailable Deep Capture prerequisites, **When** the inspection attempt cannot proceed and passive capture, required outputs, event delivery, and cleanup are reconciled, **Then** eligible passive evidence is retained under an exact Capture window and the result reports effective Capture mode without a positive Deep Capture fact.
4. **Given** an attempted packet-only fallback with a missing capture artifact, failed required output, event-delivery failure, or unresolved cleanup, **When** fallback eligibility is evaluated, **Then** the original terminal failure remains authoritative and the fallback does not convert the command to success.

---

### User Story 3 - Operate and Diagnose Game-Only Inspection (Priority: P3)

An operator can select and understand inspection scope consistently across guided calibration, advanced Deep Capture, target details, Doctor, authorization, normal and verbose human output, JSON, help, and documentation.

**Why this priority**: The native ownership and relay capabilities are not a complete product until operators can deliberately select them and understand their evidence and cleanup boundaries.

**Independent Test**: Parse every documented command, inspect authorization before effects, run controlled success and failure journeys, and compare human, JSON, target, Doctor, manifest, and package-artifact projections for the same exact scope and attempt lineage.

**Acceptance Scenarios**:

1. **Given** `--inspect-scope auto|game|launch-chain`, **When** calibration or Deep Capture prepares a session, **Then** the requested scope, effective scope, recommendation source, relayed middleware behavior, unresolved-owner handling, fallback authority, route ownership, output, and cleanup obligations are visible before effects.
2. **Given** a completed or partial attempt, **When** normal, verbose, JSON, target detail, or Doctor output is requested, **Then** game routing, game inspection, middleware relay, passive fallback, completeness, and cleanup remain separate outcomes with one evidence-backed next action.
3. **Given** quiet, silent, redirected, color-disabled, Unicode, or narrow terminal modes, **When** scope-aware output is rendered, **Then** the existing stream and measured-layout contracts remain intact.

### Edge Cases

- Existing facts and workflows have no scope dimensions during migration.
- Resume attempts to change scope, middleware handling, retry authority, fallback authority, launch case, route, family, protocol set, target revision, or candidate choice.
- A launcher relays successfully but no exact game-owned traffic reaches the proxy.
- Game routing succeeds while TLS or protocol inspection fails.
- Ownership is ambiguous, unavailable, stale, saturated, or changes generation.
- A provisional game exits and a valid successor appears during the retained launcher handoff.
- The initial attempt leaves unresolved routing, proxy, cleanup, or output obligations.
- A retry would repeat an exact scoped case or exceed the finite workflow bound.
- Passive observation started after part of the failed attempt and therefore has a missing interval.
- Writer, queue, or identity retention fails while forwarding or passive capture continues.
- Requested scope is unknown, conflicts with resume, or game scope is unavailable for the selected target.

## Requirements

### Functional Requirements

- **FR-001**: Compatibility cases, append-only facts, workflow intent, attempt identity, stored recommendations, eligibility, and applicable policy identity MUST carry requested scope, effective scope, and middleware handling as exact typed dimensions.
- **FR-002**: The additive store migration MUST preserve existing rows and bytes, represent absent scope dimensions explicitly as legacy launch-chain history, and prevent legacy rows from qualifying game-only cases.
- **FR-003**: Guided calibration MUST resolve `auto` before effects, recommend game scope with blind relay for eligible launcher-mediated topology, distinguish inferred recommendations from observed facts, and retain explicit launch-chain diagnosis.
- **FR-004**: Scope MUST participate in canonical case keys, no-progress detection, concurrency checks, stale applicability, resume equality, and the finite workflow bound.
- **FR-005**: Only exact qualifying game-owned routing and protocol observations within the correct observation window MAY write positive game-scope facts; relay success, listener acceptance, HTTP status, and unresolved ownership MUST NOT supply them.
- **FR-006**: Launch acquisition, provisional handoff, middleware relay, game routing, game protocol inspection, passive evidence, artifact completeness, route-owner release, proxy drain, and cleanup MUST remain independent typed outcomes.
- **FR-007**: A declared game-scope retry MUST create at most one fresh immutable successor attempt linked to its failed predecessor after fresh target resolution and exact cleanup reconciliation; same-invocation inheritance of the predecessor authorization MUST require the complete target and resolved launch authority to remain identical apart from the reviewed scope transition, and the successor MUST preserve the predecessor result and artifacts.
- **FR-008**: Retry MUST refuse changed target authority, unresolved obligations, repeated exact cases, exhausted bounds, missing initial authority, and resume intent mismatch before new effects.
- **FR-009**: A declared packet-only fallback MUST retain eligible passive evidence under its actual window, report effective Capture mode and omissions, and MUST NOT create a positive Deep Capture or game-inspection fact. Fallback success MUST require a written packet-capture artifact, every required artifact written, no event-delivery failure, and every cleanup obligation released or not needed; otherwise the original terminal failure MUST remain authoritative.
- **FR-010**: The system MUST NOT change an established connection's inspection decision after a TLS or protocol failure; a different scope requires a new authorized attempt.
- **FR-011**: Calibration and advanced Deep Capture MUST accept one shared public `--inspect-scope auto|game|launch-chain` vocabulary; generated resume and next commands MUST preserve exact intent.
- **FR-012**: Authorization MUST state the selected game roles, observed launch-chain scope, relayed middleware, unresolved-owner handling, managed route effects, installation CA use, exact output, phase lifetimes, permitted retry or fallback, route-owner release, and cleanup before effects.
- **FR-013**: Normal, verbose, JSON, quiet, silent, redirected, color, Unicode, and narrow output MUST preserve existing presentation contracts while reporting requested/effective scope and independent outcome dimensions consistently.
- **FR-014**: Target details and Doctor MUST distinguish general runtime readiness from current exact-scope readiness, stale or mismatched recommendations, supported relay transports, owner-evidence capability, unresolved owners, and retained route obligations without mutating state.
- **FR-015**: Native artifacts and terminal results MUST retain scope, handling, recommendation provenance, attempt/fallback lineage, exact evidence windows, relay/inspection counters, loss, completeness, actual bundle locations, and cleanup obligations while excluding middleware decrypted content and proxy-owned keys under game scope.
- **FR-016**: Help, generated commands, public schemas, glossary, first-run and calibration guidance, target/compatibility guidance, Doctor and troubleshooting prose, package certification guidance, master specification, slice chronology, and changelog MUST describe the delivered behavior and parse under the production command tree.
- **FR-017**: CI package artifact certification MUST bind exact build identity and package bytes to the delivered parser and controlled journey. Installed-product and private-title testing MUST occur only after release and MUST NOT condition S194 completion.
- **FR-018**: S194 MUST add no system-wide proxy behavior, covert process instrumentation, target memory access, target key extraction, second target-storage path, or unbounded retry.

### Key Entities

- **Scoped Compatibility Case**: The existing exact launch, route, family, protocol, backend, product, and target-version case plus requested scope, effective scope, middleware handling, and policy identity.
- **Scope Recommendation**: An inferred or measured preference bound to a target and exact applicability dimensions, with provenance and freshness.
- **Calibration Workflow Intent**: Immutable target, candidate, launch, route, family, protocols, requested scope, resolved scope, middleware handling, and allowed retry/fallback authority.
- **Calibration Attempt**: One immutable effect window with a unique identity, predecessor link, exact scoped case, phase clocks, result, bundle, and cleanup obligations.
- **Fallback Result**: The exact executed fallback kind, effective mode, evidence window, omissions, and relationship to the failed attempt.
- **Scope Readiness**: The read-only projection of runtime readiness, exact applicable facts, relay support, ownership capability, unresolved obligations, and one next action.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every legacy store fixture migrates without data loss, remains readable, and contributes zero false game-scope eligibility.
- **SC-002**: Every controlled launcher-mediated proposal resolves `auto` deterministically before effects and distinguishes inferred scope from measured compatibility.
- **SC-003**: Every retry or fallback scenario produces either one authorized successor or one typed refusal, with no repeated exact case, mutable resume intent, or unbounded attempt.
- **SC-004**: Controlled success, partial, failure, interruption, writer-loss, and cleanup scenarios reconcile the same scope, lineage, evidence windows, omissions, and verdicts across stored data, artifacts, human output, JSON, target detail, and Doctor.
- **SC-005**: Every documented scope and fallback command parses, every invalid or conflicting form fails before effects with one actionable next command, and all existing quiet, silent, redirected, color, Unicode, and narrow-layout checks continue to pass.
- **SC-006**: The complete controlled protocol/identity matrix retains no middleware decrypted application export or proxy-owned key under game scope and reports all named forwarding, observation, and evidence loss.
- **SC-007**: All applicable repository and exact-head CI checks pass, CI certifies the exact Windows package artifacts, and the PR has no unresolved hosted review finding.

## Assumptions

- S191 exact-owner decisions and S193 blind relay are the native behavior consumed by this slice.
- The existing installation CA, destination policy, session capability, route-owner, bundle, and cleanup authorities remain unchanged.
- `auto` is a recommendation resolver, not a license to invent compatibility evidence.
- One game-scope retry is the finite automatic successor; packet-only fallback may follow only when it was included in the initial plan and no conflicting obligation remains.
- Existing broad-scope history remains useful for launch-chain diagnosis and is never rewritten as game-only history.
- CI package artifact checks establish packaged bytes and build identity. Installed-product observations follow release and become concrete GitHub issues when defects appear.

## Scope Boundaries

**In scope**: #560, #561, and completion of parent #557; store schema and migration; compatibility and workflow contracts; calibration policy; linked retry and packet fallback; CLI, Doctor, target, output, artifacts, documentation, controlled checks, and CI package artifact certification.

**Out of scope**: closing #542, publishing a release, live private-title or pre-release installed-product testing, changing installation CA lifecycle, adding proxy transports, adding target injection or memory access, changing system-wide proxy settings, or deleting private evidence.

## Done When

S194 is done when #560 and #561 acceptance is delivered, #557's remaining implementation checklist is satisfied, all tasks and requirement-quality checklists are complete, analysis has no unresolved critical finding, local verification and exact-head CI are green, hosted Codex review findings are resolved within two rounds, and the PR is ready for the owner merge ritual.
