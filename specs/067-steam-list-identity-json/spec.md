# Feature Specification: Steam list identity and JSON output

**Feature Branch**: `067-steam-list-identity-json`

**Created**: 2026-08-22

**Status**: Draft

**Input**: User description: "S067: `steam list` fixes (issues #171, #172).
Label the columns with a header, join the local store to show handle and row
index, define a deterministic sort order (by name), keep the snapshot table
read-only from this command, and add a `--json` structured output mode that
honors the global --json flag and carries app id, name, install directory,
and (when available) handle/stable id/row index. Enumeration warnings keep
going to the emitter/stderr. See issue bodies for full acceptance criteria:
https://github.com/h8rt3rmin8r/fragcap/issues/171 and
https://github.com/h8rt3rmin8r/fragcap/issues/172"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read a labeled, joined listing (Priority: P1)

An operator who has already run the hero listing (`fragcap targets`) runs
`fragcap steam list` to see what Steam has installed. Today the output is two
unlabeled tab-separated columns with no connection to the local store, so the
operator cannot tell which listed title is already registered, what handle to
pass to `fragcap capture`, or what row number a title occupies in the most
recent target listing.

**Why this priority**: This is the entire content of issue #171 and the
listing's stated job (spec section 16.3): let an operator find the identifier
to act on. Without it, `steam list` is inspection output that dead-ends.

**Independent Test**: Run `fragcap steam list` on a machine with Steam
installed and at least one title already registered via discovery. Verify the
output carries a header naming every column, and that a registered title's row
shows its handle and (when it appears in the most recent snapshot) its row
index, while an installed-but-unregistered title is visibly distinct rather
than showing blank fields.

**Acceptance Scenarios**:

1. **Given** a Steam installation with titles already registered by discovery
   and a local store with a most-recent listing snapshot, **When** the
   operator runs `fragcap steam list`, **Then** the output starts with a
   header row naming each column, and each registered title's row shows its
   handle and its row index from the most recent snapshot when it appears
   there.
2. **Given** a Steam installation with a title that is registered but does not
   appear in the most recent snapshot, **When** the operator runs
   `fragcap steam list`, **Then** that title's row shows its handle with no
   row index, and is visibly distinct from both a fully-resolved row and an
   unregistered row.
3. **Given** a Steam installation with a title that has never been registered,
   **When** the operator runs `fragcap steam list`, **Then** that title's row
   shows neither a handle nor a row index, and is visibly distinct from a
   registered row.
4. **Given** no local store exists, or the store cannot be opened, **When**
   the operator runs `fragcap steam list`, **Then** the command still succeeds,
   still names every installed title, and signals (through the emitter, to
   standard error) that identity information could not be joined.
5. **Given** two consecutive runs of `fragcap steam list` on an unchanged
   machine, **When** the operator compares the output, **Then** the rows
   appear in the same order both times, sorted by title name.
6. **Given** a prior `fragcap targets` run produced a listing snapshot,
   **When** the operator runs `fragcap steam list` afterward, **Then**
   `fragcap capture <n>` still resolves exactly as it did before `steam list`
   ran, the snapshot is read, never rewritten, by this command.

---

### User Story 2 - Consume Steam titles as structured data (Priority: P2)

An operator or a script needs to consume the Steam listing programmatically,
for example to check what is installed before deciding what to register. They
pass the global `--json` flag, which every other inspection command already
honors, and expect one structured record per installed title on standard
output with nothing else mixed in.

**Why this priority**: This is issue #172. It depends on User Story 1's join
existing (the structured record carries the same identity fields), which is
why it is P2 rather than P1, but it is independently valuable: a machine
consumer needs the install directory that the human table has never shown,
and needs the guarantee that the record stream will not need to be re-parsed
every time the human table's shape changes.

**Independent Test**: Run `fragcap steam list --json` and parse standard
output as newline-delimited records. Verify every installed title appears
exactly once, each record carries at minimum app id, name, and install
directory, and no human-readable text reaches standard output.

**Acceptance Scenarios**:

1. **Given** a Steam installation with installed titles, **When** the
   operator runs `fragcap steam list --json`, **Then** standard output
   contains one structured record per title, newline-delimited, and standard
   output contains nothing else.
2. **Given** the same installation and local store state as User Story 1's
   scenarios, **When** the operator runs `fragcap steam list --json`,
   **Then** each record carries the same identity fields (handle, row index)
   under the same presence/absence rules as the human listing, with absence
   distinguishable from a zero or empty value.
3. **Given** a Steam installation with no titles enumerated, **When** the
   operator runs `fragcap steam list --json`, **Then** standard output
   contains zero records rather than a record describing zero.
4. **Given** an enumeration warning occurs during discovery, **When** the
   operator runs `fragcap steam list --json`, **Then** the warning reaches
   standard error through the emitter and standard output's record stream
   stays uncontaminated.
5. **Given** no Steam installation is found, **When** the operator runs
   `fragcap steam list --json`, **Then** the command exits with the same
   configuration-refusal exit code it uses today, unaffected by the flag.

---

### Edge Cases

- A title's Steam-reported name collides with another installed title's name
  (two different app ids, same displayed name): sort order falls back to app
  id as a tiebreak so the order stays total and reproducible.
- The local store opens but the query for a title's registration fails for a
  reason other than "not registered" (a corrupt row, an unexpected schema
  state): the row is treated as unresolved-with-a-warning rather than silently
  reported as unregistered, and the warning goes through the emitter.
- The most recent listing snapshot exists but is empty (no prior `targets` run
  produced rows, or the table was cleared): every registered title falls into
  the "registered, no row index" state, never a lookup failure.
