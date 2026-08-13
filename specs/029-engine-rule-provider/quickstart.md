# Quickstart: Validating the Engine-Rule Provider

This slice adds no CLI surface, so validation is at the library and test level:
the provider resolves an Unreal twin-exe layout through the S027 resolver at
`heuristic-unverified` fidelity, declines on a no-match, and is outranked by a
profile. All scenarios run with `cargo test`; the full gate is `cargo xtask ci`.

## Prerequisites

- Rust toolchain as pinned (`rust-toolchain.toml`), MSRV 1.82.
- No npcap, no capture driver, no game install: the provider inspects temporary
  directory trees the tests build.

## Run the slice's tests

```bash
cargo test -p fragcap-profile engine_rule
```

Expected: the `engine_rule` module tests and the provider tests pass, covering:

- Unreal: a temp tree with a root stub and
  `MyGame/Binaries/Win64/MyGame-Win64-Shipping.exe` resolves to the shipping
  executable, stamped `HeuristicUnverified` and `engine-rule`.
- Unity: a temp tree with `MyGame_Data/`, `UnityPlayer.dll`, and `MyGame.exe`
  resolves to the player executable.
- Ren'Py: a temp tree with `renpy/`, a `.rpa` archive, and a launcher exe
  resolves to the launcher.
- No match: a temp tree with no recognized layout yields no answer.
- Absent client: a recognized `Binaries/Win64` directory with no shipping exe
  yields no answer (no fabricated target).
- Ambiguous: two `*-Win64-Shipping.exe` files decline and record the note.

## Validate cascade participation and precedence

```bash
cargo test -p fragcap-profile resolver
cargo test -p fragcap-profile providers
```

Expected: a `TargetResolver` holding the real `EngineRuleProvider` plus a
profile provider resolves the same Unreal install to the profile's answer when a
matching profile exists (profile outranks engine rule), and to the engine-rule
answer when no profile matches (SC-004). Registration order does not change the
result (the S027 permutation discipline).

## Determinism check

```bash
cargo test -p fragcap-profile engine_rule::tests::deterministic
```

Expected: resolving the same fixture twice, and after shuffling the order the
test creates sibling files in, yields the identical resolved path (SC-003,
FR-006).

## Full repository gate

```bash
cargo xtask ci
```

Expected: green. This runs fmt, clippy (all targets, all features, warnings
denied), the workspace test suite, the conventions lint (which continues to
forbid `OpenProcess`/`ReadProcessMemory`/`WriteProcessMemory`, none of which the
module names), the dependency-direction check (unchanged, no new crate), the
license check, and the fixture drift check (unaffected: this slice commits no
`fixtures/` files). Also run, where a toolchain is present:

```bash
cargo xtask msrv       # builds at 1.82; exits 2 if the toolchain is absent
cargo xtask neutral    # fragcap-core builds with no backend; exits 2 if absent
```

Report an exit-2 skip honestly rather than as a pass.

## Documentation check

The new glossary entry for "engine rule" and the master-spec cascade subsection
land in this slice. The documentation linter enforces glossary completeness and
cross-links:

```bash
bash scripts/lint-docs.sh
```

Expected: green, with the "engine rule" term defined and cross-linked.
