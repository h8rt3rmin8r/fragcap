# Implementation Plan: Catalog namespace convergence

**Branch**: `063-catalog-convergence` | **Date**: 2026-08-20 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/063-catalog-convergence/spec.md`

## Summary

Make every store path an override rather than a requirement, delete the command
no shipped binary can run, replace the network remediation `doctor` offers with
an offline one it can actually perform, and collapse three seed verbs into one
`--tier` flag.

Order: the removal first, because it deletes one of the flags the next step has
to change and one of the verbs the step after that has to merge; then the flags;
then the merge; then `doctor`, which consumes the merged command's name.

## Branch base

**This slice is stacked on `062-help-surface`, not on `main`.** Two reasons, and
the second is a hard dependency rather than a convenience:

1. Both slices rewrite `crates/fragcap-cli/src/cli.rs` heavily. S062 rewrote
   nearly every doc comment on the surface; this slice rewrites the `catalog`
   block's declarations. Branching from `main` would guarantee a conflict in
   that file at merge time.
2. FR-006 extends the enumerating guard in `crates/fragcap-cli/tests/cli_help.rs`
   with a required-store-flag assertion. That guard does not exist on `main`;
   S062 created it. The requirement is literally unimplementable from `main`.

The pull request therefore targets `062-help-surface` and should merge after it.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82.

**Primary Dependencies**: none added. `http_req` stays where it is, behind
`net`, unused by any shipped build (OOS-001, OOS-002).

**Storage**: SQLite through `rusqlite`, behind the `targets` feature. Two
stores: `catalog.db` (shipped, disposable) and `local.db` (user-owned).

**Testing**: `crates/fragcap-cli/tests/cli_bootstrap.rs` (already drives
`FRAGCAP_CATALOG_DB`, the natural home for the default-store assertions),
`cli_targets.rs` (the S058 precedent for `FRAGCAP_LOCAL_DB`), `cli_doctor.rs`,
and `cli_help.rs` (the S062 enumerating guard, which gains the required-flag
assertion).

**Target Platform**: Windows for the product; the resolution logic is
platform-neutral and the default path is per-user.

**Project Type**: CLI.

**Constraints**: `.github/workflows/release.yml` is a pinned artifact and names
these subcommands. Changing it requires a dated decision fragment and, per
repository memory, is a required step rather than a follow-up because
`cargo xtask ci` does not cover it.

**Scale/Scope**: nine argument declarations, one command removed, two verbs
merged into one, one `doctor` check and one action kind, one workflow step, one
documentation page, one glossary entry, two specification sections.

## Phase 0: Research (complete)

Four things were measured rather than assumed, and two of them changed the plan.

**1. The `catalog store` check fires on absence, not emptiness.**
`checks.rs:388` returns `None` when `inputs.catalog_db_present`. An absent store
is exactly what the first-run bootstrap creates, and its signature table is
exactly what `seed-signatures` fills from a compiled-in document. Both are
offline. **The remediation was wired to a network fetch for a condition that
never needed one**, which is why FR-010 replaces the action rather than
redirecting it.

**2. The published `catalog.db` has zero title records.** `assets/hint-seed.json`
carries `"records": []` and is what `release.yml:115` imports. So the operator's
recorded offline path ("download `catalog.db` from the releases page and run
`catalog import`") would hand the user the same empty store they already have.
The decision's substance survives; its mechanism is corrected to "create it
locally", recorded in the spec's clarifications.

**3. Only the catalog arm of `doctor` is dishonest.** `action.rs:97` degrades
`ObtainNpcap` to "Open the official download page for npcap ...; this build
cannot fetch the installer", which is truthful and actionable. `action.rs:103`
degrades `FetchCatalog` to "run `fragcap catalog update` with a net-enabled
build", which asks a user holding a binary to become a Rust developer. This
bounds the slice: FR-011 leaves the npcap arms alone, and OOS-002 declines
#175's request to delete the fetch, because #175's objection is to a promise the
binary cannot keep and `ObtainNpcap` makes no such promise.

**4. `update` is `seed --steam` with a hardcoded threshold.** `catalog.rs:110`
and `catalog.rs:209` call the same function; the endpoint is
`https://steamspy.com/api.php?request=all`. Deleting it loses no capability that
`seed --tier catalog --steam` does not already provide in a build that has it.

