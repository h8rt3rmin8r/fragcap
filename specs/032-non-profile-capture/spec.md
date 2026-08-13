# Feature Specification: Non-Profile Production Capture Path

**Feature Branch**: `feat/non-profile-capture`

**Created**: 2026-08-13

**Slice**: S032 (activates the target-resolution cascade of issue #77). Slices
S027 through S030 built a cascade that resolves a capture target from install
layout (an engine rule, a Steam platform walker) and runtime observation, and
S031 detects a target's technologies. But `run` still refuses any resolved target
that has no backing profile ("resolved a target with no profile, which run cannot
capture yet"), and a profile outranks every cascade provider, so the engine rule,
the walker, and observation resolve targets that nothing can then capture. This
slice closes that gap: it lets `run` capture a resolved-but-unprofiled target.
Constitution principles in play: passive observation only, so resolution and
capture read the filesystem and the query-only process snapshot and never open a
process handle or read process memory (P-1); no silent loss, so an unresolved,
ambiguous, or unreadable target is a named, surfaced failure, never a silent
no-op (P-4); an honest fidelity, so a target synthesized from a heuristic
resolution is stamped `heuristic-unverified` and never `authored` (P-9); and a
glossary entry for any new term in the same change (P-6).

**Input**: Let `run` capture a target that the cascade resolves without a
profile. When resolution returns a target whose origin is an engine rule, the
platform walker, or runtime observation (no backing profile), synthesize a
one-stage capture identity from that target's image name and path anchors, the
same shape `watch` and the `tap` scaffold already build, stamp it
`heuristic-unverified`, and hand it to the shared capture engine. Add the entry
points that let the cascade resolve such a target from an install location: a
general `--install-dir <path>` that runs the cascade over a given install
directory, and a `--steam <app_id>` convenience that resolves the app id to its
install directory through the existing Steam library lookup and then takes the
same path. These are mutually exclusive with `--profile`; the profile path is
unchanged and byte-identical. An install location that resolves to nothing, or to
an ambiguous or unreadable layout, is a surfaced failure, not a capture of
nothing.

## Overview

The cascade was built inside-out. S027 laid down the resolver and the fidelity
model; S028 gave it a launch-agnostic capture path through `watch`; S029 and S030
added the engine rule and the platform walker, providers that resolve a client
from install layout with no per-title data. Each of those slices wired its
provider into the production resolver `run` assembles, and each deferred the same
last step: `run` resolves through the whole cascade but then calls
`into_profile()` on the result and errors if there is none. Every non-profile
provider therefore produces answers that only reach a dead end.

The reason the dead end was safe to leave is that nothing drove a non-profile
resolution from the command line either. `run` takes a `--profile` reference and
builds a reference request; the engine rule and the walker need an install
directory on the request to do anything, and a bare reference carries none, so in
practice only the profile provider ever answered. The gap is therefore two
missing halves of one path: an input that gives the cascade an install location
to resolve from, and an output that captures the resolved identity when it is not
a profile.

This slice supplies both. The input is an install location: `--install-dir <path>`
names one directly, and `--steam <app_id>` resolves one through Steam's local
library lookup (the `install_root_for` helper S029 and S030 already built for
exactly this). Either way the resolver runs with an install root, so the engine
rule can name the socket-holding client from layout, the walker can classify the
install directory when no engine is recognized, and runtime observation remains
the final arbiter. The output is a capture: when the resolved target has no
profile, its identity (an executable image name plus optional path anchors, the
`MatchPredicates` every non-profile target already carries) is placed into a
one-stage profile and handed to the same capture engine `watch` uses. The
synthesized profile is stamped `heuristic-unverified`, because the identity came
from an install-layout heuristic, not from an operator who typed it; `watch`
stamps its synthesized profile `authored` for the opposite reason, and the
difference is the whole point of the fidelity model.

Honesty about failure is as much the deliverable as the capture. An install
location the cascade cannot resolve to a single client (an unrecognized layout, a
genuinely ambiguous one, an unreadable tree, a Steam app id that is not installed)
does not produce a capture of nothing. It produces a surfaced failure that names
what happened, so an operator who expected a capture and got none can tell a
declined resolution from a game that never sent a packet. That is the same P-4
discipline the providers already apply when they decline; this slice carries it
through to the command that acts on their answer.

Nothing here changes the profile path. A `run --profile <ref>` capture resolves,
overlays, and captures exactly as before, byte for byte; the non-profile path is
a second branch reached only when the resolved target has no profile.

## Clarifications

### Session 2026-08-13

- Q: What is the command-line surface for a non-profile capture? -> A: Two inputs
  on `run`, mutually exclusive with `--profile` and with each other, exactly one
  of the three required: `--install-dir <path>` runs the cascade over a given
  install directory (the general form, and the one that is testable offline and
  covers non-Steam installs), and `--steam <app_id>` resolves the app id to its
  install directory through the existing Steam library lookup and then takes the
  same path (the convenience form). A separate subcommand was rejected: the
  operation is a capture, `run` is the capture command, and the effective-config
  overlay, the sinks, and the orchestrator are all `run`'s already. Recorded as a
  decision.
- Q: What fidelity does the synthesized profile carry? -> A: `heuristic-unverified`.
  The identity was resolved by an install-layout heuristic (engine rule or
  walker) or by runtime observation, not typed by an operator, so stamping it
  `authored` (as `watch` does for its typed identity) would be a lie (P-9). The
  schema permits `heuristic-unverified` on a profile and refuses only `observed`,
  so the synthesized profile parses. Recorded as a decision.
- Q: How is the identity extracted from a resolved non-profile target? -> A: Every
  non-profile target origin (observed, engine rule, platform walker) already
  carries its `MatchPredicates` identity; the capture path reads that identity and
  places it into a one-stage profile, exactly as `watch` places a typed identity.
  No new accessor is invented if one exists; a small uniform accessor over the
  origins is a plan-time decision if one is needed.
- Q: Does the profile path change at all? -> A: No. `run --profile` resolves and
  captures exactly as before and its output is byte-identical. The non-profile
  path is a separate branch taken only when `into_profile()` returns nothing.
- Q: What happens when the cascade cannot resolve the install location? -> A: The
  resolver's existing unresolved outcome (unrecognized, ambiguous, or unreadable,
  each already a surfaced variant) is turned into a surfaced command failure that
  names the reason; nothing is captured and the failure is not silent (P-4). A
  Steam app id that is not installed fails at the library lookup, likewise
  surfaced.
- Q: Does this open a process handle to capture a resolved target? -> A: No. The
  resolved identity drives the same launch-agnostic capture engine `watch` uses:
  the session arms, the query-only startup snapshot is folded (attach-to-running),
  and the capture binds the process when it matches, by socket-table and ETW
  attribution from outside the process. No process handle is opened and no process
  memory is read (P-1).
- Q: What exit codes do the failure cases use? -> A: The existing CLI exit
  contract. A command-line misuse (more than one of `--profile`/`--install-dir`/
  `--steam`, or none) is a usage error (exit 2), reported by the argument parser
  before any resolution. A runtime failure (an install location the cascade
  declines, an unreadable directory, a `--steam` app id not installed) is a
  surfaced failure (exit 1). This matches how `run --profile` and the other
  commands already map usage versus failure, so the non-profile path adds no new
  exit-code semantics.
- Q: What game identity does the synthesized profile carry? -> A: A generic,
  honest identity, not a fabricated title. The synthesized one-stage profile
  carries a fixed placeholder game id and name that names the path it came from
  (for example id `run` / name derived from the input), the same way `watch` uses
  `id: "watch"` / `name: "ad hoc watch"`; for `--steam <app_id>` the app id is
  carried on the game's `app_id` field, which is a fact, while the display name
  stays generic unless the library lookup already returned one. The synthesized
  identity's job is to bind the socket holder, not to assert a verified title.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Capture a game from its install directory with no profile (Priority: P1)

An operator has a game installed but has authored no profile for it. They point
`run` at the install directory with `--install-dir`. The cascade resolves the
socket-holding client from the install layout (an engine rule names an Unreal
shipping client, say), `run` synthesizes a one-stage identity from that resolved
client, stamps it `heuristic-unverified`, and captures the game the same way a
profile capture would, launch-agnostic.

**Why this priority**: This is the activation the whole of #77 was built toward
and the general, offline-testable core of the slice. Delivered alone it turns
every install-layout provider from a dead end into a working capture. It is the
MVP.

**Independent Test**: Build a fixture install directory with a recognized engine
layout (an Unreal twin-exe tree), run `run --install-dir <fixture>` under the
offline capture harness, and assert it resolves the shipping client, synthesizes a
`heuristic-unverified` one-stage identity for it, and captures through the same
engine as a profile run (reproducing the offline capture the harness produces for
an equivalent authored identity).

**Acceptance Scenarios**:

1. **Given** an install directory with a recognized engine layout and no profile,
   **When** `run --install-dir <dir>` runs, **Then** the resolved client is
   captured through a synthesized one-stage identity stamped
   `heuristic-unverified`.
2. **Given** the same directory, **When** the capture runs offline against a
   process script that starts the resolved client, **Then** the capture attributes
   its flows exactly as an equivalent authored one-stage identity would.

---

### User Story 2 - Capture a Steam-installed game by app id with no profile (Priority: P1)

An operator wants to capture a Steam-installed game with no profile and does not
want to find its install directory by hand. They pass `run --steam <app_id>`.
`run` resolves the app id to its install directory through Steam's local library
lookup, then takes the same non-profile path as User Story 1.

**Why this priority**: This is the headline convenience and the most common way an
operator will reach the non-profile path. It is a thin resolve-then-delegate over
User Story 1, so it is P1 but rides on the core.

**Independent Test**: With a fake Steam library fixture whose app id maps to a
recognized-engine install directory, resolve the app id to its install directory
through the library lookup and assert it drives the same non-profile capture path
as User Story 1. (The real-machine `install_root_for` is exercised by the library
crate's own tests; this slice tests the delegation from a resolved directory.)

**Acceptance Scenarios**:

1. **Given** a Steam-installed title with a recognized engine layout and no
   profile, **When** `run --steam <app_id>` runs, **Then** the app id resolves to
   its install directory and the resolved client is captured through a synthesized
   `heuristic-unverified` identity.
2. **Given** an app id that is not installed in any Steam library, **When**
   `run --steam <app_id>` runs, **Then** the command fails with a surfaced message
   naming the missing title, and nothing is captured.

---

### User Story 3 - Honest fidelity and honest failure (Priority: P1)

An operator relies on the capture telling the truth. A capture reached through the
non-profile path is marked as the heuristic it is, never as an authored fact. And
an install location the cascade cannot resolve to a single client produces a
surfaced failure that names the reason, not a capture that silently records
nothing.

**Why this priority**: This is the P-9 and P-4 guarantee for the slice, and it is
what makes the activation safe rather than merely convenient. A non-profile
capture that claimed `authored` fidelity, or a declined resolution that looked
like a game that sent no traffic, would each be the kind of quiet lie the
constitution forbids.

**Independent Test**: (a) Assert the synthesized profile for a resolved engine-rule
or walker target carries `heuristic-unverified`, never `authored`. (b) Run
`run --install-dir <dir>` over a directory the cascade declines (unrecognized
layout, an ambiguous one, and an unreadable one) and assert each yields a surfaced
non-zero failure naming the reason, with no capture produced.

**Acceptance Scenarios**:

1. **Given** a resolved non-profile target, **When** its capture identity is
   synthesized, **Then** the profile carries `heuristic-unverified` fidelity and
   never `authored`.
2. **Given** an install directory the cascade cannot resolve (unrecognized,
   ambiguous, or unreadable), **When** `run --install-dir <dir>` runs, **Then**
   the command fails with a surfaced message naming the reason and captures
   nothing (P-4).
3. **Given** a `run --profile <ref>` capture, **When** it runs, **Then** its
   resolution, overlay, and output are byte-identical to before this slice.

---

### Edge Cases

- Both `--profile` and `--install-dir` (or `--steam`) are given, or none of the
  three: the command-line parser reports a mutual-exclusion or missing-input
  usage error before any resolution runs.
- `--install-dir` names a path that does not exist or is not a directory: a
  surfaced failure naming the path, distinct from a directory that was scanned and
  matched no layout.
- The install directory holds a recognized engine layout but several candidate
  clients (a genuinely ambiguous layout): the cascade declines with the ambiguity
  surfaced, and `run` fails naming it, rather than picking one arbitrarily (the
  S029 decline behavior, carried through to the command).
- The resolved client is already running when the command starts: the shared
  capture engine's startup-snapshot fold attaches to it, exactly as it does for a
  `watch` or profile capture; no process handle is opened to do so.
- A `--steam <app_id>` whose library manifest is unreadable or malformed: surfaced
  as a warning by the library lookup and, if no install directory results, a
  surfaced failure; never a silent empty capture.
- The resolved target is a profile after all (a profile reference also happened to
  match): the profile path is taken; the non-profile branch is not reached. This
  cannot arise from `--install-dir`/`--steam`, which carry no profile reference,
  but the branch order makes it well-defined.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `run` MUST capture a resolved target that has no backing profile,
  by synthesizing a one-stage capture identity from the resolved target's image
  name and path anchors and handing it to the shared capture engine, replacing the
  current "run cannot capture yet" refusal.
- **FR-002**: The synthesized one-stage profile MUST be stamped
  `heuristic-unverified` fidelity, never `authored`, and MUST be produced through
  the same validating profile construction the authored and `watch`/`tap` paths
  use, so an invalid synthesized identity surfaces as a profile diagnostic rather
  than a malformed capture.
- **FR-003**: `run` MUST accept a `--install-dir <path>` input that runs the
  resolution cascade over the given install directory (supplying it as the
  request's install root so the engine rule and the platform walker can resolve
  the socket holder), and capture the resolved target.
- **FR-004**: `run` MUST accept a `--steam <app_id>` input that resolves the app
  id to its install directory through the existing Steam library lookup and then
  takes the same non-profile capture path as `--install-dir`.
- **FR-005**: `--profile`, `--install-dir`, and `--steam` MUST be mutually
  exclusive, with exactly one required; supplying more than one, or none, MUST be
  a usage error (exit 2) reported by the argument parser before resolution.
- **FR-006**: The `run --profile` path MUST be unchanged: its resolution, overlay,
  and output MUST be byte-identical to before this slice (verified against the
  existing goldens).
- **FR-007**: An install location the cascade cannot resolve to a single client
  (an unrecognized layout, a genuinely ambiguous one, or an unreadable one) MUST
  produce a surfaced command failure (exit 1) that names the reason, and MUST NOT
  capture anything (P-4). The decline reason MUST be carried from the resolver's
  existing unresolved outcome, not re-derived.
- **FR-008**: A `--steam <app_id>` that is not installed in any Steam library MUST
  fail with a surfaced message naming the missing title, and MUST NOT capture
  anything.
- **FR-009**: The non-profile capture path MUST NOT open a process handle, read
  process memory, or otherwise engage a denylisted technique; it MUST reach the
  target through the same launch-agnostic, attribution-from-outside engine the
  `watch` path uses (P-1).
- **FR-010**: The resolved target's already-running case MUST be handled by the
  shared capture engine's startup-snapshot fold (attach-to-running), the same way
  the `watch` and profile paths handle it, with no new acquisition mechanism.
- **FR-011**: Any new term this slice introduces MUST gain a glossary entry in the
  same change, and the specification MUST document the non-profile capture path
  under the resolution-cascade or run-command section (P-6).
- **FR-012**: The slice MUST add no new runtime dependency and MUST keep the
  minimum supported toolchain green.

### Key Entities *(include if feature involves data)*

- **Non-profile capture path**: The `run` branch taken when the resolved target
  has no backing profile. It reads the resolved target's identity, synthesizes a
  one-stage `heuristic-unverified` profile from it, and drives the shared capture
  engine. Distinct from the profile branch, which hands the backing profile
  through unchanged.
- **Install-location input**: The `run` inputs that give the cascade something to
  resolve from without a profile: a direct install directory (`--install-dir`) or
  a Steam app id resolved to one (`--steam`). Both produce an install root the
  resolver consumes.
- **Synthesized capture identity**: The one-stage profile built from a resolved
  non-profile target's `MatchPredicates` (image name plus optional path anchors),
  stamped `heuristic-unverified`, and validated the same way every other profile
  is.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `run --install-dir <dir>` over a recognized-engine fixture layout
  with no profile captures the resolved client through a synthesized
  `heuristic-unverified` identity, in an offline test, reproducing the attribution
  an equivalent authored identity produces.
- **SC-002**: `run --steam <app_id>` over a fake Steam library fixture resolves
  the app id to its install directory and drives the same non-profile capture
  path, in a test; a not-installed app id fails with a surfaced message.
- **SC-003**: The synthesized profile carries `heuristic-unverified` fidelity and
  never `authored`, asserted in a test.
- **SC-004**: An unrecognized, ambiguous, and unreadable install directory each
  cause `run --install-dir` to fail with a surfaced reason and capture nothing, in
  tests (P-4).
- **SC-005**: `--profile`, `--install-dir`, and `--steam` are mutually exclusive
  with exactly one required, asserted by a command-line parsing test.
- **SC-006**: The `run --profile` capture output is byte-identical to the existing
  goldens (the profile path is untouched).
- **SC-007**: The full repository gate (`cargo xtask ci`) passes and the minimum
  supported toolchain (`cargo xtask msrv`) stays green, with no new dependency.

## Assumptions

- The S027 resolver, its providers (engine rule S029, platform walker S030,
  observation), the `MatchPredicates` identity carried by every non-profile
  target origin, the `ResolutionRequest::for_install` and install-root builders,
  and the Steam `install_root_for`/`install_root_in` lookup all exist and are the
  contract this slice composes; it wires them into `run` rather than adding new
  resolution machinery.
- The shared capture engine (`orchestrator::capture`) and the one-stage-profile
  synthesis pattern used by `watch` and the `tap` scaffold are the mechanism this
  slice reuses for the capture half; the offline capture harness that drives the
  existing `run`/`watch`/`tap` tests (a process script plus the offline
  components) is the mechanism the tests reuse, so no real game, capture driver,
  or Steam installation is required.
- The schema permits `heuristic-unverified` on a profile and refuses only
  `observed` (S027), so the synthesized profile parses; the targeting fidelity
  stamped here stays distinct from the attribution fidelity (Live/Retained/None)
  in the core crate.
- The resolver's unresolved outcome already distinguishes unrecognized, ambiguous,
  and unreadable declines as surfaced variants (S029/S030); this slice maps those
  to a surfaced command failure rather than inventing new decline reasons.
- Extracting the identity from a resolved non-profile target uses the origins'
  existing `MatchPredicates`; if a uniform accessor across the observed,
  engine-rule, and walker origins is needed, adding one is a plan-time decision
  recorded there, mirroring how prior slices added small accessors.
