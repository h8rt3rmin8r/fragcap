# Quickstart: verifying S065

**Slice**: S065 | **Date**: 2026-08-20 |
**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

Two tiers. Tier 1 runs anywhere the workspace builds and covers every
functional requirement. Tier 2 is the operator's machine and covers SC-001 and
SC-002, which name real installed titles.

## Prerequisites

Tier 1 needs only a Rust toolchain. No capture driver, no elevation, no game,
no Steam install: the PE inputs are bytes this repository generates and the
engine trees are temporary directories.

## Tier 1: the ordinary gate

```bash
cargo xtask ci
```

Run it in the foreground and watch it finish. It runs, in order: `cargo fmt
--check`, `cargo clippy --all-targets --all-features -D warnings`, `cargo test
--workspace --locked`, `cargo xtask lint`, `cargo xtask deps`, and `cargo xtask
license`.

To run only this slice's tests while iterating:

```bash
cargo test -p fragcap-profile pe signature engine_rule
```

```bash
cargo test -p fragcap-targets --test signatures
```

```bash
cargo test -p fragcap-cli --test cli_targets
```

### What each requirement is verified by

| Requirement | Verified by |
| --- | --- |
| FR-001 no DRM from the SDK library | a test asserting the shipped seed carries no `steam_api` pattern, and a fixture tree with `steam_api64.dll` and no `.bind` reporting no DRM |
| FR-002 section-marker pattern form | a compile test over `section:.bind` and over a byte-marker pattern, asserting applied and inert respectively |
| FR-003 `.bind` detects the wrapper | two generated PE fixtures, one with a `.bind` section and one without, in otherwise identical trees |
| FR-004 bounded scan | a tree with an executable below the depth bound (not read) and a bounded-prefix read asserted by a fixture whose section table sits inside the prefix |
| FR-005 cap-dropped candidates counted | a tree with more than the cap of executables, asserting the counter and the incomplete coverage state |
| FR-006 accounting sums | the existing sum assertion, extended to the shipped seed and to a set carrying an unrecognized marker form |
| FR-007 passive | `cargo xtask lint`, which fails on the forbidden call names, plus review of the one new reader |
| FR-008, FR-009 new engines | fixture trees for the observed Ren'Py and GameMaker layouts |
| FR-010 written decision | the decisions fragment under `changelog.d/`, and research.md R-5 |
| FR-011 directed subset check | the test in `crates/fragcap-targets/tests/signatures.rs`, iterating `Engine::ALL` |
| FR-012 two columns | the CLI listing test asserting both headers and that neither column carries the other's category |
| FR-013, FR-018 readiness | a grep-style assertion that the two retired sentences appear nowhere, plus the readiness labels unchanged |
| FR-014 three states | three registered rows, one per state, rendering three different markers |
| FR-015 stored, plumbed, round-tripped | a store round trip, an export/import round trip, an out-of-set import rejection, and an open of a v6 store reading `None` |
| FR-016 machine surface | an export test asserting both the per-finding `category` and the `detection_scan` key |
| FR-017 width | three tests over rendered output: the non-handle budget, the fit at the longest fitting handle, and the no-clipping overflow at the real 47 character handle |

## Tier 2: the operator's machine

Needs the operator's Windows machine with the seven measured titles installed.
Nothing in tier 1 depends on it, and a tier 2 result is a claim about one
machine, so record the date with it.

Seed a scratch catalog with the new signature set and point discovery at it, so
the check does not depend on whatever the per-user catalog was seeded from:

```bash
cargo run -q -p fragcap-cli -- catalog seed --tier signature --db /tmp/s065-cat.db
```

```bash
cargo run -q -p fragcap-cli -- targets discover --catalog-db /tmp/s065-cat.db --local-db /tmp/s065-loc.db
```

Read the rendered table and check, against SC-001 and SC-002:

- `arc_raiders`, `barotrauma`, `shale_hill_secrets`, and
  `trapped_with_ivy_piper` carry no DRM product in `SENSITIVITIES`.
- `detroit_become_human`, `palworld`, and `enshrouded` still carry
  `Steam DRM`.
- `trapped_with_ivy_piper` carries `Ren'Py` in `ENGINE`.
- `shale_hill_secrets` carries `GameMaker` in `ENGINE`.
- No row carries an engine and a protection product in the same column.
- The row overhead outside the handle is 53 columns, so the table fits 80 unless
  a handle exceeds 27 characters. On this machine one does (47 characters), so
  the listing runs to 100 columns with every value intact. That is the declared
  behavior, not a defect.

A row registered before this change carries no coverage record, so it renders
`not scanned` until it is re-registered. That is correct, not a defect: the
tool does not claim a scan it did not run. To re-scan, remove the row and let
discovery re-register it.

## What a failure means

- The subset check failing means an engine was added to the launch-resolution
  rules without a detection signature. Add the signature; do not relax the
  check.
- A tier 2 row reporting `not scanned` after a fresh discovery run means a
  producing source was left unplumbed, which is the FR-015 defect. Find the
  source, not the renderer.
- The budget test failing means a column was added or a marker widened. Update
  `NON_HANDLE_COLUMNS` deliberately and say so; do not truncate a value to fit.
