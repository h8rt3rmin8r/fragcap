# Feature Specification: Target Resolution Cascade -- Resolver Core

**Feature Branch**: `feat/target-resolver-core`

**Created**: 2026-08-12

**Status**: Draft

**Slice**: S027 (GitHub issue #77, slice 1 of 4). Builds the target-resolution
cascade as a first-class abstraction above the profile-reference lookup, on the
master JSON Schema and JSON profiles delivered by S025 (#75) and S026 (#76).
Constitution principles in play: the honesty posture that a guess is labeled as
one and never presented as a fact (P-9), no silent loss when nothing resolves
(P-4), the platform neutrality of the profile crate (P-2), and passive
observation only (P-1).

**Input**: Build a target resolver that consults an ordered list of providers of
varying trust, returns the highest-precedence available answer, stamps every
answer with a targeting fidelity tier and provenance, and imposes a total order
independent of internal iteration. Introduce a first-class Target distinct from
Profile. Surface fidelity, provenance, and kind on the loaded profile (currently
discarded). Encode the fidelity rank. Wire the two providers that already have
data (profile lookup, runtime observation) and register the other three (hint,
engine rule, platform walker) as no-answer stubs their own slices fill. Keep the
resolver in fragcap-profile and add nothing to fragcap-core. Keep targeting
fidelity separate from attribution fidelity. Foundation-only: no CLI watch flag,
no engine detection, no Steam appinfo, no hint DB.

## Overview

fragcap has, in its code and especially its docs, conflated two independent
problems: how a game is launched, and which process owns the packets once it is
running. This slice starts separating them by building the launch-agnostic core
that decides what to capture. The one durable fact fragcap can rely on is that at
runtime a process exists that is the game and holds sockets; how it launched,
where it installed, and which storefront sold it are all variable and
untrustworthy.

The mechanism is a cascade. Every source that can answer "what is this game's
target identity?" is a provider. The resolver consults providers in a fixed
precedence order and returns the first available answer, and every answer carries
a fidelity stamp so the tool never presents a guess as a fact. The precedence, in
descending trust, is: a user-authored target package, a curated or verified
profile, a shipped hint database, an engine rule, a platform walker, and finally
runtime observation, which is the ultimate arbiter because it assumes nothing
about origin and works identically for a modded install, a standalone game, and a
plain storefront title.

This slice delivers the spine and the two providers that already have data behind
them. The profile provider wraps the existing four-step profile lookup and stamps
its answer with the profile's own declared fidelity, which covers the top two
precedence positions (an authored package and a verified profile are both
profiles, distinguished by the fidelity they declare). The runtime-observation
provider yields an answer stamped observed from a process matching the target
identity, and sits at the bottom as the fallback that can always answer once the
game is running. The three providers in between, the hint database, the engine
rule, and the platform walker, are registered as real providers that currently
return no answer; their slices (#78, S029, S030) fill in the data without
touching the resolver's ordering.

Three properties are load-bearing.

**Precedence is total and does not depend on iteration order.** When more than one
provider could answer, the higher-precedence one wins, and it wins whatever order
the providers happen to be registered or iterated in. This is the same discipline
the attribution join already holds itself to: an implementation that takes the
first hit off an unordered set passes an ordinary test and produces answers that
change between runs. A permutation test over the provider order is what proves the
order is imposed rather than incidental.

**Every answer is stamped, and the stamp never overstates its source.** A resolved
target carries exactly one fidelity tier and a provenance. An observation answer
is labeled observed, never verified or authored; a profile answer carries the
fidelity the profile declared. The tier enum, whose order is documented in prose
in the master schema but not machine-encoded, gains a total rank so answers can be
compared and so the invariant that provider precedence never inverts fidelity can
be asserted rather than hoped for.

**Targeting fidelity and attribution fidelity stay separate.** The targeting tier
(authored, verified, heuristic-unverified, observed) describes how trustworthy a
target definition is and is what the resolver ranks by. The attribution fidelity
(live, retained, none) describes how a captured packet was attributed and is a
different mechanism on a different type. Neither is renamed, and neither is
derived from the other; the observed targeting tier is not the same thing as a
live attribution.

The slice stops at the resolver, its providers, the Target type, the surfacing of
fidelity on the profile, and their tests, plus the spec and glossary. It does not
add a launch-agnostic CLI surface (S028), engine detection (S029), Steam appinfo
walking (S030), or the hint database (#78). It makes the cascade the thing those
slices plug into.

## Clarifications

### Session 2026-08-12

Resolved under autopilot from the spec, the constitution, issue #77, the S025 and
S026 code already on main, and the existing resolution and matching architecture.

- Q: Is precedence a property of the provider or of the fidelity tier? -> A: The
  precedence is an ordered provider list, which is the resolver's spine. Fidelity
  is a stamp carried on each answer. The two correlate (a higher-precedence
  provider carries an equal-or-higher tier) and the resolver is designed so
  provider order never inverts fidelity order, which is asserted. The three
  heuristic-unverified providers (hint, engine, walker) share a tier but occupy
  distinct precedence positions, so provider order carries information fidelity
  alone does not.
- Q: One profile provider or two (authored package versus verified profile)? ->
  A: One. The existing lookup returns a single resolved profile that declares its
  own fidelity (authored for a user package, verified for a curated one). The
  profile provider stamps its answer with the profile's declared fidelity, which
  satisfies both of the top two precedence positions without a second provider.
- Q: What does the runtime-observation provider do in this slice, given the
  confirm-and-override loop the issue describes? -> A: It yields an observed
  target from a process matching the target identity and sits at the bottom of the
  cascade as the fallback. The promotion and confirm-or-override feedback (a live
  capture upgrading a hint, fed back as a verified profile through a submission
  pipeline) is future work and explicitly out of scope here; this slice delivers
  observation as the lowest-precedence answer, not the override authority.
- Q: How far does CLI integration go in this slice? -> A: Minimal and
  behavior-preserving. The existing profile path in the run command flows through
  the resolver and receives a profile-backed target, and capture output is
  unchanged. The observation provider is exercised by tests, not by a new CLI
  flag; wiring observation into a live launch-agnostic capture is S028 (watch
  mode). There is no user-visible surface change in this slice.
- Q: Where does the Target and the resolver live, and does anything reach into
  core? -> A: In fragcap-profile, the crate that already owns resolution and
  matching. Nothing is added to fragcap-core, whose dependency allowlist stays
  ["bytes"]. The resolver is pure logic over already-parsed inputs and the
  process tree the matching module already reads; it opens no process handle and
  adds no external crate.
- Q: How is the observed target's identity formed without a profile behind it? ->
  A: From the matching machinery that already exists: a process in the tree whose
  image name and path a caller-supplied identity selects. The observation provider
  turns that matched process into a target stamped observed. It uses only the
  image name and path already in the process snapshot; it opens no handle and
  reads no process memory (P-1).
- Q: What constitutes the target identity a Target carries and the observation
  provider matches on? -> A: The match predicates that already exist: an exe
  image-name glob plus optional path anchors (a path substring or a path regex),
  which is issue #77's durable "exe image name plus a path anchor" key. A Target's
  match rules are those predicates; the observation provider selects a process
  from the tree by them; and ancestry (descends_from) is reserved for genuine
  runtime disambiguation rather than being the identity's spine. No new identity
  vocabulary is introduced in this slice.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Resolve a target by fidelity-ranked precedence (Priority: P1)

fragcap needs to decide what to capture for a game. It consults its providers in a
fixed precedence order and returns the highest-precedence available answer,
stamped with the fidelity tier and provenance of the source that produced it.

**Why this priority**: This is the cascade itself and the foundation every later
targeting slice plugs into. Without a resolver that imposes a total precedence and
stamps every answer, there is nothing for the engine-rule provider, the platform
walker, or the hint database to feed, and no honest fidelity to report.

**Independent Test**: Register providers at more than one precedence position,
have two of them able to answer for the same game, and confirm the resolver
returns the higher-precedence answer with the correct fidelity and provenance,
and returns it identically no matter what order the providers were registered in.

**Acceptance Scenarios**:

1. **Given** two providers that can both answer for a game, **When** the target is
   resolved, **Then** the higher-precedence provider's answer is returned, carrying
   that provider's fidelity tier and provenance.
2. **Given** the same providers registered in a different order, **When** the
   target is resolved, **Then** the same answer is returned; the outcome does not
   depend on registration or iteration order.
3. **Given** a resolved target, **When** it is inspected, **Then** it carries
   exactly one fidelity tier and a provenance naming the source that produced it.
4. **Given** a lower-precedence provider that can answer and a higher-precedence
   provider that cannot, **When** the target is resolved, **Then** the
   lower-precedence answer is returned, still correctly stamped.

---

### User Story 2 - Fall back to runtime observation for a game with no definition (Priority: P2)

A game has no authored package, no curated profile, no hint, and no engine or
platform answer, which is the common case for a standalone or modded install. Once
the game is running, the resolver still produces a target from the process that
matches the identity, stamped observed.

**Why this priority**: Runtime observation is what makes fragcap work for the many
titles that have no curated definition at all. It is the arbiter at the bottom of
the cascade and the reason the core is launch-agnostic. It builds on US1's
ordering (observation is the lowest-precedence provider).

**Independent Test**: With no higher-precedence provider able to answer, present a
process tree containing a process that matches a supplied identity, and confirm
the resolver returns a target stamped observed derived from that process, using
only its image name and path.

**Acceptance Scenarios**:

1. **Given** no higher-precedence provider can answer and a matching process
   exists, **When** the target is resolved, **Then** an observed target derived
   from that process is returned.
2. **Given** no higher-precedence provider can answer and no matching process
   exists yet, **When** the target is resolved, **Then** no answer is produced and
   the outcome is a distinct, named not-resolved result, not a silent empty
   answer.
3. **Given** an observed target, **When** its fidelity is read, **Then** it is
   observed, never verified or authored.

---

### User Story 3 - Read the trust level of any definition or answer (Priority: P3)

An operator, a downstream consumer, or a later confirm-and-override step needs to
know how trustworthy a target definition is. A loaded profile exposes the
fidelity, provenance, and kind it declared, and every resolved target exposes the
fidelity and provenance of its answer.

**Why this priority**: The honesty posture (P-9) depends on the trust level being
carried and readable rather than inferred. The in-memory profile currently
discards these fields after validation, so this is a real gap that the resolver
and every future consumer depend on. It builds on US1 (the answers whose trust is
being read).

**Independent Test**: Load a profile that declares a fidelity and provenance, and
confirm both, and the kind, are readable from the in-memory profile; resolve a
target and confirm the answer's fidelity and provenance are readable.

**Acceptance Scenarios**:

1. **Given** a loaded profile that declares a fidelity, a provenance, and a kind,
   **When** the in-memory profile is inspected, **Then** all three are readable and
   match what the file declared.
2. **Given** a resolved target from any provider, **When** it is inspected,
   **Then** its fidelity tier and provenance are readable.
3. **Given** the targeting fidelity of an answer and the attribution fidelity of a
   captured packet, **When** both are examined, **Then** they are distinct values
   on distinct types, neither derived from the other.

---

### Edge Cases

- No provider can answer: the resolver returns a distinct, named not-resolved
  outcome rather than an empty success, so a capture is never armed against
  nothing without saying so (P-4, P-9).
- Two providers at different precedence positions can both answer: the higher
  wins, deterministically, regardless of iteration order.
- A provider higher in the chain than observation returns no answer while
  observation can: observation answers, stamped observed, and is not shadowed by
  the higher provider's silence.
- A stub provider (hint, engine, walker) is consulted: it returns no answer
  cleanly, and its presence changes no result until its data slice lands.
- The observation provider is consulted before any matching process exists: it
  returns no answer, which is the not-resolved outcome, not an error.
- A profile that omits a required fidelity or provenance is already refused at load
  by S026; the resolver never sees an unstamped profile.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The resolver MUST consult an ordered set of providers in a fixed
  precedence order and return the highest-precedence available answer.
- **FR-002**: Every resolved target MUST carry exactly one targeting fidelity tier
  and a provenance identifying the source that produced it.
- **FR-003**: The targeting fidelity tiers MUST have a defined total rank
  (authored > verified > heuristic-unverified > observed), and the resolver's
  provider precedence MUST NOT invert that rank.
- **FR-004**: Resolution MUST be deterministic and independent of the order in
  which providers are registered or internally iterated; the precedence order is
  imposed, not incidental.
- **FR-005**: A Target MUST be a first-class value distinct from a Profile,
  carrying the resolved target identity, its match rules, its fidelity, and its
  provenance, sufficient for the capture pipeline to act on an answer that did not
  come from a profile. The identity and match rules MUST be expressed with the
  existing match predicates (an exe image-name glob plus optional path anchors);
  no new identity vocabulary is introduced.
- **FR-006**: The profile provider MUST produce a target from the existing profile
  resolution, stamped with the profile's own declared fidelity, covering the
  authored-package and verified-profile precedence positions.
- **FR-007**: The runtime-observation provider MUST produce a target stamped
  observed from a process that matches the supplied identity, and MUST be the
  lowest-precedence provider; it MUST use only the image name and path already in
  the process snapshot and MUST open no process handle (P-1).
- **FR-008**: The hint-database, engine-rule, and platform-walker providers MUST be
  registered as providers that currently return no answer, so that adding their
  data in later slices is additive and requires no change to the resolver's
  ordering.
- **FR-009**: A loaded profile MUST expose the fidelity, provenance, and kind it
  declared; these MUST NOT be discarded after validation.
- **FR-010**: The targeting fidelity MUST remain a separate mechanism from the
  attribution fidelity (live, retained, none); neither MUST be renamed, and neither
  MUST be derived from the other.
- **FR-011**: When no provider can answer, resolution MUST report a distinct, named
  not-resolved outcome, never a silent empty answer that could arm a capture
  against nothing (P-4).
- **FR-012**: The resolver and the Target type MUST live in the profile crate and
  MUST add no dependency to the core crate (allowlist ["bytes"], P-2); no new
  external crate is expected.
- **FR-013**: The master specification MUST gain a section describing the
  resolution cascade, cross-referencing target acquisition, the master schema and
  target artifacts, and the narrower profile-reference resolution order; and the
  glossary MUST gain entries for provider, target resolver, resolution cascade, and
  target in the same change (P-6).

### Key Entities *(include if feature involves data)*

- **Provider**: A source that can answer "what is this game's target identity?"
  Yields either an answer stamped with its fidelity and provenance, or no answer.
  Occupies a fixed position in the precedence order.
- **Target resolver**: The component that consults providers in precedence order
  and returns the highest-precedence available answer, or a named not-resolved
  outcome.
- **Target**: The resolved answer handed to the capture pipeline: the target
  identity (an exe image-name glob plus optional path anchors, per the existing
  match predicates), its match rules, its fidelity tier, and its provenance.
  Distinct from a profile; a profile is one way to back a target, not the only way.
- **Fidelity tier (with rank)**: The targeting trust tier (authored, verified,
  heuristic-unverified, observed) with a total order the resolver imposes and
  compares against.
- **Resolution outcome**: Either a resolved target or a distinct not-resolved
  result naming that nothing answered.
- **Profile (extended)**: The in-memory profile, now also exposing its declared
  fidelity, provenance, and kind.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: For every permutation of the provider registration order, the
  resolver returns the same answer for the same inputs, 100% of the time.
- **SC-002**: Every resolved target is stamped with exactly one fidelity tier and a
  provenance; no resolved target is ever unstamped.
- **SC-003**: A game with no authored package, curated profile, hint, engine, or
  platform answer still resolves to an observed target once a matching process
  exists.
- **SC-004**: A loaded profile's declared fidelity, provenance, and kind are
  readable programmatically 100% of the time.
- **SC-005**: Adding a later provider's data (hint, engine, or platform walker)
  requires no change to the resolver's ordering logic; the change is confined to
  that provider.
- **SC-006**: No resolved answer is ever stamped at a higher fidelity than its
  source; an observed answer is never labeled verified or authored.
- **SC-007**: When no provider can answer, resolution returns the named
  not-resolved outcome, never a silent empty success.
- **SC-008**: Capture output for an existing profile-backed run is byte-identical
  before and after the resolver is introduced; the integration is
  behavior-preserving.

## Assumptions

- **Placement and dependencies.** The resolver, the providers, and the Target type
  live in fragcap-profile. Nothing is added to fragcap-core (allowlist ["bytes"]).
  No new external crate is expected; the resolver is pure logic over already-parsed
  profiles and the process tree the matching module already reads.
- **Two live providers, three stubs.** Only the profile provider and the
  runtime-observation provider carry data in this slice. The hint-database,
  engine-rule, and platform-walker providers are registered and return no answer;
  #78, S029, and S030 fill them in.
- **Fidelity comes from the source.** A profile answer's fidelity is the profile's
  declared tier; an observation answer's fidelity is observed. The resolver does
  not invent or upgrade a tier.
- **Confirm-and-override is future.** The feedback loop in which a live capture
  confirms or overrides a higher-chain guess and is promoted to a verified profile
  through a submission pipeline is out of scope; observation here is the
  lowest-precedence answer, not the override authority.
- **Behavior-preserving integration.** The run command's existing profile path
  flows through the resolver and receives a profile-backed target with unchanged
  capture output. No launch-agnostic CLI surface is added; that is S028.
- **Fidelity separation.** Targeting fidelity (authored, verified,
  heuristic-unverified, observed) and attribution fidelity (live, retained, none)
  are distinct mechanisms on distinct types; this slice does not touch the
  attribution enum.
- **Toolchain.** The workspace minimum supported toolchain stays 1.82 and MUST
  remain green; the slice adds no dependency.
- **Text hygiene.** All artifacts are UTF-8 without BOM, LF line endings, and
  contain no em-dashes or en-dashes, including code comments and JSON string
  values.
