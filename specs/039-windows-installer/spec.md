# Feature Specification: Windows installer (MSI) and hint-database default with first-run bootstrap

**Feature Branch**: `039-windows-installer`

**Created**: 2026-08-14

**Status**: Draft

**Input**: User description: "Windows installer (MSI) and hint-DB default path
with first-run bootstrap (slice S039, implements GitHub issue #96, decision #58;
v0.3.0 pre-launch distribution readiness). Deliver, as one slice ending in one
PR: a hint-database default location with a first-run bootstrap, a barebones
hint-database artifact, an unsigned MSI installer, the release-workflow changes
that emit and checksum the new artifacts, and the docs, glossary, and
specification updates that describe them honestly."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Hints work with zero configuration (Priority: P1)

A user runs a capture without setting any hint-database option. fragcap resolves
a per-user default location, and if no database exists there yet it creates one,
so the hint provider is present and the local launch-data accumulation has a
store to write into. The user did nothing to arrange this; it simply works, and
it works the same whether the user installed fragcap or unzipped it.

**Why this priority**: This is the enabling change the rest of the slice rests
on. Without a default the shipped database is never consulted, and the local
accumulation added in the previous slice has nowhere to write, so the barebones
database and the installer would place a file that nothing reads. It is also the
smallest independently valuable change: it makes hint resolution available to
every user, including those who only ever download the raw executable.

**Independent Test**: With no hint-database option set, point the default
resolution at a controlled per-user location that does not yet exist, run the
first-run bootstrap, and confirm a valid, current-schema database now exists
there and that a second run leaves it untouched; confirm that supplying an
explicit option still overrides the default and that an explicit but absent path
is neither created nor fatal.

**Acceptance Scenarios**:

1. **Given** no hint-database flag and no environment override, **When** the
   default location is resolved, **Then** it is the per-user application-data
   path for fragcap's hint database, alongside the existing per-user profile
   directory.
2. **Given** the default location does not exist and a database template ships
   beside the executable, **When** the first-run bootstrap runs, **Then** the
   template is copied to the default location and the resulting database opens as
   a valid current-schema store.
3. **Given** the default location does not exist and no template ships beside the
   executable, **When** the first-run bootstrap runs, **Then** an empty
   current-schema database is created at the default location.
4. **Given** the default database already exists, **When** the first-run
   bootstrap runs, **Then** the existing database is left exactly as it was.
5. **Given** an explicit hint-database flag or environment value, **When**
   resolution runs, **Then** that value is used unchanged and, if it names an
   absent file, the file is not created and the run is not failed (the existing
   present-versus-absent-versus-unopenable behavior is preserved).

---

### User Story 2 - One-click Windows install (Priority: P1)

A Windows user downloads a single installer, runs it, and afterward can invoke
`fragcap` from any new terminal without editing the system path or creating any
directories by hand. The installer places the program and its barebones hint
database, registers an uninstall entry, best-effort excludes its own install
directory from Windows Defender, and points the user at the separately-required
capture driver on completion.

**Why this priority**: The installer is the headline deliverable of the slice
and the reason issue #58 was opened. For the non-expert audience it replaces a
spread of manual steps (unzip, place files, edit the path) with one action, and
it is what makes the tool approachable for first-time users.

**Independent Test**: Build the installer from the authored definition where the
installer toolchain is available; install it silently on a Windows machine and
confirm the program is on the system path in a new shell, the hint database is
present, the uninstall entry exists, and uninstalling removes the files, the
path entry, and the exclusion. The installed-installer runtime behavior is a
documented manual-verification step, like live capture.

**Acceptance Scenarios**:

1. **Given** a clean machine, **When** the installer completes a per-machine
   install, **Then** the program is installed under the platform's program files
   directory, its directory is on the system path, and `fragcap` resolves in a
   newly opened terminal.
2. **Given** a completed install, **When** the user opens the installer's final
   screen, **Then** the installer surfaces the capture driver download page so
   the user knows the one remaining prerequisite before live capture.
3. **Given** a completed install, **When** the user inspects Windows Defender
   settings, **Then** the install directory is excluded; **and** if the platform
   refuses the exclusion (for example when tamper protection is on), the install
   still completes rather than failing.
4. **Given** an installed copy, **When** the user uninstalls it, **Then** the
   program files, the system-path entry, and the Defender exclusion are all
   removed.
5. **Given** the installer is unsigned, **When** the user runs it, **Then** the
   platform's unrecognized-publisher warning appears and the documentation the
   user was given explains that this is expected and how to proceed.

---

### User Story 3 - Choose what to download (Priority: P2)

A user visiting a release picks what suits them: a portable archive that needs no
installation, the installer, or just the hint database on its own. Each download
carries a checksum so the user can verify integrity, which matters most for the
unsigned installer.

