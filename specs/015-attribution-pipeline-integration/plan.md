# Implementation Plan: Attribution Session-to-Pipeline Integration

**Branch**: `feat/attribution-pipeline-integration` (spec dir `015-`) | **Date**: 2026-08-10 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/015-attribution-pipeline-integration/spec.md`

## Summary

Close the two paired S13 follow-ups (issues #18, #19). Change
`FlowAttributor::refresh` to `&self` (a specification section 29 deviation) so
the pipeline's section 8.6 control thread can drive the socket-table refresh
through the shared `Arc<dyn FlowAttributor>`, and restrict phase-two filter
narrowing to endpoints owned by profiled processes by filtering in the
role-stamping decorator, which already holds the profiled-PID snapshot. The CLI
`RefreshDriver` stopgap collapses onto the control thread. The resolve path stays
lock-free (section 11.6). Verified entirely at tier 1; the live socket-table
wiring is `cfg`-gated and stays compiled-only. Design decisions in
[research.md](research.md).

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82 (edition 2021).

**Primary Dependencies**: No new dependency. Reuses `arc-swap` (already present,
S10) for lock-free publication and `std::sync::Mutex` for the refresh-only state.
`fragcap-core`'s allowlist is untouched (P-2).

**Storage**: N/A (in-memory attribution snapshot).

**Testing**: `cargo test --workspace --locked`; new tier-1 tests in
`fragcap-core` (pipeline drives refresh), `fragcap-attr` (owner-carrying
endpoints, `refresh(&self)` interior mutability, lock-free resolve across a
publication), and `fragcap` facade (role-stamping decorator filters to profiled).

**Target Platform**: Windows for the live backend; all tests are platform-neutral
tier 1. `cargo xtask neutral` (linux, no capture backend) must still build.

**Project Type**: Rust workspace (library crates + CLI). Compiler/systems style.

**Performance Goals**: The per-packet `resolve` path stays wait-free (one atomic
load, no lock). Refresh cost is unchanged (still one section 11.2-cadence table
read on the control thread).

**Constraints**: P-1 (no process handle; profiled PIDs come from existing session
bindings, not a new query), P-2 (no platform dep in `fragcap-core`), P-3 (session
owns stage matching; pipeline/attr consume it), P-4 (no new discard class), P-6
(glossary entry for any new term), P-9 (retention window origin unchanged),
section 11.6 (lock-free resolve). All text UTF-8 no BOM, LF, no em/en dashes.

**Scale/Scope**: A trait-signature change touching every `FlowAttributor`
implementor and test double (7 sites), one new small value type, interior
mutability on one attributor, a two-line control-thread addition, one decorator
override, and a `cfg`-gated CLI wiring change removing one stopgap thread.

## Constitution Check

*GATE: passed at plan time; re-checked after design.*

- **P-1 (no denylisted technique)**: PASS. No process handle, injection, or hook.
  The profiled-PID set is the session's existing stage bindings.
- **P-2 (core takes no platform dep)**: PASS. The new trait methods and
  `OwnedEndpoint` live in `fragcap-core`; the socket table stays in
  `fragcap-attr`. `wants_refresh` keeps the schedule type out of core. `cargo
  xtask neutral` builds.
- **P-3 (source/attributor separation; layering)**: PASS. Filtering runs in the
  facade decorator, not in `fragcap-attr` or the pipeline. No trait names another.
- **P-4 (every discard counted)**: PASS. No new discard class; existing counters
  and the conservation invariant are untouched (FR-009).
- **P-6 (glossary in same change)**: PASS. `OwnedEndpoint` and "profiled endpoint
  set" get glossary entries; the refresh-driving and narrowing resolutions are
  promoted to the specification.
- **P-9 (the instrument does not lie)**: PASS. Retention window origin (last seen
  present) is unchanged; only the caller of refresh moves.
- **Deviation process**: the `refresh(&self)` signature and the two added trait
  methods are recorded as a dated section 29 decision fragment.

No violations; Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/015-attribution-pipeline-integration/
├── plan.md              # This file
├── spec.md              # Feature spec
├── research.md          # Design decisions D-1..D-6
├── data-model.md        # Trait contract and the OwnedEndpoint type
├── quickstart.md        # Tier-1 validation guide
└── checklists/
    ├── requirements.md
    └── concurrency-and-deviation.md
```

### Source Code (repository root)

