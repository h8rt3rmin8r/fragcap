---

description: "Task list for slice S063, catalog namespace convergence"
---

# Tasks: Catalog namespace convergence

**Input**: Design documents from `/specs/063-catalog-convergence/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md)

**Tests**: Required. The store-path guard (FR-006) is written before the flags
change, so it is observed failing against the current declarations; the same
discipline S062 used, and for the same reason.

## Format: `[ID] [P?] [Story] Description`

- **[Story]**: US1 store paths, US2 `doctor`, US3 the merged seed verb

---

## Phase 1: Remove the dead command (#175)

Goes first: it deletes one of the nine flags US1 must change and one of the
verbs US3 must merge.

- [x] T001 Delete `CatalogCommand::Update` from `cli.rs`, its dispatch arm in
  `commands/catalog.rs`, `fn update`, and the `#[cfg(not(feature = "net"))]`
  error arm. Satisfies FR-007.
- [x] T002 Delete `update_default`, whose only caller is the `FetchCatalog`
  action being replaced in Phase 4. Keep `HttpCatalog`: `seed --tier catalog
  --steam` still uses it where `net` is present.
- [x] T003 Purge "the current published catalog" and every Cargo feature name
  from user-facing strings in `cli.rs` and `commands/catalog.rs`. No such
  artifact is published. Satisfies FR-008 and FR-009 in part.
- [x] T004 Update the specification and the site reference. The specification
  has **no `catalog update` entry**; issue #175's line 1898 citation is wrong.
  The two real sections are **15.7**, which names `catalog seed-signatures`, and
  **26.3**, which says "the npcap and catalog fetch actions are network-gated
  and degrade in a default build" (FR-010 reverses that for the catalog action).
  Also `site/content/docs/reference/cli.mdx:215`. This slice is spec-impacting,
  unlike S061 and S062, so the changelog fragments carry a real `spec-impact`
  list naming 15.7 and 26.3, and `cargo xtask spec` will enforce it. Satisfies
  FR-009.

---

## Phase 2: User Story 1 - Store paths become overrides (#179, P1)

- [x] T005 [US1] Extend the S062 enumeration in `cli_help.rs`: walk the clap
  command tree and assert no argument named `db`, `catalog-db`, or `local-db`
  is required, using `Arg::is_required_set`. Satisfies FR-006.
- [x] T006 [US1] Confirm the guard fails, naming every offending subcommand.
  Expected: the eight surviving declarations. A guard that passes here has been
  written wrong.
- [x] T007 [US1] Promote the S058 `default_local_store` resolution in
  `targets.rs` to a shared helper beside `ensure_catalog_store`, so `targets
  discover` and `targets` use one implementation of the local-store precedence.
  Satisfies FR-002 for the local store.
- [x] T008 [US1] Change the eight surviving fields to `Option<PathBuf>` and
  resolve each through the shared helpers: `catalog import --db`, `catalog
  export --db`, `catalog seed --db`, `technologies --catalog-db`, `targets
  discover --catalog-db` and `--local-db`. `catalog import`'s positional seed
  stays required; it is user data, not a path fragcap owns. Satisfies FR-001.
- [x] T009 [US1] Make the unresolvable case a clean failure naming what could
  not be resolved, not a clap usage error. `ensure_catalog_store` already
  returns `Ok(None)` for it. Satisfies FR-003.
- [x] T010 [US1] Confirm FR-004's asymmetry is preserved: a defaulted store is
  created with parents, an operator-named path is opened as given and never
  created by a read-only command. Inherited from `ensure_catalog_store` rather
  than reimplemented; assert it rather than assume it.
- [x] T011 [US1] Make every success line name the resolved store. Satisfies
  FR-005.
- [x] T012 [US1] Add the default-store assertions to `cli_bootstrap.rs`, which
  already drives `FRAGCAP_CATALOG_DB`: no flag resolves the default, an explicit
  flag beats the override, the override beats the default.
- [x] T013 [US1] Confirm T006's guard is now green, with no edit to the guard.

---

## Phase 3: User Story 3 - One seed verb (#180, P2)

- [x] T014 [US3] Add `--tier` to `catalog seed`: a repeatable value enum over
  `SeedTier`'s four members, including `launch`, so the fourth member needs no
  fifth top-level verb. Satisfies FR-013.
- [x] T015 [US3] Collapse the `catalog_source` and `engine_source` `ArgGroup`s
  into one group over `--from`, `--steam`, and `--pcgamingwiki`, keeping the
  mutual exclusion that already refuses an ambiguous invocation.
