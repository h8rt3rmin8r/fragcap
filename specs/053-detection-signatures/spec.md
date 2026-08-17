# Feature Specification: Data-driven detection signatures

**Feature Branch**: `053-detection-signatures`

**Created**: 2026-08-17

**Status**: Draft

**Input**: Slice S053 (issue #140, milestone v0.5.0). Depends on S052 (the
TargetSource discovery seam), now merged. Source:
fragcap-v0.5.0-UX-Handoff-Plan.md sections 3.6, 8, and Appendix B. Fills the
`DirectoryClassifier` seam S052 left open.

## Overview

fragcap already recognizes game technologies (engines, anti-cheat, DRM) from an
install directory, but the knowledge is baked into the binary: an embedded,
vendored ruleset compiled into `fragcap-profile`. Recognizing a newly shipped
anti-cheat, or a game engine that did not exist at build time, requires a code
change and a release.

This slice moves that knowledge out of code and into data. The signature set
becomes a table in the shipped catalog database, so `fragcap catalog update`
refreshes detection capability the same way it refreshes the title catalog, with
no release cycle. A single generic matcher runs the table against a directory's
shape. That matcher is the real implementation of the `DirectoryClassifier` seam
S052 introduced, so detection runs automatically inside the scan phase of every
discovery source (Steam, known-roots, a user-pointed folder) with no user aware
the step exists, and the standalone `fragcap technologies --path <dir>` command a
researcher uses to inventory an unknown binary is repointed at the same table.

Local evidence outranks remote claims. An engine identified from files on disk is
stamped `verified`; the same engine claimed by the remote catalog stays
`heuristic-unverified`. This is constitution principle P-9 working as intended: a
`UnityPlayer.dll` you can see is stronger evidence than a catalog attribution
derived from a third-party wiki.

One rule is load-bearing and absolute (specification section 3.6): **detected
DRM and anti-cheat are neutral evidence, never gates.** fragcap does not restrict,
block, warn against, or discourage capture based on what it detects. A detected
product is a fact recorded in an evidence column, nothing more. No status value,
color, or wording anywhere in any output may imply that a title is off limits,
risky, or discouraged. A title with no recorded online multiplayer mode is still
listed as fully capturable, because a single-player title that produces network
traffic is one of the most interesting results this tool can surface.

## Clarifications

### Session 2026-08-17

Two load-bearing decisions were resolved with the operator after reading the
existing detection code.

- Q: Which signature match kinds does this slice implement? The Appendix B set
  mixes path-shape signals (a filename, a directory tree) with content signals (a
  PE version string, a byte or section marker inside an executable). Content
  matching requires opening and reading file contents and cannot be exercised from
  empty fixture files. -> A: Implement filename, directory-shape, AND
  PE-version-string matching this slice. PE-version-string reads the version
  resource in a binary's PE header (a bounded, on-disk read, never process memory,
  so P-1 is not engaged) and needs only a small crafted-header fixture. Raw
  byte/section-marker scanning is deferred: the schema still carries a
  `binary-marker` kind and the three content-marker-only DRM products (Denuvo,
  Arxan, VMProtect) are seeded, but their rows are inert until a later slice, and
  the seed count surfaces them as not-yet-matchable (P-4).
- Q: What becomes of the existing embedded `CompiledRuleset` in `fragcap-profile`?
  Investigation found it is the vendored SteamDB `FileDetectionRuleSets` set (376
  path-only patterns, hash-locked), authored for Steam *depot manifests* rather
  than on-disk install layouts, stamping every finding `heuristic-unverified`, and
  never validated against a real installed game (all its tests are synthetic). The
  depot-vs-install path mismatch is why it detects little on a real install. -> A:
  Replace it. The `signature` table in the catalog database becomes the single
  source of truth for both the classifier (US1) and the `technologies` command
  (US3), seeded with the Appendix B install-layout set, which is authored for
  on-disk markers and so actually matches real installs. The embedded ruleset and
  its `fragcap-profile` code path are removed. The breadth the SteamDB set covered
  beyond Appendix B (extra SDKs, launchers, emulators) is regained later as
  additional install-authored table rows, never by restoring a compiled-in
  fallback. On an absent or unseeded catalog database, detection reports nothing
  matched (honest per P-4) rather than falling back to a detector that misleads.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Detection runs automatically inside every discovery scan (Priority: P1)

