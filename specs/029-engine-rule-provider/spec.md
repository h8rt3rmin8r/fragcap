# Feature Specification: Engine-Rule Provider (Unreal First)

**Feature Branch**: `feat/engine-rule-provider`

**Created**: 2026-08-12

**Status**: Draft

**Slice**: S029 (GitHub issue #77, slice 3 of 4). Adds the first general-purpose
target provider to the target-resolution cascade delivered by S027 (#77 slice 1)
and made launch-agnostic by S028 (#77 slice 2). It recognizes the socket-holding
game client from the engine's documented on-disk install layout alone, with no
per-game data. Constitution principles in play: passive observation only, so the
provider inspects the filesystem and never opens a process handle or reads
process memory (P-1), a heuristic answer is stamped as a heuristic and never
dressed up as a fact (P-9), no silent loss when a layout does not match (P-4),
the platform neutrality of the profile crate (P-2), and a glossary entry for the
new term in the same change that introduces it (P-6).

**Input**: Build an engine-rule provider that implements the S027 provider trait
and resolves a game's real networked client executable from the engine's
documented install-directory layout. Unreal Engine first and mandatory: a stub
executable in the install root corresponds to a `*-Win64-Shipping.exe` under a
`Binaries\Win64` directory, which is the process that holds the sockets. Then
Unity (`*_Data` directory plus `UnityPlayer.dll`) and Ren'Py (`renpy` directory
plus `.rpa` archives) as additional rules in the same provider. Every answer is
stamped at `heuristic-unverified` targeting fidelity with `provenance.source`
`engine-rule`. The provider takes a launch entry point (a directory-derived stub
here, a platform walker's output in S030) and hops from stub to the real client.
No Steam appinfo work, no CLI changes, no hint database.

## Overview

fragcap's job is to attribute captured packets to the process that produced them.
For that it must first know which process to watch. The target-resolution cascade
built in S027 consults providers of descending trust and returns the highest
one that can answer. Two providers had data behind them from the start: a
user-authored or curated profile, and runtime observation once the game is
already running. Three sat as no-answer stubs their own slices would fill. This
slice fills the engine-rule stub.

The problem engine rules solve is the launcher stub. A large class of modern
games ship a thin executable in the install root whose only job is to relaunch
the real client, which is the process that actually opens sockets. Before the
game has ever run, nothing but the on-disk layout distinguishes the stub from
the client, and neither standard capture tooling nor the profile provider can
name the client without per-title authored data. But game engines lay their
files out in documented, stable conventions. Unreal Engine's shipping client is
always a `*-Win64-Shipping.exe` under `Binaries\Win64`. Unity's player sits
beside a `*_Data` directory and a `UnityPlayer.dll`. Ren'Py's ships with a
`renpy` directory and `.rpa` archives. Recognizing those conventions resolves a
whole class of titles for free, with no per-game data to author or maintain.

An engine rule is a heuristic, and the cascade already has the vocabulary to say
so. The provider stamps every answer at `heuristic-unverified` fidelity, which
ranks below an authored package and a verified profile and above raw runtime
observation. That ordering is exactly right: an engine rule is a better-than-
nothing guess grounded in a documented convention, not a fact an author vouched
for. If the guess is wrong, or if the game is not built on a recognized engine,
the provider returns no answer and the cascade falls through to the next
provider and ultimately to runtime observation, which assumes nothing about the
install and always works once the game is running.

The provider is designed to compose with the platform walker that S030 will
build. The walker's job is to find a launch entry point (a stub executable) for
a title; the engine rule's job is to hop from that stub to the socket-holding
client. In this slice the entry point comes from scanning an install directory
directly, so the provider is independently testable and useful now; in S030 the
same provider consumes the walker's output unchanged.

## Clarifications

### Session 2026-08-12

- Q: When more than one `*-Win64-Shipping.exe` matches under `Binaries\Win64`,
  what does the provider do? -> A: It declines (returns no answer) and records
  the ambiguity in the resolution notes, so the cascade falls through to runtime
  observation, which disambiguates at runtime. The provider never silently picks
  one candidate (P-9), mirroring the runtime disambiguation S028 reserves for
  the identical-process case.
- Q: Are Unity and Ren'Py implemented in this slice or deferred? -> A:
  Implemented in this slice alongside Unreal. They share the provider's rule
  trait and their fixtures are cheap, so all three land together; Unreal remains
  the mandatory acceptance gate, and Unity/Ren'Py may split to a follow-up only
  if unforeseen complexity forces it.
- Q: What does an engine-rule answer carry as its resolved target? -> A: A new
  engine-rule target origin, distinct from the profile and observed origins,
  naming the resolved client executable (file name and full path) plus the match
  predicates the pipeline binds it by. This is an architecture-affecting change
  recorded in the plan and the slice changelog.
- Q: How does the provider receive its input, and how does it compose with the
  S030 walker? -> A: Through an optional install-root / launch-entry-point input
  on the resolution request; the provider declines when it is absent. The S030
  platform walker will populate that same input, so the walker composes with the
  provider without changing it.
- Q: Does this slice wire the full provider set into the CLI or facade? -> A:
  No. S029 delivers the provider and demonstrates its participation through the
  S027 resolver in tests, as S027 did for its own providers. Production CLI
  assembly of the whole provider cascade is out of scope for this slice, which
  makes no CLI changes.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Resolve an Unreal title's socket-holding client (Priority: P1)

An operator wants to capture an Unreal Engine game installed on disk. They point
fragcap at the install directory (or, later, let the platform walker find it).
The install root holds a stub `MyGame.exe` that, when run, relaunches
`MyGame-Win64-Shipping.exe` from `MyGame\Binaries\Win64`. The operator has no
authored profile for this title. The engine-rule provider recognizes the Unreal
layout, resolves the target to the shipping executable, and stamps the answer
`heuristic-unverified` with provenance `engine-rule`, so the cascade can hand a
correct capture target to the pipeline that the operator never had to author.

**Why this priority**: Unreal is the mandatory acceptance target for this slice
and the single highest-leverage engine, covering the largest share of the
launcher-stub problem the slice exists to solve. Delivering this story alone is
a viable, valuable increment: it turns a class of previously unresolvable titles
into resolvable ones with zero per-game data.

**Independent Test**: Construct a fixture directory tree in the Unreal twin-exe
shape (a stub in the root, a `*-Win64-Shipping.exe` under `Binaries\Win64`),
run it through the S027 resolver with only the engine-rule provider able to
answer, and assert the resolved target names the shipping executable at
`heuristic-unverified` fidelity with provenance `engine-rule`.

**Acceptance Scenarios**:

1. **Given** an install tree with a root stub and a matching
   `*-Win64-Shipping.exe` under `Binaries\Win64`, **When** the engine-rule
   provider resolves it, **Then** the target names the shipping executable,
   stamped `heuristic-unverified` fidelity and `engine-rule` provenance.
2. **Given** an install tree with no `Binaries\Win64` directory and no shipping
   executable, **When** the engine-rule provider resolves it, **Then** it
   returns no answer and the cascade falls through to the next provider.
3. **Given** an install tree that an authored profile already targets, **When**
   the resolver runs with both the profile provider and the engine-rule provider
   available, **Then** the profile provider's answer wins on precedence and the
   engine rule does not override a higher-trust answer.

---

### User Story 2 - Resolve Unity and Ren'Py titles by layout (Priority: P2)

An operator points fragcap at a Unity or Ren'Py game with no authored profile.
The engine-rule provider recognizes the Unity layout (a `*_Data` directory
beside the player executable, with a `UnityPlayer.dll`) or the Ren'Py layout (a
`renpy` directory and `.rpa` archives) and resolves the player executable at
`heuristic-unverified` fidelity with `engine-rule` provenance.

**Why this priority**: Unity and Ren'Py extend the same mechanism to two more
large engine populations at low marginal cost once the Unreal rule and the
provider scaffolding exist. They are valuable but not mandatory for this slice;
the plan permits deferring them to a follow-up if the slice grows.

**Independent Test**: Construct fixture directory trees in the Unity and Ren'Py
shapes and assert each resolves its player executable at `heuristic-unverified`
fidelity with `engine-rule` provenance, and that a directory matching neither
returns no answer.

**Acceptance Scenarios**:

1. **Given** an install tree with a `*_Data` directory and a `UnityPlayer.dll`
   beside a player executable, **When** the provider resolves it, **Then** the
   target names the player executable at `heuristic-unverified` fidelity with
   `engine-rule` provenance.
2. **Given** an install tree with a `renpy` directory and one or more `.rpa`
   archives, **When** the provider resolves it, **Then** the target names the
   Ren'Py launcher executable at `heuristic-unverified` fidelity with
   `engine-rule` provenance.

---

### Edge Cases

- A stub in the root with no `Binaries\Win64` and no shipping executable: the
  Unreal rule does not match; the provider returns no answer rather than
  guessing at the stub itself.
- Multiple `*-Win64-Shipping.exe` files under `Binaries\Win64` (rare, but
  possible with bundled tools): the provider declines and records the ambiguity
  rather than silently picking one, letting runtime observation disambiguate.
- A layout matching more than one engine rule (for example an Unreal game that
  also ships a stray `*_Data` directory): rule evaluation order must be defined
  and total so the answer does not depend on iteration order.
- A directory with no recognized engine layout at all: the provider returns no
  answer and the cascade continues; nothing is emitted at a fidelity higher than
  the evidence supports.
- The install directory does not exist or is unreadable: the provider returns no
  answer without erroring the whole resolution, and the reason is observable.
- An engine layout is recognized but the socket-holding executable named by the
  rule is absent from disk: the provider does not fabricate a target for a file
  that is not there.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide an engine-rule provider that implements the
  S027 target provider trait and participates in the target-resolution cascade at
  the engine-rule precedence position established by S027.
- **FR-002**: The provider MUST recognize the Unreal Engine install layout: a
  launch stub in the install root whose corresponding socket-holding client is a
  `*-Win64-Shipping.exe` file located under a `Binaries\Win64` directory, and
  MUST resolve the target to that shipping executable.
- **FR-003**: The provider MUST stamp every answer it produces at
  `heuristic-unverified` targeting fidelity with `provenance.source` equal to
  `engine-rule`, and MUST NOT stamp any answer at a higher fidelity tier.
- **FR-004**: The provider MUST return "no answer" (not an error, not a fabricated
  target) when no recognized engine layout is present, so the cascade falls
  through to the next provider.
- **FR-005**: The provider MUST derive its identification solely from
  filesystem inspection of the install directory. It MUST NOT open a process
  handle, read process memory, launch the target, or depend on artifacts that
  exist only after the game has run (such as per-user AppData files) or on
  launcher tokens.
- **FR-006**: The provider MUST resolve deterministically. Where more than one
  engine rule could match, rule evaluation order MUST be total and independent of
  iteration order. Where more than one candidate executable matches a single
  rule (for example two `*-Win64-Shipping.exe` files under `Binaries\Win64`), the
  provider MUST decline (return no answer) and record the ambiguity in the
  resolution notes rather than pick one arbitrarily, so the cascade falls through
  to runtime observation, which disambiguates at runtime.
- **FR-007**: The provider MUST accept a launch entry point (a stub executable or
  install directory) as its input so that a platform walker (S030) can feed the
  same provider without modification.
- **FR-008**: The provider MUST recognize the Unity install layout (a `*_Data`
  directory and a `UnityPlayer.dll` beside the player executable) and the Ren'Py
  install layout (a `renpy` directory and `.rpa` archives), resolving each to its
  player executable under the same fidelity and provenance rules, as additional
  rules in the same provider. Unreal (FR-002) is the mandatory acceptance target;
  Unity and Ren'Py land in this slice and may split to a follow-up only if
  unforeseen complexity forces it.
- **FR-009**: When the provider declines to answer or cannot read the install
  directory, the reason MUST be observable to the caller rather than swallowed,
  consistent with the no-silent-loss posture.
- **FR-010**: The term "engine rule" MUST have a full glossary entry in
  `docs/glossary/process-and-attribution.md`, and the target-resolution cascade
  section of `docs/fragcap-specification.md` MUST document engine rules, both
  landing in this slice.

### Key Entities *(include if feature involves data)*

- **Engine rule**: A named recognizer for one game engine's documented on-disk
  install layout. It takes an install directory or launch stub and, when the
  layout matches, names the socket-holding client executable. Attributes: the
  engine it recognizes, the layout signature it matches on (filename suffix plus
  directory convention), and the executable it resolves to.
- **Engine-rule provider**: The cascade provider that evaluates the set of engine
  rules against an input and emits a resolved target at `heuristic-unverified`
  fidelity with `engine-rule` provenance, or no answer.
- **Resolved target (engine-rule origin)**: The Target value the provider hands
  back: the resolved client identity plus its fidelity stamp and provenance,
  consumed by the cascade exactly like any other provider's answer.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: For a sample of at least three distinct Unreal twin-exe install
  layouts (differing game names), the provider resolves each to the correct
  `*-Win64-Shipping.exe` when run through the S027 resolver, at
  `heuristic-unverified` fidelity with `engine-rule` provenance.
- **SC-002**: For an install layout matching no recognized engine, the provider
  returns no answer 100% of the time and the resolver falls through to the next
  provider, with zero targets emitted at a fidelity higher than the evidence
  supports.
- **SC-003**: The resolution outcome is identical across repeated runs and across
  reordered directory contents for the same fixture, demonstrating that the
  answer does not depend on iteration order.
- **SC-004**: An authored or verified profile targeting the same install always
  outranks the engine-rule answer in the cascade, demonstrating the fidelity
  ordering holds end to end.
- **SC-005**: The full repository gate (`cargo xtask ci`), including the fixture
  drift check, passes with the new provider and fixtures in place.

## Assumptions

- The S027 provider trait, the `Target` type, the targeting-fidelity ranking
  (`authored` > `verified` > `heuristic-unverified` > `observed`), and the
  provenance representation exist and are the contract this provider implements;
  `engine-rule` is already a named `provenance.source` value in the master schema
  and glossary. This slice adds an implementation behind the existing engine-rule
  stub, not a new cascade position.
- The provider lives in the crate that already owns resolution and matching so
  the deps-direction check and the core allowlist are respected; no dependency is
  added to `fragcap-core`. Whether it is a submodule of that crate or a small new
  crate is a plan-time decision recorded there, decided against the dependency-
  direction gate.
- Fixture directory trees are built with a temporary-directory helper in the
  spirit of the existing Steam-crate `TempTree` test fixtures; no real game
  install is required to test the provider.
- Windows path conventions (`Binaries\Win64`, backslash separators) are the
  documented target; the rule matching is expressed so it is correct on the
  capture platform and does not assume a case-sensitive filesystem.
- Unity and Ren'Py (FR-008) are in scope for this slice alongside Unreal; they
  share the rule trait and cheap fixtures. Unreal (FR-002) is the hard
  acceptance gate, and Unity/Ren'Py split to a follow-up only if unforeseen
  complexity forces it, with the provider structured to admit more rules without
  rework either way.
