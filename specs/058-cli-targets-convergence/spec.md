# Feature Specification: CLI targets convergence

**Feature Branch**: `058-cli-targets-convergence`

**Created**: 2026-08-18

**Status**: Draft

**Input**: Slice S058, issues #157 and #156 (packed), post-v0.5.0 convergence sprint slice 1.

## Context

After S054's CLI surface rework and S055's targets hero command, two parts of the
`fragcap-cli` surface still do not speak the targets model:

- The bare `fragcap targets` hero command resolves the default local store
  automatically, but the explicit `targets` subcommands (`add`, `show`, `remove`,
  `export`, `import`, `list`) still require an explicit `--db <path>`. A first-time
  user cannot follow the natural `fragcap targets` then `fragcap targets add` path
  without discovering and typing the store path (#157).
- The Wireshark extcap capture path was never converged by S054: `commands/extcap.rs`
  still resolves a retired profile *file* (via the profile search-path / bundled-set
  cascade) and declares a required `--profile` field, instead of resolving a stored
  target selector the way `capture` does (#156). S057 added a "legacy" callout to the
  CLI reference documenting this gap; this slice closes it.

This is a `fragcap-cli`-only slice: no change to `fragcap-core`, the pipeline,
attribution, or the capture orchestrator's behavior, and no new dependency.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Registering a target with no store path (Priority: P1)

A user who has just run `fragcap targets` (which discovered and listed titles from
the default local store) wants to register one more title by hand. They run
`fragcap targets add --steam <app_id>` with no `--db`, and it registers into the
same default local store the listing used. The same holds for `show`, `remove`,
`export`, `import`, and `list`: none requires an explicit `--db` to operate on the
default store, and an explicit `--db` still overrides it.

**Why this priority**: This removes the wall the getting-started guide had to route
around (S057 deferred the manual-registration examples to the reference precisely
because `--db` was required). It is the ergonomic payoff of the targets model and is
independently testable.

**Independent Test**: With `FRAGCAP_LOCAL_DB` pointed at a scratch store (or the
default), run each affected subcommand with no `--db` and confirm it operates on the
resolved default store; run one with an explicit `--db` and confirm the flag still
wins. The full `cargo xtask ci` is green.

**Acceptance Scenarios**:

1. **Given** no `--db` flag and a resolvable default local store, **When** the user
   runs `fragcap targets add <name> --steam <app_id>` (or `show`/`remove`/`export`/
   `import`/`list`), **Then** the command operates on the default local store, the
   same one the bare `fragcap targets` listing uses.
2. **Given** an explicit `--db <path>`, **When** the user runs any affected
   subcommand, **Then** it operates on the named store (the flag overrides the
   default).
3. **Given** neither a `--db` flag, a `FRAGCAP_LOCAL_DB` override, nor a resolvable
   application-data base, **When** the user runs a subcommand that must open a store,
   **Then** it fails with a named error rather than a panic or a silent no-op.

### User Story 2 - Capturing through Wireshark by target selection (Priority: P1)

A user drives fragcap as a Wireshark extcap capture source. The capture
configuration the analyzer dialog presents selects a **stored target** by a single
selector string (a handle, a case-insensitive exact name, or a 1-based row index),
resolved against the local store exactly as `fragcap capture` resolves a positional
`--target`. No part of the extcap capture path resolves a profile file through the
retired search-path / bundled-set cascade.

**Why this priority**: extcap capture is currently bound to the retired profile-file
mechanism, which after S054 resolves nothing usable; converging it onto targets is
what makes analyzer-driven capture work in the shipped model, and it closes the
"legacy" gap S057 documented.

**Independent Test**: The extcap capture handler, driven offline (the existing
replay/attr/process substrate) against a registered target selector plus a local
store, resolves the target and proceeds to capture; the extcap config declaration
names a target selector, not a profile; no code path in the extcap capture handler
calls the profile-file resolver.

**Acceptance Scenarios**:

1. **Given** a registered target and an extcap capture invocation naming its
   selector, **When** the analyzer starts capture, **Then** fragcap resolves the
   stored target (the same resolution `capture` uses) and streams the capture.
2. **Given** the extcap configuration declaration, **When** the analyzer queries it,
   **Then** the selection option names a target selector, and the extcap control
   grammar (interfaces, link types, the config block as arg lines, FIFO streaming)
   is otherwise unchanged so unmodified Wireshark still drives fragcap.
3. **Given** the extcap capture handler, **When** its resolution path is inspected,
   **Then** it resolves a stored target and never resolves a profile file via the
   search-path / bundled-set cascade.

### User Story 3 - One resolution implementation, shared (Priority: P2)

The stored-target resolution logic that `capture` uses is a single shared
implementation that the extcap command also calls, rather than two copies. The seam
is clean enough that a later slice (S059, launch-and-observe) can extend it in one
place.

**Why this priority**: It is the structural precondition for US2 and prevents drift
between the two capture entry points; it is lower user-facing priority but
load-bearing for maintainability and the next slice.

**Independent Test**: Grep confirms the resolution functions live in one module and
are called by both `capture` and `extcap`; there is no duplicated target-resolution
body.

**Acceptance Scenarios**:

1. **Given** the codebase after the slice, **When** the target-resolution functions
   are located, **Then** they live in one shared location and both `capture` and
   `extcap` call them.

### Edge Cases

- A subcommand invoked with no `--db` and no resolvable default store must fail with
  a clear message, never a panic or a silent success (US1 scenario 3).
- `add`/`import` against a defaulted, not-yet-created store must still work (the
  store is created on first open), matching today's explicit-`--db` behavior.
- The extcap config block must remain a set of arg lines the analyzer parses; only
  the meaning of the one selection arg changes (profile file to target selector), so
  the wire contract stays one config string.
- Parallel tests that set the process-wide `FRAGCAP_LOCAL_DB` env var must isolate
  their store paths to avoid racing each other.

## Clarifications

### Session 2026-08-18

Resolved by decision under the operator's standing autopilot preference (decide and
proceed; surface, do not present confirmation menus). Recorded for traceability.

- Q: Does the extcap selection arg keep the call name `--profile` (new meaning) or
  become `--target`? A: Rename it to a target selector (`--target`), matching the
  `capture` command's selector flag, so the analyzer dialog and the command line
  name the selection identically. The wire contract stays one config string; only
  the arg's call name and tooltip change.
- Q: Does the extcap command gain store-path overrides? A: Yes -- add `--local-db`
  (and `--catalog-db` for parity and steam-anchor resolution) to the extcap args,
  mirroring `capture`, so the resolver can reach the same stores and tests can
  isolate a scratch store. They default the same way `capture` does.
- Q: For `targets list` with no resolvable store, does it error like the other
  must-open subcommands or degrade to an empty listing like the bare hero command?
  A: `list` degrades to an empty listing on an unresolvable-path-at-all case,
  matching the bare `fragcap targets` hero behavior it mirrors; `add`/`show`/
  `remove`/`export`/`import` error (FR-003), since operating on a store with no
  resolvable location is a genuine failure for them.

## Requirements *(mandatory)*

### Functional Requirements

**Part A (#157) - default `--db`**

- **FR-001**: The `targets add`, `show`, `remove`, `export`, `import`, and `list`
  subcommands MUST accept an optional `--db`; when omitted, they MUST resolve the
  same default local store the bare `fragcap targets` hero command resolves
  (`FRAGCAP_LOCAL_DB`, else the per-user application-data default).
- **FR-002**: An explicit `--db <path>` MUST continue to take precedence over the
  environment override and the default.
- **FR-003**: A subcommand that must open a store, given no `--db`, no
  `FRAGCAP_LOCAL_DB`, and no resolvable application-data base, MUST fail with a named
  error, not a panic or a silent no-op.
- **FR-004**: `add` and `import` against a defaulted, not-yet-existing store MUST
  work on first use, including on a clean machine where the per-user application-data
  directory does not exist yet: resolving the per-user default MUST ensure its parent
  directory exists before opening, since the store layer does not create missing
  parent directories. An explicit `--db` or `FRAGCAP_LOCAL_DB` path is operator-named
  and used as given.
- **FR-005**: The `discover` subcommand's separate two-store `catalog_db`/`local_db`
  flag pattern MUST remain unchanged (out of scope for this slice).

**Part B (#156) - extcap uses target selection**

- **FR-006**: The extcap capture path MUST resolve a stored target from a single
  positional selector string (a handle, a case-insensitive exact name, or a 1-based
  row index) against the local store, using the same resolution the `capture` command
  uses for a positional `--target`. The durable stable-id form (`--id`) is a
  `capture`-command flag and is out of scope for the extcap single-string dialog.
- **FR-007**: No code path in the extcap capture handler MUST resolve a profile file
  through the retired profile search-path / bundled-set cascade.
- **FR-008**: The extcap configuration declaration MUST present a target-selection
  option (naming a target selector), and the extcap control grammar otherwise
  (interfaces, link types, the config block structure as arg lines, FIFO streaming)
  MUST be unchanged so unmodified Wireshark still drives fragcap.
- **FR-009**: The stored-target resolution logic MUST be a single shared
  implementation called by both `capture` and `extcap` (no duplicated body), and the
  seam MUST be reusable by a later slice.

**Cross-cutting**

- **FR-010**: The change MUST be confined to `fragcap-cli` (plus the CLI reference
  doc and spec text); no change to `fragcap-core`, the pipeline, attribution, or the
  orchestrator's behavior, and no new dependency or `Cargo.lock` delta.
- **FR-011**: The CLI reference (`site/content/docs/reference/cli.mdx`) MUST replace
  the S057 extcap "legacy" callout with the converged options; no documentation page
  MUST describe extcap capture as a legacy profile-file path.
- **FR-012**: The specification's command-surface section MUST be reconciled with the
  converged extcap behavior, and the Applies-To lockstep (`cargo xtask spec`) MUST
  pass. Any new user-facing term MUST carry a glossary entry in the same change
  (P-6); this slice is expected to introduce none.
- **FR-013**: `cargo xtask ci` MUST be green (fmt, clippy `--all-features`, test
  `--workspace`, lint, deps, license, docs check).

### Key Entities

- **Local store (`local.db`)**: the user-owned store of registered targets; the
  default the hero command and (after this slice) the subcommands resolve.
- **Target selector**: an exact handle, a case-insensitive exact name, or a 1-based
  row index over the current listing -- the positional selector both `capture` and
  (after this slice) `extcap` resolve. The durable stable id (`--id`) is an additional
  `capture`-command flag, not part of the extcap single-string dialog.
- **Shared target-resolution seam**: the one implementation of stored-target
  resolution (selector to a synthesized capture `Profile`), called by both `capture`
  and `extcap`.
- **Extcap config block**: the set of arg lines fragcap prints for the analyzer's
  capture dialog; after this slice its selection arg names a target selector.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Each of `targets add`/`show`/`remove`/`export`/`import`/`list` runs
  against the default local store with no `--db` flag, and an explicit `--db` still
  overrides; verified by tests.
- **SC-002**: The extcap capture path resolves a stored target selector and never
  resolves a profile file via the search-path / bundled-set cascade; verified by the
  extcap tests and by inspection of the handler.
- **SC-003**: The target-resolution logic exists in exactly one shared location,
  called by both `capture` and `extcap`.
- **SC-004**: No documentation page references extcap capture as a legacy
  profile-file path; the CLI reference documents the converged options.
- **SC-005**: `cargo xtask ci` is green, and there is no `Cargo.lock` delta.

## Assumptions

- The existing `paths` helpers (`local_db_path`, `default_local_db_path`) and the
  default-resolution chain already used by the `scan` variant and the bare hero
  listing are the mechanism to reuse; no new path helper is added.
- The store layer creates a fresh store at a nonexistent path but does not create a
  missing parent directory, so the default-store resolution creates the per-user
  default's parent before opening (as `capture`'s store bootstrap does).
- The extcap analyzer wire contract passes back a single selection string; that stays
  one string, with only its meaning changing from profile reference to target
  selector.
- The stored-target resolution functions currently private to `capture.rs`
  (`resolve_stored`, `setup_stores`, `build_resolver`, `resolve_from_install`,
  `synthesize_named_profile`, `synthesize_profile`, `steam_app_id`) are the seam to
  extract; the extraction preserves their behavior for `capture`.
- Launch-and-observe / capture-time promotion (#152), the installer npcap wording
  (#133), and the `discover` two-store flag pattern are out of scope and deferred to
  their own slices.

## Dependencies

- Reflects the shipped reality of S054 (CLI surface rework), S055 (targets hero
  command), and S057 (the extcap legacy callout this slice removes).
- Produces the clean shared resolution seam that slice S059 (launch-and-observe)
  extends.
