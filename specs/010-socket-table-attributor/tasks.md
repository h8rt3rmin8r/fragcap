# Tasks: Socket Table Attributor

**Slice**: S10

**Branch**: `feat/socket-table-attributor`

**Created**: 2026-08-09

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md),
[data-model.md](data-model.md),
[contracts/attribution-api.md](contracts/attribution-api.md),
[quickstart.md](quickstart.md)

Tests are included and are not optional. This is the first attributor in the
project that can be wrong, so the tests are the slice's actual deliverable and
the code is what makes them pass.

Three notes on the shape.

**Phase 2 changes the trait before anything new is built.** The `Sync` bound
and the removal of the pipeline's per-packet mutex touch a declared surface and
a hot path. Doing them first means the workspace compiles green before any new
behavior exists, so a later failure is attributable to the behavior rather than
to the churn. It also proves research R-5's claim that both existing
implementors are already `Sync`, at the point where the claim is cheapest to
disprove.

**Phases 3 through 5 build the immutable half before anything that owns state.**
Matching, ranking, retention lookup, and fidelity are pure functions of a
declared table and a declared instant. Every one of them is testable before a
clock, a source, or a publication cell exists, and building them first means
the attributor in phase 6 composes parts that are already proven rather than
being debugged as a whole.

**Phase 7 is the part that needs Windows.** It is last for the reason
`docs/plans/README.md` gives for placing the platform slices after the offline
pipeline: it integrates against a consumer already known to be correct. It is
also the part this slice cannot verify, and phase 9 says so rather than
implying otherwise.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel with other `[P]` tasks in the same phase
  (different files, no dependency on an incomplete task)
- **[Story]**: The user story from [spec.md](spec.md) this serves
- Every task names the file it changes

## Phase 1: Setup

- [X] T001 Add `arc-swap` 1.9 to `[workspace.dependencies]` in the root
      `Cargo.toml` and take it in `crates/fragcap-attr/Cargo.toml` as a
      non-optional runtime dependency (research R-3).
- [X] T002 Add `windows-sys` 0.36 to `[workspace.dependencies]` with the four
      features research R-4 names, and take it in
      `crates/fragcap-attr/Cargo.toml` as optional and target-gated to Windows.
      Declare the feature as `socket-table` with `default = []`, and pass it
      through from `crates/fragcap/Cargo.toml` as its own feature rather than
      by widening `live`.

      **Not named `live`.** The analyze gate caught the collision. The facade
      already declares `live = ["fragcap-capture/live"]`, meaning "links
      against the npcap import library". This backend needs no npcap: it reads
      Windows socket tables through the IP Helper API, which is present on
      every Windows machine. Folding it into `live` would make a build that
      wanted attribution also require a capture driver software development
      kit, and would make the workflow that builds this backend fail for a
      reason that has nothing to do with it. S09's own rule applies and gives
      the answer: a feature is named for the capability it gates, and these are
      two capabilities.
- [X] T003 [P] Create the module files `seam.rs`, `table.rs`, `index.rs`,
      `schedule.rs`, and `socket.rs` under `crates/fragcap-attr/src/`, declare
      them in `crates/fragcap-attr/src/lib.rs`, and update that file's module
      narrative, which currently says the socket table attributor arrives in
      S10.
- [X] T004 Verify the dependency additions leave `cargo xtask deps` and
      `cargo xtask license` passing, with `fragcap-core` taking no platform
      dependency and no edge from `fragcap-attr` to any sibling crate (FR-037,
      FR-038, SC-011), and record what `Cargo.lock` gained. Research R-4
      predicts `windows-sys` adds nothing and `arc-swap` adds one package; a
      different result means the version pin is wrong.

## Phase 2: The trait bound and the pipeline mutex

- [X] T005 Change `FlowAttributor: Send` to `FlowAttributor: Send + Sync` in
      `crates/fragcap-core/src/traits.rs`, with the doc comment recording it as
      a deviation in the same form the `PacketSource` `Send` bound uses.
- [X] T006 Extend the cross-thread trait test in
      `crates/fragcap-core/src/traits.rs` to assert `FlowAttributor` is `Sync`
      behind a pointer, alongside the existing `Send` assertions.
- [X] T007 [US4] Replace `Arc<Mutex<Box<dyn FlowAttributor>>>` with
      `Arc<dyn FlowAttributor>` in `crates/fragcap-core/src/pipeline/mod.rs`,
      remove the per-packet lock in the capture loop, and update the
      `Pipeline::new` doc comment that explains why the attributor is owned by
      the acquisition side. `Pipeline::new` keeps its `Box` parameter.
- [X] T008 [US4] Add a test in `crates/fragcap-core/src/pipeline/mod.rs`
      asserting the pipeline drives several capture threads against one shared
      attributor and that the conservation identity still holds (SC-007).
