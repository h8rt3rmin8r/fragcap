# Tasks: Header Parsing and Flow Keys

**Slice**: S03

**Branch**: `feat/header-parsing-flow-keys`

**Created**: 2026-08-08

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md),
[data-model.md](data-model.md),
[contracts/parse-api.md](contracts/parse-api.md),
[quickstart.md](quickstart.md)

Tests are included and are not optional. This is the first slice with logic, so
every claim it makes about what it parses and what it declines is a claim only a
test can support. Tasks are ordered test-first within each phase.

One honesty note on the phase split. User stories 1 and 2 are the same code read
two ways: a parser that produces keys and a parser that declines with a reason
are one function, and nothing useful can be written that does the first without
the second. The phases separate their tests, not their implementation. Phase 3
carries the layer code and its success-path tests; Phase 4 carries the rejection
tests and the counter isolation harness that makes them meaningful. Phase 3 is
not shippable alone and is not pretended to be.

## Phase 1: Setup

- [X] T001 Create the module tree `crates/fragcap-core/src/parse/` with
  `mod.rs`, `link.rs`, `ip.rs`, `transport.rs`, `fragment.rs`, and
  `direction.rs`, each carrying the SPDX header and a module doc comment naming
  the specification section it implements
- [X] T002 Declare `pub mod parse;` in `crates/fragcap-core/src/lib.rs` and add
  the crate-level documentation paragraph stating that S03 added the first
  behavior to this crate, replacing the "carries no behavior" claim in the
  existing crate docs

## Phase 2: Foundational

Leaf types every later phase needs. No parsing yet.

- [X] T003 [P] Correct the documentation on `LinkType::NULL` and
  `LinkType::RAW` in `crates/fragcap-core/src/link.rs` per plan D-7: code 0 is
  BSD loopback encapsulation with a four byte host-order address family field,
  code 101 is the encapsulation with no link layer header
- [X] T004 [P] Define `ParseReject` in `crates/fragcap-core/src/parse/mod.rs` as
  a closed enum of the twelve variants in `data-model.md`, deriving copy,
  equality, and debug, with `as_str` returning the counter name for each, and a
  doc comment on each variant stating the remedy it implies
- [X] T005 [P] Define `ParseOutcome` in `crates/fragcap-core/src/parse/mod.rs`
  with the `Parsed` and `Rejected` variants and the `flow`, `direction`, and
  `reject` accessors, deriving copy so that FR-003's no-borrow guarantee is
  structural
- [X] T006 [P] Define `InterfaceAddrs` in
  `crates/fragcap-core/src/parse/direction.rs` over a `Vec<IpAddr>` with `new`,
  `contains`, `is_empty`, and `len`, documenting that `contains` is a linear
  scan and allocates nothing
- [X] T007 Define `ParseStats` in `crates/fragcap-core/src/stats.rs` with one
  `u64` per `ParseReject` variant plus `direction_ambiguous` and
  `fragment_evicted`, and a `rejected()` method summing only the twelve, with a
  doc comment stating why there is no success counter and no grand total
- [X] T008 Add `pub parse: ParseStats` to `CaptureStats` in
  `crates/fragcap-core/src/stats.rs`, held by value beside `source`
- [X] T009 Write the test in `crates/fragcap-core/src/stats.rs` that advances
  every field of `ParseStats` and asserts `CaptureStats::fragcap_dropped` and
  `total_dropped` are still zero, pinning FR-036 and SC-008 at the type level
  before any parser exists
- [X] T010 Define `HeaderParser` in `crates/fragcap-core/src/parse/mod.rs` with
  its three fields, `new`, `set_interface_addrs`, `interface_addrs`, and
  `stats`, leaving `parse` and `apply` unimplemented, and re-export
  `HeaderParser`, `InterfaceAddrs`, `ParseOutcome`, `ParseReject`, and
  `ParseStats` from `crates/fragcap-core/src/lib.rs`

## Phase 3: User Story 1, a conversation gets an identity (Priority: P1)

**Goal**: A supported frame yields the flow key it carries, with the capturing
host's endpoint in the local position.

**Independent test**: `cargo test -p fragcap-core parse` passes the
success-path rows of the contract table in
[contracts/parse-api.md](contracts/parse-api.md).

- [X] T011 [US1] Write the frame-building test helpers in
  `crates/fragcap-core/src/parse/mod.rs` under `#[cfg(test)]`: an Ethernet
  header, an IPv4 header with settable header length, protocol, fragment flags,
  and addresses, an IPv6 fixed header, an extension header of each of the five
  handled types, and TCP and UDP headers. Each helper takes fields and returns
  bytes, so a test reads as the packet it describes
