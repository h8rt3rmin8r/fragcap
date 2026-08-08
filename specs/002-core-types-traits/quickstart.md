# Quickstart: Validating Core Types and Traits

**Slice**: S02

**Created**: 2026-08-08

How to confirm this slice actually did what it claims. Every command here runs
in the foreground and is watched to completion, per the constitution's
verification rule.

## Prerequisites

A clean clone and `rustup`. The pinned toolchain installs itself from
`rust-toolchain.toml`. For the platform-neutrality check, one extra target:

```sh
rustup target add x86_64-unknown-linux-gnu
```

## The full gate set

```sh
cargo xtask ci
```

Expected final line:

```text
ci: all checks passed
```

This runs format, clippy with warnings denied, the workspace test suite with a
locked dependency set, the repository conventions linter, the dependency
direction check, and the per-crate licensing check.

## The two checks that became meaningful in this slice

Both passed vacuously before this slice because the dependency graph was empty.
S01 recorded that rather than hiding it. Run them and read the output:

```sh
cargo xtask msrv
```

The declared minimum is 1.82 and the single new dependency declares 1.57, so the
check should pass. It is no longer vacuous, but it is not yet tight. See
`research.md` R-6.

```sh
cargo xtask neutral
```

Builds `fragcap-core` for a target with no capture backend. This is the P-2
proof, and it is the check that would catch a platform dependency entering core.

## The dependency audit, against a real graph

```sh
cargo deny check licenses
```

`bytes` is MIT, which is on the allowlist in specification section 20.4, so
`deny.toml` needed no amendment. This is the first time this check has had a
subject.

If `cargo-deny` is not installed locally, the `audit` workflow runs it. That
workflow is scheduled weekly and dispatch-only, and per its own header comment
it has not yet completed a run, so do not treat it as green without reading a
run.

## What the tests prove

The test names correspond to the validation rules in `data-model.md`. The ones
worth reading rather than only running:

- The UDP attribution key derivation, which asserts a UDP flow key never yields
  a key carrying a remote endpoint. This is the type-level enforcement of
  specification section 8.4's prohibition.
- The trait object test, which constructs each behavioral trait behind a
  pointer. It fails if a later change makes a trait method generic, which would
  break the pipeline in section 8.6.
- The attribution state test, which constructs each of the three states and
  asserts which one is read back.
- The statistics test, which asserts on an individual named discard counter
  rather than a total, and confirms no total is stored.

Run a single one by name:

```sh
cargo test -p fragcap-core udp
```

## What this slice does not do

Nothing here captures a packet, resolves an attribution, parses a header, or
writes a file. There is no binary to run and no output to inspect. If you are
looking for something to point at an interface, that arrives at S09.

The observable result of this slice is that `cargo doc -p fragcap-core --open`
shows a documented vocabulary, and that the checks above pass against it.