## Constitution Check

*GATE: passed before Phase 0. Re-checked after design; still passing.*

| Principle | Bearing | Verdict |
| --- | --- | --- |
| P-1 Passive Observation Only | No capture or process code. Removing the only user-facing network command reduces outbound surface. | Satisfied |
| P-2 Core Stays Platform-Neutral | All changes in `fragcap-cli`; `fragcap-core` untouched. | Satisfied |
| P-3 Capture And Attribution Stay Separate | No crate-boundary change. | Not engaged |
| P-4 No Silent Loss | FR-015: bare `catalog seed` names every tier it skipped and why. FR-016: the seed counters are unchanged. A merged verb that quietly filled fewer tiers would be the configuration-side form of an uncounted discard. | **Satisfied by design** |
| P-5 Compatibility Outranks Richness | Output formats unchanged. `catalog export` still emits the schema-conformant document. | Not engaged |
| P-6 Glossary First | FR-018: `docs/glossary/anti-cheat-and-security.md:72` names `catalog seed-signatures` and moves with the verb. The "catalog seeder" and "engine seeder" entries name internal components that survive the merge and need no edit. No new term; `--tier` names an existing model concept (`SeedTier`). | **Gated** |
| P-7 Wrappers Stay Thin | Wrappers pass through and do not name `catalog` subcommands; the `cargo xtask ci` wrapper checks confirm. | Satisfied |
| P-8 House Standards Apply | UTF-8 no BOM, LF, no em-dashes or en-dashes. | **Gated** |
| P-9 The Instrument Does Not Lie (NON-NEGOTIABLE) | The driver. "The current published catalog" names an artifact that does not exist (FR-009); a remediation the binary cannot perform is a false promise (FR-010); FR-014 refuses to guess a tier from an ambiguous document, because guessing writes the wrong columns silently. | **Primary driver** |
| P-10 One Path To A Target | Not target creation, but the reasoning transfers: one seed verb rather than one per tier keeps the surface from multiplying by the number of sources. | Satisfied |
| P-11 The Specification Describes What Shipped | The specification carries no `catalog update` entry (#175's line 1898 citation is wrong), but **15.7** names `catalog seed-signatures` and **26.3** states the catalog fetch action is network-gated and degrades, which FR-010 reverses. Both must change with the code. That makes this slice spec-impacting, unlike S061 and S062. | **Engaged; see below** |

No violations. Complexity Tracking omitted.

**P-11 consequence**: the changelog fragments for this slice carry a real
`spec-impact` section list rather than `none`, and `cargo xtask spec` will
require `docs/fragcap-specification.md` to have changed in the release diff.
This is the first slice in the campaign where that applies.

## Design

### 1. Remove `catalog update` (FR-007 to FR-009)

Delete `CatalogCommand::Update`, its dispatch arm, `fn update`, and the
`#[cfg(not(feature = "net"))]` error arm. `update_default` exists only for the
`doctor --fix` `FetchCatalog` action, which is being replaced, so it goes too,
and with it the last caller of `HttpCatalog` outside the seeders.

`HttpCatalog` itself stays: `seed --tier catalog --steam` still uses it in a
build that has `net`.

Purge "the current published catalog" from `cli.rs`, `doctor/checks.rs:397`,
specification section 26.3, and `site/content/docs/reference/cli.mdx:215`.
`CHANGELOG.md` keeps it: those lines are the historical record of releases that
did ship the command, and it is never edited from a feature branch.

### 2. Store paths become overrides (FR-001 to FR-006)

Eight fields survive the removal and become `Option<PathBuf>`. Each command
resolves through one shared helper rather than repeating the precedence:

- catalog store: `target_resolve::ensure_catalog_store(flag)`, which already
  implements flag, then environment, then per-user default with the first-run
  bootstrap, and already returns `Ok(None)` for "no location determinable" so
  FR-003's clean failure has a place to attach.
- local store: the S058 `default_local_store` path in `targets.rs`, promoted to
  a shared helper so `targets discover` uses the same one.

FR-004's asymmetry is already `ensure_catalog_store`'s behavior (operator-named
returned as given, default seeded) and is inherited rather than re-implemented.

`catalog import`'s positional seed stays required: it is user data, not a path
fragcap owns.

The guard (FR-006) extends the S062 enumeration in `cli_help.rs`: walk the clap
command tree and assert no argument named `db`, `catalog-db`, or `local-db` is
required. It reuses `fragcap_cli::command()` and clap's `Arg::is_required_set`,
so it needs no new machinery.

### 3. One seed verb (FR-012 to FR-017)

```
fragcap catalog seed                        # every tier fillable with no source flag
fragcap catalog seed --tier engine --from <doc>
fragcap catalog seed --tier catalog --steam           # a build with network access
fragcap catalog seed --tier engine --pcgamingwiki     # a build with network access
```

`--tier` is a repeatable value enum over `SeedTier`'s four members. `--from`
requires exactly one `--tier`; zero or many is a usage error at exit 2 (FR-014).
Bare `seed` fills the signature tier from the compiled-in document and names
each skipped tier with its reason (FR-015).

The existing `catalog_source` / `engine_source` `ArgGroup`s collapse into one
group over `--from`, `--steam`, and `--pcgamingwiki`, keeping the mutual
exclusion that already refuses an ambiguous invocation.

`release.yml:119` becomes `catalog seed --tier signature`, and FR-017 requires
running it rather than reading it.

### 4. `doctor` offers something the binary can do (FR-010, FR-011)

`ActionKind::FetchCatalog` becomes `ActionKind::InitializeCatalog`:

- removed from `net_required()`, so it never degrades;
- `primary_label` becomes an initialize-and-seed sentence;
- `degraded_label` loses its arm entirely, since it cannot degrade;
- `is_guidance_only()` (currently true for a degraded `FetchCatalog`) no longer
  special-cases it: it is always performable.

`checks.rs:397`'s remediation becomes the merged seed command. The `--fix`
performer calls the same code path the command does, against the resolved
default store, which is what `update_default` did for the fetch.

The npcap arms are untouched (FR-011).

## Project Structure

```text
specs/063-catalog-convergence/
├── spec.md
├── plan.md                    # this file, carrying Phase 0 inline
├── tasks.md
└── checklists/requirements.md
```

No `research.md`: Phase 0 produced four measured answers, short enough to sit
beside the design that depends on them. No `data-model.md`; `SeedTier` and
`SeedSummary` are unchanged, which is FR-016.

### Files changed

```text
crates/fragcap-cli/src/cli.rs                     # 8 fields, --tier, Update removed
crates/fragcap-cli/src/commands/catalog.rs        # dispatch, merged seed, update removed
crates/fragcap-cli/src/commands/target_resolve.rs # shared local-store helper
crates/fragcap-cli/src/commands/targets.rs        # discover resolves both stores
crates/fragcap-cli/src/commands/technologies.rs   # catalog store resolution
crates/fragcap-cli/src/doctor/action.rs           # InitializeCatalog
crates/fragcap-cli/src/doctor/checks.rs           # the remediation sentence
crates/fragcap-cli/tests/cli_bootstrap.rs         # default-store assertions
crates/fragcap-cli/tests/cli_help.rs              # the required-flag guard
crates/fragcap-cli/tests/cli_doctor.rs            # the offline remediation
.github/workflows/release.yml                     # the seed invocation
docs/fragcap-specification.md                     # sections 15.7 and 26.3
docs/glossary/anti-cheat-and-security.md          # names catalog seed-signatures
site/content/docs/reference/cli.mdx               # the catalog section
changelog.d/S063-catalog-convergence.*.md
```

**Structure Decision**: no new module. The merged seed lives where the three
verbs lived.

## Verification

`cargo xtask ci` in the foreground, watched to completion. Then, against the
rebuilt binary:

1. every affected command with no store flag, on a machine with a resolvable
   per-user directory;
2. the precedence ladder, flag over environment over default;
3. `doctor` with the catalog store deleted, and `--fix` performing the offered
   action with no network;
4. **the release workflow's catalog build step, executed locally** against the
   new grammar, per FR-017 and the repository memory that release infrastructure
   names CLI subcommands and `cargo xtask ci` does not cover it.

The S062 help guard and lint rule must stay green over the rewritten `catalog`
block, which is the first test of whether that guard was worth building.