- [X] T012 [P] [US1] Write the failing success-path tests for the link layer in
  `crates/fragcap-core/src/parse/link.rs`: Ethernet dispatching on each
  EtherType, raw IP dispatching on each version nibble, and BSD loopback
  accepting each known address family value in both byte orders, per research
  R-1 and R-2
- [X] T013 [P] [US1] Write the failing success-path tests for IPv4 in
  `crates/fragcap-core/src/parse/ip.rs`: a minimum-length header, a header
  carrying options skipped by the declared length, and both transport protocols
- [X] T014 [P] [US1] Write the failing success-path tests for IPv6 in
  `crates/fragcap-core/src/parse/ip.rs`: a bare fixed header, and chains
  carrying each of hop-by-hop, routing, destination options, fragment, and
  authentication, asserting the transport ports are read from the correct
  offset in each, per research R-4
- [X] T015 [US1] Implement the link layer dispatch in
  `crates/fragcap-core/src/parse/link.rs`, returning the network protocol and
  the offset at which the network header begins
- [X] T016 [US1] Implement the IPv4 header parse in
  `crates/fragcap-core/src/parse/ip.rs`, validating version and header length,
  skipping options by the declared length, reading protocol, fragment flags and
  offset, identification, and the address pair, and bounding every read by the
  captured length rather than the declared total length, per research R-3
- [X] T017 [US1] Implement the IPv6 fixed header parse and the extension header
  chain walk in `crates/fragcap-core/src/parse/ip.rs`, with the per-type advance
  formulas from research R-4, the eight header bound, and the zero-advance
  guard from plan D-8 carrying a comment stating it is unreachable by
  construction and why it is kept
- [X] T018 [US1] Implement the TCP and UDP port reads in
  `crates/fragcap-core/src/parse/transport.rs`, requiring both port fields to be
  present in the captured bytes before returning
- [X] T019 [US1] Implement `HeaderParser::parse` in
  `crates/fragcap-core/src/parse/mod.rs`, sequencing the five validation stages
  in the order `data-model.md` fixes, and `HeaderParser::apply` writing `flow`
  and `direction` onto a `CapturedPacket`
- [X] T020 [US1] Write the test in `crates/fragcap-core/src/parse/mod.rs` that
  inserts both directions of one conversation into a map and asserts one entry,
  pinning SC-005

## Phase 4: User Story 2, a frame fragcap cannot parse says why (Priority: P1)

**Goal**: Every path that ends without a flow key ends in exactly one named,
surfaced counter.

**Independent test**: one test per `ParseReject` variant, each asserting the
full `ParseStats` delta is exactly one field.

- [X] T021 [US2] Write the counter isolation harness in
  `crates/fragcap-core/src/parse/mod.rs` under `#[cfg(test)]`: snapshot
  `ParseStats`, parse, diff every field, and assert exactly one advanced by
  exactly one. Asserting only that the expected counter moved would pass a
  parser that moves three
- [X] T022 [P] [US2] Write the failing rejection tests for the link layer in
  `crates/fragcap-core/src/parse/link.rs`, covering `UnsupportedLinkType`,
  `UnsupportedEtherType` including a VLAN tagged frame per FR-009, and
  `UnsupportedAddressFamily`
- [X] T023 [P] [US2] Write the failing rejection tests for the network layer in
  `crates/fragcap-core/src/parse/ip.rs`, covering `UnsupportedIpVersion`,
  `MalformedNetworkHeader` for an IPv4 header length below five,
  `ExtensionChainTooLong` for a nine header chain, and `NoNextHeader`
- [X] T024 [P] [US2] Write the failing rejection tests for the transport layer
  in `crates/fragcap-core/src/parse/transport.rs`, covering
  `UnsupportedTransport` for a non-TCP non-UDP protocol and for an IPv6 chain
  terminating in the encapsulating security payload, and
  `MalformedTransportHeader` for a UDP header declaring a length shorter than
  itself
- [X] T025 [P] [US2] Write the failing `ShortHeader` tests in
  `crates/fragcap-core/src/parse/mod.rs`, truncating a frame at each header
  boundary in turn, asserting the short cause rather than the malformed one
- [X] T026 [US2] Thread the rejection causes through every parse stage in
  `crates/fragcap-core/src/parse/`, advancing exactly one counter at exactly one
  site per cause, until Phase 4's tests pass
