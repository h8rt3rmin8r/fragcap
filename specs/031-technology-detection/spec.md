# Feature Specification: Technology-Detection Surface

**Feature Branch**: `feat/technology-detection`

**Created**: 2026-08-13

**Status**: Draft

**Slice**: S031 (operator decision 2026-08-12; follows the S027 through S030
resolution cascade of GitHub issue #77, but is a distinct concern). Where the
engine-rule provider (S029) recognizes an engine's install layout only to name
the socket-holding client, this slice reports the full set of technologies a
game's install directory reveals: engine, anti-cheat, SDK, emulator, container,
and launcher. Constitution principles in play: passive observation only, so
detection reads file paths and never file contents, and never opens a process
handle or reads process memory (P-1); no silent loss, so a ruleset pattern that
cannot compile is a named, counted, surfaced skip rather than a quiet drop
(P-4); a glossary entry for the new terms in the same change (P-6); and an
honest fidelity stamp that names a filename heuristic as `heuristic-unverified`,
never as fact (P-9). Surfacing anti-cheat is itself an on-mission P-9
user-safety and consent signal: an operator has a right to know a game they are
about to capture ships an anti-cheat that watches process behavior.

**Input**: Vendor the SteamDB `FileDetectionRuleSets` `rules.ini` (MIT, (c) 2021
SteamDB) as a committed, integrity-locked data asset (a NOTICE carrying the
attribution and a lock file recording the source repository, the pinned commit,
the license, and a SHA-256 over the vendored bytes, in the same spirit as
`skills-lock.json`). Build a technology-detection surface that scans a target's
local install directory and reports the technologies present per target, grouped
by category: game engine, anti-cheat, SDK, emulator, container, and launcher.
Detection matches the vendored ruleset's path regexes against the install's file
paths (filenames and relative paths only, never file contents), consistent with
P-1: fragcap detects technologies, it does not evade them or interact with them.
Results surface to the user on the command line and are written into output
metadata as a multi-category `technologies` structure carrying, per finding, its
category, the technology's name, the marker path that matched, and a
`heuristic-unverified` fidelity. This detection labels technologies from install
layout; it does not replace the resolver's separate logic for deciding which
executable is the socket-holding client. No new runtime dependency is added (the
existing regex engine is reused); no file contents are read; the minimum
supported toolchain stays green; and every ruleset pattern the regex engine
cannot compile is skipped through a named, counted, surfaced path.

## Overview

fragcap already reasons about a game's install directory in two narrow ways. The
engine-rule provider (S029) reads the layout to decide which executable holds the
sockets, and the Steam walker (S030) classifies a directory's executables into a
likely client. Both answer exactly one question: which process is the game.
Neither tells the operator what the game is built on or what watches it while it
runs.

This slice adds that missing surface. The same install directory that reveals a
`*-Win64-Shipping.exe` also, very often, reveals an `EasyAntiCheat` directory, a
`BattlEye` service, a Steamworks SDK dll, an Electron container, or a bundled
emulator. Those facts matter to an operator for two reasons. First, an
anti-cheat is a consent and safety signal: fragcap is designed so that nothing it
does resembles what anti-cheat watches for, and an operator deserves to be told,
before a capture, that a game ships one. Second, knowing the engine, SDK, and
container of a target is exactly the context that makes a capture
interpretable later; it belongs in the output's metadata next to the flows.

The detection method is deliberately the same shape as everything else fragcap
does with an install directory: it reads paths, never contents. SteamDB's open
`FileDetectionRuleSets` ruleset is the maintained, community-curated source
behind SteamDB's own technology attribution, and it detects technologies from
depot file paths alone, which is precisely fragcap's P-1 constraint. S029 already
aligned its four hand-written engine signatures to a subset of that ruleset but
deliberately did not vendor or execute the whole thing, recording that
catalog-scale detection (the full anti-cheat, SDK, and emulator coverage) was a
separate concern. This slice is that separate concern. Because fragcap runs
against games the operator has installed, it walks a real filesystem and applies
the whole ruleset to it; the license gate SteamDB itself faces (depot manifests
for games nobody owns) does not apply to a local install the operator already
has on disk.

Two honest boundaries frame the slice. The ruleset labels a technology from a
marker file; it does not name the client executable, so this surface never
overrides the resolver's socket-holder decision. And a filename is a guess, not a
guarantee, so every finding is stamped `heuristic-unverified`, matching the
ruleset's own stated posture that its detections are educated guesses.

## Clarifications

### Session 2026-08-13

- Q: Is the whole `rules.ini` vendored, or only the sections this slice
  surfaces? -> A: The whole file is vendored verbatim, unmodified, so the lock
  hash is a meaningful integrity check against the pinned upstream commit and the
  NOTICE attribution covers the complete work. The detection engine applies the
  category sections it understands; vendoring the whole file and applying a
  subset are independent decisions, and vendoring the whole keeps the asset
  honest and re-verifiable.
- Q: The ruleset's regexes are written for a PCRE-style engine and use features
  (for example possessive quantifiers such as `\w++`) that fragcap's regex engine
  does not support. What happens to a pattern that fails to compile? -> A: It is
  skipped, and the skip is counted and surfaced (a count of skipped patterns, and
  the ability to name which technologies were affected), never silently dropped
  (P-4). Detection proceeds with every pattern that does compile. The vendored
  bytes are never edited to make a pattern compile; the asset stays a faithful
  copy and the incompatibility is handled at load time in code.
- Q: Does the ruleset's two-pass "Evidence" deduction (secondary patterns that
  build hints, then deduction logic that infers an engine when the first pass is
  ambiguous) get implemented? -> A: No, not in this slice. This slice applies the
  direct category sections (engine, anti-cheat, SDK, emulator, container,
  launcher), which each match a marker path to a named technology directly. The
  Evidence second-pass inference engine is a larger, separable concern and is
  deferred, recorded as a decision. Deferring it does not weaken the anti-cheat,
  SDK, engine, emulator, container, or launcher findings, which are all
  first-pass direct matches.
