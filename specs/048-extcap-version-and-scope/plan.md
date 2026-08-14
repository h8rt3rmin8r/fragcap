# Implementation Plan: exe FileVersion stamp and extcap scope flags

**Branch**: `048-extcap-version-and-scope` | **Date**: 2026-08-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/048-extcap-version-and-scope/spec.md`

## Summary

Two changes in `fragcap-cli` that clear the remaining items of issue #104:
(A) stamp the exe PE FileVersion from the crate version via a Windows-only
`winresource` build-dependency in `build.rs`; (B) add `--user`/`--system` scope
flags to `extcap install`/`uninstall`, reusing the existing per-user and
machine-wide path resolvers, with `--dir` kept as an override. No pinned artifact
changes; the MSI stays on `--dir`; the doctor strings and golden are unchanged.

## Technical Context

**Language/Version**: Rust (workspace MSRV 1.82). Change in `fragcap-cli` only.

**Primary Dependencies**: adds `winresource 0.1.31` with `default-features = false`
as the workspace's first build-dependency (Windows target only). This adds two
packages to `Cargo.lock` (`winresource`, `version_check`), both MIT/Apache-2.0;
`default-features = false` drops the optional `toml` (MSRV 1.85). No runtime dep
is added.

**Testing**: `cargo test -p fragcap-cli` (the extcap install/uninstall integration
tests and the doctor classifier tests), `cargo xtask ci`, and `cargo xtask msrv`
(the crux: the build-dep must compile under 1.82).

**Target Platform**: Windows-MSVC for the stamped resource; the classifier/CLI is
platform-neutral and the scope resolvers already exist.

**Project Type**: Rust workspace (CLI crate).

**Constraints**: MSRV 1.82 (winresource must compile under it; fallback hand-rolled
`.rc`); license allowlist MIT/Apache/BSD/ISC/Unicode-DFS/Zlib; the resource step
runs only on Windows-MSVC and must not break `cargo xtask neutral` (which does not
build fragcap-cli) or the MSRV build; no pinned artifact touched; UTF-8, LF, no
dashes.

**Scale/Scope**: one `build.rs` restructure + a `stamp_version_resource()` fn, one
`Cargo.toml` build-dep table, a regenerated `Cargo.lock`, one args-struct change,
one resolver change, new tests, one docs section, one AGENTS.md row, two changelog
fragments.

## Constitution Check

- **P-1 Passive Observation**: No capture behavior changes. PASS.
- **P-2 Core Platform-Neutral**: `fragcap-core` untouched; the Windows build-dep is
  in `fragcap-cli`. PASS.
- **P-3 Capture/Attribution Separate**: unaffected. PASS.
- **P-6 Glossary First**: no new term. PASS.
- **P-8 House Standards**: UTF-8, LF, no dashes; dependency recorded in the
  inventory. PASS.
- **P-9 The Instrument Does Not Lie**: the exe stops reporting a false `0.0.0.0`
  version. PASS.

No violations. `cargo xtask deps` ignores build-dependencies, so the first
`[build-dependencies]` does not perturb the runtime-graph gate; the only new gate
interaction is MSRV, addressed explicitly.

## Project Structure

### Documentation (this feature)

```text
specs/048-extcap-version-and-scope/
├── spec.md
├── plan.md
├── research.md
├── quickstart.md
├── checklists/requirements.md
└── tasks.md
```

data-model.md and contracts/ omitted: no data entity; the CLI's contract (the
extcap command surface and doctor output) is covered by existing tests plus the
new scope tests.

### Source (paths touched)

```text
crates/fragcap-cli/build.rs                      # split guard; stamp_version_resource()
crates/fragcap-cli/Cargo.toml                    # [target.'cfg(windows)'.build-dependencies] winresource
Cargo.lock                                       # + winresource, version_check (regenerated)
crates/fragcap-cli/src/cli.rs                    # ExtcapInstallArgs: --user/--system in an ArgGroup
crates/fragcap-cli/src/commands/extcap.rs        # scope-aware resolve_dir; thread through install/uninstall
crates/fragcap-cli/tests/cli_extcap.rs           # --system/--user/conflict tests
site/content/docs/reference/cli.mdx              # document --user/--system
AGENTS.md                                        # dependency-inventory row for winresource
changelog.d/048-*.added.md , 048-*.fixed.md      # scope flags (added) + FileVersion (fixed)
```

**Structure Decision**: Keep both changes in `fragcap-cli`. The version resource is
a `cfg(windows)` build-dep so no non-Windows graph is perturbed and the neutral
build (which does not compile fragcap-cli) is unaffected; the MSRV build is the one
that compiles it and is verified explicitly. The scope flags reuse the S044
resolvers so no new path logic is written.

## Design decisions (see research.md)

- `winresource` `default-features = false` chosen over `embed-resource` (which
  pulls `toml` unconditionally, breaking MSRV) and over a hand-rolled `.rc` (kept
  as the fallback if winresource fails under 1.82).
- Version resource stamped on every Windows-MSVC build (not `live`-gated); npcap
  linkargs stay `live`-gated. `if let Err -> cargo:warning` so a box without
  `rc.exe` still builds.
- Scope precedence: `--dir` > `--system` > `--user`/default; clap `ArgGroup`
  enforces at most one. MSI unchanged (keeps `--dir`).

## Complexity Tracking

No constitution violations; no entries.
