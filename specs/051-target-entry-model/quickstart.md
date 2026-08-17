# Quickstart: Validating the target entry model

This is a validation/run guide, not an implementation guide. It proves the slice
end to end against the spec's Success Criteria. Implementation detail lives in
`tasks.md`.

## Prerequisites

- The `targets` feature enabled (it carries the store and the new deps).
- In this environment, use the GNU-host toolchain for SQLite-backed crates:
  `cargo +1.96.0-x86_64-pc-windows-gnu ...`. CI runs the real MSVC build.

## 1. Handle vectors (SC-001, SC-002)

Run the Appendix A table as unit tests:

```sh
cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-targets --features targets handle
```

Expected: every Appendix A vector passes, including the fallback cases (`2048`
and whitespace-only fall back; a 90-character title truncates to 64 with no
trailing `_`). No stored handle is purely numeric.

## 2. Collision (SC-003)

Register `Portal 2` twice (unit/integration test): the first yields `portal_2`,
the second yields `portal_2_2`, and the first entry's handle is byte-identical
before and after the second registration.

## 3. Stable identifier (SC-004)

```sh
cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-targets --features targets identifier
```

Expected:
- Two entries built independently from `steam:2221490` have equal `stable_id`.
- Entries from `steam:2221490` and `steam:620` differ.
- An unanchored entry later matched to an anchor adopts the anchored `stable_id`
  and its former value appears in `target_id_aliases`; `--id <old>` still
  resolves it.

## 4. Fidelity-ordered resolution and declines (SC-005)

```sh
cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-targets --features targets hint_cascade
```

Expected:
- A title `authored` in `local.db` beats the same title `heuristic-unverified` in
  `catalog.db`.
- Competing `local.db` rows resolve highest-fidelity-first.
- Each of the four declines (sparse, engine-only, launcher-mediated, multi-exe)
  declines and the cascade continues; the pre-existing mediation and ambiguity
  tests still pass.

## 5. Selector resolution (SC-006)

Integration test over the CLI selector:
- A unique handle resolves exactly one target.
- `--id <N>` resolves by `stable_id` regardless of name/handle collisions.
- A name matching two targets lists both (with handle and id) and exits **2**,
  resolving nothing.
- A bare integer selects by ephemeral row index.

## 6. Retirement (SC-007) - DEFERRED to S054

Not applicable in S051. Retiring `--profile`, the AppData profile directory, and
the `profile` command is deferred to S054's capture rework (they are the only
capture entry point; see the deferrals clarification). In S051 those surfaces are
unchanged and `fragcap schema validate <file>` still validates against the
published schema. This step's validation runs when S054 lands.

## 7. Full gate

```sh
cargo xtask ci
```

Plus, on a linker-capable machine, verify the dependency additions:

```sh
cargo tree -p fragcap-targets --features targets       # inspect the blake3 + unicode delta
cargo deny check licenses                               # blake3/constant_time_eq/unicode licenses resolve to the allowlist
cargo xtask msrv                                        # default features only: new deps are not compiled under 1.82
```

Expected: `cargo xtask ci` green; `Cargo.lock` delta limited to `blake3`,
`unicode-normalization`, the chosen category crate, and their small transitive
graphs; every license in the allowlist (or added via a dated `deny.toml`
exception fragment); MSRV gate unaffected because the new crates sit behind the
default-off `targets` feature.
