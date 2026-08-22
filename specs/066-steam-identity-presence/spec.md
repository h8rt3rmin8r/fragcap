# Feature Specification: Steam Install-Path Resolution, Target Presence, and Multi-Name Identity

**Feature Branch**: `066-steam-identity-presence`

**Created**: 2026-08-21

**Status**: Draft

**Input**: User description: "S066: Steam install-path resolution, target presence, and
multi-name identity (issues #166, #167, #173). Bundle three related, cross-cutting fixes
to Steam discovery and the targets store into one slice: (1) the Steam library walk
hard-codes `steamapps/common/<installdir>` as every title's install root, but Steam
soundtrack/music apps install under `steamapps/music/<installdir>`, producing a spurious
warning and a bogus `ready` row; fix by resolving the install root from the app's actual
type. (2) a registered target whose install root no longer exists on disk renders
identically to a healthy target in `fragcap targets`; fix by deriving per-row presence and
rendering a missing root in the existing warning color with a note. (3) a Steam title
carries at least three names (display name, installdir, launch executable) and fragcap
keeps only the derived handle, so a renamed title cannot be found by its folder name or
executable; fix by storing all three names verbatim and extending selector resolution to
match on installdir and executable stem, surfacing a semantic divergence in `targets
show`. All three share the same TargetEntry/discovery/registration surface and are
sequenced together per the issues' own cross-links."

## Clarifications

### Session 2026-08-21

Answered under autopilot decision policy (shruggie-speckit): each question was resolved
against the constitution, the source issues' own stated tradeoffs, and existing code
patterns, rather than escalated, since none is materially irreversible or
architecture-defining.

- Q: Should a discovered Steam app of the Music (soundtrack) type be registered as a
  capture target at all? → A: No. It is counted under the existing
  `considered_not_a_game` discovery-account outcome (the same bucket a known-root
  directory matching no signature already uses), never produced as a candidate, and
  therefore never registered. This reuses an existing, already-conserved outcome rather
  than adding a new one (P-4).
- Q: Does a missing-install-root row change the CAPTURE column's readiness label? → A:
  No. `capture_readiness` (spec 3.6) is documented as answering from launch data alone and
  deliberately never asserting validity; overloading it would give the column two
  meanings. The missing-root fact is carried instead as a note prefixed to the
  free-running SENSITIVITIES column value (the one column already exempt from padding and
  truncation), so an unaffected row's rendering is untouched by construction: the prefix
  is empty for a row that is not in this state.
- Q: Is a row with a missing install root still eligible to be the hero listing's
  suggested next `fragcap capture <n>` command? → A: No. The "next command" selection
  additionally skips a row whose install root is recorded and absent, falling through to
  the next ready row, or to the first row if none qualify, since suggesting a capture
  against files that are gone is a bad first suggestion (raised in issue #167 itself).
- Q: Should an ampersand (`&`) in a source name expand to the word "and" during handle
  derivation, instead of disappearing as it does today? → A: Yes. `Trapped with Ivy &
  Piper` derives `trapped_with_ivy_and_piper` going forward. This is forward-looking only
  (no migration of already-derived handles); the handle-derivation reference vectors are
  updated to match in the same change.
- Q: What concrete rule distinguishes a "semantic" name divergence (surfaced in `targets
  show`) from a "cosmetic or truncation" one (kept silent)? → A: Normalize both the
  display name and the installdir through the existing handle-normalization function
  (case, punctuation, whitespace collapsed to the same rules a handle already uses).
  If the two normalized forms are equal, or one is a substring of the other, the
  divergence is cosmetic or truncation and stays silent; otherwise it is semantic and is
  surfaced. This reuses existing, already-tested normalization rather than introducing a
  second string-comparison rule set.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A soundtrack no longer masquerades as a capturable game (Priority: P1)

A user who owns a Steam soundtrack or other music-type add-on runs `fragcap targets` (or
the bare `fragcap` hero command). Today this prints a spurious `could not read install
directory during detection` warning and lists the soundtrack as a `ready` capture target,
because the walk assumes every title installs under `steamapps/common/`. The user should
see no warning for an app that is, in fact, correctly installed, and the tool should make
a deliberate, recorded decision about whether such an app belongs in the target list at
all rather than appearing as noise.

**Why this priority**: This is the most concrete, reproducible defect (observed on a real
machine, exact repro steps in issue #166) and it actively degrades trust in every other
warning the hero command prints: a user who sees two warnings for titles that are
installed correctly starts ignoring warnings generally, which defeats P-4's purpose of
surfacing loss.

**Independent Test**: Register a fixture Steam library containing one music-type app
(`steamapps/music/<installdir>`) and one ordinary game
(`steamapps/common/<installdir>`); run discovery and confirm the music app resolves to its
real directory with no warning, and the game's resolution is byte-identical to before.

**Acceptance Scenarios**:

1. **Given** a Steam library with a music-type app installed under
   `steamapps/music/<installdir>`, **When** discovery runs, **Then** the resolved install
   root points at the real directory and no "could not read install directory" warning is
   printed for it.
2. **Given** a Steam library with an ordinary game under `steamapps/common/<installdir>`,
   **When** discovery runs, **Then** its resolved install root and detection outcome are
   unchanged from before this feature (no regression to the existing common-app path).
3. **Given** the fix is in place, **When** a real machine with an installed Steam
   soundtrack runs `fragcap targets`, **Then** no `could not read install directory`
   warning is printed for it.

---

### User Story 2 - A dead registration says so instead of pretending to be healthy (Priority: P1)

A user runs `fragcap targets` after uninstalling a game, disconnecting a removable drive
that held a second Steam library, or deleting a folder they had previously pointed a scan
at. The corresponding row today renders exactly like a healthy, capturable target: same
color, same `ready` label, with the only clue being an unattributed warning line printed
above the whole table. The user should be able to look at the table itself and tell, per
row, which registrations point at files that are no longer there, without the tool ever
deleting or hiding that registration.

**Why this priority**: This is the direct usability consequence a user hits every time
issue #166's underlying condition (or any of its several other causes: a removed library, a
deleted scanned folder) occurs, and it is what makes the warning in User Story 1
actionable rather than an orphaned line the reader cannot connect to a row.

**Independent Test**: Register a target whose `install_root` is a path that does not
exist, list targets, and confirm that row (and only that row) renders with the presence
indicator; unaffected rows must render exactly as they did before this feature (byte
identical to the committed CLI goldens for the non-color output).

**Acceptance Scenarios**:

1. **Given** a registered target whose `install_root` is recorded and does not exist on
   disk, **When** `fragcap targets` runs with color enabled (a real terminal, `NO_COLOR`
   unset), **Then** that row renders in the existing warning color with a short note
   identifying the situation.
2. **Given** the same registration, **When** `fragcap targets` runs with `NO_COLOR=1` set
   or with output piped, **Then** the row is plain text carrying the same note, with no
   escape sequences.
3. **Given** a registered target with no `install_root` recorded at all, **When**
   `fragcap targets` runs, **Then** that row is not reported as missing (absent and
   unrecorded are different states, and only a recorded-but-missing path counts as
   missing).
4. **Given** any row not in the missing-install-root state, **When** `fragcap targets`
   runs in any color mode, **Then** its rendering is byte-for-byte identical to what this
   feature's predecessor produced (no drift for a healthy row).
5. **Given** a missing-install-root registration, **When** a user inspects the store or
   runs subsequent commands, **Then** the registration is still present, still selectable,
   and still resolvable by its stable identifier (nothing is ever auto-removed).

---

### User Story 3 - A renamed title can be found by the name the user actually sees (Priority: P2)

A user has a Steam title whose store display name and installed folder name disagree (a
publisher renamed the storefront listing after the depot's `installdir` was fixed at
first publish). The user knows the folder name from Explorer, or knows the executable
name from a shortcut, but has no way to find or reference the registration using either,
because fragcap derives and keeps only one name (the handle, from the display name). The
user should be able to select the registration by any of the names it actually has, and
where the display name and the installed folder name mean genuinely different things (not
just a casing or truncation difference), the tool should say so rather than silently
picking one.

**Why this priority**: This is real and reproducible (documented against a real, sampled
library where 11 of 34 installed titles show some divergence), but it is a findability
improvement rather than a correctness defect the way User Story 1 and 2 are: the
registration already exists and is already capturable, it is only harder to locate by
one of its names.

**Independent Test**: Register a Steam title whose known display name, installdir, and
launch executable stem are all different strings; confirm the registration can be
resolved by a substring of any of the three, and that `targets show` states the
divergence when the names are not just cosmetic variants of each other.

**Acceptance Scenarios**:

1. **Given** a registered Steam title whose display name and Steam `installdir` are
   different strings, **When** the user selects it by a substring of the display name, a
   substring of the `installdir`, or the launch executable's file stem, **Then** the
   selector resolves to that one registration in every case.
2. **Given** the same registration, **When** the user runs `targets show` against it,
   **Then** all three names (display name, installdir, executable) are visible verbatim,
   with none reconstructed from another.
3. **Given** a title whose display name and installdir differ only cosmetically (casing,
   whitespace, a subtitle truncation), **When** the user runs `targets show`, **Then** no
   divergence note is printed (only a semantically different pair is called out).
4. **Given** a title whose display name and installdir name genuinely different things
   (for example "Trapped with Ivy & Piper" installed as "Escape from Ivy & Piper"),
   **When** the user runs `targets show`, **Then** a note states both names so the reader
   knows the folder does not match the storefront title.

---

### Edge Cases

- What happens when a Steam manifest's app type cannot be determined at all (no
  `appinfo.vdf` entry, or an unreadable/corrupt appinfo cache)? The install root falls
  back to the current `common/<installdir>` assumption rather than failing discovery for
  the whole title (a game is still the overwhelmingly common case).
- What happens when a discovered candidate's directory type is `Music` and the slice's
  recorded decision is not to register it as a capture target at all? It is counted in
  the discovery account under an existing or new outcome bucket (never silently dropped,
  P-4) and does not appear as a registered row.
- What happens when an `install_root` that was missing at one listing reappears (a drive
  is reconnected) before the next listing? The row renders as healthy again; presence is
  derived fresh on each listing, never cached as a stored verdict.
- What happens when a selector token (a substring) matches one target by its `installdir`
  and a different target by its display name? Both are candidate matches and the existing
  ambiguity handling (`Selection::Ambiguous`) applies exactly as it does for two targets
  whose names collide today.
- What happens to a target registered before this feature shipped, which has no stored
  `installdir` or executable value at all? It behaves exactly as before: selectable by
  handle and display name, with the divergence note never printed (there is nothing to
  compare).
- What happens to a Music-type target a pre-fix build already registered? It is left
  exactly as it is; this feature changes what gets registered going forward, and does
  not retroactively remove, hide, or reclassify an existing registration (the same
  nothing-is-ever-auto-unregistered guarantee User Story 2 states for a missing install
  root applies here too, for a different cause).
- Does an explicit, user-directed registration of a Music-type title (naming its app id
  directly, rather than the automatic discovery walk finding it) get blocked the same
  way? No: the Music-type exclusion in FR-004 applies to what the automatic discovery
  walk produces as a candidate. A user who deliberately asks to register a specific
  title by its app id is taking a distinct, intentional action, not being handed
  discovery noise, and that registration proceeds normally.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Steam discovery walk MUST resolve an installed title's install
  directory using the app's actual install-directory type rather than unconditionally
  assuming `steamapps/common/<installdir>`.
- **FR-002**: A fixture Steam tree with a title installed under
  `steamapps/music/<installdir>` MUST resolve to that real directory, and detection over
  it MUST NOT emit a "could not read install directory" warning caused by an
  incorrect root.
- **FR-003**: A fixture Steam tree with a title installed under the ordinary
  `steamapps/common/<installdir>` MUST resolve identically to how it resolved before this
  feature (no observable change to the existing, dominant case).
- **FR-004**: A discovered app of the Music (soundtrack) type MUST NOT be registered as a
  capture target: it is counted under the existing `considered_not_a_game` discovery
  outcome and never produced as a candidate, consistently in both the `targets discover`
  inspection output and the hero listing's registration behavior (see Clarifications).
- **FR-005**: `fragcap targets` (and the equivalent bare `fragcap` hero listing) MUST
  derive, per registered row, whether its `install_root` is present on disk, absent on
  disk, or not recorded at all, evaluated fresh at listing time.
- **FR-006**: A row whose `install_root` is recorded and absent MUST render, when color
  output is active, in the same color the rest of the CLI already uses for a warning-level
  condition, accompanied by a short textual note identifying the situation.
- **FR-007**: The same row, when color output is inactive (`NO_COLOR` set, or output not
  a terminal), MUST render as plain text carrying the same note, with no escape sequences.
- **FR-008**: A row whose `install_root` is not recorded at all MUST NOT be reported as
  missing; "absent" and "never recorded" are distinct, observable states.
- **FR-009**: Every row not in the missing-install-root state MUST render byte-for-byte
  identically, in every color mode, to how it rendered before this feature (no drift for
  a healthy row, in the color output or the plain output).
- **FR-010**: Discovering that a row's `install_root` is missing MUST NOT delete, hide, or
  otherwise mutate that target's registration; the row remains listed, selectable, and
  resolvable by its stable identifier.
- **FR-011**: A row with a missing install root MUST keep its existing CAPTURE readiness
  label unchanged; the missing-root fact MUST be carried as a note prefixed to that row's
  SENSITIVITIES column value; and such a row MUST NOT be offered as the hero listing's
  suggested "next command" target (see Clarifications).
- **FR-012**: The target entry model MUST be extended to store, verbatim and without
  reconstruction from one another, the platform display name, the platform installdir (or
  equivalent folder-identifying value), and the launch executable name, for every source
  that has one available.
- **FR-013**: Selector resolution (used by `targets show`, `targets remove`, `targets
  export`, and any other selector-driven command) MUST match a candidate token against the
  stored installdir and the launch executable's file stem, in addition to the existing
  handle and display-name matching, so a target is findable by any of its recorded names.
- **FR-014**: `targets show` MUST state, for a target whose display name and installdir
  represent semantically different names (not merely a casing, whitespace, or
  truncation-only difference), both names together in one note, so the reader can tell the
  folder does not match the storefront title.
- **FR-015**: `targets show` MUST NOT print any divergence note for a target whose display
  name and installdir differ only cosmetically (case, whitespace) or by truncation
  (one is a prefix or subtitle-stripped form of the other).
- **FR-016**: An ampersand (`&`) in a source name MUST expand to the word `and` during
  handle derivation, rather than disappearing as it does today (forward-looking only; no
  migration of already-derived handles), and the handle-derivation reference vectors MUST
  be updated to match (see Clarifications).
- **FR-017**: A target registered before this feature shipped, carrying no stored
  installdir or executable value, MUST continue to resolve by handle and display name
  exactly as before, and MUST NOT trigger a divergence note (there is nothing recorded to
  compare).
- **FR-018**: Every new discard, decline, or non-registration path introduced by FR-004
  MUST be counted in a named outcome so the discovery account remains conserved.

### Key Entities *(include if feature involves data)*

- **Installed Steam title**: A title discovered on disk, now carrying not just its
  resolved install directory but the app-type information used to resolve it, so a
  non-`common` install location is a known fact rather than an assumption.
- **Target entry**: The stored, registered capture target. Extended to carry the
  platform's raw installdir/folder-identifying value and the raw launch executable name
  as verbatim fields alongside the existing display name and derived handle, none
  reconstructed from another.
- **Presence state**: A derived (never stored) per-listing fact about a target's
  `install_root`: present, absent-but-recorded, or not recorded. Distinct from
  classification, fidelity, and capture readiness, which it does not alter.
- **Name divergence**: A derived (never stored) per-target fact comparing the display name
  and the installdir: no divergence, a cosmetic/truncation divergence (silent), or a
  semantic divergence (surfaced in `targets show`).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user with an installed Steam soundtrack or other music-type app sees zero
  spurious install-directory warnings for it when running `fragcap targets`.
- **SC-002**: A user scanning a table of registered targets can identify, from the table
  alone (no need to correlate a warning line printed above it), which specific rows point
  at install locations that no longer exist.
- **SC-003**: A user who knows a title only by its installed folder name or its launch
  executable name can locate its registration without first learning the tool's derived
  handle or the storefront display name.
- **SC-004**: Of the rows unaffected by either the missing-install-root state or a
  semantic name divergence, 100% render identically, in every color mode, to their
  pre-feature output (zero CLI golden regressions for the unaffected majority).
- **SC-005**: Zero registered targets are deleted, hidden, or otherwise altered as a side
  effect of detecting a missing install root or a name divergence.

## Assumptions

- The three source issues (#166, #167, #173) are sequenced into one slice because they
  read and write the same `TargetEntry`/discovery/registration surface, per the issues'
  own cross-linking; splitting them would mean touching the same schema and rendering code
  three separate times.
- Reading `appinfo.vdf` for an app's install-directory type is acceptable added cost per
  discovered title, since `fragcap-steam` already parses that file for launch entries and
  the read is local and passive (no network, no elevated privilege, consistent with P-1).
- "Semantically different" names (FR-014) versus "cosmetic or truncation" (FR-015) is
  decided in Clarifications: reuse the existing handle-normalization rules, and treat an
  equal-after-normalization or substring relationship as cosmetic/truncation, anything else
  as semantic.
- The existing CLI goldens (`cli_targets.rs` and friends) are the authority for "byte
  identical to before"; any output shape change for an affected row is validated by new,
  explicit test fixtures rather than by loosening the existing goldens.
- Non-Steam discovery sources (the known-roots walker, a directory scan) do not have a
  separate "installdir" concept distinct from their existing path-derived name; FR-012's
  installdir field is populated when a source has one available (Steam) and left absent
  otherwise, consistent with P-9 (never inventing an observation).