When any discovery source scans a directory, the generic signature matcher
classifies it by shape. A directory whose shape matches an engine signature is a
game: the scan emits one candidate for it, stamps the detected engine at
`verified` fidelity, and stops descending into that subtree. No user knows the
step exists or invokes it; detection is a phase of the scan, not a separate
command.

**Why this priority**: This is the point of the slice. S052 shipped the
`DirectoryClassifier` seam with a placeholder that treated every immediate child
of a known root as a game. This story replaces the placeholder with the real
matcher, which is what lets a standalone or non-catalog title be recognized by its
shape rather than by a curated list. Without it the seam classifies nothing.

**Independent Test**: Point a source at a fixture directory tree containing an
engine marker (for example a `*_Data/` folder beside a `UnityPlayer.dll`). Verify
one candidate is emitted, its detected engine is Unity at `verified` fidelity, and
descent stopped at the matched directory rather than enumerating executables
beneath it. Exercised entirely from fixture directories with no real game.

**Acceptance Scenarios**:

1. **Given** a fixture directory whose shape matches a seeded engine signature,
   **When** a source scans it, **Then** exactly one candidate is emitted for that
   directory, its detected engine is stamped `verified`, and the scan does not
   descend into the matched subtree.
2. **Given** a fixture directory with no signature match, **When** a source scans
   it, **Then** no engine is stamped and the directory is accounted as
   considered-not-a-game (the S052 discovery account stays conserved).
3. **Given** a candidate that also carries a remote catalog engine attribution,
   **When** a local engine signature matches, **Then** the local `verified`
   attribution is the one presented and the remote `heuristic-unverified` claim
   does not override it.

---

### User Story 2 - Detection capability refreshes as data, without a release (Priority: P2)

The signature set lives in a table in the catalog database. `fragcap catalog
update` refreshes it alongside the title catalog. An operator who adds a signature
row for a newly shipped anti-cheat gets that product detected on the next scan
with no new fragcap build.

**Why this priority**: This is the durable value of the slice over the embedded
ruleset it replaces. It is P2 rather than P1 because the matcher (US1) must exist
before there is anything for the data to drive, but the "no release cycle"
property is the reason the work is worth doing at all.

**Independent Test**: Seed a signature table, run a scan, and confirm a product is
detected. Add one filename or directory-shape signature row directly to the table,
re-run the scan against a matching fixture, and confirm the new product is
detected with no code change and no rebuild.

**Acceptance Scenarios**:

1. **Given** a catalog database seeded with the Appendix B signature set, **When**
   a directory matching any seeded signal is scanned, **Then** the corresponding
   product is detected.
2. **Given** a running installation, **When** one new filename or directory-shape
   signature row is added to the table, **Then** a subsequent scan of a matching
   directory reports the new product with no code change.
3. **Given** `fragcap catalog update`, **When** it runs, **Then** the signature
   table is refreshed as part of the same update that refreshes the catalog.

---

### User Story 3 - A researcher inventories an unknown binary directory (Priority: P2)

A researcher points `fragcap technologies --path <dir>` at a directory to see what
engine, anti-cheat, and DRM it contains, without registering it as a capture
target and without launching anything. The report groups findings by category and
labels every finding as neutral evidence.

**Why this priority**: This is the first-class standalone use of the matcher
(specification section 3.6). It is retained from the existing command, repointed
at the table, and it is the surface where the neutral-evidence rule is most
visible to a user.

**Independent Test**: Run `technologies --path` against a fixture directory
containing an anti-cheat marker and a DRM marker. Confirm both are listed as
neutral facts grouped by category, that no status, color, or wording frames either
as a reason not to capture, and that an unreadable subtree is surfaced as a
coverage warning rather than reported as absence.

**Acceptance Scenarios**:

1. **Given** a fixture directory containing an anti-cheat marker, **When**
   `technologies --path` scans it, **Then** the anti-cheat is listed as a neutral
   fact and nothing in the output frames it as risky, blocked, or discouraged.
