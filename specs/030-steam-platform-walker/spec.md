# Feature Specification: Steam Platform-Walker Refactor

**Feature Branch**: `feat/steam-platform-walker`

**Created**: 2026-08-12

**Status**: Draft

**Slice**: S030 (GitHub issue #77, slice 4 of 4, the final slice). Demotes Steam
from "the feature" to one optional provider feeding the target-resolution cascade
built by S027 and composed with the engine-rule provider of S029. Constitution
principles in play: passive observation only, so the walker reads the filesystem
and registry and never opens a process handle or reads process memory (P-1); the
dependency direction, so the walker provider lives in `fragcap-steam` and
`fragcap-profile` never depends on it (P-2, P-3); no silent loss, so a declined or
degraded resolution is a named, surfaced outcome (P-4); a glossary entry for the
new term in the same change (P-6); and an honest fidelity stamp and an honest
provenance that names what the walker actually did (P-9).

**Input**: Make `fragcap-steam`'s installed-title enumeration flow through the
S027 resolver as a platform-walker provider at `Precedence::PlatformWalker`,
replacing the no-op stub. The walker supplies a title's install directory as the
request's `install_root` so the higher-precedence engine-rule provider (S029) can
hop to the socket holder; when the engine rule does not recognize the layout, the
walker answers at its own precedence by classifying the install directory's
executables. Every answer is `heuristic-unverified` with an honest provenance.
When enumeration finds nothing or is genuinely ambiguous, the walker declines and
the cascade falls through to runtime observation. Steam's managed `steam://`
launch stays a convenience adapter. Appinfo/PICS reading is deferred; this slice
wires the existing library walk and install-directory classification into the
cascade with no new dependency.

## Overview

fragcap's targeting has, in its code and especially its history, treated Steam as
the spine: the way a game is found, launched, and identified. The resolution
cascade built across S027 through S029 reframes that. The durable fact is that at
runtime a process exists that is the game and holds the sockets; how it was found,
where it installed, and which storefront sold it are all variable. Steam is one
source of "where is this game and what might its client be," no more privileged
than an engine rule or a hint database, and strictly less trustworthy than a
user-authored profile.

This slice makes that concrete. `fragcap-steam` already reads Steam's local
library metadata to enumerate installed titles, each with an application id, a
name, and an install directory, and it already has a classifier that picks a
likely client executable from an install directory's contents by dropping
installers and helpers and preferring the largest non-launcher binary. Until now
that machinery only produced a standalone scaffold file for a human to review.
S030 wires it into the cascade as a provider.

The provider's place in the cascade is deliberate. It sits below the engine rule,
because an engine's documented layout is a more specific signal than a
storefront's generic "here is the install directory." So the walker's first and
best contribution is to supply the install directory as the request's
`install_root`, letting the engine-rule provider name the socket holder from
layout (an Unreal title installed through Steam resolves through the engine rule,
not the walker). When the engine rule does not recognize the layout, the walker
answers at its own lower precedence by classifying the install directory's
executables into a client. Both answers are `heuristic-unverified`, because a
storefront's install record and an executable-naming heuristic are guesses, not
facts an author vouched for.

Honesty about method matters here. The walker resolves from Steam's library
manifests and from classifying files in the install directory; it does not read
Steam's application info (the `config.launch` array that names the executable
Steam itself invokes). That data lives only in Steam's networked product-info
service or a local binary cache this slice does not parse. So the walker's
provenance is named for what it did, a library walk and install-directory
classification, and not for a source it did not consult. Reading application info
is a documented future direction, not part of this slice.

When Steam has nothing to offer, the cascade still works. A title that is not
installed, or an install directory whose executables the classifier cannot
confidently reduce to one client (several near-identical binaries, the shape of a
few launcher-mediated titles), makes the walker decline rather than guess. The
cascade then falls through to runtime observation, the arbiter that assumes
nothing about origin and resolves the game once it is running. That degradation is
the point of the cascade, and it is a required, tested behavior of this slice.

## Clarifications

### Session 2026-08-12

- Q: Does this slice read Steam application info (the `config.launch` launch
  array) for the launch executable? -> A: No. The launch executable lives only in
  Steam's networked product-info service or a local binary cache, which would
  require a heavy networked dependency or a binary-format parser this slice does
  not add. The engine rule (S029) and the existing install-directory classifier
  already name the client for the common cases, and hard titles degrade to runtime
  observation, so appinfo/PICS reading is deferred to a follow-up slice. Recorded
  as an architecture decision.
- Q: What provenance does a walker answer carry? -> A: A source that names what
  the walker actually did, `steam-library` (a library-manifest walk plus
  install-directory classification), not `steam-appinfo`, which would claim a
  source the walker did not read (P-9).
- Q: Where does the walker provider live, given the cascade lives in
  `fragcap-profile`? -> A: In `fragcap-steam`, which already depends on
  `fragcap-profile` and so can implement its provider trait. `fragcap-profile`
  must not depend on `fragcap-steam` (the dependency-direction check forbids it),
  so the no-op stub in `fragcap-profile` is retired and the CLI/facade, which
  depends on both crates, assembles the resolver with the real walker.
