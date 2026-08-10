# Tasks: Live Capture Source and Interfaces

**Slice**: S09

**Branch**: `feat/live-capture-interfaces`

**Created**: 2026-08-09

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md),
[data-model.md](data-model.md),
[contracts/capture-api.md](contracts/capture-api.md),
[quickstart.md](quickstart.md)

Tests are included and are not optional. Most of this slice cannot be exercised
in continuous integration at all, so the parts that can be must be, and the
parts that cannot must be honest about it rather than quietly untested.

Four notes on the shape, because the phase order does not follow the priority
order and the reason is worth stating once.

**Phase 2 changes three types before anything new is built.** The `Send` bound,
the interface identifier on `CapturedPacket`, and per-interface source
statistics all ripple through the workspace, and two of them change a signature
every test calls. Doing them in one sweep means the workspace compiles again
before any new behavior is added, so a later failure is attributable to the
behavior rather than to the churn.

**Phase 3 is user story 3 despite it being P2.** Selection is a pure decision
over a value, both P1 stories consume it, and it is the only part of this slice
that can be exhaustively tested on any machine. Building it first means the two
stories above it are written against something already proven rather than
alongside it.

**Phase 4 delivers the multi-interface capture without a capture driver.**
Two replay sources standing in for two interfaces exercise the identifier, the
per-thread parser, the retirement path, both writers, and the conservation
identity. Everything the project can verify about multi-interface capture is
verified here, on any machine.

**Phase 5 is the part that needs Windows, npcap, and elevation.** It is last
because it is the least verifiable, and putting it last means it integrates
against a consumer already known to be correct rather than being debugged
alongside one. That is the same reasoning `docs/plans/README.md` gives for
placing S09 after the offline pipeline in the first place.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel with other `[P]` tasks in the same phase
  (different files, no dependency on an incomplete task)
- **[Story]**: The user story from [spec.md](spec.md) this serves
- Every task names the file it changes

## Phase 1: Setup

- [X] T001 Add `pcap` 2.4 as an optional dependency and declare the `live`
      feature in `crates/fragcap-capture/Cargo.toml`, with `default = []`, and
      pass the feature through from `crates/fragcap/Cargo.toml`. Do not enable
      it anywhere by default (plan D-2).
- [X] T002 [P] Create `crates/fragcap-core/src/interface.rs` and declare the
      module in `crates/fragcap-core/src/lib.rs`.
- [X] T003 [P] Create the live backend skeleton under
      `crates/fragcap-capture/src/live/` (`mod.rs`, `enumerate.rs`,
      `driver.rs`, `route.rs`), gated
      `#[cfg(all(windows, feature = "live"))]`, and declare it in
      `crates/fragcap-capture/src/lib.rs`.
- [X] T004 [P] Add the six glossary entries named in plan D-11 to
      `docs/glossary.md` following the specification section 4.3 template:
      bootstrap filter, interface identifier, interface inventory, selection
      outcome, virtual interface, interface retirement. Constitution P-6
      requires these in the same change that introduces the terms.

## Phase 2: Foundational (blocking every story)

**Purpose**: land the three recorded deviations and the interface vocabulary,
and get the workspace compiling again before any new behavior exists.

- [X] T005 Define `InterfaceId`, `InterfaceRecord`, and `InterfaceInventory` in
      `crates/fragcap-core/src/interface.rs` per
      [data-model.md](data-model.md). No `is_virtual` field on the record; the
      verdict is carried separately.
- [X] T006 Define `SelectionSettings`, `VirtualVerdict`, `SelectionReason`,
      `ExclusionReason`, `SelectedInterface`, `SelectionOutcome`, and
      `SelectionError` in `crates/fragcap-core/src/interface.rs`.
      `ExclusionReason` is a closed enumeration, following `ParseReject`'s
      discipline, so a new exclusion path cannot be added without naming
      itself.
- [X] T007 [P] Define `InterfaceRetirement` and `RetirementReason` in
      `crates/fragcap-core/src/interface.rs`, documenting on the type that a
      retirement advances no drop counter and why (FR-028).