- Q: The schema categories named for this surface (engine, anti_cheat, sdk,
  framework, emulator, container, runtime, launcher) are a superset of the
  ruleset's sections (Engine, AntiCheat, SDK, Emulator, Container, Launcher, plus
  Evidence). How are they reconciled? -> A: The `technologies` schema defines the
  full superset as its category vocabulary so the structure is stable for future
  sources (a hint database, or the Evidence pass). This slice populates the
  categories the vendored ruleset's direct sections map to; the extra categories
  (framework, runtime) are valid, defined, and simply unpopulated by this source
  today. The ruleset's `Launcher` section maps to the `launcher` category, and
  `Evidence` is not a category (it is deferred inference machinery).
- Q: Where does the vendored asset and the detection code live? -> A: The asset
  lives beside the existing embedded schema in the profile crate's assets
  directory, and the detection module lives in the profile crate next to the
  engine-rule module, because it is filesystem-and-path reasoning over an install
  directory with no platform-specific dependency, exactly like the engine rule.
  It adds nothing to the core crate's dependency allowlist and nothing to the
  lockfile (the regex engine is already a dependency).
- Q: Is anti-cheat detection framed as an evasion aid? -> A: No, the opposite. It
  is a user-safety and consent signal (P-9): the operator is told what watches
  the game so they can make an informed choice before capturing. fragcap detects;
  it never hooks, injects, or interacts with the anti-cheat, and the detection
  reads only file paths.
- Q: How is technology detection invoked, and does it run automatically during a
  live capture? -> A: Through a dedicated on-demand CLI subcommand that scans a
  given install directory and prints the grouped report, and additionally the
  profile-scaffold path enriches the target artifact it materializes with the
  detected technologies (the scaffold already reasons about an install directory,
  so it is the natural place for the persisted result). Detection does NOT run
  automatically inside the live `run` capture loop, and the pcapng and JSON Lines
  packet writers are unchanged: adding per-capture detection would put a
  filesystem walk on the capture path and change output files unmodified analyzers
  read, for no benefit the on-demand-plus-scaffold surface does not already
  deliver (P-5). Recorded as a decision.