- Q: What does the walker do on a genuinely ambiguous install (several
  near-identical client candidates)? -> A: It declines and the ambiguity is a
  surfaced outcome, letting runtime observation disambiguate once the game is
  running, rather than the classifier picking one arbitrarily for an automatic
  capture (P-9).
- Q: Does the `steam://` managed launch change? -> A: No. It stays a convenience
  adapter, unchanged; it is simply no longer framed as the spine.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Capture a Steam-installed engine title through the cascade (Priority: P1)

An operator wants to capture a game installed through Steam that is built on a
recognized engine (for example an Unreal title) and has no authored profile. The
walker enumerates the Steam library, finds the title's install directory, and
supplies it to the resolver. The engine-rule provider recognizes the layout and
resolves the socket-holding client, stamped `heuristic-unverified`. Steam did the
finding; the engine rule did the naming; the operator authored nothing.

**Why this priority**: This is the composition the whole of #77 was built toward
and the most common installed-game shape. It proves the walker feeds the cascade
and composes with the engine rule end to end. Delivered alone it is a viable,
valuable increment.

**Independent Test**: Build a fake Steam library fixture whose title install
directory is an Unreal twin-exe layout, run it through the resolver assembled with
the walker and the engine-rule provider, and assert the shipping executable is
resolved at `heuristic-unverified` (the engine rule's answer, because it outranks
the walker).

**Acceptance Scenarios**:

1. **Given** a Steam library with an installed Unreal title and no matching
   profile, **When** the resolver runs with the walker supplying `install_root`,
   **Then** the engine-rule provider resolves the shipping executable at
   `heuristic-unverified`.
2. **Given** the same title with a matching authored profile, **When** the
   resolver runs, **Then** the profile outranks both the engine rule and the
   walker, and the profile's answer is returned.

---

### User Story 2 - Capture a Steam-installed non-engine title through the walker (Priority: P1)

An operator wants to capture a Steam-installed game whose install directory the
engine rule does not recognize (no engine layout), with no authored profile. The
engine rule declines; the walker classifies the install directory's executables,
drops installers and helpers, and names the most likely client, stamped
`heuristic-unverified` with provenance `steam-library`.

**Why this priority**: This is the walker's own distinct contribution below the
engine rule, and it is what makes Steam a useful provider rather than only a
finder of install directories. Without it, non-engine Steam titles would fall
straight through to runtime observation even when the install directory names an
obvious single client.

**Independent Test**: Build a fake Steam library whose title install directory
holds one clear client executable plus installers and helpers (no engine markers),
run it through the resolver, and assert the walker resolves the client at
`heuristic-unverified` with provenance `steam-library`.

**Acceptance Scenarios**:

1. **Given** an installed non-engine title with one clear client executable among
   installers and helpers, **When** the resolver runs and the engine rule
   declines, **Then** the walker resolves the client at `heuristic-unverified`
   with provenance `steam-library`.

---

### User Story 3 - Graceful degradation to runtime observation (Priority: P1)

An operator targets a Steam title that is either not installed or whose install
directory is genuinely ambiguous (several near-identical executables). The walker
declines rather than guess, and the cascade falls through to runtime observation,
which resolves the game once a matching process is running.

**Why this priority**: Degradation is the safety property of the whole cascade and
an explicit done-condition of this slice. A walker that guessed on ambiguity would
produce a confident wrong target; declining preserves honesty and hands the
question to the arbiter that can actually answer it at runtime.

**Independent Test**: (a) A resolver over a Steam library that does not contain the
requested title declines at the walker and resolves via the observation provider
when a matching process is present. (b) A resolver over an install directory with
several near-identical client candidates declines at the walker with the ambiguity
surfaced, and resolves via observation.

**Acceptance Scenarios**:

1. **Given** a requested title not present in the Steam library, **When** the
   resolver runs, **Then** the walker declines and runtime observation resolves a
   matching live process.
2. **Given** an installed title whose install directory is genuinely ambiguous,
   **When** the resolver runs, **Then** the walker declines with the ambiguity
   surfaced and runtime observation resolves the live process.

---

### Edge Cases

- The requested title is not installed in any Steam library: the walker declines;
  nothing is fabricated; the cascade continues.
- The install directory holds no plausible client executable (only installers and
  helpers): the walker declines rather than name a helper.
- The install directory holds several near-identical client candidates the
  classifier cannot reduce to one: the walker declines and the ambiguity is
  surfaced.
- A Steam library path or manifest is unreadable or malformed: it is reported as a
  non-fatal warning and enumeration continues over the readable remainder, never
  silently dropped.
- No Steam installation is present on the machine at all: the walker declines
  cleanly; the rest of the cascade is unaffected.
- The engine rule already resolved the title (higher precedence): the walker's own
  classification is not consulted, and the engine rule's answer stands.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide a platform-walker provider that implements
  the S027 target-provider trait at the `PlatformWalker` precedence position and
  participates in the target-resolution cascade, replacing the no-op stub.