- [X] T008 [P] Define `DriverReport` in
      `crates/fragcap-core/src/interface.rs`, with the two installation-option
      fields as `Option<bool>` so that "could not determine" is distinct from
      "absent" (FR-045).
- [X] T009 Add the `Send` bound to `PacketSource` in
      `crates/fragcap-core/src/traits.rs`, with a doc comment recording the
      deviation and the specification section it diverges from (FR-023).
- [X] T010 Add a compile-time assertion that `dyn PacketSource` is `Send`,
      beside the existing dyn-compatibility test in
      `crates/fragcap-core/src/traits.rs`, so an implementor that stops being
      `Send` fails at the trait rather than in the pipeline.
- [X] T011 Add the non-optional `interface: InterfaceId` field to
      `CapturedPacket` and the corresponding parameter to
      `CapturedPacket::from_raw` in `crates/fragcap-core/src/packet.rs`. No
      default value (data-model.md states why). Leave `RawPacket` unchanged.
- [X] T012 Update every `CapturedPacket::from_raw` call site in the workspace.
      The compiler enumerates them; work through the list rather than
      guessing at it.
- [X] T013 Replace `CaptureStats::source` with
      `sources: Vec<(InterfaceId, SourceStats)>` and add the computed
      `source()` and `source_for()` methods in
      `crates/fragcap-core/src/stats.rs`, widening each `u32` to `u64` before
      summing. Keep the module's rule that no aggregate is stored.
- [X] T014 Update the pcapng writer under `crates/fragcap-sink/src/pcapng/`,
      `crates/fragcap-sink/src/json.rs`, and the corpus helper in
      `crates/fragcap/tests/` to read `source()` rather than the removed field.
- [X] T015 [P] Add unit tests in `crates/fragcap-core/src/stats.rs` asserting
      that `source()` equals the sum of its parts, that `source_for()` isolates
      one interface, that changing one interface's counter changes the total,
      and that no driver-reported count reaches a fragcap counter or vice versa
      (SC-007). This is the per-interface form of the existing "changing one
      cause changes the total" test.
- [X] T016 Run `cargo test --workspace` and confirm the workspace compiles and
      the existing suite passes before any new behavior is added. A red result
      here is churn, not a defect in this slice's new code, and is much cheaper
      to find now.

## Phase 3: User Story 3, interface selection (P2, built first)

**Goal**: the section 12.1 precedence, as a pure decision over an inventory.

**Independent test**: `cargo test -p fragcap-core --lib interface` on any
machine, with no capture driver and no network.

- [X] T017 [US3] Write the selection matrix test first, in
      `crates/fragcap-core/src/interface.rs`, covering explicitly named,
      default route, loopback present, loopback absent, virtual, down, no
      address, and broad capture. It must fail before T020 exists (SC-002).
- [X] T018 [P] [US3] Write the accounting invariant test in
      `crates/fragcap-core/src/interface.rs`: for every case,
      `selected.len() + excluded.len()` equals the inventory length. This is
      the selection-side analogue of the conservation identity, and it fails
      loudly if a future rule drops an interface on the floor (SC-003).
- [X] T019 [P] [US3] Add the virtual-interface pattern list as data in one
      place in `crates/fragcap-core/src/interface.rs`, with a comment stating
      that it is a heuristic and that its verdict is only ever used to exclude
      from automatic selection (plan D-9, FR-004).
- [X] T020 [US3] Implement `select` precedence step one, explicitly named
      interfaces, in `crates/fragcap-core/src/interface.rs`, including that an
      explicit name overrides the virtual verdict (FR-006).
- [X] T021 [US3] Implement precedence step two, the default-route interface
      plus the loopback adapter when requested, matching the inventory's
      `default_route_source` against interface addresses (FR-005).
- [X] T022 [US3] Implement precedence step three, broad capture over every
      interface that is up, addressed, and not virtual (FR-005, FR-008).
- [X] T023 [US3] Implement `SelectionError::UnknownInterface` carrying the
      available names, and `SelectionError::NothingSelected` (FR-007, FR-011).
      Both are returned; neither fails a run from inside a pure function.