- [X] T009 Run `cargo test --workspace --locked` and confirm the workspace is
      green before any new behavior exists. A failure here is the `Sync` bound
      finding an implementor that is not, which research R-5 predicts does not
      exist.

## Phase 3: The immutable half, tables and entries (US1)

- [X] T010 [P] [US1] Define `SocketTableEntry` and `SocketTable` in
      `crates/fragcap-attr/src/table.rs` per
      [contracts/attribution-api.md](contracts/attribution-api.md) section 3,
      with `remote` as an `Option` and no constructor that accepts a remote for
      a UDP entry.
- [X] T011 [P] [US1] Tests in `table.rs`: a declared table round-trips, a UDP
      entry cannot carry a remote, and `taken_at` is preserved (FR-001 through
      FR-004).

## Phase 4: The immutable half, matching and the order (US1, US2)

- [X] T012 [US1] Implement `MatchRank` and candidate ranking in
      `crates/fragcap-attr/src/index.rs`: the four exactness ranks, tested in
      order so each is mutually exclusive by construction (FR-005 through
      FR-008).
- [X] T013 [US1] Tests (SC-001): TCP matches on both endpoints; UDP matches on the local
      endpoint alone and never against a remote; a UDP wildcard bind matches a
      specific local address on the same port; a TCP entry does not take the
      wildcard allowance (FR-005, FR-006, FR-007).
- [X] T014 [US1] Implement dual-stack matching: an IPv6 unspecified bind
      matches an IPv4 local endpoint on the same port, for UDP only (FR-007),
      and a test that a specific bind beats the dual-stack bind.
- [X] T015 [US1] Implement the creation-instant filter: an entry whose
      `created` is later than the packet's instant is not a candidate, for both
      protocols, and an entry with no `created` is still a candidate (FR-009,
      SC-004).
- [X] T016 [US1] Implement the within-rank order: latest `created` at or before
      the packet's instant first, `None` last, then ascending `pid` as the
      total tiebreak, with a comment saying the last rule is arbitrary and
      exists only to make the order total (FR-008a, FR-008b).
- [X] T017 [US1] Test that permuting the entries of a declared table changes no
      answer, for a table containing every rank and a within-rank tie
      (SC-014). This is the test a first-hit matcher fails.
- [X] T018 [US1] Test that the packet's instant and not the present moment
      selects the answer, driven through `resolve` rather than an inherent
      method, mirroring the property S04 established for the scripted
      attributor (FR-010).

## Phase 5: Retention, fidelity, and the index (US2)

- [X] T019 [US2] Define `RetentionMap` and `RetainedEntry` in `index.rs`,
      keyed on `Endpoint`, carrying `last_seen` (FR-018, FR-018a).
- [X] T020 [US2] Define `AttributionIndex` holding the table, the names, and
      the retention map, with `resolve`, `endpoints`, and `carries` per
      contract section 4.
- [X] T021 [US2] Implement fidelity: an answer from the current table carries
      `Live`, an answer from the retention map carries `Retained`, and the
      value is supplied at each construction site rather than derived
      (FR-019).
- [X] T022 [US2] Implement live-beats-retained: when both match one endpoint,
      the live entry wins and the answer carries `Live` (FR-020).
- [X] T023 [US2] Implement expiry against the packet's instant, measured from
      `last_seen` (FR-021), and a test that a retention of zero makes an
      absent endpoint immediately unresolvable.
- [X] T024 [US2] Test the whole life of one endpoint in a single test: present
      and `Live`, absent and `Retained`, expired and unresolved (SC-002). Three
      separate tests can each pass while the transitions are wrong.
- [X] T025 [US2] Test port reuse: an endpoint whose owner changed inside the
      window resolves to the new owner (SC-003).
- [X] T026 [US2] Implement `endpoints` reporting current plus retained
      (FR-023), and `carries` for the FR-014 trigger.
- [X] T027 [US2] Implement name attachment: `AttributionIndex::resolve` reads
      the image name from the names map, and produces an attribution carrying
      the observed identifier when no name is present (FR-032), with `role` and
      `stage` absent.

## Phase 6: The schedule (US3)

- [X] T028 [P] [US3] Define `Clock` and `SystemClock` in
      `crates/fragcap-attr/src/seam.rs`, plus `TestClock` with `at`, `set`, and
      `advance`. `set` and `advance` take `&self` over an atomic, because the
      clock is held as `Arc<dyn Clock>` and a test that cannot advance it
      through a shared handle cannot drive the cadence at all.
- [X] T029 [US3] Define `RefreshSchedule` in
      `crates/fragcap-attr/src/schedule.rs` over atomics, per contract section
      7, `Send + Sync`.
