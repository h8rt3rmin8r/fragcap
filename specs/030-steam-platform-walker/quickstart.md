# Quickstart: Validating the Steam Platform-Walker

This slice adds no new capture CLI surface (see the plan's scope boundary), so
validation is at the library and resolver level: the walker provider flows through
the S027 cascade, composes with the engine rule over a Steam install directory,
answers directly for a single-client non-engine title, and degrades to runtime
observation on a not-installed, ambiguous, or unreadable install. All scenarios run
with `cargo test`; the full gate is `cargo xtask ci`.

## Prerequisites

- Rust toolchain as pinned (`rust-toolchain.toml`), MSRV 1.82.
- No real Steam install, no npcap, no game: tests build fake Steam library trees
  and install directories under the system temp dir.

## Run the slice's tests

```bash
cargo test -p fragcap-steam walker
cargo test -p fragcap-profile target
```

Expected: the walker module tests and the `TargetOrigin::PlatformWalker` tests
pass, covering:

- A single-client non-engine install resolves via the walker at
  `heuristic-unverified` with provenance `steam-library`.
- An install with only installers and launchers (no plausible client) declines.
- An install with several plausible clients declines as ambiguous with the count
  recorded.
- An unreadable install declines with the path recorded.

## Validate composition and degradation through the resolver

```bash
cargo test -p fragcap-steam --test walker_cascade
```

Expected (fake Steam library composed with engine-rule fixtures):

- A Steam-installed Unreal title, with the walker supplying `install_root`,
  resolves to the shipping executable via the engine rule (which outranks the
  walker).
- A Steam-installed single-client non-engine title resolves via the walker.
- A requested title that is not installed, an ambiguous install, and an unreadable
  install each decline at the walker and resolve via the observation provider when
  a matching process tree is present.
- An authored profile for the same title outranks both the engine rule and the
  walker.

## Dependency-direction gate

```bash
cargo xtask deps
```

Expected: green. The walker provider lives in `fragcap-steam` (edge
`fragcap-steam -> fragcap-profile`, allowed); `fragcap-profile` has gained no
dependency on `fragcap-steam`.

## Full repository gate

```bash
cargo xtask ci
```

Expected: green (fmt, clippy with warnings denied, workspace tests, conventions
lint including the forbidden process-handle APIs which the walker does not name,
the dependency-direction check, license, wrappers, docs check). Also run where a
toolchain is present:

```bash
cargo xtask msrv       # builds at 1.82; exits 2 if the toolchain is absent
cargo xtask neutral    # fragcap-core builds with no backend; exits 2 if absent
```

Report an exit-2 skip honestly rather than as a pass.

## Documentation check

The "platform walker" glossary entry, the section 15.7 walker subsection, and the
section 16 reframe land in this slice. The documentation linter enforces glossary
completeness and cross-links:

```bash
bash scripts/lint-docs.sh check
```

Expected: green, with "platform walker" defined and cross-linked.