- [X] T024 [US3] Record an `ExclusionReason` for every enumerated interface not
      selected, and assert in a test that no interface reaches the outcome
      without one (FR-009).

## Phase 4: User Story 2, several interfaces told apart (P1)

**Goal**: identity from acquisition through to both writers, with several
capture threads, and no capture driver anywhere in the test.

**Independent test**: `cargo test -p fragcap --test multi_interface`.

- [X] T025 [US2] Add `SourceBinding` and change `Pipeline::new` to take
      `Vec<SourceBinding>` in `crates/fragcap-core/src/pipeline/mod.rs`, per
      [contracts/capture-api.md](contracts/capture-api.md). Do not add a
      multiplexing source (FR-024).
- [X] T026 [US2] Spawn one capture thread per source in
      `crates/fragcap-core/src/pipeline/mod.rs`, all delivering into the single
      existing bounded buffer with its drop-oldest semantics unchanged
      (FR-022, FR-025).
- [X] T027 [US2] Give each capture thread its own parser, reading its link type
      from its own source, in `crates/fragcap-core/src/pipeline/mod.rs`
      (FR-026).
- [X] T028 [US2] Implement interface retirement: a failed capture thread
      retires its interface and the run continues; the run ends when every
      source has retired or the stop handle is set. Record the retirement with
      the interface and reason in the pipeline report, advancing no drop
      counter (FR-027, FR-028).
- [X] T029 [P] [US2] Add `ConfigError::NoSources` in
      `crates/fragcap-core/src/pipeline/mod.rs`, so a pipeline over nothing
      fails at construction rather than after capturing nothing.
- [X] T030 [US2] Allow more than one `InterfaceDeclaration` in
      `crates/fragcap-sink/src/pcapng/interface.rs`, each with its own link
      type and snapshot length, and remove the error variant refusing a second
      (FR-031, FR-032).
- [X] T031 [US2] Resolve each packet's `InterfaceId` to its declared block in
      the pcapng writer, keeping the refusal of an undeclared interface intact
      (FR-035).
- [X] T032 [P] [US2] Name the interface on every record in
      `crates/fragcap-sink/src/json.rs` when the capture holds more than one,
      and continue omitting the key when it holds exactly one (FR-033, FR-034).
- [X] T033a [US2] Add a retirement test to
      `crates/fragcap/tests/multi_interface.rs`: one of two sources fails part
      way through, the other keeps delivering, the run ends when the last has
      retired, and the report names the failed interface and the reason. Assert
      also that no drop counter moved as a result (SC-012, FR-027, FR-028).
      Analyze finding C3: this was the only success criterion with an
      implementation task and no test.
- [X] T033 [US2] Add the tier 1 test `crates/fragcap/tests/multi_interface.rs`
      driving two replay sources declared as distinct interfaces with different
      link types, asserting both declarations, correct per-packet references in
      the pcapng output, the interface key in the JSON output, and the
      conservation identity with two capture threads running (SC-004, SC-006).
- [X] T034 [US2] Run `cargo test -p fragcap --test corpus_pipeline` and confirm
      the committed goldens reproduce byte for byte. Do not regenerate them; a
      diff here means S09 changed single-interface output, which SC-005
      forbids.

## Phase 5: User Story 1, live capture (P1)

**Goal**: the first packet fragcap has ever read from a network interface.

**Independent test**: `cargo test -p fragcap-capture --features live --test live`
on Windows with npcap installed and an elevated shell. Tier 2.

- [X] T035 [P] [US1] Implement default-route determination in
      `crates/fragcap-capture/src/live/route.rs` using a bound and connected
      UDP socket's chosen source address, adding no dependency (plan D-3). A
      machine with no route yields `None`, which selection already handles.
- [X] T036 [US1] Implement `enumerate` in
      `crates/fragcap-capture/src/live/enumerate.rs`, adapting
      `pcap::Device::list()` into an `InterfaceInventory` and mapping
      `DeviceFlags::is_loopback`, `is_up`, and `is_running` onto the record.
      It must not open a handle (FR-003).