**Why this priority**: The operator chose to offer all three forms so the user
decides. It depends on the barebones database and the installer existing, and it
is independently verifiable by inspecting a release's artifact set.

**Independent Test**: Run the release-artifact assembly and confirm it produces
the portable archive (carrying the program, its legal texts, and the hint
database), the installer, and a loose copy of the hint database, and that a
checksum file accompanies each of the three.

**Acceptance Scenarios**:

1. **Given** a tagged release build, **When** artifact assembly runs, **Then** it
   produces a portable archive containing the program, the license, the notice,
   and the hint database.
2. **Given** the same build, **When** artifact assembly runs, **Then** it also
   produces the unsigned installer and a loose copy of the hint database.
3. **Given** the three artifacts, **When** checksums are generated, **Then** each
   of the archive, the installer, and the loose database has its own checksum
   file.

---

### User Story 4 - An honest, self-consistent record (Priority: P3)

A user or reviewer reading the project's documentation finds the installer, the
bundled database, and the new default location described accurately: what the
installer does, that it is unsigned by design and how to handle that, that the
bundled database is empty and grows from the user's own machine, and that the
capture driver is still a separate install. Every new term the change introduces
is defined in the same change.

**Why this priority**: The project's honesty posture (surface what is unverified,
define terms as they are introduced) is a standing requirement, not an optional
polish. It layers on the other stories and is independently checkable by the
documentation gate.

**Independent Test**: Run the documentation gate and confirm the glossary defines
every newly introduced term, the generated glossary index reproduces exactly, and
the specification's artifact and data-directory sections match what the release
and the binary actually do.

**Acceptance Scenarios**:

1. **Given** the change introduces new distribution terms, **When** the
   documentation gate runs, **Then** each term has a glossary entry and the
   generated index reproduces byte-for-byte.
2. **Given** the installer is unsigned, **When** a user reads the installation
   documentation, **Then** it explains the unrecognized-publisher warning, how to
   proceed, and that verifying the checksum is the integrity check in the absence
   of a signature.
3. **Given** the specification describes release artifacts and the per-user data
   directory, **When** a reader compares it to the release and the binary,
   **Then** the description matches (the portable archive now carries the hint
   database, the installer and loose database are listed, and the new per-user
   default is documented).

---

### Edge Cases

- **No application-data location resolvable**: if the per-user application-data
  base cannot be determined, the default hint-database path is simply
  unavailable, exactly as the existing profile directory already degrades; the
  run proceeds without a default database rather than failing.
- **First-run bootstrap cannot write**: creating or copying the default database
  fails (a read-only or denied location). This is surfaced as a warning and the
  capture still proceeds; it is never fatal, matching how the accumulation walk
  already treats its own faults.
- **Explicit option names an absent file**: unchanged from today. The file is not
  created, the hint provider is simply absent, and the run is not failed; only a
  present-but-unopenable explicit database is a loud error.
- **The installer toolchain is absent on the build runner**: the release job
  installs it explicitly; a missing toolchain is a build failure surfaced on the
  tagged run, not a silent omission of the installer.
- **Windows Defender refuses the exclusion**: the exclusion is best-effort. A
  refusal (disabled Defender, tamper protection) does not fail the install; the
  behavior is documented as best-effort.
- **A major upgrade over a previous install**: unexercised until a second release
  exists; the installer carries a stable upgrade identity so a later version
  replaces rather than duplicates the install.

## Clarifications

### Session 2026-08-14

