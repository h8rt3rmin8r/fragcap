# Tasks: Replay Source and Fixture Corpus

**Slice**: S04

**Branch**: `feat/replay-source-fixtures`

**Created**: 2026-08-08

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md),
[data-model.md](data-model.md),
[contracts/replay-api.md](contracts/replay-api.md),
[quickstart.md](quickstart.md)

Tests are included and are not optional. This slice's whole output is test
infrastructure, so a substrate whose own correctness is unasserted would put
every later slice's tests on sand.

Two notes on the shape.

The three tracks in Phases 3, 4, and 5 are genuinely independent. The reader
touches only `fragcap-capture/src/pcap.rs`, the script and attributor only
`fragcap-attr`, and the corpus only a test target. They can be built in any
order, and the phase numbering is priority rather than dependency.

Phase 6 is where they meet, and it is the only phase that proves the slice's
actual claim. Nothing before it demonstrates that the pipeline runs without a
capture driver; it demonstrates that three parts each work alone.

## Phase 1: Setup

- [X] T001 Verify the repository's ignore rules admit the corpus before writing
  any of it: confirm `.gitignore` re-includes `fixtures/**/*.pcap` by creating a
  throwaway file there and checking `git status` sees it. The Assumptions
  section requires this be checked rather than trusted, and discovering it after
  generating eight fixtures would be an unpleasant surprise
- [X] T002 Create `fixtures/` with a `README.md` stating what the corpus is,
  that it is generated rather than hand-made, how to regenerate it, and that a
  regenerated diff must be read

## Phase 2: Foundational

Leaf types the tracks need. No behavior yet.

- [X] T003 [P] Define `ReplayStats` in `crates/fragcap-capture/src/pcap.rs`
  with the four counters from `data-model.md` and a `skipped()` method summing
  only the two that stop delivery, with a doc comment on each counter saying
  whether the record is still delivered and why
- [X] T004 [P] Define `ScriptError` in `crates/fragcap-attr/src/script.rs` with
  a named variant per load failure, each carrying the line number, since a
  script is hand-authored as often as generated
- [X] T005 Declare the module set and re-exports in
  `crates/fragcap-capture/src/lib.rs` and `crates/fragcap-attr/src/lib.rs`,
  replacing the "skeleton only" notes both currently carry

## Phase 3: The reader (Priority: P1)

**Goal**: Bytes in, packets out, with every skip counted and nothing altered.

**Independent test**: `cargo test -p fragcap-capture pcap`, against byte arrays
built in the test rather than files on disk.

- [X] T006 [US1] Write the pcap-building test helper in
  `crates/fragcap-capture/src/pcap.rs` under `#[cfg(test)]`: a file header with
  settable magic, snaplen, and link type, and a record with settable timestamp,
  captured length, on-wire length, and payload. Settable independently, so a
  test can build a record whose fields contradict each other
- [X] T007 [P] [US1] Write the failing tests for the four magic numbers,
  asserting byte order and timestamp unit come from the file and never from the
  host, per research R-2
- [X] T008 [P] [US1] Write the failing tests for timestamp conversion: a
  microsecond file and a nanosecond file whose fractions differ by a thousand
  must yield the same instant, and a sub-microsecond value must survive, per
  research R-3
- [X] T009 [P] [US2] Write the failing determinism tests: the same bytes read
  twice yield identical sequences, and the same capture written in all four
  magic variants yields one sequence
- [X] T010 [P] [US3] Write the failing tests for each of the four skip causes,
  each asserting exactly one counter moved and asserting whether the record was
  delivered, per the table in `data-model.md`
- [X] T011 [P] [US3] Write the failing tests for what must not be skipped: a
  zero-length record, an out-of-order timestamp, and a link type fragcap cannot
  parse all arrive
- [X] T011a [P] [US1] Write the failing fidelity test for FR-006: a record with
  a distinctive payload arrives byte-identical, and its original on-wire length
  arrives as recorded rather than as the captured length. Added at the analyze
  gate, which found FR-006 asserted only indirectly through a length check that
  never looked at the bytes
- [X] T012 [US1] Implement `PcapReader` in
  `crates/fragcap-capture/src/pcap.rs`: parse the file header, dispatch on the
  magic, and decode records in the five-stage order `data-model.md` fixes
- [X] T013 [US3] Implement the skip accounting, with the two stop-reading causes
  distinguished from the two deliver-anyway causes, and a comment at the
  `caplen_exceeds_wire` site recording that reconciling the two lengths is the
  alteration P-9 forbids