2. **Given** a fixture directory with a readable root and an unreadable subtree,
   **When** it is scanned, **Then** the unreadable subtree is surfaced as a
   coverage warning and the scan still succeeds.
3. **Given** a directory with no recognized technology, **When** it is scanned,
   **Then** the command reports no technologies detected and exits successfully
   (an empty result is not an error).

---

### Edge Cases

- A directory matches more than one signature (for example an engine marker and an
  anti-cheat marker): all matched products are recorded; the engine match drives
  the game classification and descent-stop, the others are recorded as additional
  evidence.
- A signature row has a `kind` the current matcher does not implement (a
  content-scan kind under the default scope): the row is counted as seeded but
  not-yet-matchable and surfaced, never silently dropped (P-4 conservation).
- A signature's pattern is malformed or empty: it is rejected at load with a
  surfaced diagnostic; one bad row does not disable the rest of the table.
- The catalog database is absent or carries no signature rows: detection reports
  nothing matched rather than erroring, with no compiled-in fallback (FR-008a).
- Two signatures for different products share a marker filename: both products are
  reported; the matcher does not assume a filename maps to a single product.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST store detection signatures as data in a table in the
  catalog database, with fields sufficient to express a product, its category
  (engine, anti-cheat, or DRM), the kind of match (filename, directory-shape,
  PE-version-string, and binary-marker; the first three are implemented this
  slice, binary-marker is carried in the schema but inert), the match pattern, and
  a confidence value.
- **FR-002**: The system MUST seed the table with the full Appendix B signature
  set: engines Unity, Unreal, Source, Godot, CryEngine, and RE Engine; anti-cheat
  Easy Anti-Cheat, BattlEye, Vanguard, mhyprot, nProtect GameGuard, and Xigncode3;
  DRM Denuvo, Steam DRM, Arxan, and VMProtect.
- **FR-003**: The system MUST detect technologies by running a single generic
  matcher over the table. Detection behavior MUST be a function of the table's
  contents, not of per-product code branches.
- **FR-004**: Adding a signature row of an implemented match kind to the table
  MUST be honored on the next scan with no code change and no rebuild.
- **FR-005**: The signature table MUST be refreshable through the same
  catalog-seed family that refreshes the title catalog (the `targets seed*`
  offline-from-bundled-asset path), so detection capability updates without a
  release cycle. A single seed operation reloads the table idempotently. (The
  handoff's conceptual `catalog update` name maps onto this seed family; see
  research D2.)
- **FR-006**: The matcher MUST implement the `DirectoryClassifier` seam introduced
  in S052, so detection runs automatically in the scan phase of every
  `TargetSource` with no separate user action.
- **FR-007**: A directory-shape match MUST stop descent: the scan emits one
  candidate for the matched directory and does not descend into its subtree, and
  MUST NOT enumerate a directory's executables first and then ask whether each is a
  game.
- **FR-008**: An engine identified from local files MUST be stamped `verified`
  fidelity. A remote catalog engine attribution MUST remain
  `heuristic-unverified`.
- **FR-008a**: The existing embedded `CompiledRuleset` in `fragcap-profile` and its
  vendored SteamDB ruleset asset MUST be removed; the signature table is the single
  source of detection for both the classifier and the `technologies` command. When
  the catalog database is absent or holds no signatures, detection MUST report
  nothing matched rather than fall back to any compiled-in ruleset.
- **FR-008b**: PE-version-string matching MUST read only the version resource of a
  binary's on-disk PE header. It MUST NOT open a process handle, read process
  memory, or read a file's contents beyond what the version resource requires
  (P-1).
- **FR-009**: When both a local `verified` engine and a remote
  `heuristic-unverified` engine attribution exist for the same candidate, the local
  `verified` value MUST be the one presented (P-9: local evidence outranks remote).
- **FR-010**: The system MUST retain `fragcap technologies --path <dir>` as a
  standalone command that reports the technologies in a directory, grouped by
  category, without registering the directory as a capture target and without
  launching anything.
- **FR-011**: Detected DRM and anti-cheat MUST be recorded and displayed as
  neutral evidence. No output path (any command, any format) may emit a status
  value, color, or wording that characterizes a detected anti-cheat or DRM product
  as a reason not to capture, or that implies a title is off limits, risky, or
  discouraged.
