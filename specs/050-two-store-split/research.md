# Research: The two-store split (S050)

Phase 0 decisions. Each resolves an unknown for the plan.

## R-1: One provider reads both stores (local first, then catalog)

**Decision**: `HintDatabaseProvider` holds an ordered `Vec<Store>` and queries them
in order, returning the first store's usable answer. `new(store)` stays (single
store, for existing tests); add `new_layered(Vec<Store>)`. The CLI builds it with
`new_layered(vec![local, catalog])`, so `local.db` (this machine's learned data)
is consulted before `catalog.db` (the shipped seed).

**Rationale**: `TargetResolver::new` rejects two providers at the same precedence,
and `HintDatabaseProvider` occupies the single `Precedence::HintDatabase` slot. A
second provider would conflict. One provider over both files keeps the one slot,
preserves the existing decline logic (launcher-mediated, ambiguous, engine-only),
and gives learned data priority over the seed. The full fidelity-ordered cascade
collapse is S051; this is the minimal dual-read that prevents an attribution
regression.

**Alternatives**: a precedence parameter to place two providers (adds cascade
surface S051 will rework); merging both stores into one at open time (a runtime
merge is exactly the mixing the split removes).

## R-2: First-run bootstrap creates both stores

**Decision**: Generalize `ensure_default_hint_db(default, template)` to
`ensure_store(path, template)`. Bootstrap `catalog.db` from the beside-binary
template (as today), and bootstrap `local.db` with `template = None` (an empty
current-schema store). Both live in the per-user AppData root, writable, no
elevation. `copy_writable_hint_db` (clearing the read-only attribute the MSI
template carries) is reused for the catalog copy.

**Rationale**: This is the "rename and redirect" the handoff describes: the
existing bootstrap already does exactly this for one file; it now runs for two.
`local.db` never has a template (it is the user's, created empty).

## R-3: Store paths

**Decision**: `paths.rs` gains `default_catalog_db_path()` and
`default_local_db_path()` (`%APPDATA%\fragcap\catalog.db` and `local.db`), factored
through a shared `default_db_from(appdata, filename)`. Overrides:
`catalog_db_path(flag)` and `local_db_path(flag)`, each flag-over-env with
`FRAGCAP_CATALOG_DB` and `FRAGCAP_LOCAL_DB`. `HINT_DB_ENV` (`FRAGCAP_HINT_DB`) is
removed.

**Rationale**: mirrors the existing single-path helper, one per store, so the
bootstrap and the resolver address each file independently and tests can point
either at a scratch path.

## R-4: CLI flags

**Decision**: replace `--hint-db <path>` with `--catalog-db <path>` and
`--local-db <path>`; no `--hint-db` alias (v0.5.0 ships no deprecation shims).

**Rationale**: the single hint store became two; each needs an override for
testing and advanced use. The clarify session settled this.

## R-5: Accumulation target

**Decision**: `accumulate_launch` writes to `local.db`. `run` accumulates into the
local store before building the resolver.

**Rationale**: learned launch data is user-owned; it must survive a catalog
refresh (FR-006, FR-007), so it lives in `local.db`.

## R-6: The seed file is renamed hint.db to catalog.db

**Decision**: the beside-binary seed is `catalog.db`. Rename in
`bundled_hint_db_template` (sibling `catalog.db`), the WiX component/file/source,
and the release workflow's build/stage/archive/checksum steps.

**Rationale**: the shipped seed IS the catalog; the name should say so, and the
first-run copy targets `catalog.db`.

## R-7: No migration

**Decision**: an existing `hint.db` in AppData is neither read nor written; both
stores are created fresh. No migration code.

**Rationale**: the user base is two people who can delete a folder (handoff 2.2).

## R-8: doctor reports both stores

**Decision**: the doctor data-directory report names both `catalog.db` and
`local.db` (and their presence), replacing the single hint-db line (issue #106).

**Rationale**: doctor's honesty guarantee (v0.4.0) must reflect the real layout.

## R-9: Pinned artifacts and the spec

**Decision**: `.github/workflows/release.yml` is pinned and names `hint.db`
(builds the barebones store, stages it, archives it, checksums it); its rename to
`catalog.db` lands with a dated `decisions` fragment. `crates/fragcap-cli/wix/main.wxs`
is not in the pinned list, so it needs no decisions fragment (a `changed`
fragment covers it). Specification sections describing the store (15.x, 24.5) are
edited in this slice, and the changelog fragment naming those sections satisfies
the S049 release gate.

## R-10: Glossary (P-6)

**Decision**: add glossary entries for `catalog.db` and `local.db` under
`platform-and-distribution.md`, update the existing hint-database entry to point
at them, and regenerate the index.

**Rationale**: P-6 requires a glossary entry in the change that introduces a term;
the doc linter (in `cargo xtask ci`) enforces it.

## R-11: Verification in this environment

**Decision**: build, test, check, clippy, and fmt run under the GNU-host toolchain
`1.96.0-x86_64-pc-windows-gnu` (`cargo +1.96.0-x86_64-pc-windows-gnu ...`).

**Rationale**: this host has no MSVC linker, and build scripts and proc-macros
(serde, rusqlite) compile for the host, so the pinned msvc toolchain cannot link
them here. The GNU-host toolchain links build scripts and the bundled SQLite
through the on-PATH mingw gcc; it builds `fragcap-targets` and `fragcap-cli` and
runs their tests. The pinned msvc build is verified by CI.