- [X] T014 [US1] Write the failing tests for opening: a file shorter than a
  header, and a file with an unrecognized magic, each a terminal error rather
  than an empty sequence
- [X] T015 [US1] Implement `ReplaySource` in
  `crates/fragcap-capture/src/replay.rs` over the reader, with `open` and
  `from_bytes`
- [X] T016 [US1] Write and satisfy the seam tests: exhaustion is
  `Err(Closed)` and never `Ok(None)`; `set_filter` succeeds and changes nothing
  delivered; `link_type` is the file's; `received` is what was delivered and
  both backend drop counts are zero, per FR-016a

## Phase 4: The script and the attributor (Priority: P1)

**Goal**: Predetermined answers, including different answers at different
times, without being able to express an answer the real attributor could not
give.

**Independent test**: `cargo test -p fragcap-attr`, against strings rather than
files.

- [X] T017 [US4] Write the failing parser tests for the three statements in
  plan D-8, including comments, blank lines, and `always` versus an explicit
  window
- [X] T018 [US4] Write the failing tests for the two combinations that must not
  load: a UDP entry naming a remote endpoint, and a TCP entry with `*`. These
  are what stop the double expressing an attribution S10 could never make
- [X] T019 [US4] Write the failing overlap tests: intersecting windows for one
  flow fail; abutting half-open windows load; `always` alongside any other
  window for the same flow fails
- [X] T020 [US4] Implement `AttributionScript` and its parser in
  `crates/fragcap-attr/src/script.rs`, with every error naming its line
- [X] T021 [US4] Write the failing resolution tests: an owner returned within
  its window, nothing outside it, nothing for an unmentioned flow, and nothing
  for an `unowned` entry
- [X] T022 [US4] Write the failing port reuse test, one local endpoint resolving
  to two different owners in two windows. This is the reason the time dimension
  exists and is worth writing before the attributor
- [X] T023 [US4] Write the failing wildcard bind test, a UDP entry bound to a
  wildcard address matching a datagram on a specific interface address, so the
  double agrees with the rule S10 must implement
- [X] T023a [US4] Write the failing tests for the `endpoint` statement: it
  parses, a malformed address names its line, and the declared endpoints are
  what `active_endpoints` reports. Caught at the analyze gate: FR-023 is the
  one requirement in this slice that had no task, and the trait method would
  have surfaced as a compile error with nothing specifying its behavior
- [X] T024 [US4] Implement `ScriptedAttributor` in
  `crates/fragcap-attr/src/scripted.rs`, resolving through
  `FlowKey::attribution_key` and `AttributionKey::local_matches_bind` rather
  than a parallel comparison, per plan D-6, and reporting the script's declared
  endpoints from `active_endpoints`
- [X] T025 [US4] Implement `set_now` as an inherent method and write the test
  asserting the `FlowAttributor` trait is still usable as a trait object with no
  timestamp parameter, pinning SC-006b so a later widening is noticed
- [X] T025a [US4] Add an `endpoint` line to each generated script in T028 for
  the flows that fixture carries, so the statement is exercised by the corpus
  rather than only by a unit test

## Phase 5: The corpus (Priority: P2)

**Goal**: Eight fixtures that are what they say they are, and stay that way.

**Independent test**: `cargo test -p fragcap-capture --test corpus`.

- [X] T026 [US5] Write the pcap writer in
  `crates/fragcap-capture/tests/corpus.rs`, the inverse of the reader, and the
  frame builders the fixtures need. Deterministic by construction: a constant
  timestamp base, constant addresses, and the filler payload pattern, per plan
  D-10 and FR-032a
- [X] T027 [US5] Generate the eight fixtures section 25.3 names, each from a
  readable description in the generator, each with the condition it exists to
  exercise, per research R-7
- [X] T028 [US5] Generate the eight attribution scripts, one per fixture, with
  comments annotating what each absolute timestamp means. `port-reuse.script`
  carries the two-window case
- [X] T029 [US5] Implement the drift check: regenerate into memory, compare
  against what is committed, and fail naming the file that differs. Cover
  scripts as well as captures, per FR-033
- [X] T030 [US5] Implement the regeneration path behind
  `FRAGCAP_UPDATE_FIXTURES`, in the same target so generation and checking
  cannot disagree about the format
- [X] T031 [US5] Write the pairing check: a capture with no script and a script
  with no capture are both reported
- [X] T032 [US5] Write the size check: 64 KiB per fixture, 256 KiB for the
  directory