- Q: The metadata lands in "output metadata" - which artifact, the packet-stream
  capture files (pcapng/JSON Lines) or the target/profile artifact? -> A: The
  target/profile artifact, as the master target schema's `technologies` field on a
  materialized target. The packet-stream writers stay byte-compatible with
  unmodified analyzers (P-5) and are not touched; the durable, schema-validated
  home for a per-target technology set is the target artifact the schema already
  governs. Recorded as a decision.
- Q: Is the technology set threaded through every cascade resolver answer (so a
  Steam-walker or engine-rule resolution automatically carries its
  technologies)? -> A: No, not in this slice. Detection is an independent
  capability consumed on demand and at scaffold time (FR-015: it labels
  technologies, it does not participate in resolution). Weaving it into the
  resolver's `Target` for every provider is a larger, separable change and is out
  of scope; keeping it independent preserves the resolver's single job of naming
  the socket holder.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See a target's technologies, including its anti-cheat, before capture (Priority: P1)

An operator is about to capture a game and wants to know what it is built on and
what watches it. They point fragcap at the game's install directory (or a target
that already carries one). fragcap reports the technologies it detected, grouped
by category: the engine, any anti-cheat, the SDKs, and so on, each with the
marker file that revealed it, and each marked as a heuristic guess. The operator
sees, for example, that the game ships EasyAntiCheat, and decides accordingly.

**Why this priority**: This is the whole point of the slice and its on-mission
safety contribution. An operator has a right to know a game ships an anti-cheat
before fragcap runs alongside it. Delivered alone, the CLI surface is a viable,
valuable increment: it turns an install directory into an honest technology
report.

**Independent Test**: Build a temporary install directory containing marker
files for a known engine and a known anti-cheat (for example an `EasyAntiCheat`
directory and a `*-Win64-Shipping.exe`), run the detection, and assert the report
lists both under their categories with the correct marker paths and a
`heuristic-unverified` fidelity.

**Acceptance Scenarios**:

1. **Given** an install directory containing an anti-cheat marker and an engine
   marker, **When** the operator runs technology detection on it, **Then** the
   output lists the anti-cheat under the anti-cheat category and the engine under
   the engine category, each naming the marker path that matched, each stamped
   `heuristic-unverified`.
2. **Given** an install directory with no recognizable technology markers,
   **When** detection runs, **Then** the output reports no technologies found
   (an empty result), not an error and not a fabricated guess.
3. **Given** an install directory whose files match markers in several categories
   (engine, SDK, container), **When** detection runs, **Then** all findings are
   reported, grouped by category, with no category suppressing another.

---

### User Story 2 - Technologies recorded in output metadata (Priority: P1)

An operator captures a target and later reads the output. The detected
technologies are written into the capture's metadata as a multi-category
structure, so a person or a tool reading the output can see, alongside the flows,
that the game used a given engine, anti-cheat, and SDK set, each as a
heuristic-unverified finding with its marker path.

**Why this priority**: A technology report that exists only on the console at
capture time is lost the moment the terminal scrolls. Writing it into the output
metadata is what makes it durable and interpretable later, and it is the second
half of "surfaced to the user and into output metadata." It is independently
testable against a written artifact.

**Independent Test**: Materialize a target with a detected technology set into
the output metadata structure and assert it serializes to the multi-category
`technologies` shape (category, name, marker path, fidelity per finding) and
validates against the master target schema.

**Acceptance Scenarios**:

1. **Given** a target with detected technologies, **When** its metadata is
   written, **Then** the metadata carries a `technologies` structure listing each
   finding's category, technology name, marker path, and `heuristic-unverified`
   fidelity.
2. **Given** the written `technologies` structure, **When** it is validated
   against the master target schema, **Then** it conforms (the schema defines the
   category vocabulary and the per-finding shape).
3. **Given** a target for which no technologies were detected, **When** the
   scaffold writes its metadata, **Then** the `technologies` structure is present
   and empty (an empty array), so a scan that ran and found nothing is
   distinguishable from an older artifact that predates the field; the schema
   still keeps the field optional so those older artifacts validate.

---

### User Story 3 - Incompatible ruleset patterns are skipped, counted, and surfaced (Priority: P1)

The vendored ruleset is written for a PCRE-style engine and contains patterns
fragcap's regex engine cannot compile. When the ruleset is loaded, every such
pattern is skipped so detection can still run on the compatible majority, and the
number of skipped patterns (and which technologies they belonged to) is available
rather than hidden.