- **FR-002**: The walker provider MUST be located in the Steam crate, which
  already depends on the profile crate; the profile crate MUST NOT gain a
  dependency on the Steam crate (the dependency-direction check must stay green).
- **FR-003**: The walker MUST make a Steam title's install directory available to
  the resolver as the request's install-root input, so the higher-precedence
  engine-rule provider can resolve the socket holder from layout when it
  recognizes one.
- **FR-004**: When the engine rule does not resolve, the walker provider MUST
  answer by classifying the install directory's executables (dropping installers
  and helpers, preferring the client over launcher stubs) into a single client
  identity, stamped `heuristic-unverified`.
- **FR-005**: The walker MUST stamp every answer at `heuristic-unverified`
  targeting fidelity with a provenance source of `steam-library`, naming the
  library walk and install-directory classification actually performed, and MUST
  NOT claim a source (such as `steam-appinfo`) it did not read (P-9).
- **FR-006**: The walker MUST decline (return no answer) when the requested title
  is not installed, when the install directory holds no plausible client, or when
  the install directory is genuinely ambiguous (several near-identical
  candidates); a declined resolution MUST let the cascade continue to runtime
  observation, and an ambiguity MUST be a surfaced outcome rather than a silent
  drop (P-4).
- **FR-007**: The walker MUST derive its answer from filesystem and registry
  reads only, as the Steam crate already does. It MUST NOT open a process handle,
  read process memory, or perform network access (P-1).
- **FR-008**: Non-fatal enumeration problems (an unreadable library path, a
  malformed manifest) MUST be surfaced as warnings and MUST NOT abort enumeration
  or silently drop titles.
- **FR-009**: The walker's resolved target MUST be usable by the capture pipeline
  the same way other resolved targets are (it carries a client identity: an
  executable image name and optional path anchors), and it MUST validate against
  the master target schema where it is materialized as a target artifact.
- **FR-010**: The `steam://` managed launch MUST remain available and unchanged as
  a convenience adapter; this slice MUST NOT make managed launch a precondition of
  resolution or capture.
- **FR-011**: The term "platform walker" MUST gain a full glossary entry in the
  same change, and the specification MUST document the platform walker under the
  resolution-cascade section and reframe the Steam-integration section to present
  Steam as one adapter feeding the cascade (P-6).

### Key Entities *(include if feature involves data)*

- **Platform walker**: The cascade provider that turns a storefront's installed
  library into cascade answers: it locates a title's install directory (so the
  engine rule can use it) and, as a fallback, classifies that directory's
  executables into a client identity. Attributes: the enumerated installed titles
  (application id, name, install directory) and the classifier that reduces a
  directory to a client or declines.
- **Walker-resolved target**: The target the walker hands back when it answers
  directly: the resolved client identity, its `heuristic-unverified` fidelity, and
  its `steam-library` provenance, plus the Steam application id and title name it
  came from, consumed by the cascade exactly like any other provider's answer.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A Steam-installed Unreal title with no authored profile resolves to
  its shipping executable through the resolver (via the engine rule, fed the
  install directory by the walker) at `heuristic-unverified`, in a fixture test.
- **SC-002**: A Steam-installed non-engine title with one clear client resolves
  through the walker at `heuristic-unverified` with provenance `steam-library`, in
  a fixture test.
- **SC-003**: A requested title that is not installed, and an install directory
  that is genuinely ambiguous, each cause the walker to decline and the cascade to
  fall through to runtime observation, in fixture tests; zero targets are emitted
  from an ambiguous install.
- **SC-004**: An authored or verified profile for the same Steam title outranks
  both the engine rule and the walker in the cascade, demonstrating the fidelity
  order holds end to end.
- **SC-005**: The dependency-direction check confirms the walker provider lives in
  the Steam crate and the profile crate has not gained a dependency on it.
- **SC-006**: The full repository gate (`cargo xtask ci`) passes with the walker
  wired into the production resolver.

## Assumptions

- The S027 cascade, the provider trait, the `Target` type, the targeting-fidelity
  ranking, the provenance representation, and the S029 `install_root` request
  input all exist and are the contract this walker implements. The `install_root`
  input was added by S029 precisely for the walker to populate.
- `fragcap-steam` already enumerates installed titles (application id, name,
  install directory) from Steam's local library manifests and already has an
  executable classifier that names a likely client from an install directory; this
  slice reuses that machinery rather than adding new enumeration or a new
  dependency.
- A walker answer needs a target origin distinct from the profile, engine-rule,
  and observed origins (it is none of those); adding one is a plan-time decision
  recorded there, mirroring how S029 added its engine-rule origin.
- Reading Steam application info (the launch array) is out of scope and deferred:
  it lives only in a networked product-info service or a local binary cache, and
  the engine rule plus the install-directory classifier already cover the common
  cases while hard titles degrade to runtime observation. The launcher-mediated
  flag and the full launch-array model belong with the hint-database revision, not
  this refactor.
- Fixtures are temporary directory trees built at test time (a fake Steam library
  with library manifests and install directories, composed with the engine-rule
  install-layout fixtures), in the spirit of the existing Steam-crate and
  profile-crate test helpers; no real Steam installation is required.
