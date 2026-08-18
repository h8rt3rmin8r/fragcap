# Feature Specification: CLI surface rework

**Feature Branch**: `054-cli-surface-rework`

**Created**: 2026-08-17

**Status**: Draft

**Input**: GitHub issue #141 (slice S054, milestone v0.5.0). Source:
`fragcap-v0.5.0-UX-Handoff-Plan.md` sections 9.1 through 9.4. Depends on S053
(merged). Inherits the US5 profile-file-retirement deferral carried forward from
S051.

## Overview

fragcap today exposes one capture engine behind three verbs, `run`, `tap`, and
`watch`, and each verb subtracts capability rather than adding convenience: `tap`
hard-codes a file capture and cannot ring-buffer, neither `tap` nor `watch` can
launch a title under capture, and only `watch` can wait for a process to start.
The three verbs are backed by near-identical assembly code that differs by a
single field.

At the same time the tool's data namespaces no longer match its two stores. Since
the two-store split (S050) there is a shipped, disposable `catalog.db` and a
user-owned `local.db`, but the commands that write them are scattered: catalog
seeding lives under `targets`, and the target-registration commands that belong to
the user store share that namespace with them. Profiles have stopped being files
(S050, S051), yet a `profile` command, a profile directory, and a `--profile`
capture selector still exist and still imply a file that no longer backs anything.

This feature collapses the three capture verbs into a single `capture` verb,
realigns the command namespaces to the two stores, retires the profile-file
surface completely, and groups the command surface for discoverability. It is a
usability and coherence change: no capture capability is removed, and five
previously-inexpressible captures become expressible.

## Clarifications

### Session 2026-08-17

Resolved by decision under the operator's standing autopilot mandate (decide and
proceed); each answer is grounded in the issue source text or an existing project
pattern, and none is an irreversible or architecture-defining choice.

- Q: Do the current ad-hoc `run --install-dir` / `run --steam`
  capture-without-registration paths carry onto `capture`, or are they dropped in
  favour of the two named target inputs? → A: Dropped. Issue #141 section 9.1 names
  the target inputs as exactly `--target` or `--process`, mutually exclusive, one
  required. Capturing an installed title is a register-then-capture flow (`targets
  add --steam` / `targets scan` / discovery, then `capture --target`). The "removes
  nothing" claim is about capture *behaviours* (ring, wait, launch), which are all
  preserved; the ad-hoc unregistered-directory shortcut is folded into the target
  model, not a lost behaviour.
- Q: What is the scope of `catalog update` in this slice? → A: Establish the
  `catalog` namespace and the `catalog update` command as the fetch-the-published-
  catalog home, wiring its live network fetch to the existing net-gated seeder
  machinery (S035). Like that machinery it is compiled behind the `net` feature and
  not run in continuous integration. The slice does not define or ship a new remote
  published-catalog artifact; if none is reachable, `catalog update` reports that
  honestly rather than inventing one.
- Q: What remains under the `steam` namespace after `steam profile` moves to
  `targets add --steam`? → A: The genuinely Steam-specific inspection operations
  (installed-title enumeration and any Steam metadata reads). `steam` is not removed
  outright; only the profile-scaffolding verb leaves it. Exact residual subcommand
  shape is settled in planning.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - One capture verb (Priority: P1)

An operator wants to capture a game's traffic. Instead of choosing among `run`,
`tap`, and `watch`, and discovering that the verb they picked cannot do what they
need (ring-buffer a named process, launch a named process under capture, wait for
one to start), they use a single `capture` verb whose flags are orthogonal. They
name the target one of two ways, by a stored target selector (`--target`) or by a
raw process image name (`--process`), and then choose the capture behaviour with
independent flags that compose freely.

**Why this priority**: This is the heart of the slice and the MVP. It delivers the
five previously-inexpressible captures and removes the arbitrary per-verb
restrictions. The namespace and presentation work only matters once there is a
single capture surface to organize around.

**Independent Test**: Fully testable by driving `capture` with each of the five
capture combinations from the section 9.1 table through the existing offline
capture substrate (no capture driver, no game) and asserting each is accepted and
produces the expected capture, plus asserting the retired verbs no longer parse.

**Acceptance Scenarios**:

