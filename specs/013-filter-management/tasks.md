# Tasks: Filter Management

Dependency-ordered. Test-driven: within each component the tests are written with
or before the implementation, and the component is not complete until they pass.
All tests are tier 1 (no capture driver, no elevation, no game).

## T001 - Endpoint ordering (`fragcap-core`)

- Add `Ord`/`PartialOrd` derives to `Endpoint` and `Proto` in
  `crates/fragcap-core/src/flow.rs` if absent, so an endpoint set can key a
  `BTreeSet` for deterministic compilation and set-difference gap counting.
- Add a test that endpoints sort deterministically.
- Depends on: nothing. Blocks: T002, T003.

## T002 - Filter compilation (`fragcap-core`)

- Grow `crates/fragcap-core/src/filter.rs`: `FilterProgram::narrowed(endpoints:
  &[Endpoint]) -> FilterProgram`, the OR of one protocol/host/port clause per
  endpoint, sorted and deduplicated, spanning IPv4 and IPv6; empty input yields an
  empty program.
- Tests: exact admission per endpoint; IPv4+IPv6 mix; determinism under reorder and
  duplication; empty input is `is_empty()`.
- Depends on: T001. Blocks: T004, T005.

## T003 - Maintenance policy (`fragcap-core`)

- In `crates/fragcap-core/src/filter.rs`: `FilterConfig` (+ `PRODUCTION`, `Default`)
  and `FilterManager` with `new`, `poll(&mut self, wanted, now) -> Vec<Install>`,
  and `filter_gaps`. Internal per-handle `Installed`/`HandleState`; capture-wide
  debounce state; gap accumulation per the contract.
- Tests with synthetic `Instant`s: debounce holds installs for two seconds and
  resets on change; per-handle five-second rate limit defers (not drops) a
  reinstall; churn coalesces; idempotence (no reinstall when unchanged); empty set
  installs nothing and keeps bootstrap/prior; gap counting (bootstrap-to-first
  records zero, a narrowed-to-narrowed addition records the added endpoints).
- Depends on: T001, T002. Blocks: T005.

## T004 - Stats doc refinement (`fragcap-core`)

- Refine the `CaptureStats::filter_gaps` doc comment in
  `crates/fragcap-core/src/stats.rs` to the section 12.3 definition (gap
  occurrences, distinct from the drop counters, never a fabricated packet count).
  No field or behavior change.
- Depends on: nothing. Blocks: T007 (glossary consistency).

## T005 - Control thread wiring (`fragcap-core::pipeline`)

- In `crates/fragcap-core/src/pipeline/mod.rs`: add `Pipeline::set_filter_config`
  and a `filter_config` field (default `PRODUCTION`). In `run`, create a per-source
  `mpsc` channel, spawn a control thread that reads `active_endpoints()`, drives
  `FilterManager::poll(_, Instant::now())`, sends each `Install`'s program to its
  handle's channel, and returns `CaptureStats { filter_gaps, ..default }`. Give
  `acquire` a `Receiver<FilterProgram>`; drain to latest and `set_filter` between
  reads, treating a maintenance `set_filter` error as non-fatal (keep prior
  program, do not retire). After joining capture threads, signal stop, join the
  control thread, and `absorb` its stats.
- Depends on: T002, T003. Blocks: T006.

## T006 - Pipeline filter tests (`fragcap-core::pipeline`)

- A recording `PacketSource` double that captures the sequence of installed
  `FilterProgram`s. Tests: with a zero-debounce `FilterConfig`, a scripted
  attributor with a non-empty active-endpoint set drives bootstrap-to-narrowed on
  the double; every packet is still attributed regardless of the filter;
  `filter_gaps` surfaces separately from the drop counters and the conservation
  identity still holds; the control thread does not block shutdown. Confirm the
  existing corpus/pipeline tests still pass unchanged (default 2s debounce means a
  fast run installs nothing, so goldens are unaffected).
- Depends on: T005.

## T007 - Glossary (P-6)

- Add entries to `docs/glossary.md` under `## Capture and Networking`: `Filter
  gap` (resolving the dangling reference from `Bootstrap filter`), `Filter
  manager`, `Filter program`, and the narrowing (phase two) and maintenance (phase
  three) phases. Cross-link `Bootstrap filter`. Do not re-add existing terms.
- Depends on: T002, T003, T005 (terms are stable by then).

## T008 - Changelog fragments

- `changelog.d/S13-filter-management.added.md` (feature line, present tense,
  citing sections 12.2/12.3) and
  `changelog.d/S13-filter-management.decisions.md` (D-a through D-f dated; note the
  two section-29 deviation candidates).
- Depends on: T001..T007.

## T009 - Verify and commit

- Run `cargo xtask ci` in the foreground to completion (fmt, clippy, test, lint,
  deps, license); run `cargo xtask neutral` and `cargo xtask msrv` (expect exit 0
  or a clean can-not-run 2).
- Stage the slice's files and commit with a conventional message. Halt before push.
- Depends on: T001..T008.