**Why this priority**: This is the P-4 no-silent-loss guarantee for this slice. A
detector that silently dropped a third of its rules would under-report
technologies while looking complete, which is exactly the invisible loss the
constitution forbids. Making the skip counted and surfaced is a required,
tested behavior, not a nicety.

**Independent Test**: Load the vendored ruleset, assert the count of compiled
patterns plus the count of skipped patterns equals the total pattern count in the
file (conservation), and assert the skipped count is exposed (non-negative, and
zero only if every pattern happened to compile).

**Acceptance Scenarios**:

1. **Given** the vendored ruleset containing at least one pattern the engine
   cannot compile, **When** it is loaded, **Then** that pattern is skipped, the
   remaining patterns compile and are usable, and the skipped count is at least
   one and is exposed to the caller.
2. **Given** the loaded ruleset, **When** the compiled and skipped counts are
   summed, **Then** the sum equals the total number of patterns in the file (no
   pattern is unaccounted for).
3. **Given** a skipped pattern belonged to a named technology, **When** the load
   result is inspected, **Then** the affected technology is identifiable, so an
   operator can tell that coverage was reduced rather than that the technology is
   absent.

---

### User Story 4 - The vendored asset is attributed and integrity-locked (Priority: P2)

A contributor or auditor wants to confirm the vendored ruleset is a faithful,
attributed copy of a known upstream version. A NOTICE file carries the MIT
license text and the SteamDB copyright, and a lock file records the source
repository, the pinned upstream commit, the license identifier, and a SHA-256
over the vendored bytes. Re-hashing the committed file reproduces the recorded
hash.

**Why this priority**: Vendoring third-party content without attribution and an
integrity record is a licensing and supply-chain gap. The lock file is what lets
a reviewer confirm the asset was not tampered with and came from where it claims,
mirroring how `skills-lock.json` governs vendored skills. It is P2 because the
detection surface can be demonstrated without it, but the slice is not shippable
without it.

**Independent Test**: Compute a SHA-256 over the committed `rules.ini` bytes
(normalized the way the lock records) and assert it equals the hash recorded in
the lock file; assert the NOTICE contains the MIT text and the SteamDB copyright.

**Acceptance Scenarios**:

1. **Given** the committed vendored `rules.ini`, **When** its bytes are hashed the
   way the lock file records, **Then** the result equals the lock file's recorded
   hash.
2. **Given** the vendored asset, **When** the NOTICE is read, **Then** it contains
   the MIT license text and the SteamDB copyright attribution.
3. **Given** the lock file, **When** it is read, **Then** it records the source
   repository, the pinned upstream commit, the SPDX license identifier, and the
   hash, so the exact upstream version is identifiable.

---

### Edge Cases

- The install directory does not exist or is unreadable: detection reports the
  unreadable path as a surfaced condition and does not fabricate an empty "no
  technologies" answer that would be indistinguishable from a clean scan (mirrors
  the S029 `Unreadable` treatment, P-4).
- A subtree of the install directory is unreadable: the unreadable path is
  surfaced; detection continues over the readable remainder rather than aborting
  or silently under-reporting.