1. **Given** a stored target with a Steam anchor, **When** the operator runs
   `capture --target eso --launch --duration 30m --out capture.fcapng`, **Then**
   the title is launched through its platform launcher and captured to the file
   for the bounded duration.
2. **Given** a named process, **When** the operator runs `capture --process
   eso64.exe --duration 5m --out capture.fcapng`, **Then** the running process is
   captured to the file (the capability the old `tap` had).
3. **Given** a named process that is not yet running, **When** the operator runs
   `capture --process unknown.exe --wait 5m --mode ring --ring 10m`, **Then** the
   command waits up to five minutes for the process to appear and captures it into
   a ten-minute ring window (a named process, ring buffer, waiting for start, none
   of which any single old verb could do).
4. **Given** neither `--target` nor `--process`, **When** the operator runs
   `capture`, **Then** the command reports a usage error (exit 2) naming the
   missing required target input.
5. **Given** both `--target` and `--process`, **When** the operator runs
   `capture --target eso --process eso64.exe`, **Then** the command reports a usage
   error (exit 2) because the two target inputs are mutually exclusive.
6. **Given** any of `run`, `tap`, or `watch` on the command line, **When** the
   operator runs it, **Then** the command is rejected as unknown (no alias, no
   deprecation shim), because fragcap has no userbase to shim for.

---

### User Story 2 - Namespaces follow the stores (Priority: P2)

A maintainer refreshing shipped data and a user managing their own targets each
reach for a namespace that matches the store they are touching. Everything that
writes the disposable, shipped `catalog.db` lives under `catalog`; everything that
writes the user-owned `local.db` lives under `targets`. Registering a Steam title
as a target is a `targets` operation, not a `steam` one, because the result lands
in the user store.

**Why this priority**: The command surface should teach the two-store model rather
than contradict it. This is high-value coherence work but depends on the capture
verb existing first, and it is independently testable once the moves are made.

**Independent Test**: Testable by asserting each relocated command resolves under
its new namespace and writes the correct store, that the old locations no longer
resolve, and that `targets add --steam <app_id>` registers a target equivalent to
what `steam profile <app_id>` scaffolded before.

**Acceptance Scenarios**:

1. **Given** the `catalog` namespace, **When** the operator runs `catalog import`,
   `catalog export`, `catalog seed`, `catalog seed-engine`, or `catalog
   seed-signatures`, **Then** each operates on `catalog.db` and the same operation
   no longer resolves under `targets`.
2. **Given** an installed Steam title, **When** the operator runs `targets add
   --steam <app_id>`, **Then** a target for that title is registered in `local.db`
   (the capability `steam profile <app_id>` used to provide).
3. **Given** the `steam` namespace, **When** the operator inspects it, **Then** it
   retains only genuinely Steam-specific operations and no longer carries profile
   scaffolding.
4. **Given** the retired profile-file surface, **When** the operator runs `profile
   validate`, **Then** the command is rejected as unknown, while `schema validate
   <file>` still validates a JSON artifact (documented as an advanced operation for
   someone about to share a JSON export).

---

### User Story 3 - A discoverable surface (Priority: P3)

A new user runs `fragcap --help` and sees the commands grouped by purpose rather
than as a flat alphabetical list, so the shape of the tool is legible at a glance.
A user who runs `fragcap` with no arguments is shown their registered targets plus
a one-line pointer to `--help`, rather than a bare usage error.

**Why this priority**: Presentational polish that raises discoverability without
changing any capability. It depends on the final command set (US1, US2) being
settled so the groupings and the bare-invocation default are stable.

**Independent Test**: Testable by asserting `--help` output carries the four
grouped headings with the right commands under each, that bare `fragcap` prints the
target listing followed by the footer line, and that `fragcap targets` (explicit)
prints the same listing without the footer.

**Acceptance Scenarios**:

1. **Given** the top-level help, **When** the operator runs `fragcap --help`,
   **Then** commands appear under the headings Capture, Targets, Environment, and
   Data, with nothing hidden.
2. **Given** no arguments, **When** the operator runs `fragcap`, **Then** the
   registered targets are listed and a footer line points at `--help`.
3. **Given** an explicit listing, **When** the operator runs `fragcap targets`,
   **Then** the same listing prints without the footer line.

---

