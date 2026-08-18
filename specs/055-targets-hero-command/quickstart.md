# Quickstart / Validation: The targets hero command

**Feature**: S055 | **Date**: 2026-08-18

Runnable scenarios that prove the slice. Local builds here use the GNU toolchain
(no MSVC linker on this machine); CI runs the real `cargo xtask ci`.

```sh
# Build/test the CLI crate locally (GNU toolchain)
cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-cli
cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-targets
```

## Scenario 1 - Hero listing on a fresh store (US1)

Prereq: an empty local.db (point `FRAGCAP_LOCAL_DB` at a temp path).

```sh
fragcap targets
```

Expected: with no targets and no discovery hits, the empty-case block prints
actionable next commands and names a next command; exit 0. Seed a few entries
(via `add`/`import`) and re-run: a numbered table with CAPTURE/KNOWN columns,
handle-ordered, ending in `fragcap capture <n>`.

Automated: `crates/fragcap-cli/tests/cli_targets.rs` (listing shape, ordering,
footer vs no-footer, empty-case next commands). Golden for the table renderer.

## Scenario 2 - Durable row index (US1, SC-003)

```sh
fragcap targets            # shows row 3 = "rust"
fragcap targets add ...    # register a new target that sorts before "rust"
fragcap capture 3          # still resolves to "rust", the row the user saw
```

Expected: `capture 3` resolves to the snapshot's row 3, not the shifted live
order. A row index past the snapshot exits 2.

Automated: `crates/fragcap-targets/tests` snapshot round-trip + a
`cli_capture.rs` / `cli_targets.rs` case asserting resolution against the snapshot
after a mutation.

## Scenario 3 - Interactive add, all three answers (US2)

Drive the scripted prompt seam (no terminal needed):

```sh
# non-interactive equivalent used in tests
fragcap targets add ./game.exe --name "My Game" --socket-holder unsure
```

Expected: entry registered with an unresolved launch chain and no fabricated
holder; `Y` -> resolved client + `ready`; `n` -> unresolved holder + `needs a
target`. Inline scan evidence shown before the socket-holder decision.

Automated: `cli_targets.rs` add-flow cases for Y / n / unsure via the scripted
seam; a `fragcap-targets` unit test asserting the stored `launch_entries` shape
and that no answer writes an unobserved holder.

## Scenario 4 - Promotion on capture (US2, FR-013)

```sh
# fixture pipeline: run capture against an unsure-authored target
fragcap capture <selector>   # over a fixture; observes the socket holder
fragcap targets show <selector>   # fidelity now "verified", launch chain resolved
```

Expected: the row promotes to `verified` with the observed client; a run that
observes no holder leaves it unresolved.

Automated: a unit test of the promotion function; an integration test over the
fixture pipeline (`crates/fragcap/tests`) asserting the write-back. Live-capture
demonstration, if required, is Tier 2 (not CI) and labeled as such.

## Scenario 5 - Export / import round-trip (US3, SC-005)

```sh
fragcap targets export > targets.json
FRAGCAP_LOCAL_DB=fresh.db fragcap targets import targets.json
FRAGCAP_LOCAL_DB=fresh.db fragcap targets export > targets2.json
diff targets.json targets2.json   # identical id set, no duplicates
```

Expected: identical identifiers, no duplicate rows; importing twice is idempotent
on identity; a nonconforming file is rejected whole.

Automated: `fragcap-targets` export/import round-trip test; `cli_targets.rs`
import merge-on-id and reject-nonconforming cases.

## Scenario 6 - Remove (US3)

```sh
fragcap targets remove <selector>   # removes exactly that target
fragcap targets remove <ambiguous-name>   # lists matches, refuses, exit 2
```

Automated: `cli_targets.rs` remove cases (exact, ambiguous, cascade of aliases).

## Full gate (CI parity, run before commit)

```sh
cargo xtask ci
```

`fmt`, `clippy --all-targets --all-features`, `test --workspace --locked`,
`xtask lint`, `xtask deps`, `xtask license`, plus the fixture drift and
`xtask spec` (P-11 Applies-To). `neutral` and `msrv` exit 2 where the toolchain
is absent and are watched in CI.