- Q: What does the shipped barebones database contain? -> A: An empty
  current-schema store (no rows). It is the substrate; the previous slice's local
  accumulation fills it from the user's own machine, and the full curated corpus
  remains an out-of-band maintainer artifact plus a future opt-in community sync
  (issue #94). Shipping specific unverified titles would bake in staleness and is
  rejected under the honesty posture (P-9).
- Q: How does the release expose downloads? -> A: Three forms, each with its own
  checksum: a portable archive (program plus legal texts plus the hint database),
  the unsigned installer, and a loose copy of the hint database. The user chooses.
- Q: Is the local launch-data accumulation on by default once a default database
  exists? -> A: Yes. With the default database present, the previous slice's
  accumulation runs automatically at capture start (local only, no network, no
  process handle). Sharing the learned data remains a separate opt-in (issue #94).
- Q: How is the bundled database made writable when the installer places it under
  a program-files directory the user cannot write at runtime? -> A: The installer
  ships the database as a read-only template beside the program; the binary's
  first-run bootstrap creates the writable per-user default (copying the template
  when present, else creating an empty store). One code path serves both the
  installer and the portable-archive user.
- Q: Is the installer signed? -> A: No. It is unsigned by design for this
  release; code signing is tracked separately (issue #79) and is out of scope
  here. The documentation explains handling the unrecognized-publisher warning
  and that the checksum is the integrity check.
- Q: Which capture entry points trigger the first-run bootstrap and the default
  hint database? -> A: The primary capture run path only, where hint resolution
  and local accumulation are already wired. The other capture entry points (the
  launch-agnostic watch path and the ad-hoc tap path) are unchanged by this
  slice: they neither create the default database nor gain the default when it is
  absent. A user who only ever uses those paths simply gets no default database
  until a run creates one, which is acceptable and honest for this release; broad
  co-location across every entry point is a later, separable change.

## Requirements *(mandatory)*

### Functional Requirements

Hint-database default and first-run bootstrap

- **FR-001**: The system MUST resolve a per-user default location for the hint
  database when the operator supplies neither the hint-database flag nor its
  environment override, alongside the existing per-user profile directory. When
  the per-user application-data base is not resolvable, the default is
  unavailable and the run proceeds without it.
- **FR-002**: An explicitly supplied hint-database flag or environment value MUST
  continue to take precedence over the default and MUST retain its current
  semantics: an absent explicit path is not created and is not fatal, a present
  one is consulted, and a present-but-unopenable one is a loud error.
- **FR-003**: On a capture run, when the default location is in effect and no
  database exists there, the system MUST create one before resolution and
  accumulation: by copying a database template shipped beside the executable when
  such a template exists, otherwise by creating an empty current-schema store.
- **FR-004**: The first-run bootstrap MUST leave an already-existing default
  database untouched, and MUST run only for the defaulted location, never for an
  explicitly supplied path (so FR-002's semantics for explicit paths are
  preserved).
- **FR-005**: A failure of the first-run bootstrap MUST be surfaced as a warning
  and MUST NOT fail the capture; the run proceeds as it would with no database.
- **FR-006**: Once the default database exists, the existing hint resolution and
  the existing local launch-data accumulation MUST operate against it with no
  further operator action, so hint resolution and accumulation become the
  zero-configuration default.

Barebones hint-database artifact

- **FR-007**: The system MUST provide a committed seed document, in the existing
  exportable hint-record format, that carries no records, from which an empty
  current-schema database is produced by the existing offline import path with no
  new code.
- **FR-008**: The produced database MUST be a valid current-schema store that the
  existing export path round-trips to a valid, empty exportable document.

Unsigned installer

- **FR-009**: The system MUST provide an installer definition that installs the
  program per-machine under the platform's program-files directory and adds the
  install directory to the system path, taking effect in newly opened terminals.
- **FR-010**: The installer MUST place, beside the program, the barebones hint
  database as a read-only template, the license, and the notice.
- **FR-011**: The installer MUST register a standard uninstall entry and MUST
  carry a stable upgrade identity so a later version replaces rather than
  duplicates the install.
- **FR-012**: The installer MUST attempt, with elevated rights it already holds,
  to exclude its own install directory from Windows Defender on install and to
  remove that exclusion on uninstall; a refusal MUST NOT fail the install
  (best-effort). The exclusion MUST be scoped to fragcap's own install directory
  and MUST NOT alter any other security setting.
- **FR-013**: The installer MUST surface the capture-driver download page to the
  user on completion, without downloading, bundling, or installing the driver
  itself.
- **FR-014**: The installer MUST be producible unsigned; code signing is out of
  scope for this slice.

Release artifacts

- **FR-015**: The release MUST produce, for a tagged build, a portable archive
  containing the program, the license, the notice, and the hint database (the
  database placed beside the program so the portable-archive user gets the same
  first-run bootstrap as the installed user).
- **FR-016**: The release MUST additionally produce the unsigned installer and a
  loose copy of the hint database.
- **FR-017**: The release MUST generate a checksum for each of the portable
  archive, the installer, and the loose hint database.
- **FR-018**: The release build MUST acquire the installer toolchain explicitly
  rather than assuming it is present on the runner, and MUST build the barebones
  database through the existing offline import path.

Documentation, glossary, and specification

- **FR-019**: The documentation MUST describe the installer (what it installs,
  the system-path change, the best-effort Defender exclusion, the capture-driver
  link) and MUST explain handling the unsigned installer: the unrecognized-
  publisher warning is expected by design, how to proceed, that verifying the
  checksum is the integrity check, and that signing is tracked separately.
- **FR-020**: The installation flow documentation MUST include installing fragcap
  (via the installer) as a step, with the unsigned-installer note, ahead of the
  existing capture-driver step.
- **FR-021**: Every distribution term the change introduces MUST receive a
  glossary entry in the same change, and the generated glossary index MUST be
  regenerated so it reproduces exactly.
- **FR-022**: The specification's artifacts section MUST be amended to list the
  portable archive (now carrying the hint database), the loose hint database, and
  the unsigned installer, while stating that the archive still ships no game
  profiles and that a hint database is not a game profile.
- **FR-023**: The specification MUST clarify that the no-bundling obligation binds
  only the capture driver, and MUST document the new per-user hint-database
  default and its first-run bootstrap alongside the existing per-user profile
  directory.

Constraints (constitution)

- **FR-024**: No part of this feature may open a process handle or read another
  process's memory to recognize or attribute a target (P-1). The Defender
  exclusion is an installer/OS-configuration action about fragcap's own files, not
  an action against any target process, its memory, its traffic, or the network
  stack.
- **FR-025**: The feature MUST NOT bundle, download, or install the capture
  driver; it may only link to its download page. The bundled hint database is
  fragcap's own data artifact and is not subject to the driver's no-bundling rule.
- **FR-026**: The binary changes MUST add no new third-party dependency to the
  workspace and MUST keep the minimum supported toolchain build green.
- **FR-027**: The change MUST NOT bump the workspace version; the release-cut
  process performs the version bump separately.
- **FR-028**: All added or edited text files MUST be encoded as the project
  requires (no byte-order mark, line-feed endings, no em or en dashes), including
  the installer definition and any code comments.
- **FR-029**: The release workflow and any other pinned process artifact this
  slice changes MUST be accompanied by a dated decision record in the changelog.

### Key Entities *(include if data involved)*

- **Default hint-database location**: the per-user, writable path fragcap uses
  for its hint database when no explicit option is given, a sibling of the
  existing per-user profile directory.
- **Hint-database template**: a read-only copy of the barebones database shipped
  beside the program by both distribution forms, from which the first-run
  bootstrap seeds the writable per-user default.
- **Barebones hint database**: an empty, current-schema hint store built offline
  from a committed empty seed document; the substrate the local accumulation and
  future community sync fill.
- **Installer**: the per-machine installer artifact that places the program and
  the template, adds the system-path entry, best-effort excludes the install
  directory from Defender, registers uninstall and a stable upgrade identity, and
  surfaces the capture-driver link.
- **Release artifact set**: for a tagged build, the portable archive, the
  installer, and the loose hint database, each with a checksum.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: With no hint-database option set and no database present, a first
  capture run leaves a valid current-schema database at the per-user default; a
  second run leaves it unchanged.
- **SC-002**: Supplying an explicit hint-database option overrides the default;
  an explicit absent path is neither created nor fatal, and a present-but-
  unopenable one still errors.
- **SC-003**: The committed empty seed produces, through the existing offline
  import path, a database that the existing export path round-trips to a valid
  empty exportable document.
- **SC-004**: A tagged release produces a portable archive (program, license,
  notice, hint database), an unsigned installer, and a loose hint database, and a
  checksum file exists for each of the three.
- **SC-005**: On a Windows install (manual verification), the program is on the
  system path in a new terminal, the hint database bootstraps to the per-user
  default, the capture-driver link is presented, the Defender exclusion is applied
  when the platform permits and its refusal does not fail the install, and
  uninstalling removes the files, the path entry, and the exclusion.
- **SC-006**: The documentation gate passes: every newly introduced term is
  defined and the generated glossary index reproduces exactly.
- **SC-007**: The change adds no new entry to the workspace dependency lockfile.
- **SC-008**: The full repository check set, the core-neutrality build, and the
  minimum supported toolchain build all stay green, and the workspace version is
  unchanged.

## Assumptions

- The per-user application-data base is the right home for the writable hint
  database, mirroring the existing per-user profile directory; where that base is
  unavailable the default is simply absent, as the profile directory already is.
- The installed program's directory is a stable place to ship the read-only
  database template, so the first-run bootstrap can find it beside the executable
  for both the installer and the portable archive.
- The hint database at rest is a single file with no side-car journal once it has
  been opened and closed by the offline import, so copying the template is safe.
- Local launch-data accumulation running automatically at capture start, once a
  default database exists, is the operator-chosen default; it stays local and
  private and its sharing is a separate opt-in (issue #94).
- The installer's runtime behavior (the unrecognized-publisher warning, the real
  per-machine install and path change, the Defender exclusion, the capture-driver
  link, uninstall, and any future upgrade) is verified manually and recorded, the
  same honesty posture the project already holds for live capture, because it
  cannot be exercised by the automated check set.
- The installer is unsigned for this release; signing is a separate, non-blocking
  track (issue #79). The checksum is the integrity mechanism in the meantime.
- The workspace version bump to the next release is performed by the release-cut
  process, not by this slice.
