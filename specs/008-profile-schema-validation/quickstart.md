# Quickstart: Validating S05

**Slice**: S05 | **Date**: 2026-08-09 | **Plan**: [plan.md](plan.md)

How to convince yourself the profile schema is correct, in the order that finds
problems soonest. Every command runs in the foreground and is watched to
completion; none of it needs a capture driver, elevated privilege, a game, or a
network interface.

## Prerequisites

The repository as checked out. Network access once, to fetch the two new
dependencies.

## The one command

```bash
cargo xtask ci
```

Formatting, Clippy with warnings denied, the workspace test suite, the
conventions linter, the dependency direction check, and the license check. If it
passes, the slice's mechanical obligations are met.

Two checks sit outside it because they need a toolchain or a target the runner
may not have. Both exit 2 rather than 0 when they cannot run, so a check that
did not run cannot look like one that passed.

```bash
cargo xtask msrv
```

```bash
cargo xtask neutral
```

`msrv` is the one that matters most for this slice, and it is the reason
`toml-span` is here instead of `toml`. The obvious dependency declares Rust 1.85
against a declared minimum of 1.82, and this check is what would have caught
that after the fact rather than before. Research R-1 has the measurements.

`neutral` matters for a subtler reason: a duration module was added to
`fragcap-core`, and this is the mechanical proof it brought no platform surface
with it.

## The four layers, individually

Run these when something in `ci` fails and you want to know where.

**Layer 1, the duration grammar.** Pure arithmetic over a string, in the crate
three later slices will call it from.

```bash
cargo test -p fragcap-core duration
```

**Layer 2, the glob and the ambiguity decision.** Pattern pairs with no profile
involved. This is where the intricate code is, so it is where the table-driven
tests are.

```bash
cargo test -p fragcap-profile glob
```

**Layer 3, diagnostics.** Every code produced at least once, and the
multiple-fault accumulation that is the slice's stated purpose.

```bash
cargo test -p fragcap-profile --test diagnostics
```

**Layer 4, the worked examples and resolution.** The two section 15.2 profiles
parsed field by field, and the four-step order against directories the test
builds under `CARGO_TARGET_TMPDIR`.

```bash
cargo test -p fragcap-profile --test examples --test resolution
```

## What each layer would catch

| Symptom | Layer that finds it | What it means |
| --- | --- | --- |
| A section 15.2 example is refused | 4 | The schema disagrees with the architecture of record. The specification wins; the code is the defect. |
| The three-stage example is refused | 4 | The ambiguity check is firing on the profile it exists to protect. Both its stages are pinned; check `pinned()`. |
| Two patterns wrongly reported disjoint | 2 | A false negative in the intersection walk, which is the silent empty capture the check exists to prevent. |
| Two patterns wrongly reported intersecting | 2 | A false positive, which refuses a legal profile with advice its author cannot act on. |
| One diagnostic where several were expected | 3 | Accumulation broke, most likely a `?` reintroduced into the extraction path. |
| Diagnostic order varies between runs | 3 | Something is emitting in traversal order rather than sorting. `toml-span` iterates in key order, which is neither the author's reading order nor stable. |
| A resolution test passes for the wrong reason | 4 | Check that the assertion is on `ProfileSource` and not only on the profile contents. Two steps can hold identical files. |
| `cargo xtask msrv` fails | outside | A dependency, direct or transitive, raised its declared minimum above 1.82. |
| `cargo xtask deps` fails | in `ci` | `fragcap-core` acquired a dependency. The duration module must not bring one. |

## Confirming the traversal check by hand

The rule is that a reference is refused before any path is joined to it, not
that the open fails. Those are different guarantees, and only the first holds
regardless of what happens to be at the target.

```bash
cargo test -p fragcap-profile --test resolution traversal
```

The test asserts the error is `InvalidReference` rather than `NotFound`. A
`NotFound` here would mean the join happened and the filesystem was consulted,
which passes today because nothing is at that path and stops passing on a
machine where something is.

## Confirming what the dependencies actually are

```bash
cargo tree -e normal -p fragcap-profile
```

Expected, and nothing more:

```text
fragcap-profile
├── fragcap-core
│   └── bytes
├── regex
│   ├── regex-automata
│   │   └── regex-syntax
│   └── regex-syntax
└── toml-span
    └── smallvec
```

If `aho-corasick` or `memchr` appears, `regex` default features were re-enabled;
they are literal-scanning optimizations that buy nothing when the haystack is
one image path. If `serde` appears, something reached for a derive-based
deserializer, which cannot satisfy FR-013 because it stops at the first error.

`Cargo.lock` is the wrong place to check this, and it will look wrong.
`aho-corasick` appears there because a lockfile records a resolution for every
optional dependency regardless of which features select it. `cargo tree`
respects the feature selection and is the answer to what actually builds.

```bash
git diff main -- Cargo.toml Cargo.lock crates/*/Cargo.toml
```

The expected diff is exactly two dependency lines in the workspace manifest and
two in `crates/fragcap-profile/Cargo.toml`. This workspace has added one runtime
dependency in eight slices, so a third and fourth are an architectural event;
both are argued in the plan and recorded in the decisions fragment. A dependency
that appears here without appearing there is the defect, whichever one is
missing.

## What this slice does not let you do yet

Nothing here evaluates a predicate against a process, so a valid profile is not
yet a capture. `fragcap profile validate` does not exist; S14 owns command
surfaces, and this slice supplies the diagnostics that command will print. No
bundled profiles ship, so resolution step four is exercised against a set the
test constructs.
