# Implementation Plan: CLI targets convergence

**Branch**: `058-cli-targets-convergence` | **Date**: 2026-08-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/058-cli-targets-convergence/spec.md`

## Summary

Two `fragcap-cli`-only convergences onto the targets model. **#157**: make `--db`
optional on the `targets` subcommands (`add`/`show`/`remove`/`export`/`import`/
`list`), defaulting to the same local store the bare `fragcap targets` hero command
resolves. **#156**: extract the stored-target resolution seam currently private to
`capture.rs` into a shared module, and repoint the Wireshark extcap capture path at
it so extcap selects a stored target instead of resolving a retired profile file.
Remove the S057 extcap "legacy" callout from the CLI reference. No core/pipeline/
attribution change, no new dependency, no `Cargo.lock` delta.

## Technical Context

**Language/Version**: Rust (workspace MSRV 1.82; local build/test on the GNU
toolchain `cargo +1.96.0-x86_64-pc-windows-gnu ...` since there is no MSVC linker
here; CI runs the MSVC `--all-features` gate).

**Primary Dependencies**: none added. Reuses existing `paths` helpers, the
`fragcap::targets` resolution API, `fragcap::profile` types, and the extcap/assemble
machinery already in `fragcap-cli`.

**Storage**: SQLite `local.db` / `catalog.db` via the existing `Store` (no schema
change).

**Testing**: `cargo test -p fragcap-cli` (cli_targets.rs, cli_extcap.rs, cli_args.rs),
`cargo xtask ci`, `cargo xtask spec` (Applies-To lockstep), docs check. GNU locally.

**Target Platform**: Windows (the tool); the docs site is static-exported.

**Project Type**: CLI (Rust workspace) with a co-located docs site.

**Performance Goals**: N/A.

**Constraints**: fragcap-cli only. Extcap analyzer wire contract (interfaces, DLTs,
config block as arg lines, FIFO streaming) unchanged; only the one selection arg's
meaning changes. UTF-8 no BOM, LF, no em/en dashes (including comments). No
`Cargo.lock` delta.

**Scale/Scope**: ~5 arg-struct field changes + ~6 handler default-resolutions
(#157); one new module extracting ~7 fns + extcap repoint + `ExtcapArgs` additions +
`config_block` change (#156); test updates in cli_targets.rs and cli_extcap.rs; one
CLI-reference doc edit; spec section 17 reconcile + `xtask spec`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 (technique denylist / observe-only)**: Not engaged. No capture/attribution/
  handle/injection code changes; the extcap path already captures, only its target
  resolution changes. PASS.
- **P-4 (every discard counted)**: Not engaged (no pipeline/drop change). PASS.
- **P-6 (new term -> glossary same change)**: The slice reuses existing vocabulary
  (target, selector, handle, local store). No new user-facing term expected; if one
  appears it gets a glossary entry. PASS (verified in tasks).
- **P-9 (no fabrication / honest reporting)**: extcap converges to the same honest
  target resolution `capture` uses; no dead/fake path. The removed profile-file path
  was the stale one. PASS.
- **P-11 (spec describes what shipped)**: The extcap "legacy" callout is removed
  because the behavior converges; spec section 17 is reconciled and `cargo xtask
  spec` keeps the Applies-To lockstep. PASS.
- **Architecture (fragcap-core takes no platform dep; deps flow concrete->abstract)**:
  Unchanged; the change is entirely in `fragcap-cli`, the top of the graph. `cargo
  xtask deps` unaffected. PASS.
- **Compatibility / wrappers thin**: The extcap wire contract is preserved, so
  unmodified Wireshark still drives fragcap. PASS.
- **Pinned artifacts**: none touched (no workflows/toolchain/release.toml/scripts).
  PASS.
- **Encoding / no dashes**: enforced across edited files. PASS (verified in tasks).

No violations. Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/058-cli-targets-convergence/
├── plan.md              # This file
├── spec.md              # Feature spec
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   ├── targets-db-default.md     # The default-db resolution contract (#157)
│   └── extcap-target-selection.md# The extcap config + resolution contract (#156)
└── checklists/
    ├── requirements.md
    └── convergence.md
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/
├── cli.rs                        # TargetsAddArgs/ShowArgs/ExportArgs + List/Import db -> Option; ExtcapArgs gains target inputs; config-arg rename
├── commands/
│   ├── target_resolve.rs         # NEW: extracted shared resolution seam (pub(crate))
│   ├── capture.rs                # calls the extracted seam (behavior preserved)
│   ├── extcap.rs                 # capture handler uses the seam; config_block selection arg -> target
│   ├── targets.rs                # add/show/remove/export/import/list resolve default db when omitted
│   └── mod.rs                    # declare target_resolve
└── paths.rs                      # unchanged (reuse local_db_path/default_local_db_path)

crates/fragcap-cli/tests/
├── cli_targets.rs                # add default-db (no --db) coverage, isolated FRAGCAP_LOCAL_DB
├── cli_extcap.rs                 # offline_substrate -> register target + selector + --local-db; config-block assertions
└── cli_args.rs                   # (unchanged unless a surface assertion shifts)

site/content/docs/reference/cli.mdx  # replace the extcap "legacy" callout with converged options
docs/fragcap-specification.md        # section 17 extcap/targets reconcile; `cargo xtask spec`
changelog.d/S058-*.md               # changelog fragment
```

**Structure Decision**: Introduce `commands/target_resolve.rs` as the single home
for stored-target resolution, so `capture` and `extcap` share one implementation
(FR-009) and slice S059 has one clean place to add its unresolved-entry branch. All
other changes are edits to existing files; no new crate, no new dependency.

## Complexity Tracking

No constitution violations; no entries.