- The install directory is very large: the scan is bounded so detection stays
  affordable and does not walk without limit (the depth or breadth bound is a
  plan-time decision, consistent with S029's bounded scan).
- A single marker file matches rules in more than one category (or more than one
  technology): every distinct match is reported; the surface does not collapse
  multiple technologies into one.
- The same technology is revealed by more than one marker file: it is reported
  once per technology (deduplicated by technology within a category), with a
  representative marker path, rather than once per matching file.
- The vendored ruleset file is missing or its recorded hash does not match: this
  is a build-or-load-time error surfaced clearly, not a silent fallback to zero
  rules that would make every target look technology-free.
- Every pattern in the ruleset happens to compile (a future upstream that avoids
  incompatible constructs): the skipped count is zero and detection is complete;
  the counted-skip machinery still runs and simply reports zero.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST vendor the SteamDB `FileDetectionRuleSets`
  `rules.ini` as a committed data asset, copied verbatim and unmodified from a
  pinned upstream commit.
- **FR-002**: The vendored asset MUST be accompanied by a NOTICE that carries the
  MIT license text and the SteamDB copyright attribution ((c) 2021 SteamDB), and
  by a lock record that records the source repository, the pinned upstream
  commit, the SPDX license identifier, and a SHA-256 over the vendored bytes
  (normalized to a documented, reproducible form), in the same spirit as
  `skills-lock.json`.
- **FR-003**: Re-hashing the committed vendored bytes the way the lock records
  MUST reproduce the recorded hash; a mismatch MUST be a surfaced error, checkable
  in the repository gate.
- **FR-004**: The system MUST load the vendored ruleset and compile its category
  rules (engine, anti-cheat, SDK, emulator, container, launcher) into an
  executable matcher over file paths.
- **FR-005**: When a ruleset pattern cannot be compiled by fragcap's regex engine,
  the system MUST skip that pattern, MUST count the skip, and MUST make the
  skipped count (and the affected technology) available to the caller; it MUST NOT
  silently drop the pattern, and it MUST NOT edit the vendored bytes to force
  compilation (P-4).
- **FR-006**: The count of compiled patterns plus the count of skipped patterns
  MUST equal the total number of patterns in the vendored file (conservation), and
  this MUST be asserted in a test.
- **FR-007**: The system MUST scan a target's install directory and match compiled
  rules against the file paths it finds (filenames and relative paths only), and
  MUST NOT read file contents (P-1).
- **FR-008**: Detection MUST NOT open a process handle, read process memory, or
  perform any network access; it derives entirely from filesystem path reads
  (P-1).
- **FR-009**: The directory scan MUST be bounded (a documented depth or breadth
  limit) so detection stays affordable on a large install and does not walk
  without limit.
- **FR-010**: An unreadable install directory or an unreadable subtree MUST be
  surfaced as a named condition and MUST be distinguishable from a directory that
  was fully scanned and contained no technologies; a partial read MUST NOT be
  reported as a complete empty result (P-4).
- **FR-011**: Detection MUST report each finding with its category, the
  technology's name, the marker path that matched, and a `heuristic-unverified`
  targeting fidelity; findings MUST be grouped by category, and a technology MUST
  be reported once per category (deduplicated), not once per matching file.
- **FR-012**: The system MUST provide a dedicated on-demand command-line surface
  that scans a given install directory and prints the detected technologies in a
  readable grouped form (by category, each finding naming its technology, marker
  path, and heuristic fidelity).
- **FR-013**: The detected technologies MUST be representable in a materialized
  target artifact as a multi-category `technologies` structure whose category
  vocabulary is engine, anti_cheat, sdk, framework, emulator, container, runtime,
  and launcher, carrying per finding the category, the technology name, the marker
  path, and the `heuristic-unverified` fidelity; the profile-scaffold path MUST
  populate this structure for the target it materializes.
- **FR-013a**: Technology detection MUST NOT run automatically inside the live
  capture loop, and the pcapng and JSON Lines packet-stream writers MUST remain
  unchanged and byte-compatible with unmodified analyzers (P-5). "Output metadata"
  for this slice is the target/profile artifact, not the packet-stream capture
  files.
- **FR-014**: The `technologies` structure MUST be defined in the master target
  schema (the category vocabulary and the per-finding shape), and a materialized
  `technologies` structure MUST validate against that schema; the fidelity value
  MUST remain the targeting fidelity vocabulary, distinct from the attribution
  fidelity (Live/Retained/None).
- **FR-015**: This detection surface MUST NOT change or override the resolver's
  decision about which executable is the socket-holding client; it labels
  technologies and is consumed independently of resolution.
- **FR-016**: The slice MUST add no new runtime dependency (the existing regex
  engine is reused) and MUST NOT add any crate to the lockfile; the minimum
  supported toolchain MUST stay green.
- **FR-017**: The new terms this slice introduces (for example "technology
  detection" and the ruleset it is built on) MUST gain full glossary entries in
  the same change, and the specification MUST document the technology-detection
  surface (P-6).
- **FR-018**: The vendored asset, its NOTICE, and its lock record are pinned
  artifacts; adding them (and any hash-check step added to the gate) MUST be
  recorded as a dated decision in the changelog.

### Key Entities *(include if feature involves data)*

- **Vendored ruleset**: The SteamDB `FileDetectionRuleSets` `rules.ini`, a
  committed copy of a pinned upstream commit. Attributes: the raw bytes, the
  category sections it defines, and the per-technology path-regex rules within
  them (including array-form rules where one technology has several markers).
- **Ruleset lock record**: The integrity and provenance record for the vendored
  asset. Attributes: source repository, pinned upstream commit, SPDX license
  identifier, and the SHA-256 over the vendored bytes.
- **Compiled ruleset**: The load-time product of the vendored ruleset: the set of
  successfully compiled category rules plus the accounting of skipped
  (incompatible) patterns. Attributes: the usable matchers by category and
  technology, the compiled count, the skipped count, and the affected technologies
  for skips.
- **Technology finding**: One detected technology for one target. Attributes: the
  category, the technology name, the marker path that matched, and the
  `heuristic-unverified` fidelity.
- **Technologies report**: The per-target collection of findings, grouped by
  category, surfaced to the operator and materializable into output metadata as
  the multi-category `technologies` structure.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An install directory containing an anti-cheat marker and an engine
  marker yields a report listing both, under their categories, with correct marker
  paths and `heuristic-unverified` fidelity, in a fixture test.
- **SC-002**: The compiled-pattern count plus the skipped-pattern count equals the
  total pattern count in the vendored ruleset, asserted in a test; the skipped
  count is exposed to the caller.
- **SC-003**: Re-hashing the committed vendored `rules.ini` reproduces the hash
  recorded in the lock file, checkable in the repository gate.
- **SC-004**: A detected technologies set materializes into the multi-category
  `technologies` metadata structure and validates against the master target
  schema, in a test.
- **SC-005**: An unreadable install directory (or subtree) is surfaced as a named
  condition distinct from a clean empty scan, in a test.
- **SC-006**: The dependency-direction and dependency-inventory checks confirm no
  new runtime dependency and no new lockfile crate were added; the minimum
  supported toolchain build stays green.
- **SC-007**: The full repository gate (`cargo xtask ci`) passes with the
  technology-detection surface and the vendored-asset hash check in place.

## Assumptions

- The master target schema (S025) and the targeting-fidelity vocabulary
  (`heuristic-unverified`, from S027) exist and are the contract this slice
  extends with a `technologies` structure; the fidelity here is the targeting
  fidelity, deliberately separate from the attribution fidelity
  (Live/Retained/None) in the core crate.
- The regex engine already vendored (added S05, used by the profile match
  predicates) is the matcher for the ruleset's path regexes; it is RE2-style and
  does not support PCRE constructs such as possessive quantifiers, which is why
  incompatible patterns are skipped rather than compiled. No new dependency is
  needed or added.
- The vendored `rules.ini` is applied against a real, installed game directory the
  operator already has on disk; the depot-manifest license gate SteamDB faces for
  its catalog is a catalog-scale problem that does not apply to a local install,
  so the whole ruleset may be applied.
- The Steam-Catalog-Research.md that named the schema categories (engine,
  anti_cheat, sdk, framework, emulator, container, runtime) is the operator's own
  design note; its category vocabulary is adopted as the schema's category enum,
  extended with `launcher` to cover the ruleset's `Launcher` section. The
  `framework` and `runtime` categories are defined for stability and future
  sources and are simply unpopulated by the vendored ruleset today.
- The ruleset's two-pass "Evidence" deduction (secondary hint patterns plus
  engine inference) is out of scope for this slice; only the direct category
  sections are applied. This is recorded as a decision and does not weaken the
  first-pass engine, anti-cheat, SDK, emulator, container, or launcher findings.
- The detection module lives in the profile crate beside the engine-rule module
  (filesystem-and-path reasoning, no platform dependency), and the vendored asset
  lives beside the embedded schema in that crate's assets directory; the CLI, which
  already depends on the profile crate, surfaces the report. This mirrors the
  S029/S030 placement decisions and keeps the core crate's dependency allowlist
  unchanged.
- Fixtures are temporary directory trees built at test time (marker files in an
  install layout), in the spirit of the existing engine-rule and Steam-crate test
  helpers; no real game installation is required.
