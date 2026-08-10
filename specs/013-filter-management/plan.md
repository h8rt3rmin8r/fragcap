# Implementation Plan: Filter Management

**Branch**: `feat/filter-management` | **Date**: 2026-08-10 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/013-filter-management/spec.md`

## Summary

S13 adds phases two and three of the section 12.2 filter lifecycle to the S09
bootstrap. Three pieces, all in `fragcap-core`: a pure compiler that turns an
endpoint set into a `FilterProgram` (a libpcap expression); a pure maintenance
policy that decides, under a two-second debounce and a one-per-five-seconds
per-handle rate limit, when to reinstall and counts the filter gaps it opens; and
a control thread wired into `Pipeline::run` that reads the attribution map's
`active_endpoints()`, drives the policy, and hands each capture thread its current
program over a `std::sync::mpsc` channel. Each capture thread installs the program
on its own handle between reads via the existing `PacketSource::set_filter`. The
narrowed filter is never the authority: userspace attribution still runs on every
packet. Everything is tier-1 testable, the policy against a supplied instant and
the wiring against a recording source double.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82

**Primary Dependencies**: none new. The per-source publication uses
`std::sync::mpsc`; the endpoint source is `FlowAttributor::active_endpoints()`
(S10); BPF application is `PacketSource::set_filter` (S09). `fragcap-core`'s
dependency allowlist stays `["bytes"]`.

**Storage**: N/A (in-memory endpoint sets and per-handle policy state).

**Testing**: `cargo test` at tier 1. The compiler and the policy are pure and
tested directly (the policy with caller-supplied `Instant` values); the control
thread is tested through `Pipeline::run` with a recording `PacketSource` double
and a scripted attributor. No capture driver, elevation, or game.

**Target Platform**: platform-neutral. All new code is in `fragcap-core`, which a
BPF expression reaches only as text; `cargo xtask neutral` still builds core for a
backend-free target.

**Project Type**: Rust library workspace (the library is the product).

**Performance Goals**: the per-packet path gains one non-blocking channel drain
per read-loop iteration (once per read timeout, not per packet). Compilation and
policy run on the control thread, off the acquisition path. No per-packet
allocation is added.

**Constraints**: `fragcap-core` stays platform-neutral and dependency-minimal
(P-2); `PacketSource` and `FlowAttributor` stay separate and `PacketSource` gains
no `Sync` bound (P-3); every discard and gap is counted (P-4); no fabricated
packet counts (P-9).

**Scale/Scope**: an endpoint set is tens of endpoints; handles are one per
interface; the control thread ticks a few times a second.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive Observation**: no denylisted technique. The filter is a capture
  filter compiled onto the NDIS driver through the permitted `pcap` binding
  (section 19.2), the same install path S09 already uses for bootstrap. No handle,
  no injection, no hook. PASS.
- **P-2 Core Stays Platform-Neutral**: all new code is in `fragcap-core` and uses
  only `std` plus the existing `bytes`; the dependency allowlist is unchanged. A
  compiled filter is a `String` to core; only `fragcap-capture` treats it as npcap
  syntax. `arc-swap` was deliberately not added to core to avoid widening the
  allowlist (see research D-c). PASS.
- **P-3 Capture And Attribution Stay Separate**: the filter manager reads
  `active_endpoints()` off the shared attributor and hands programs to sources
  over a channel; neither `PacketSource` nor `FlowAttributor` names the other, and
  `PacketSource` gains no bound. The manager bridges them from the control thread,
  which is where section 8.6 places it. PASS.
- **P-4 No Silent Loss**: a stale-filter exclusion is counted in the existing
  named `filter_gaps` counter and surfaced; it is distinct from the three drop
  counters. A deferred or failed maintenance reinstall advances no drop counter
  because nothing was observed and then discarded. PASS.
- **P-6 Glossary First**: `Filter gap` (resolving the existing dangling
  reference), `Filter manager`, `Filter program`, and the narrowing and
  maintenance phases get glossary entries in this change. PASS.
- **P-9 The Instrument Does Not Lie**: `filter_gaps` counts gap occurrences
  (wanted endpoints not yet admitted), never a fabricated count of kernel-excluded
  packets fragcap did not observe. Userspace attribution still runs on every
  packet, so the filter alters no observation. PASS.

No violations; Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/013-filter-management/
├── plan.md
├── spec.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── filter-management-api.md
├── checklists/
│   ├── requirements.md
│   └── filtering.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-core/src/filter.rs            # grow: compile(endpoints) -> FilterProgram;
                                             #   FilterManager + FilterConfig + install decisions + gap counting
crates/fragcap-core/src/pipeline/mod.rs      # + control thread in run(); per-source mpsc channels;
                                             #   acquire() drains its receiver and installs; absorb control-thread filter_gaps
crates/fragcap-core/src/stats.rs             # refine the filter_gaps doc comment to the section 12.3 definition
crates/fragcap-core/src/lib.rs               # re-exports (FilterManager, FilterConfig) if needed
crates/fragcap-core/tests/ or #[cfg(test)]   # recording source double; end-to-end narrowing/gap test
docs/glossary.md                             # + Filter gap, Filter manager, Filter program, narrowing, maintenance (P-6)
changelog.d/S13-filter-management.added.md
changelog.d/S13-filter-management.decisions.md
```

**Structure Decision**: everything lands in `fragcap-core`. Compilation and policy
are pure over core types (`Endpoint`, `Proto`, `SocketAddr`, `Instant`); the
control thread belongs to the pipeline, which section 8.2 places in core and which
is the one place already holding both the sources (to install) and the attributor
(to read endpoints), so it satisfies P-3 without either trait naming the other.
`fragcap-capture` and `fragcap-profile` are untouched; the bootstrap install and
BPF application already exist in `fragcap-capture` and are consumed.

## Phase 0: Research

See [research.md](research.md). Decisions D-a through D-f record the endpoint
source, the filter-gap counting unit, the per-source delivery mechanism, the BPF
grammar and empty-set behavior, the maintenance timing model and its
injectability, and where gap counting is accumulated.

## Phase 1: Design

See [data-model.md](data-model.md) for the entities and
[contracts/](contracts/filter-management-api.md) for the public surface.
[quickstart.md](quickstart.md) shows the tier-1 test path.

## Complexity Tracking

No constitution violations; no entries.