- [x] T016 [US3] Require exactly one `--tier` with `--from`; zero or many is a
  usage error at exit 2. Never sniff the document: both offline documents are
  bare JSON arrays with no discriminator, so a guess writes the wrong columns
  silently (P-9). Satisfies FR-014.
- [x] T017 [US3] Make bare `catalog seed` fill every tier reachable with no
  source flag and name every skipped tier with its reason. A silent skip is a
  P-4 defect. Satisfies FR-015.
- [x] T018 [US3] Delete `CatalogCommand::SeedEngine` and
  `CatalogCommand::SeedSignatures` and route their bodies through the merged
  command. Satisfies FR-012.
- [x] T019 [US3] Confirm the `SeedSummary` counters and their meanings are
  unchanged. Satisfies FR-016.
- [x] T020 [US3] Update `.github/workflows/release.yml:119` to `catalog seed
  --tier signature`. A pinned artifact, so it carries a dated decision fragment.
  Satisfies FR-017 in part.
- [x] T021 [US3] Update `docs/glossary/anti-cheat-and-security.md:72`, which
  names `fragcap catalog seed-signatures`, and the `catalog` section of
  `site/content/docs/reference/cli.mdx`. The "catalog seeder" and "engine
  seeder" entries in `process-and-attribution.md` name internal components that
  survive the merge unchanged and need no edit. Satisfies FR-018.

---

## Phase 4: User Story 2 - `doctor` offers something the binary can do (#175, P1)

Last, because it names the merged command from Phase 3.

- [x] T022 [US2] Rename `ActionKind::FetchCatalog` to
  `ActionKind::InitializeCatalog`, remove it from `net_required()`, give it a
  primary label describing the offline initialize-and-seed, delete its
  `degraded_label` arm, and stop `is_guidance_only()` special-casing it. It can
  no longer degrade, because it needs nothing to degrade from. Satisfies FR-010.
- [x] T023 [US2] Rewrite the `catalog store` remediation in `checks.rs:397` to
  name the merged seed command. Satisfies FR-010.
- [x] T024 [US2] Point the `--fix` performer at the same code path the command
  uses, against the resolved default store, as `update_default` did for the
  fetch.
- [x] T025 [US2] Leave the npcap actions unchanged. Their degraded text
  truthfully offers the official download page and says the build cannot fetch,
  which is not the state #175 objects to. Assert by reading, and record the
  decision. Satisfies FR-011.
- [x] T026 [US2] Add the assertion to `cli_doctor.rs`: with the catalog store
  absent, the offered remediation is performable by the running binary and
  produces a populated signature table with no network.

---

## Phase 5: Record and verify

- [x] T027 Write `changelog.d/S063-catalog-convergence.changed.md` and
  `.removed.md`, both carrying a real `spec-impact` section list, since this
  slice edits `docs/fragcap-specification.md`.
- [x] T028 Write `changelog.d/S063-catalog-convergence.decisions.md` recording:
  the release-workflow change (a pinned artifact); the correction to the
  operator's offline-path mechanism, with the measurement that forced it (the
  shipped catalog has zero title records); and the decision to decline #175's
  request to enable `net` in release builds or delete the npcap fetch, with the
  reasoning that its degraded form is already truthful.
- [x] T029 Run `cargo xtask ci` in the foreground, watched to completion.
- [x] T030 Verify by running, not reading: every affected command with no store
  flag; the precedence ladder; `doctor` with the catalog store deleted and
  `--fix` performing the offered action with no network.
- [x] T031 **Execute the release workflow's catalog build step locally** against
  the new grammar. Repository memory records that release infrastructure names
  CLI subcommands and that `cargo xtask ci` does not cover it, so reading the
  diff is not verification. Satisfies FR-017 and SC-005.
- [x] T032 Confirm the S062 help guard and lint rule are still green over the
  rewritten `catalog` block. This is the first real test of whether that guard
  was worth building.
- [x] T033 Stage only this slice's files and commit. Never stage
  `.specify/feature.json`; never edit `CHANGELOG.md` from a feature branch.

---

## Dependencies

- Phase 1 blocks Phases 2 and 3: it removes a flag one must change and a verb
  the other must merge.
- T005 and T006 precede T008: the guard is written to fail first.
- Phase 4 follows Phase 3, because the remediation names the merged command.
- Phase 5 follows everything, and T031 follows T020.

## Out of scope

Per `spec.md`: adding `net` to the release feature set (OOS-001), deleting
S056's npcap fetch (OOS-002), populating the shipped catalog's title tier
(OOS-003), and the #183 help accuracy audit (OOS-004).