### Edge Cases

- **Path anchor on a raw process**: the old `watch` distinguished a modded install
  (an executable outside `steamapps` launched from a mod manager) by a path anchor
  in addition to the image name. That anchoring capability must survive on
  `capture` so `--process` captures do not lose the ability to disambiguate two
  processes of the same image name.
- **`--launch` without a launchable target**: launching under capture needs a
  platform anchor (a Steam app id). A `--process` capture, or a `--target` whose
  target carries no anchor, cannot be launched; the command must report that as a
  usage error rather than silently ignoring `--launch`.
- **Bare invocation with an empty local store**: `fragcap` with no arguments and no
  registered targets still prints a coherent empty listing plus the footer, not an
  error.
- **A documentation example that names a retired command**: every example in the
  shipped docs must name a command that exists after this change; a stale example
  is a defect this slice must not leave behind.

## Requirements *(mandatory)*

### Functional Requirements

#### The capture verb (US1)

- **FR-001**: The tool MUST expose a single `capture` verb that supersedes `run`,
  `tap`, and `watch`. The three old verbs MUST be removed with no aliases and no
  deprecation shims.
- **FR-002**: `capture` MUST identify its target by exactly one of two mutually
  exclusive, required inputs: `--target` (a stored-target selector per S051, section
  5.4) or `--process` (a raw process image name). Supplying neither or both MUST be
  a usage error (exit 2).
- **FR-003**: Every capture-behaviour flag on `capture` MUST be orthogonal to the
  target input, so that all five captures named in the section 9.1 table are
  expressible: profile-equivalent plus ring buffer, named process plus ring buffer,
  named process plus wait-for-start, named process plus launch-under-capture, and
  profile-equivalent plus give-up timeout.
- **FR-004**: `capture` MUST retain, as orthogonal optional flags, the path-anchor
  capability the old `watch` provided (a case-insensitive path substring and a path
  regular expression), so a `--process` capture can disambiguate two processes that
  share an image name.
- **FR-005**: `capture --launch` MUST launch a target that carries a platform anchor
  through its launcher before capturing it; when the selected target input carries no
  launchable anchor, `--launch` MUST be reported as a usage error rather than
  ignored.
- **FR-006**: The offline capture substrate that drives the capture path in tier-1
  tests (recorded source, scripted attributor, scripted process timeline) MUST
  remain available on `capture`, hidden from help, so the whole capture path stays
  testable with no capture driver, no elevation, and no game.

#### Profile-file surface retirement (US1, the S051 US5 deferral)

- **FR-007**: The `profile` management command and all its subcommands MUST be
  removed.
- **FR-008**: The profile directory concept, the `--profile-dir` selector, and the
  file-backed profile provider MUST be removed, together with the `--profile`
  capture selector, as one coherent surface.
- **FR-009**: `schema validate <file>` MUST remain as the general JSON-artifact
  validator and MUST be documented as an advanced operation for someone about to
  share a JSON export.

#### Namespaces follow the stores (US2)

- **FR-010**: A `catalog` namespace MUST own every operation that writes the shipped,
  disposable `catalog.db`. The catalog-seeding operations currently under `targets`
  (import, export, seed, seed-engine, and the S053 signature seed) MUST move under
  `catalog`.
- **FR-011**: The `targets` namespace MUST own only operations on the user-owned
  `local.db` (target registration, listing, showing, discovery, and directory scan).
- **FR-012**: Registering a Steam title as a target MUST be expressed as `targets
  add --steam <app_id>`, replacing `steam profile <app_id>`. The `steam` namespace
  MUST retain only genuinely Steam-specific operations.
- **FR-013**: `catalog update` MUST be the command that fetches the current published
  catalog into `catalog.db`.

#### Presentation and discoverability (US3)

- **FR-014**: `--help` MUST group the command surface under four headings: Capture
  (`capture`, `replay`), Targets (`targets`, `technologies`, `steam`), Environment
  (`doctor`, `extcap`), and Data (`catalog`, `schema`). The grouping MUST be
  presentational only and MUST hide nothing.
- **FR-015**: Invoking `fragcap` with no arguments MUST run the `targets` listing and
  append a footer line pointing at `--help`.