```text
crates/fragcap-core/src/
├── traits.rs            # FlowAttributor: refresh(&self); + wants_refresh; + active_endpoints_owned; update StubAttributor
├── flow.rs              # + OwnedEndpoint value type
└── pipeline/mod.rs      # control thread drives refresh; update StubAttributor, PanicOnEndpoints doubles

crates/fragcap-attr/src/
├── socket.rs            # SocketTableAttributor: Mutex<RefreshState>; refresh(&self); wants_refresh/active_endpoints_owned trait impls
├── index.rs             # AttributionIndex::endpoints_owned(at) -> Vec<OwnedEndpoint>
├── resolver.rs          # PublishedResolver::refresh(&self) (no-op); soften module doc
└── scripted.rs          # ScriptedAttributor::refresh(&self) (no-op)

crates/fragcap/src/
└── session.rs           # RoleStampingAttributor: refresh(&self) forwards to inner; wants_refresh forwards; active_endpoints() filters to profiled; update Fixed double

crates/fragcap-cli/src/
├── assemble.rs          # remove RefreshDriver; live_components wraps real attributor  [cfg socket-table,windows]
└── orchestrator.rs      # narrowed message reports profiled count; drop refresh_driver stop  [cfg gated]

docs/glossary.md         # + OwnedEndpoint, profiled endpoint set
docs/plans/README.md     # note: roadmap S15-S18 shift to dirs 018-021; 015 is an S13 follow-up
changelog.d/             # S015 added fragment + dated section-29 decision fragment
```

**Structure Decision**: Existing workspace layout; no new crates or modules. The
change is concentrated in the attribution trait and its implementors, with a
two-line pipeline addition and a `cfg`-gated CLI simplification.

## Implementation order (TDD, tier 1)

1. **`OwnedEndpoint`** in `fragcap-core::flow` (value type + unit test).
2. **Trait change** in `traits.rs`: `refresh(&self)`, `wants_refresh(&self) ->
   bool { false }`, `active_endpoints_owned(&self) -> Vec<OwnedEndpoint>`
   default. Update the in-file `StubAttributor`. Keep the dyn/Send/Sync
   assertions green.
3. **`fragcap-attr`**: index `endpoints_owned`; `SocketTableAttributor`
   `Mutex<RefreshState>` + `refresh(&self)` + trait `wants_refresh` /
   `active_endpoints_owned`; `PublishedResolver` and `ScriptedAttributor`
   `refresh(&self)`. Add the tier-1 tests: owner-carrying endpoints; lock-free
   resolve across a refresh publication (the existing several-threads test,
   adapted to drive `refresh(&self)` through a shared `Arc`).
4. **Pipeline** (`fragcap-core`): control thread drives `wants_refresh`/`refresh`;
   update the `StubAttributor` and `PanicOnEndpoints` doubles. Add the tier-1
   test: a test attributor whose `refresh` flips its answer becomes resolvable
   mid-run through the pipeline (User Story 1 / SC-001).
5. **Facade** (`session.rs`): `RoleStampingAttributor::refresh` forwards to
   inner; `wants_refresh` forwards; `active_endpoints()` filters to the profiled
   snapshot via `inner.active_endpoints_owned()`; update `Fixed`. Add the tier-1
   test: profiled vs unprofiled split admits only profiled (User Story 2 /
   SC-002).
6. **CLI** (`cfg`-gated): remove `RefreshDriver`; `live_components` wraps the real
   attributor; narrowed message reports the profiled count.
7. **Docs**: glossary entries; `docs/plans/README.md` numbering note; specification
   promotion to sections 11, 12.2, 29; changelog fragments.
8. **Verify**: `cargo xtask ci`, `cargo xtask neutral`, `cargo xtask msrv` in the
   foreground; confirm offline goldens byte-identical.

## Risks

- **Lock-free regression**: the Mutex must not touch the resolve path. Guarded by
  the adapted concurrency test (SC-003) and the `concurrency-and-deviation.md`
  checklist.
- **Missed implementor**: a trait change that skips one implementor fails to
  compile; the dyn/Send/Sync assertions catch bound regressions.
- **Offline golden drift**: `wants_refresh` default-false and the scripted
  owned-endpoints default keep offline behavior identical; the corpus goldens are
  the guard.
- **cfg-gated code unbuilt in CI**: the `live_components`/`RefreshDriver` changes
  compile only under `socket-table,windows`; build them locally under that
  feature before the pre-push halt, and report that tier-2 stays unexecuted.
