# Feature Specification: Native Deep Capture Parser Fuzzing

**Feature Branch**: `codex/126-native-parser-fuzzing`

**Created**: 2026-09-03

**Status**: Draft

**Input**: Issue #324 and the user's S126 autopilot instruction.

## User Scenarios & Testing

### User Story 1 - Exercise Every Owned Input Boundary (Priority: P1)

A maintainer can run coverage-guided fuzzing against every fragcap-owned native
protocol parser, protocol state machine, and Deep Capture artifact reader from
one explicit, versioned surface inventory.

**Why this priority**: Attacker-controlled bytes cross these boundaries. A
partial or implicit target set can leave the most security-sensitive parser
unexercised while still reporting a successful fuzz job.

**Independent Test**: Validate the surface inventory against the compiled fuzz
entry points and run every target over its committed seed corpus.

**Acceptance Scenarios**:

1. **Given** every shipped fragcap-owned parser and state machine, **when** the
   fuzz inventory is validated, **then** each surface maps to exactly one
   executable target and a nonempty synthetic corpus.
2. **Given** arbitrary bytes and arbitrary fragmentation points, **when** a
   target executes, **then** it terminates within fixed input and allocation
   limits without panic, unsafe memory error, hang, or silent truncation.
3. **Given** wire decoding owned by rustls, h2, h3, or Quinn, **when** scope is
   audited, **then** the dependency boundary is named rather than claimed as a
   fragcap-owned parser.

---

### User Story 2 - Reproduce Every Corpus Case Deterministically (Priority: P2)

A contributor can replay all committed seeds on the pinned stable toolchain
without installing a fuzzing engine, and can reproduce a discovered failure by
promoting its minimized input to the same corpus and a focused regression test.

**Why this priority**: A coverage campaign is useful only when its discoveries
become fast, deterministic, reviewable evidence in the ordinary gate.

**Independent Test**: Run the stable seed-smoke command twice and require the
same target order, case identities, outcomes, and resource ceilings.

**Acceptance Scenarios**:

1. **Given** the committed corpus, **when** stable seed replay runs, **then**
   every target and seed executes in deterministic order with no network,
   privilege, trust, process, or filesystem escape effect.
2. **Given** a truncated JSONL stream or fragmented protocol frame, **when** it
   is replayed at every selected split, **then** the reader or state machine
   returns a truthful complete, partial, refused, or malformed outcome.
3. **Given** a future minimized crashing input, **when** it is committed, **then**
   documentation requires a named regression test and a scrubbed synthetic
   corpus case.

---

### User Story 3 - Run Reproducible Bounded Campaigns (Priority: P3)

A maintainer receives a bounded CI smoke campaign on a pinned fuzz toolchain and
can run longer local campaigns with exact commands for reproduction,
minimization, coverage inspection, and artifact handling.

**Why this priority**: Unpinned nightly tooling and unbounded CI jobs are neither
reproducible nor operationally safe.

**Independent Test**: Build all libFuzzer targets and run the matrix with fixed
per-target input, time, and timeout limits on the pinned Linux nightly.

**Acceptance Scenarios**:

1. **Given** a pull request, **when** fuzz smoke CI runs, **then** it installs
   exact declared tool versions and runs every target under finite limits.
2. **Given** a crash, timeout, or sanitizer failure, **when** CI exits, **then**
   the minimized artifact is retained for diagnosis and the job fails.
3. **Given** a maintainer running a longer campaign, **when** they follow the
   guide, **then** the same target, seed, dictionary, toolchain, and limits can
   be reproduced without using real traffic or secrets.

### Edge Cases

- Empty input, one-byte input, maximum accepted input, and input beyond the cap.
- A length field near integer boundaries or much larger than retained bytes.
- Every frame or record split before, within, and after a header or delimiter.
- Valid prefix followed by malformed, duplicate, contradictory, or torn data.
- Cancellation before input, between states, and after a terminal state.
- Invalid UTF-8, embedded NUL, unusual Unicode, mapped and scoped IPv6, and
  duplicate JSON keys.
- A seed exists on disk but is empty, duplicated, untracked, or contains a
  forbidden marker resembling a credential, public address, or captured trace.
- A target builds but is absent from either the registry or CI matrix.

## Requirements

### Functional Requirements

- **FR-001**: The repository MUST carry one versioned exhaustive inventory of
  every fragcap-owned native protocol parser, protocol state machine, and Deep
  Capture artifact reader in the shipped product through S125.
- **FR-002**: Each inventory surface MUST map to exactly one coverage-guided
  fuzz target and at least one minimized deterministic seed.
- **FR-003**: Protocol coverage MUST include HTTP/1 request and response heads,
  chunk framing, Basic proxy authentication, SOCKS5 authentication and request
  transitions, SOCKS5 UDP datagrams, WebSocket frames/messages, SSE, gRPC
  envelopes, destination and certificate identities, QUIC classification, and
  fragcap-owned HTTP/3 evidence state.
- **FR-004**: Artifact coverage MUST include application JSONL, lifecycle JSONL,
  resource journals, process traces, and versioned bundle manifests.