- **FR-016**: The footer line MUST be suppressed when `targets` is invoked explicitly,
  so `fragcap targets` and bare `fragcap` differ only by the footer.

#### Documentation coherence (cross-cutting)

- **FR-017**: Every command example in the shipped documentation MUST name a command
  that exists after this change. No example may reference a retired verb, the retired
  `profile` command, or a relocated command under its old namespace.
- **FR-018**: Every new term this slice introduces MUST receive a glossary entry in
  the same change, per the house rule.

### Key Entities *(include if feature involves data)*

- **Capture invocation**: the resolved intent of one `capture` call, composed of a
  target input (stored-target selector or raw process image name, plus optional path
  anchors), a capture mode (file, ring, stream), duration and stop bounds, an
  acquisition wait, a launch flag, sinks, and role/direction scoping. It is the union
  of what `run`, `tap`, and `watch` each expressed partially.
- **Command namespace**: a grouping of subcommands bound to a store or purpose:
  `catalog` (writes `catalog.db`), `targets` (writes `local.db`), `steam`
  (Steam-specific), `schema` (artifact validation), plus the environment and capture
  groups.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All five captures named in the section 9.1 table are expressible with a
  single `capture` invocation and are each covered by a test.
- **SC-002**: The three retired capture verbs and the retired `profile` command are
  each rejected as unknown; none parses.
- **SC-003**: `--help` renders the four grouped headings with every command present
  and none hidden.
- **SC-004**: Bare `fragcap` prints the target listing followed by the footer, and
  explicit `fragcap targets` prints the same listing without the footer; the two
  outputs differ only by that line.
- **SC-005**: Every catalog-writing operation resolves under `catalog` and no longer
  under `targets`; `targets add --steam <app_id>` registers a target equivalent to
  the former `steam profile <app_id>`.
- **SC-006**: Zero shipped documentation examples name a command that does not exist
  after the change (verifiable by scanning docs against the final command set).
- **SC-007**: The full repository gate set (`cargo xtask ci` and the spec/impact
  checks) passes on the change.

## Assumptions

- **Target input reconciliation**: The current `run` target inputs `--profile`,
  `--install-dir`, and `--steam` are subsumed by the new model. `--profile` is
  retired with the profile-file surface (FR-008). Capturing an installed title is
  done by first registering it as a target (`targets add --steam`, `targets scan`, or
  discovery) and then `capture --target <selector>`; the ad-hoc `--install-dir` and
  `--steam` capture-without-registration paths are not carried onto `capture`. This
  keeps `capture` to the two target inputs the issue names.
- **`catalog seed-signatures` placement**: The S053 signature seed writes `catalog.db`
  and therefore moves under `catalog` with the other catalog-seeding commands, even
  though the issue's prose enumerates only import/export/seed/seed-engine.
- **`catalog update` scope**: `catalog update` establishes the fetch-the-published-
  catalog command home. Its live network fetch reuses the existing net-gated seeder
  machinery and is exercised the same way that machinery already is (compiled behind
  the `net` feature, not run in continuous integration). The exact remote source and
  whether a published catalog artifact exists yet is settled in clarification /
  planning; this slice at minimum establishes the namespace and the command.
- **`steam` residual surface**: After `steam profile` moves to `targets add --steam`,
  the `steam` namespace retains title enumeration and any other genuinely
  Steam-specific inspection; it is not removed outright.
- **`replay` stays a stub**: `replay` remains the not-yet-implemented capture-file
  playback command and is placed in the Capture help group; this slice does not
  implement it.
- **No behavioural change to capture internals**: the pipeline, attribution, ring,
  and launch mechanics are unchanged; this slice reorganizes the command surface that
  drives them and the assembly code behind the three old verbs.
- **Build/verify environment**: local verification uses the GNU-host toolchain
  (`cargo +1.96.0-x86_64-pc-windows-gnu`) because this machine has no MSVC linker;
  the real MSVC gate runs in continuous integration.

## Dependencies

- **S053** (merged): the signature seed command that moves under `catalog`.
- **S051** (merged): the stored-target model and the `--target` selector grammar
  (section 5.4) that `capture` resolves; this slice completes the US5 profile-file
  retirement S051 deferred.
- **S050** (merged): the two-store split (`catalog.db` / `local.db`) the namespaces
  realign to.