- [X] T027 [US2] Write the test asserting the stage ordering in
  `data-model.md`, namely that a frame wrong at more than one layer is counted
  at the first, using a truncated frame carrying an unsupported EtherType

## Phase 5: User Story 3, direction is honest about loopback (Priority: P1)

**Goal**: The four locality combinations produce four defined outcomes, and
neither undetermined case is silently resolved.

**Independent test**: `cargo test -p fragcap-core parse::direction`.

- [X] T028 [US3] Write the failing tests for all four locality combinations in
  `crates/fragcap-core/src/parse/direction.rs`: source local, destination local,
  both local, neither local, asserting the flow key, the direction, and the
  counter for each
- [X] T029 [US3] Write the failing test that both halves of one loopback
  conversation produce one key, exercising the canonical ordering rule from plan
  D-5
- [X] T030 [US3] Write the failing test that replacing the interface address set
  changes the direction of an identical subsequent frame, pinning FR-032's
  requirement that no derivation of a previous set survives
- [X] T031 [US3] Implement the locality rule in
  `crates/fragcap-core/src/parse/direction.rs`, returning the local and remote
  assignment plus the direction, with the canonical ordering from plan D-5 used
  only in the both-local case
- [X] T032 [US3] Implement `NoLocalEndpoint` as a rejection in
  `crates/fragcap-core/src/parse/mod.rs` per plan D-3, with a comment recording
  that the rejected packet is still retained by the caller and why a key is not
  produced
- [X] T033 [US3] Write the test asserting an empty interface address set rejects
  every packet with `NoLocalEndpoint` and drops none, pinning the edge case the
  spec calls the most likely misconfiguration

## Phase 6: User Story 4, fragments attributed without reassembly (Priority: P2)

**Goal**: A non-initial fragment resolves to its first fragment's key, or to
nothing with a counter, and no payload is ever joined.

**Independent test**: `cargo test -p fragcap-core parse::fragment`.

- [X] T034 [US4] Write the failing tests for `FragmentTable` in
  `crates/fragcap-core/src/parse/fragment.rs`: record and look up, look up a
  key never recorded, remove on last fragment, and overflow at 257 distinct
  identities asserting the oldest was evicted and the counter advanced
- [X] T035 [US4] Write the failing tests that IPv4 and IPv6 fragment identities
  are constructed differently, per research R-6: the IPv4 key carries the
  protocol number and a sixteen bit identification, the IPv6 key carries neither
  a protocol number nor a sixteen bit field
- [X] T036 [US4] Implement `FragmentKey`, `FragmentPorts`, and `FragmentTable`
  in `crates/fragcap-core/src/parse/fragment.rs` as a fixed 256 slot array with
  a write cursor and a linear scan, per plan D-4, with no allocation on any path
- [X] T037 [US4] Implement fragment classification in
  `crates/fragcap-core/src/parse/ip.rs` per research R-7: IPv4 by the
  more-fragments flag and the offset, IPv6 by the presence of a fragment
  extension header, distinguishing initial from non-initial from not fragmented
- [X] T038 [US4] Wire the table into `HeaderParser::parse` in
  `crates/fragcap-core/src/parse/mod.rs`: record on an initial fragment that
  parsed, do not record on one that did not per FR-021a, look up on a
  non-initial one, remove on a last fragment
- [X] T039 [US4] Write the test that a non-initial fragment's direction is
  recomputed from its own addresses rather than inherited, per FR-022a, by
  replacing the address set between the first and second fragment
- [X] T040 [US4] Write the test that an orphaned non-initial fragment yields
  `UnmatchedFragment` and that a first fragment whose transport header could not
  be parsed records no identity, per FR-021a
- [X] T040a [US4] Write the test pinning FR-025, that a fragment's bytes are
  identical before and after parsing and that two fragments of one datagram
  parse to two independent outcomes with nothing joined. The property is
  structural, since `parse` takes a shared slice and returns a `Copy` outcome,
  but FR-025 is the one requirement with no other assertion behind it
- [X] T040b [US4] Write the test that an unfragmented IPv4 packet records no
  fragment table entry, per FR-021 as corrected at the analyze gate. Without
  it, ordinary traffic fills all 256 slots and evicts the entries that matter,
  and the symptom would be intermittently unmatched fragments under load

## Phase 7: User Story 5, parsing costs nothing per packet (Priority: P2)

**Goal**: Zero heap allocations per parse, asserted rather than intended.