- `--json` is combined with an empty enumeration (`no installed titles
  enumerated` in human mode): the JSON stream is simply empty, with the
  human-only sentence never emitted in JSON mode.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `fragcap steam list` MUST print a header row naming every
  column before any data rows, in human (non-JSON) mode.
- **FR-002**: `fragcap steam list` MUST resolve each installed title's local
  identity by its exact Steam anchor (app id), not by name matching, against
  the same local store resolution the other commands use
  (`FRAGCAP_LOCAL_DB`, then the per-user default).
- **FR-003**: For a title registered in the local store and present in the
  most recent listing snapshot, `fragcap steam list` MUST show its handle and
  its 1-based row index from that snapshot.
- **FR-004**: For a title registered in the local store but absent from the
  most recent listing snapshot, `fragcap steam list` MUST show its handle and
  MUST NOT show a row index, and this state MUST be visibly distinct from
  FR-003's state.
- **FR-005**: For a title with no local registration at all, `fragcap steam
  list` MUST show neither a handle nor a row index, and this state MUST be
  visibly distinct from FR-003's and FR-004's states.
- **FR-006**: `fragcap steam list` MUST NOT write to the listing snapshot
  table under any circumstance; it only reads the most recent snapshot
  produced by another command.
- **FR-007**: `fragcap steam list` MUST sort rows by title name (case-
  insensitive ordinal comparison), breaking ties by app id, and MUST produce
  the same order across repeated runs against unchanged installed and
  registered state.
- **FR-008**: If the local store is absent or cannot be opened, `fragcap
  steam list` MUST still succeed and still name every installed title, with
  every row falling back to the no-identity state (FR-005), and MUST emit a
  warning through the emitter noting that identity information could not be
  joined.
- **FR-009**: `fragcap steam list` MUST accept the global `--json` flag and,
  when set, emit one newline-delimited structured record per installed title
  to standard output instead of the human table, with no human-readable text
  interleaved on standard output.
- **FR-010**: Each JSON record MUST carry at minimum the app id, the title
  name, and the install directory.
- **FR-011**: Each JSON record MUST carry the same identity fields (handle,
  stable id, row index) as the human listing under the same presence/absence
  rules as FR-003 through FR-005, with an absent field distinguishable from a
  present-but-empty or zero value.
- **FR-012**: When JSON mode enumerates zero installed titles, `fragcap steam
  list --json` MUST emit zero records rather than a record describing the
  empty state, and MUST NOT emit the human-mode "no installed titles
  enumerated" sentence.
- **FR-013**: Enumeration warnings from Steam discovery MUST continue to
  reach standard error through the emitter in both human and JSON mode, never
  standard output.
- **FR-014**: The configuration-refusal exit behavior for "no Steam
  installation found" MUST be unchanged by the presence or absence of
  `--json`.

### Key Entities

- **Installed title**: A title Steam reports as installed on this machine,
  identified by app id, carrying a name and an install directory. Discovered
  fresh on every `steam list` run; never persisted by this command.
- **Local registration**: The target store's record of a title, reached by
  its exact Steam anchor (`steam:<app_id>`). Carries a durable handle and a
  stable id. Existing entirely outside this command's control.
- **Listing snapshot row**: The 1-based position a stable id occupied in the
  most recent `fragcap targets` listing. Read-only from this command; the
  thing `fragcap capture <n>` resolves against.
- **Joined identity state**: The per-title result of combining the three
  above, one of "registered and positioned" (handle + row index),
  "registered, unpositioned" (handle only), or "unregistered" (neither),
  carried identically in the human table and the JSON record.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Given a machine with Steam installed and at least one title
  already registered by discovery, an operator can identify the handle to
  pass to `fragcap capture` for that title by reading a single `fragcap steam
  list` run, with no second lookup command needed.
- **SC-002**: An operator can tell apart all three identity states
  (registered+positioned, registered-only, unregistered) for every row in the
  listing by sight, without cross-referencing another command's output.
- **SC-003**: Running `fragcap steam list` any number of times in sequence
  never changes what `fragcap capture <n>` resolves to.
- **SC-004**: A script piping `fragcap steam list --json` into a JSON-lines
  parser recovers every installed title, including the install directory,
  with zero non-JSON bytes on standard output.
- **SC-005**: The row order of `fragcap steam list` is identical across two
  consecutive runs on an unchanged machine, in both human and JSON mode.

## Assumptions

- The local store resolution order (`FRAGCAP_LOCAL_DB` env var, then the
  per-user default path) used by `targets` and `capture` is the correct
  resolution for `steam list` too, per issue #171's own analysis; this slice
  does not introduce a separate `--db` flag, matching the issue's framing that
  such a flag is a naming exercise worth doing but not required for the
  acceptance criteria as written.
- "Row index" means the 1-based position in the `listing_snapshot` table as
  written by the most recent `fragcap targets` run, read via a new reverse
  lookup (stable id -> position) rather than a new snapshot.
- The structured JSON form is newline-delimited records (JSON Lines), matching
  the existing `doctor --json` precedent, not a single JSON array.
- Sorting by name is case-insensitive ordinal comparison (matching the hero
  listing's existing sort-by-handle behavior's general spirit); ties break on
  app id since that is the one field guaranteed unique per installed title.
- Since both issues are filed as one coherent piece of the same command and
  are explicitly cross-referenced ("Related: #172" / "Related: #171"), this
  slice implements both together rather than splitting them, matching the
  campaign plan's framing of S067 as covering "steam list" as a whole.