- [X] T030 [US3] Implement `is_due` against the interval (FR-011, FR-012) and
      `mark_refreshed`.
- [X] T031 [US3] Implement `request_triggered` with the rate limit, returning
      whether the request was recorded (FR-014, FR-015).
- [X] T032 [US3] Implement `request_immediate`, which ignores the rate limit
      (FR-013, FR-016), and `take_request`.
- [X] T033 [US3] Test the whole cadence against `TestClock`: the interval
      elapsing, a triggered request, a second request inside 200 ms refused, a
      third after 200 ms accepted, and an immediate request accepted regardless
      (SC-005). No test sleeps.
- [X] T034 [US3] Test that a triggered request arriving just before the
      interval elapses does not produce two refreshes (checklist CHK024).

## Phase 7: Publication (US4)

- [X] T035 [US4] Define `PublishedIndex` over `ArcSwap<AttributionIndex>` in
      `index.rs`, with `load` and `publish`, `Send + Sync` (FR-027, FR-028).
- [X] T036 [US4] Test concurrent resolution across a publication: several
      reader threads resolving while a publisher alternates between two indices
      whose answers are distinct, asserting every observed answer is one of the
      two and never a mixture, for a bounded iteration count (FR-027, SC-006).
- [X] T037 [US4] Test that `load` returns a handle that stays valid across a
      subsequent `publish`, so a reader mid-lookup is unaffected.

## Phase 8: The attributor (US1, US2, US3, US4)

- [X] T038 Define `SocketTableSource` and `ProcessNamer` in `seam.rs` per
      contract section 6, plus `DeclaredTable` and `DeclaredNames`, both public
      and both able to be scripted to fail or to return nothing.
- [X] T039 Define `AttributorConfig` with the three defaults, in
      `crates/fragcap-attr/src/socket.rs` (FR-011, FR-011a, FR-015, FR-018).
- [X] T040 Define `SocketTableAttributor` per contract section 5, taking every
      seam explicitly with no defaults, and exposing `published`, `schedule`,
      and `config`.
- [X] T041 Implement `refresh`: read a table, age the retention map against it
      measuring from `last_seen`, discard entries past the grace period
      (FR-022), resolve names for the identifiers the table reported (FR-033a),
      build an index, publish it, and mark the schedule refreshed.
- [X] T042 Implement `refresh`'s failure path: a read error leaves the
      published index in place and returns the error (FR-030), and a test that
      resolution after a failed refresh answers exactly as it did before it
      (SC-008).
- [X] T043 Test the first-refresh failure case, where there is no previous
      index to keep (checklist CHK021), and that resolution against an
      unpublished attributor is unresolved rather than a panic.
- [X] T044 Implement `resolve`: load the published index, match, and on an
      unresolved lookup against an endpoint the index does not carry, read the
      injected clock and record a triggered request against it (FR-014,
      FR-015). The clock read happens only on that path; a resolved lookup
      touches nothing but the index. Assert that no lookup reads the socket
      table, enumerates a process, or opens a handle (FR-017, SC-015).

      **Why the clock and not the packet's instant.** The analyze gate found
      this unspecified. The rate limit bounds how often fragcap reads the
      platform's table, which is a wall-clock cost, so it must be measured in
      wall-clock time; measuring it against packet instants would make a replay
      of an hour of traffic in one second request three hundred refreshes, and
      a capture of a quiet interface request none. The clock is injected, so
      this costs no determinism in a test. It does mean `resolve` is not a pure
      function of the index and the packet, and that is the honest reading of
      section 11.2, which describes the trigger as rate limited in
      milliseconds rather than in capture time.
- [X] T045 Implement `active_endpoints` delegating to the index, supplying the
      injected clock's instant because the trait method carries none (FR-023).
      Document that choice at the call site: "currently active" is a question
      about now, unlike `resolve`, which is always a question about then.
- [X] T046 Test that an unresolved lookup returns `None` and is not an error,
      and that a packet with no flow key never reaches the attributor so the
      never-attempted distinction is preserved (FR-024, FR-025, FR-026,
      SC-009).
- [X] T047 Test the two-refresh sequence end to end through the trait: refresh,
      resolve `Live`, advance, refresh with the endpoint gone, resolve
      `Retained`, advance past the window, resolve unresolved.

## Phase 9: The platform backend

- [X] T048 [P] Create `crates/fragcap-attr/src/platform/mod.rs` gated on the
      `live` feature and `target_os = "windows"`, absent otherwise (FR-035,
      FR-036).
- [X] T049 Implement `IpHelperTable` in `platform/iphelper.rs`:
      `GetExtendedTcpTable` with `TCP_TABLE_OWNER_MODULE_ALL` and
      `GetExtendedUdpTable` with `UDP_TABLE_OWNER_MODULE`, for both address
      families, with the two-call size negotiation and a retry bound
      (FR-034, research R-1).