**Independent test**: `cargo test -p fragcap-core --test no_alloc`.

- [X] T041 [US5] Write `crates/fragcap-core/tests/no_alloc.rs` with a counting
  global allocator backed by a const-initialized thread-local cell with no
  destructor, per research R-8, wrapping the system allocator
- [X] T042 [US5] Write the corpus in `crates/fragcap-core/tests/no_alloc.rs`
  covering every supported combination and every rejection cause, construct the
  parser, snapshot the counter, parse the whole corpus, and assert the delta is
  zero
- [X] T043 [US5] Remove any allocation the test finds, and record in
  `plan.md` under Complexity Tracking if any could not be removed

## Phase 8: Polish and cross-cutting concerns

- [X] T044 [P] Add glossary entries to `docs/glossary.md` for every term this
  slice introduces, per P-6 and FR-037, each following the section 4.3
  structure with primary source references. Ten terms: EtherType, extension
  header chain, IP fragment, fragment identity, fragment identity table,
  interface address set, and BSD loopback encapsulation under Capture and
  Networking; parse outcome, parse rejection cause, and parse statistics under
  Process and Attribution, beside the existing seam entries. The last four were
  missed on the first pass and caught at the analyze gate: they are public type
  names the spec introduces in its Key Entities section, so P-6 binds them the
  same as the wire format terms
- [X] T045 [P] Review every new public item for the documentation FR-038
  requires, naming the later slice for anything this slice leaves incomplete,
  specifically the loopback direction resolution that S13 finishes
- [X] T046 Write `changelog.d/S03-header-parsing-flow-keys.added.md` describing
  the parser, and
  `changelog.d/S03-header-parsing-flow-keys.decisions.md` recording the four
  deviations in plan D-10 for promotion to specification section 29
- [X] T047 Update the Current state section of `AGENTS.md` to say S03 is
  complete and that `fragcap-core` now carries behavior, and update the
  corresponding paragraph in `CLAUDE.md`
- [X] T048 Run `cargo xtask ci` in the foreground and watch it to completion,
  then `cargo xtask neutral` and `cargo xtask msrv`, recording the result of
  each including any that exits 2

## Dependencies

```text
Phase 1 Setup
   |
Phase 2 Foundational  (T003..T010; T007 -> T008 -> T009; T004,T005 -> T010)
   |
   +--> Phase 3 US1 (T011 -> T012..T014 -> T015..T019 -> T020)
   |         |
   |    Phase 4 US2 (T021 -> T022..T025 -> T026 -> T027)
   |         |
   |    Phase 5 US3 (T028..T030 -> T031..T032 -> T033)
   |         |
   |    Phase 6 US4 (T034,T035 -> T036,T037 -> T038 -> T039..T040b)
   |         |
   |    Phase 7 US5 (T041 -> T042 -> T043)
   |
Phase 8 Polish (T044..T048)
```

Phases 3 through 7 are sequential rather than independent, which is a departure
from the template's usual shape and is deliberate. Every phase after the third
edits `HeaderParser::parse`, so running them in parallel would mean three
branches editing one function. The stories remain independently *testable*,
which is the property that matters for review, but they are not independently
*implementable* here.

Phase 4 is what makes Phase 3 trustworthy: until the counter isolation harness
exists, a passing success-path test says nothing about what the parser does
with everything else.

## Parallel opportunities

- **T003 through T006** are four different files with no shared symbol.
- **T012, T013, T014** are test writing in two files against helpers that
  already exist after T011.
- **T022, T023, T024, T025** are test writing in four different locations.
- **T044 and T045** are documentation in disjoint files.

Within the implementation tasks there is essentially no parallelism, because
the layers compose into one call chain and each needs the one below it.

## Implementation strategy

Bottom up, test first, one layer at a time. Write the frame helpers, then the
link layer with its successes and its rejections, then the network layer, then
the transport layer, then the assembly in `HeaderParser::parse`. Each layer's
rejections are pinned before the layer above it is written, so that a rejection
appearing at the wrong layer is caught by the layer that should have produced
it rather than at the end.

The minimum viable increment is Phases 1 through 5. That is a parser that
handles every supported combination, declines everything else with a named
reason, and reports direction honestly. Fragments and the allocation proof are
genuinely separable and could be a second commit if the first grows too large
for one review, though the intent is one.

Phase 7 is placed last for a practical reason rather than a priority one: the
allocation test needs the full corpus, and the corpus is only complete once
every rejection cause exists.