- **FR-005**: Targets MUST exercise fragmentation, repeated transitions,
  terminal transitions, cancellation, configured limits, and serialization
  round trips where the surface supports them.
- **FR-006**: Every target MUST reject or bound input above a shared maximum and
  MUST configure parser retention limits independently of attacker-declared
  lengths.
- **FR-007**: No target or seed replay may open a listener, contact a network,
  mutate trust, launch a process, use a real session capability, or write
  outside an isolated temporary test location.
- **FR-008**: The corpus MUST be synthetic and MUST reject real traffic,
  credentials, private keys, tokens, public network endpoints, and other secret
  or personal material.
- **FR-009**: Stable seed replay MUST execute every registered target and corpus
  case deterministically in the ordinary repository test gate.
- **FR-010**: CI MUST build and execute every coverage-guided target on an exact
  nightly toolchain and exact cargo-fuzz release with bounded runs, per-input
  timeout, maximum input length, and uploaded failure artifacts.
- **FR-011**: The fuzz crate and its dependency lock MUST remain isolated from
  the product workspace and MUST NOT change the shipped dependency graph or
  minimum supported Rust version.
- **FR-012**: A repository validator MUST reject missing targets, missing or
  empty corpora, duplicate surface ownership, CI matrix drift, unsafe target
  settings, untracked corpus inputs, and forbidden corpus content.
- **FR-013**: A finding MUST be minimized and promoted to a focused regression
  test plus a synthetic corpus seed before it is considered resolved.
- **FR-014**: Campaign documentation MUST cover installation, stable replay,
  bounded smoke, longer campaigns, reproduction, minimization, coverage, corpus
  review, and artifact cleanup.
- **FR-015**: S126 MUST distinguish fragcap-owned parsers from rustls, h2, h3,
  Quinn, httparse, and serde_json dependency boundaries and MUST NOT claim to
  fuzz dependency-owned wire decoders directly.
- **FR-016**: Fuzz execution MUST preserve P-1, P-4, and P-9: no target process
  access, no silent discarded input, and no success claim for a skipped target,
  seed, incomplete stream, or campaign failure.
- **FR-017**: S126 MUST update the master specification, outline, roadmap,
  glossary or testing guidance, AGENTS architecture record, and changelog while
  leaving Deep Capture incomplete until issue #334.

### Key Entities

- **Fuzz Surface**: Stable identifier for one owned parser or state machine,
  with owner, input cap, target, corpus, exercised properties, and dependency
  exclusions.
- **Fuzz Target**: Coverage-guided binary that passes bounded arbitrary input to
  one or more related fuzz surfaces.
- **Seed Case**: Minimal synthetic byte sequence with a stable descriptive name.
- **Campaign Profile**: Exact engine/toolchain versions and finite execution
  limits for CI or longer local runs.
- **Finding Artifact**: Engine-produced reproducer that is untrusted until
  scrubbed, minimized, and promoted to permanent regression evidence.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One hundred percent of registered owned surfaces map to one fuzz
  target, a nonempty corpus, stable replay, and the CI matrix.
- **SC-002**: Every committed seed passes two deterministic stable replays with
  zero panic, sanitizer failure, hang, out-of-cap allocation, or unexplained
  truncation.
- **SC-003**: Every coverage-guided target builds and completes its bounded CI
  campaign under the exact pinned toolchain with zero surviving finding.
- **SC-004**: Controlled registry, target, corpus, and CI drift cases each cause
  deterministic validator failure.
- **SC-005**: Corpus inspection finds zero real traffic, secret, credential,
  private key, public endpoint, or untracked input.
- **SC-006**: The full repository gate passes with the product dependency graph,
  product lockfile packages, runtime behavior, and P-1 prohibition unchanged.

## Assumptions

- The shipped fragcap-owned boundary is the product state through S125. Later
  parser additions must update the inventory and campaign in the same change.
- Third-party protocol libraries remain responsible for their internal wire
  decoders; this slice exercises only fragcap-owned inputs and transitions
  around those libraries.
- The stable seed replay is the fast blocking gate. Coverage-guided CI is a
  separate Linux job because libFuzzer requires nightly and sanitizer support.

## Clarifications

### Session 2026-09-03

- “Every parser” means every fragcap-owned native protocol and Deep Capture
  artifact input boundary, not every parser in unrelated Capture or target
  discovery code and not internal parsers owned by third-party dependencies.
- A seed corpus is minimized by construction for S126; future findings use the
  engine minimizer before review and promotion.
- CI smoke is bounded per target and is evidence that targets continuously run,
  not a claim that a short campaign exhausts the input space.

## Requirements Quality Checklist

- [x] Owned and dependency parsing boundaries are explicit.
- [x] Every required protocol and artifact family is named.
- [x] Resource, fragmentation, cancellation, and truncation outcomes are testable.
- [x] Reproducibility and corpus secrecy requirements are measurable.
- [x] Product dependency, runtime, P-1, and completion boundaries are preserved.