- [X] T050 Convert `liCreateTimestamp`, which is a FILETIME, into the
      project's `Timestamp`, and test the conversion against known values. A
      wrong epoch here produces plausible instants that are wrong, which is the
      P-9 failure no test over synthetic data catches.
- [X] T051 Implement `ToolhelpNamer` in `platform/toolhelp.rs` over
      `CreateToolhelp32Snapshot`, `Process32FirstW`, and `Process32NextW`,
      opening no process handle against any target (FR-033, research R-2).
- [X] T052 Mark the backend's behavioral tests `#[ignore]` so they are
      requested explicitly rather than failing on a machine without the
      platform (SC-010).
- [X] T053 Add an assertion to `cargo xtask lint` that no fragcap source names
      `OpenProcess`, in the same shape as the existing transmit-call assertion,
      so the P-1 argument for this slice is mechanical rather than remembered
      (SC-012).
- [X] T054 Extend `cargo xtask neutral` to build `fragcap-attr` alongside
      `fragcap-core` and `fragcap-capture`, for the reason S09 extended it
      (checklist CHK042).
- [X] T054a Add `crates/fragcap-attr/**` to the path filters in
      `.github/workflows/platform.yml`, and add a build step
      `cargo build -p fragcap-attr --features socket-table --locked`.

      **Found by the analyze gate.** As written, that workflow's filters name
      only `fragcap-capture/**` and `fragcap-core/**`, and its only build step
      is the capture crate, so nothing anywhere would ever compile this
      backend. The step is placed **before** the npcap acquisition and is not
      gated on the driver being present, because this backend links against no
      npcap import library and needs no software development kit. That makes it
      the first thing in that workflow which can go green on a bare Windows
      runner, and the first Windows compile check the project has that does not
      depend on an external download succeeding.

      `.github/workflows/**` is a pinned artifact, so this needs a dated
      decision recorded in `changelog.d`. T060 carries it.

## Phase 10: The facade, the pipeline, and the corpus

- [X] T055 Add `crates/fragcap/tests/attribution.rs`: a pipeline built from a
      replay source and a `SocketTableAttributor` over a declared table,
      resolving attributions end to end with no capture driver (US1, SC-007).
- [X] T056 Assert in that test that the conservation identity holds and that
      the attributed and unattributed counters partition the packets that
      carried a flow key.
- [X] T057 Run the corpus tests and confirm the committed goldens are
      unchanged, because nothing here alters what the scripted attributor
      answers (SC-013).

## Phase 11: Documentation and integration

- [X] T058 [P] Add six glossary entries to `docs/glossary.md`: socket table,
      socket table entry, attribution index, retention window, refresh trigger,
      and dual-stack socket, following the section 4.3 template with
      primary-source references (FR-039, P-6).
- [X] T059 [P] Write `changelog.d/S10-socket-table-attributor.added.md`
      describing what the slice adds.
- [X] T060 [P] Write `changelog.d/S10-socket-table-attributor.decisions.md`
      recording the four deviations, the two dependency additions, the feature
      naming decision, and the `.github/workflows/platform.yml` change, dated.
- [X] T061 Update the `## Current state` section of `AGENTS.md`: S10 complete,
      the dependency inventory gains two rows with their justifications, the
      pipeline no longer locks per packet, and the Appendix D correction about
      the UDP creation timestamp.
- [X] T062 Update `crates/fragcap-attr/README.md` and the crate's module
      narrative to describe what now exists.
- [X] T063 Resolve `checklists/attribution.md` item by item, recording the
      answer for any item whose resolution is not obvious from the diff, in the
      form S09's resolution pass used.
- [X] T064 Run `cargo xtask ci` in the foreground and watch it to completion.
      Record the result.

## Dependencies

```text
Phase 1 ─▶ Phase 2 ─▶ Phase 3 ─▶ Phase 4 ─▶ Phase 5 ─┐
                          │                          ├─▶ Phase 8 ─▶ Phase 10 ─▶ Phase 11
                          └─▶ Phase 6 ───────────────┤
                                  Phase 7 ───────────┘
                                                Phase 9 (independent after Phase 3)
```

Phase 5 depends on phase 4 because retention lookup reuses the matcher. Phases
6 and 7 depend only on phase 3 and 5's type definitions respectively and can
proceed alongside phase 4. Phase 9 needs only the seam definitions from phase 8
task T038 and the table type from phase 3, and is independent of everything
else; it is placed late because it is the least verifiable, not because it is
blocked.

## What "done" requires

Every task checked, `cargo xtask ci` green in the foreground with its output
read, the checklist resolved, and the pre-push breakdown presented. The platform
backend compiling under its feature is not the same as it working, and the
slice report must say which of the two has been demonstrated.