- **FR-012**: A title with no recorded online multiplayer mode MUST still be
  presented as fully capturable.
- **FR-013**: Every considered directory MUST land in exactly one named outcome of
  the S052 discovery account (the account stays conserved, P-4). A signature row of
  an unimplemented kind, an unreadable subtree, and a malformed signature MUST each
  be surfaced rather than silently dropped.
- **FR-014**: The full detection path MUST be exercisable entirely from fixture
  directories with no real game and no launched process.
- **FR-015**: The signature table schema change MUST be an additive migration from
  the current catalog schema version, applied transactionally, leaving an empty
  table on a database that has no signatures.

### Key Entities *(include if feature involves data)*

- **Signature**: One detection rule. Carries an identity, a category (engine /
  anti-cheat / drm), a match kind (filename / directory-shape / PE-version-string /
  binary-marker), a pattern to match, the product it identifies, and a confidence
  value. Lives as a row in the catalog database.
- **Detection finding**: The result of a signature matching a directory. Carries
  the product, its category, and the evidence (the matched path or marker). Neutral
  by construction; carries no gate, status, or risk value.
- **Detected engine attribution**: The engine a directory's shape identifies,
  stamped `verified`, attached to the candidate the scan emits. Distinct from and
  outranking any remote catalog engine attribution for the same candidate.
- **Discovery account (from S052)**: The conserved account of every considered
  directory. Extended here so a not-yet-matchable signature kind, an unreadable
  subtree, and a malformed row each have a named home.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of the Appendix B products are represented by at least one
  seeded signature row after a fresh catalog seed.
- **SC-002**: A newly added signature row of an implemented match kind is honored
  on the next scan with zero code changes and zero rebuilds.
- **SC-003**: Every seeded product whose match kind is implemented this slice is
  detected from a fixture directory with no real game present.
- **SC-004**: A directory-shape match stops descent: the number of candidates
  emitted for a matched subtree is exactly one, and no executable beneath the match
  is enumerated as a separate candidate.
- **SC-005**: A locally detected engine is presented at `verified` fidelity, and
  where a remote catalog attribution for the same title also exists, the locally
  detected value is the one shown in 100% of such cases.
- **SC-006**: Zero output paths emit a status, color, or wording that
  characterizes a detected anti-cheat or DRM product as a reason not to capture,
  verified across every command and format that can surface a detection.
- **SC-007**: The discovery account stays conserved (every considered directory in
  exactly one named outcome) across every source test, including directories that
  hold an unimplemented signature kind, an unreadable subtree, or a malformed row.

## Assumptions

- **Match-kind scope (resolved)**: This slice implements filename,
  directory-shape, and PE-version-string matching, which covers the Appendix B set
  except the three content-marker-only DRM products. The schema carries a fourth
  `binary-marker` kind so the table shape is complete and `catalog update` can ship
  it later; Denuvo, Arxan, and VMProtect are seeded but their rows are inert until
  a later slice implements raw content scanning, and the seed count surfaces them
  as not-yet-matchable. The "honored with no code change" guarantee (FR-004) is
  scoped to the implemented kinds.
- **Embedded-ruleset fate (resolved)**: The signature table is the single source of
  truth; the embedded `CompiledRuleset` and its vendored SteamDB asset are removed
  (FR-008a). The SteamDB set's breadth beyond Appendix B is regained later as
  additional install-authored table rows, not by restoring a compiled-in fallback.
- The matcher and the signature store live in the portable targets crate that
  already owns the catalog database and the S052 seam, keeping the platform-neutral
  crate boundary the constitution requires (P-3); this is confirmed at plan time,
  not asserted here.
- The catalog schema advances by one additive version for the signature table,
  following the same sequential-migration discipline S051 and S052 used.
- "Directory shape" means the presence and relative arrangement of files and
  subdirectories by name, matched from the directory listing without reading file
  contents; this is the same path-only basis the existing `technologies` command
  uses.
- The neutral-evidence rule (FR-011) governs the existing `technologies` output
  and every new surface that can show a detection; no exception exists for any
  category or product.
