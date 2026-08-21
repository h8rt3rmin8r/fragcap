# Feature Specification: Catalog namespace convergence

**Feature Branch**: `063-catalog-convergence`

**Created**: 2026-08-20

**Status**: Implemented

**Input**: Make store paths optional everywhere, collapse the three catalog seed
verbs into one, and remove the command that no shipped binary can run. Closes
issues #175, #179, #180.

## Context

Three issues rewrite the same declarations in the `catalog` block of
`crates/fragcap-cli/src/cli.rs` and the same dispatch in
`crates/fragcap-cli/src/commands/catalog.rs`. #180's own acceptance says landing
it with #179 is cheaper than two passes, and #175's structural half removes one
of the nine flags #179 has to change. They are one slice.

The through-line is that the `catalog` namespace asks the user to know things
fragcap already knows, and offers one thing fragcap cannot do at all.

## Evidence

Measured 2026-08-20 against `main` at `e2d655d` (v0.5.1).

### Seven subcommands refuse to run without a path fragcap owns

`catalog import`, `catalog export`, `catalog seed`, `catalog seed-engine`,
`catalog seed-signatures`, `catalog update`, `technologies`, and `targets
discover` require an explicit filesystem path to a store that fragcap installs,
manages, and already knows how to find. Nine `PathBuf` fields, at `cli.rs:340`,
`:414`, `:417`, `:485`, `:491`, `:504`, `:510`, `:575`, `:610`.

The resolution machinery exists and is correct; it is simply not wired to these
commands. `paths::catalog_db_path` (flag, then `FRAGCAP_CATALOG_DB`),
`paths::default_catalog_db_path` (`%APPDATA%\fragcap\catalog.db`), the same pair
for `local.db`, and `target_resolve::ensure_catalog_store` (the full chain
including the first-run bootstrap that copies the template shipped beside the
exe). One command in the same module already does it right: `update_default`
(`catalog.rs:243`) resolves `catalog_db_path(None).or_else(default_catalog_db_path)`
and hands the result to the same function the flagged path calls. It exists only
because `doctor --fix` needed a no-argument entry point; the user-facing command
was left as it was.