- [X] T033 [US6] Write the privacy check over every packet of every fixture:
  every address from a documentation range or loopback, every link layer address
  from the stated locally-administered pair, every payload byte the filler
  pattern, per FR-029 and FR-029a
- [X] T034 [US6] Write one condition assertion per fixture, from the table in
  research R-7, so a generator change that drops a fragment or flattens a chain
  fails here rather than in S08
- [X] T035 [US5] Run the regeneration, commit the corpus, and confirm the check
  passes against the committed bytes on a clean tree

## Phase 6: The claim (Priority: P1)

**Goal**: Demonstrate what the slice is actually for.

- [X] T036 Write the end-to-end test in
  `crates/fragcap-capture/tests/corpus.rs` that opens a fixture, parses every
  packet with the S03 parser, sets the attributor's clock from each packet's
  timestamp, resolves each flow against the fixture's script, and asserts the
  attributed and unattributed counts. This is SC-001, and the first time S02,
  S03, and S04 are exercised together
- [X] T037 Write the same test against `port-reuse.pcap`, asserting the owner
  changes partway through the capture, which no other test in the project can
  currently express

## Phase 7: Polish and cross-cutting concerns

- [X] T038 [P] Add glossary entries to `docs/glossary.md` for every term this
  slice introduces, per P-6 and FR-036: pcap distinguished from pcapng, replay
  source, scripted attributor, attribution script, fixture, fixture corpus, and
  test tier. Under Capture and Networking, Process and Attribution, and Rust and
  Tooling as each fits
- [X] T039 [P] Review every new public item for the documentation FR-037
  requires, naming the later slice for anything left incomplete: filtering for
  S13, golden comparison for S08, and the two fixtures whose consumers are S08
  and S10
- [X] T040 Write `changelog.d/S04-replay-source-fixtures.added.md`, and
  `changelog.d/S04-replay-source-fixtures.decisions.md` recording the two
  deviations in plan D-11 for promotion to specification section 29
- [X] T041 Update the Current state section of `AGENTS.md` and the
  corresponding paragraph of `CLAUDE.md`: S04 is complete, the tier 1 substrate
  exists, and the section 25.1 claim is now demonstrated rather than asserted
- [X] T041a Add `*.script text eol=lf` to `.gitattributes`, beside the formats
  whose parsing depends on line endings. The wildcard rule already covers it;
  listing it matches the file's own stated convention and keeps the corpus
  drift check from depending on autodetection
- [X] T042 Run `cargo xtask ci` in the foreground and watch it to completion,
  then `cargo xtask neutral` and `cargo xtask msrv`, recording each result
  including any that exits 2. `cargo xtask deps` is the evidence for FR-018 and
  FR-026: it rejects any edge between `fragcap-capture` and `fragcap-attr`, so
  neither can reach into the other's concern even by accident. Record that in
  the pull request rather than leaving the two requirements looking unverified

## Dependencies

```text
Phase 1 Setup (T001 blocks everything that writes a fixture)
   |
Phase 2 Foundational (T003, T004 -> T005)
   |
   +--> Phase 3 reader   (T006 -> T007..T011 -> T012..T013 -> T014..T016)
   +--> Phase 4 attr     (T017..T019 -> T020 -> T021..T023 -> T024..T025)
   +--> Phase 5 corpus   (T026 -> T027..T028 -> T029..T034 -> T035)
   |         (T026 needs T012, to write files the reader can read back)
   |
Phase 6 the claim (T036, T037; needs all three tracks)
   |
Phase 7 Polish (T038..T042)
```

T001 comes first and is not ceremonial. If the ignore rules do not admit
`fixtures/`, everything in Phase 5 is wasted work discovered at `git add`.

## Parallel opportunities

- **T003 and T004** are different crates.
- **T007 through T011** are test writing against a helper that already exists.
- **Phases 3, 4, and 5** touch disjoint files and could be three sessions.
- **T038 and T039** are documentation in disjoint files.

Within each track the implementation tasks are sequential, because each layer
composes into the one above it.

## Implementation strategy

Reader first, because the corpus generator needs something to verify its output
against and writing both from the same misunderstanding of the format is the
easiest way to produce a self-consistent corpus that nothing else can read. The
reader is tested against hand-built byte arrays for exactly that reason: they
are written from the format description in research R-1, not from the
generator.

The minimum viable increment is Phases 1 through 4 plus 6, which is a working
substrate over one hand-built fixture. The corpus is what makes it useful to
other slices, and is P2 only because a substrate with one fixture is still a
substrate.

Phase 6 is small and is the point. If it is awkward to write, the seams are
wrong and that is worth finding out in this slice rather than in S08.