- [X] T037 [US1] Implement `LiveSource::open` in
      `crates/fragcap-capture/src/live/mod.rs`, applying snapshot length,
      promiscuous mode, and read timeout before activating (FR-020).
- [X] T038 [US1] Implement `next_packet`, mapping `Error::TimeoutExpired` to
      `Ok(None)` and carrying the driver's timestamp and original length
      unaltered (FR-014, FR-015, FR-016).
- [X] T039 [US1] Implement the error mapping in
      [contracts/capture-api.md](contracts/capture-api.md), including plan
      D-5: on a terminal backend error, re-enumerate and report `DeviceLost`
      when the interface is gone and `Backend` when it is still present. Do not
      match on the driver's message text (FR-019).
- [X] T040 [P] [US1] Implement `stats`, copying `pcap::Stat`'s three cumulative
      counts into `SourceStats` with `u32` widened to `u64` and no
      accumulation of fragcap's own (FR-017).
- [X] T041 [P] [US1] Implement `link_type` from `Capture::get_datalink()`,
      mapping `pcap::Linktype` onto `fragcap_core::LinkType` and reporting an
      unmapped value as an error rather than a guess (FR-018).
- [X] T042 [US1] Implement `set_filter` against `Capture::filter`, and install
      the bootstrap filter admitting only IPv4 and IPv6 on each handle before
      any packet is delivered. Report a rejected program as
      `FilterRejected` with the backend's detail (FR-036, FR-037, FR-038).
      Assert in the same place that the userspace scope decision does not
      consult the installed filter, so correctness never depends on filter
      freshness (FR-040). Analyze finding A1: true today by construction, which
      is exactly how it would stop being true unnoticed.
- [~] T043 [US1] Add the tier 2 test `crates/fragcap-capture/tests/live.rs`,
      gated `#[cfg(feature = "live")]`, generating its own loopback traffic and
      asserting it comes back with the driver's timestamps and lengths. A run
      with no driver present prints the reason and returns rather than failing
      (SC-001, CHK038).

## Phase 6: User Story 4, driver detection (P2)

**Goal**: the most common first-run condition fails usefully.

**Independent test**: interrogate detection on a machine without npcap.

- [X] T044 [US4] Implement `detect_driver` in
      `crates/fragcap-capture/src/live/driver.rs`, reporting presence and
      version, returning a report rather than a `Result` because absence is an
      answer (FR-041).
- [X] T045 [US4] Detect the loopback-capture and WinPcap-compatibility
      installation options, reporting `None` when undeterminable rather than
      `Some(false)` (FR-042, FR-045).
- [X] T046 [US4] Produce the absence message naming the driver and the official
      download location, in `crates/fragcap-capture/src/live/driver.rs`
      (FR-042, SC-008).
- [X] T047 [US4] Add a test asserting no code path in the live backend spawns a
      process, downloads, or writes outside the capture output (FR-043). A
      source-level assertion is acceptable and should say so.

## Phase 7: Polish and cross-cutting

- [X] T048 Add the transmit-API check to `xtask/src/lint.rs`, failing if any
      fragcap crate's source names the capture binding's send call, with a
      comment explaining that this is the mechanical form of the P-1 argument
      in plan D-8.
- [X] T048a Extend `cargo xtask lint` to fail if any capture driver binary,
      installer, or software development kit file is present in the repository,
      matching by extension and by the known SDK directory names. SC-010 says
      this is verified mechanically and nothing verified it. Analyze finding
      C2.
- [X] T048b Extend the `neutral` subcommand in `xtask/src/main.rs` to build
      `fragcap-capture` for the backend-free target in addition to
      `fragcap-core`, with the `live` feature off. FR-021 and SC-009 both claim
      `fragcap-capture` builds there and `neutral` only ever built
      `fragcap-core`. Analyze finding C1, which was a claim nothing checked.
- [X] T049 Give `.github/workflows/platform.yml` real triggers and enable the
      `live` feature, keeping the software development kit acquisition step.
      This is a pinned artifact and requires the dated decision in T050.