Slice S058 (issue #157) established the opposite contract for `targets`: a store
path is an override, never a requirement. Its FR-005 explicitly scoped
`discover`'s two-store pattern out, and nothing was filed to pick it up.

### `catalog update` cannot run in any shipped binary, and describes something that does not exist

`.github/workflows/release.yml:63` is the only release build path and its feature
set is `live,socket-table,etw`. No `net`. So in every released artifact `catalog
update` fails at exit 2 with a message naming a Cargo feature.

It also claims to "fetch the current published catalog". It calls
`HttpCatalog::new()`, whose endpoint is `https://steamspy.com/api.php?request=all`
(`http_catalog.rs:25`). That is the maintainer's title-tier seed source. `update`
is therefore `seed --steam` with `min_reviews` hardcoded; compare `catalog.rs:110`
with `catalog.rs:209`, where the bodies are the same call. **No fragcap catalog
artifact is published anywhere**, so "the current published catalog" describes
nothing, in help text, in `doctor` output, in specification section 26.3, and in
`site/content/docs/reference/cli.mdx:215`. (#175 cites
`docs/fragcap-specification.md:1898` for this; that line number is wrong and the
specification carries no `catalog update` entry at all. See FR-009.)

### `doctor` sends the user to a dead end, and it is the only arm that does

`checks.rs:397` tells a user whose catalog store is absent to "run `fragcap
catalog update` to fetch the current published catalog", and when the build
cannot, `action.rs:103` offers:

> This build cannot fetch; run `fragcap catalog update` with a net-enabled build

The remediation for a shipped binary is to obtain a source checkout, a C
toolchain, and rebuild with a Cargo flag. For a tool whose distribution decision
(#58) is "raw exe plus barebones DB", that is not a remediation.

Worth stating precisely, because it bounds the fix: **this is the only dishonest
arm.** `ObtainNpcap` and `RelaunchNpcapInstaller` degrade truthfully
(`action.rs:97`, "Open the official download page for npcap ...; this build
cannot fetch the installer"), which is a real thing a user can do. Only
`FetchCatalog` names a rebuild.

### The remediation that was needed was never a fetch

The `catalog store` check fires when the store is **absent**, not when it is
empty (`checks.rs:388`, `if inputs.catalog_db_present { return None }`). An
absent store is created by the first-run bootstrap, which copies the template
shipped beside the exe or creates an empty store, and its signature table is
populated by `catalog seed-signatures` from a compiled-in document, offline and
idempotently. Both are things a shipped binary can do with no network at all.
The check was wired to a network fetch for a condition that never needed one.

### Three seed verbs are accretion, and the shipped catalog has no titles

`seed` (title tier), `seed-engine` (engine tier), and `seed-signatures` (the
signature table) arrived with the slices that needed them (S035, S036, S053) and
S054 moved all three verbatim from `targets` to `catalog`. No recorded decision
defends three verbs over one, and the scheme is already broken: `SeedTier` has a
member, `Launch` (`model.rs:137`), that no command reaches, so the pattern's own
next step is a fourth top-level verb. (#180 calls `Launch` "a fourth member";
the enum has three, `Catalog`, `Launch`, and `Engine`. `Signature` is not a
`SeedTier` at all, being a separate table with no source or cursor. The
substance holds: one tier has no verb, and the pattern's answer is another one.)

`seed-signatures` is not the same kind of operation as the other two: no source,
no cursor, no resume, no network. The release workflow already runs it at build
time (`release.yml:119`), so every shipped `catalog.db` arrives with the
signature table populated.

Measured, and it sharpens #180's closing question: `assets/hint-seed.json` has
**zero records**, and it is what `release.yml:115` imports to build the shipped
store. So a released `catalog.db` carries detection signatures and **no titles**,
and with `net` absent from the release build there is no compiled-in way to gain
any. This is not fatal (discovery resolved 33 titles on the developer machine
from Steam and signatures alone), but it means the title tier is a
maintainer-populated enrichment that is empty in every released build.

## Clarifications

### Session 2026-08-20

- Q: How does a user refresh `catalog.db`? -> A: **Operator decision, taken
  before this slice: drop `update` and keep no network code in the shipped
  binary.** Recorded here because it governs everything below.
- Q: The operator's decision named "download `catalog.db` from the releases page
  and run `fragcap catalog import`" as the offline path, following #175's option
  2. Measurement says that path is hollow: the release does publish `catalog.db`
  as a standalone download (`release.yml:211`), but it is the same
  zero-title store the user already has, so downloading and importing it changes
  nothing. -> A: **Keep the decision's substance, correct its mechanism.** The
  honest offline remediation is to *create* the store locally rather than fetch
  it: the bootstrap plus the compiled-in signature seed produce exactly what the
  shipped template contains, with no network and no download. `FetchCatalog`
  therefore becomes an offline action that always succeeds rather than a network
  action that always degrades. This satisfies the decision as given (no network
  code in the shipped binary, no dead end) by a better route than the issue
  proposed.
- Q: With `update` gone, `net` gates only the maintainer seeders and S056's
  npcap installer fetch. #175 says shipping a binary that structurally cannot
  perform the remediation it was authorized to perform is "the worst of the
  three states", and asks that the release build enable `net` or that the fetch
  be deleted. -> A: **Neither. Keep the fetch, keep `net` out of release
  builds, and rest on the fact that its degraded form is already truthful.**
  The operator's decision forbids network code in the shipped binary, which
  settles the first option. Deleting the fetch is the wrong reading of "the
  worst state": #175's complaint is a binary that *promises* a remediation it
  cannot perform, and `ObtainNpcap` promises nothing it cannot do, offering the
  official download page and saying plainly that this build cannot fetch. The
  code stays exercised in maintainer builds and under `--all-features` in
  continuous integration. Recorded as a decision fragment because it declines an
  explicit request in an issue.
- Q: `--from` names a different document in `seed` and `seed-engine`, and the two
  are not self-identifying: both are bare JSON arrays with no discriminator
  (`catalog.rs:89`, `engine_feed.rs:106`). -> A: **A merged command requires an
  explicit `--tier` with `--from`.** Sniffing by element keys is a guess, and a
  guess that picks the wrong tier writes the wrong columns silently, which is a
  P-9 defect rather than a convenience.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A user never types a path fragcap owns (Priority: P1)

Someone who installed fragcap runs `fragcap catalog export` and is told
`--db <DB>` is required, with nothing in the error saying what path to type.
Meanwhile `fragcap targets list` in the same binary resolves its store with no
flag at all.

**Why this priority**: it is the largest surface in the slice, it is a regression
against a contract S058 already established, and it is the one a first-time user
hits.

**Independent Test**: run each affected command with no flag on a machine with a
resolvable `%APPDATA%`.

**Acceptance Scenarios**:

1. **Given** a machine with a resolvable per-user directory, **When** any of the
   affected commands runs with no store flag, **Then** it resolves the same
   default store the hero command resolves and succeeds.
2. **Given** an explicit flag and an environment override, **When** a command
   runs, **Then** the flag wins over the override, which wins over the default.
3. **Given** no flag, no override, and no resolvable default, **When** a command
   runs, **Then** it fails cleanly naming what it could not resolve, not with a
   clap usage error.
4. **Given** a defaulted store that does not exist, **When** a command runs,
   **Then** it is created with its parents; **Given** an operator-named path that
   does not exist, **When** a read-only command runs, **Then** it is not created.
5. **Given** any successful run, **When** it reports, **Then** the line names the
   store that was read or written.

---

### User Story 2 - `doctor` offers a remediation the binary can perform (Priority: P1)

A user runs `fragcap doctor --fix` on a released binary with no catalog store and
is told to rebuild fragcap from source with a Cargo feature.

**Why this priority**: `doctor` exists to tell an operator what to do next.
Telling them to become a Rust developer is worse than saying nothing, and it is
the concrete dead end #175 was filed about.

**Independent Test**: delete the catalog store, run `doctor`, and perform what it
says using only the shipped binary.

**Acceptance Scenarios**:

1. **Given** an absent catalog store, **When** `doctor` reports, **Then** the
   remediation names a command the running binary can execute.
2. **Given** the offered action, **When** `doctor --fix` performs it, **Then** a
   catalog store exists with its signature table populated, with no network
   access.
3. **Given** a released binary, **When** any `doctor` output is read, **Then** no
   sentence names a Cargo feature or asks for a rebuild.
4. **Given** the npcap actions, **When** they degrade, **Then** they keep today's
   truthful text offering the official download page.

---

### User Story 3 - One seed verb, and the surface stops growing (Priority: P2)

A maintainer learns three commands where one would do, and a user reading
`catalog --help` sees `seed-signatures` presented as a peer step they are
expected to perform, when the release already ran it at build time.

**Why this priority**: real, but it is ergonomics for a mostly-maintainer
surface, below the two correctness stories.

**Independent Test**: seed each tier through the merged command and compare the
resulting store against what the three verbs produced.

**Acceptance Scenarios**:

1. **Given** the merged command, **When** `catalog seed` runs with no flags,
   **Then** it fills every tier that has a source available without one being
   named, and names every tier it skipped with the reason.
2. **Given** `--from` with zero or more than one `--tier`, **When** the command
   runs, **Then** it is a usage error at exit 2, never a guess.
3. **Given** any tier, **When** it is seeded, **Then** the `SeedSummary` counters
   and their meanings are unchanged.
4. **Given** the command surface, **When** it is enumerated, **Then**
   `seed-engine`, `seed-signatures`, and `update` are absent.

---

### Edge Cases

- **A defaulted store that cannot be initialized** must warn and degrade, never
  abort, matching the FR-005 behavior `ensure_catalog_store` already implements
  for the resolution path.
- **`catalog import` takes a user-supplied positional seed** which stays
  required; only its `--db` becomes optional. User data is not a path fragcap
  owns.
- **`targets discover` takes two stores**, not one. Both become optional, and
  each resolves through its own chain.
- **The release workflow names these subcommands.** `release.yml:119` invokes
  `catalog seed-signatures`. Repository memory records that release
  infrastructure names CLI subcommands and that `cargo xtask ci` does not cover
  it, so a rename that misses this file breaks the release build silently. It is
  a required step of this slice, not a follow-up.
- **A user with an empty title tier is not broken.** Discovery resolves titles
  from Steam and the signature table. The slice must not imply the catalog is
  required for capture, because it is not.

## Requirements *(mandatory)*

### Functional Requirements

**Store paths (#179)**

- **FR-001**: Every store-path argument MUST be optional. The nine fields at
  `cli.rs:340`, `:414`, `:417`, `:485`, `:491`, `:504`, `:510`, `:575`, `:610`
  become `Option<PathBuf>`, less any removed with `update`.
- **FR-002**: Resolution MUST go through the existing chain (flag, then
  `FRAGCAP_CATALOG_DB` / `FRAGCAP_LOCAL_DB`, then the per-user default) in one
  place, reusing `paths::*` and `target_resolve::ensure_catalog_store` rather
  than reimplementing precedence per command.
- **FR-003**: With no flag, no override, and no resolvable default, a command
  MUST fail cleanly naming what it could not resolve, not with a clap usage
  error.
- **FR-004**: A defaulted store that does not exist MUST be created with its
  parents. An operator-named path MUST be opened as given and never created on
  the operator's behalf by a read-only command.
- **FR-005**: Every success line MUST name the resolved store, so removing the
  flag does not remove the operator's ability to know what was written.
- **FR-006**: A guard MUST enumerate the command surface and assert that no
  subcommand declares a *required* argument named `db`, `catalog-db`, or
  `local-db`, so a new subcommand inherits the rule.

**The dead command and the dead end (#175)**

- **FR-007**: `catalog update` MUST be removed from the command surface, along
  with its dispatch arm and its `#[cfg(not(feature = "net"))]` error arm.
- **FR-008**: No user-facing string MUST name a Cargo feature or ask for a
  rebuild. This covers `cli.rs`, `commands/catalog.rs`, and
  `doctor/action.rs:103`. The S062 lint rule and help guard already assert the
  `cli.rs` half; this extends the obligation to the command bodies and `doctor`.
- **FR-009**: The phrase "the current published catalog" MUST be removed from
  every live surface that carries it, because no such artifact is published:
  `cli.rs`, `doctor/checks.rs:397`, `site/content/docs/reference/cli.mdx:215`,
  and the specification.

  **Corrected against the source.** Issue #175 cites
  `docs/fragcap-specification.md:1898`; that line number is wrong and the
  specification carries no `catalog update` entry in its command grammar at all.
  The two sections that do change are **15.7 (Target Resolution Cascade)**,
  which names `fragcap catalog seed-signatures`, and **26.3 (Diagnostics)**,
  which describes "fetch the published catalog" as a doctor action and states
  that "the npcap and catalog fetch actions are network-gated and degrade in a
  default build". FR-010 reverses the second half of that sentence for the
  catalog action, so 26.3 must move with the code (P-11).
- **FR-010**: The `catalog store` check's remediation MUST name a command the
  running binary can execute with no network. Its action becomes an offline
  initialize-and-seed rather than a network fetch, and MUST NOT be marked
  net-required or degrade.
- **FR-011**: The npcap actions MUST keep their current degraded text, which
  truthfully offers the official download page. They are not in scope beyond
  remaining unchanged.

**One seed verb (#180)**

- **FR-012**: `catalog seed-engine` and `catalog seed-signatures` MUST be gone
  from the command surface, replaced by `catalog seed --tier`.
- **FR-013**: `--tier` MUST accept `catalog`, `launch`, `engine`, and
  `signature`, and MUST be repeatable, so the fourth `SeedTier` member needs no
  fifth top-level verb.
- **FR-014**: `--from` MUST require exactly one `--tier`. Zero or more than one
  is a usage error at exit 2, matching the existing `ArgGroup` refusals. The
  command MUST NOT sniff the document to guess a tier.
- **FR-015**: Bare `catalog seed` MUST fill every tier reachable without a
  source flag and MUST name every tier it skipped, with the reason. A silent
  skip is a P-4 defect.
- **FR-016**: The `SeedSummary` counters and their meanings MUST be unchanged.
- **FR-017**: `.github/workflows/release.yml:119` MUST be updated to the new
  invocation in this change, and the update MUST be verified by running the new
  invocation, not by inspection.
- **FR-018**: `site/content/docs/reference/cli.mdx` and every glossary entry
  naming a removed verb MUST be updated in the same change (P-6).

  **Corrected against the source.** The "catalog seeder" and "engine seeder"
  entries in `docs/glossary/process-and-attribution.md` name internal
  components, and those components survive the command merge unchanged, so they
  need no edit. The entry that does break is
  `docs/glossary/anti-cheat-and-security.md:72`, which names `fragcap catalog
  seed-signatures` as the way to refresh detection capability.

**Raised in review of PR #190**

- **FR-019**: `--from` MUST be refused for a tier that reads no document. The
  signature tier seeds from a compiled-in set and the launch tier has no seeder,
  so accepting `--from` with either took the operator's file, never opened it,
  and exited 0. Validating only the *count* of `--tier` values was not enough;
  the tier must be one that can consume the input. Discarding a named input
  silently is the configuration-side form of the loss P-4 forbids.
- **FR-020**: No user-facing string MUST name a command this slice removes. The
  `technologies --catalog-db` help still said "Seed it with `catalog
  seed-signatures`", so following the displayed remediation produced an
  unknown-subcommand error. Note that neither the S062 help guard nor its lint
  rule catches this class: they check vocabulary and width, not whether a
  backticked command resolves. That cross-reference check is issue #183's, and
  this is direct evidence it is worth building.
- **FR-021**: FR-005's "name the resolved store" MUST hold for `technologies`
  and `targets discover` as well, which reported findings and candidates without
  naming the store they came from or the store they wrote to. Both were missed
  because their output is a listing rather than a success line.
- **FR-022**: A user-facing message MUST NOT contain runs of literal spaces. Three
  messages added by this slice did, because a line continuation was written as an
  escaped `
` rather than a real one, so the text carried a newline and the
  source indentation into the terminal. They now use `concat!`, which cannot fail
  this way.

### Out of scope

- **OOS-001**: Adding `net` to the release feature set. The operator's recorded
  decision is no network code in the shipped binary.
- **OOS-002**: Deleting S056's npcap installer fetch. See the clarification: its
  degraded form is truthful, so it is not the state #175 objects to.
- **OOS-003**: Populating the title tier of the shipped catalog. Measured as
  empty and reported; filling it is a maintainer data-publishing decision, not a
  command-surface change.
- **OOS-004**: The #183 help accuracy audit, which is sequenced after this slice.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every affected command runs with no store flag on a machine with a
  resolvable per-user directory. The count of required store-path arguments on
  the surface falls to zero, asserted by an enumerating guard.
- **SC-002**: `fragcap catalog update` is not a command. `catalog seed-engine`
  and `catalog seed-signatures` are not commands.
- **SC-003**: No `doctor` output on a released binary names a Cargo feature or a
  rebuild, and the catalog remediation, when performed, produces a populated
  signature table with no network access.
- **SC-004**: The string "published catalog" appears nowhere in the shipped
  command surface, the specification, the site documentation, or the glossary.
  `CHANGELOG.md` is excluded: it carries the phrase at lines 264, 364, and 366
  as the historical record of releases that did ship that command, it is
  assembled from fragments at release time, and it is never edited from a
  feature branch.
- **SC-005**: The release workflow's catalog build step runs successfully under
  the new grammar, verified by executing it rather than by reading it.
- **SC-006**: `cargo xtask ci` is green, and the S062 help guard and lint rule
  stay green over the rewritten `catalog` block.

## Assumptions

- The per-user default is resolvable on the target platform. Where it is not,
  FR-003's clean failure is the specified behavior.
- `ensure_catalog_store`'s bootstrap semantics (operator-named used as given,
  default seeded from the sibling template) are correct and are reused rather
  than re-derived.
- Discovery does not require a populated title tier, verified on the developer
  machine where 33 titles resolved with a zero-record catalog.