- [X] T050 Add `changelog.d/S09-live-capture-interfaces.added.md` and
      `changelog.d/S09-live-capture-interfaces.decisions.md`. The decisions
      fragment carries the three deviations, the `pcap` dependency with what
      was measured, the transmit-API lint, and the dated `platform.yml`
      decision. Do not edit `CHANGELOG.md`.
- [X] T051 Update the current state section of `AGENTS.md`: S09 complete, the
      dependency inventory table gains `pcap`, the two writer restrictions are
      lifted, `PacketSource` now requires `Send`, and the npcap acquisition
      step is exercised for the first time.
- [X] T052 Work through `checklists/acquisition.md`, resolving each item or
      recording why it does not apply. CHK013, CHK016, CHK019, and CHK026 are
      answered by this slice's design and should cite where.
- [X] T053 Run `cargo xtask ci` in the foreground and read the output. It must
      pass on this machine, which has no capture driver and no software
      development kit (SC-011).
- [X] T054 Run `cargo xtask neutral` and `cargo xtask msrv` in the foreground.
      Both exit 2 rather than 0 when they cannot run, so read the exit code as
      well as the output (SC-009).
- [ ] T055 Dispatch the `audit` workflow and watch it to completion. It owns
      `cargo deny`, it is weekly and dispatch-only, and it has never run. This
      slice adds the first substantive dependency graph, so it is the first to
      give the license allowlist a real subject (FR-051). Analyze finding C4.
      A result nobody watched is not a result; report what it actually said.

## What was not completed

- **T043 is written but has never run.** The tier 2 test exists and is
  type-checked with `cargo check -p fragcap-capture --features live --tests`,
  verified by introducing an error and watching the check fail. It has never
  been linked or executed, because this workspace has no npcap software
  development kit and linking needs `wpcap.lib`. Marked `[~]` rather than `[X]`.
- **T055 is open.** Dispatching the `audit` workflow and watching `cargo deny`
  needs the branch pushed, which is past the halt. The dependency licenses were
  verified by hand against the constitution's allowlist; nobody has watched the
  automated check confirm it.

## Dependencies

```text
Phase 1 (setup)
   |
Phase 2 (type changes, blocking)
   |
Phase 3 (US3 selection)  <- both P1 stories consume this
   |
   +---> Phase 4 (US2 multi-interface, tier 1)
   |         |
   +---> Phase 5 (US1 live source, tier 2) ----+
   |                                            |
   +---> Phase 6 (US4 detection) ---------------+
                                                |
                                          Phase 7 (polish)
```

Phase 4 does not depend on Phase 5. That is the point of the ordering: the
multi-interface behavior is fully verified before any code that needs a driver
exists.

Phases 5 and 6 both touch `crates/fragcap-capture/src/live/` and share
`enumerate`, so they are sequential rather than parallel despite serving
different stories.

## Parallel opportunities

- Phase 1: T002, T003, and T004 are three different files.
- Phase 2: T007 and T008 are independent type definitions; T015 is a test file
  concern once T013 lands.
- Phase 3: T018 and T019 are independent of T017's matrix.
- Phase 4: T029 and T032 touch files nothing else in the phase touches.
- Phase 5: T035, T040, and T041 are independent of the `next_packet` path.

## Implementation strategy

**The minimum viable increment is Phases 1 through 4.** At that point fragcap
has multi-interface capture, per-interface identity in both output formats,
per-interface loss accounting, and the whole section 12.1 precedence, all
verified on any machine with no capture driver. That increment is worth
committing and reviewing on its own terms even though it captures nothing live,
because everything in it is testable and everything after it is not.

**Phase 5 is where the claim stops being offline.** It should be written
against Phase 4's already-passing tests, and it should be run by hand on a
Windows machine with npcap before the pull request, because the `platform`
workflow has never completed a run and must not be treated as green until it
has been watched.

**Do not regenerate a golden to make T034 pass.** A diff there is the finding,
not the obstacle.
