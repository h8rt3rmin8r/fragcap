# Feature Specification: Detection truthfulness and the column split

**Feature Branch**: `065-detection-truthfulness`

**Created**: 2026-08-20

**Status**: Draft

**Input**: Make the detection evidence say what was actually observed, add the
two engines this machine has installed but fragcap cannot see, and split the one
overloaded evidence column into two columns with one job each. Closes issues
#169, #168, and #174.

## Context

The listing a user sees first is `fragcap targets`. Every row carries a KNOWN
column that reports what was detected in the title's install directory. Three
separate defects meet in that column, and they compound: the detection is wrong
on most rows, two installed engines are invisible to it, and the column that
renders the result mixes unrelated kinds of fact into one comma-joined string.

The ordering here is deliberate. Fixing the detection first makes the new
columns demonstrably worth having rather than mostly empty, which is what #174
itself asks for.

### The label records an observation that was not made (#169)

28 of 32 rows report "Steam DRM". They are reporting the presence of
`steam_api64.dll`, the Steamworks SDK redistributable, which ships with
essentially every Steam title whether or not any DRM wrapper is applied. That
is the P-9 failure in its purest form: the observation recorded is not the
observation made. A signal that fires on almost every row also carries no
information, so the column is simultaneously wrong and useless.

The real Steam DRM wrapper is detectable, passively, from bytes already on
disk. It appends a PE section named `.bind` to the wrapped executable. Measured
on the operator's machine by reading the section table out of each on-disk PE
header:

| Title | ships `steam_api64.dll` | has `.bind` | fragcap says | truth |
| --- | --- | --- | --- | --- |
| Detroit Become Human | yes | yes | Steam DRM | Steam DRM |
| Palworld | yes | yes | Steam DRM | Steam DRM |
| Enshrouded | yes | yes | Steam DRM | Steam DRM |
| ARC Raiders | yes | no | Steam DRM | wrong |
| Barotrauma | yes | no | Steam DRM | wrong |
| Shale Hill Secrets | yes | no | Steam DRM | wrong |
| Trapped with Ivy and Piper | yes | no | Steam DRM | wrong |

Wrong on four of the seven sampled titles, and the correct discriminator is one
section-name read away. The vocabulary for it already exists and is switched
off: the signature match kind for a binary marker is carried but inert, and the
PE header reader that the version-string kind uses already parses the same file
format.

### Two installed engines are invisible, and the detectors have diverged (#168)

Ren'Py and GameMaker are both installed on the operator's machine and neither
is detectable. Ren'Py is recognized by the launch-resolution rules but has no
evidence signature; GameMaker has neither.

That surfaces an architectural divergence. fragcap has two independent engine
detectors that read the same directory to answer related questions, and nothing
keeps their engine lists in sync:

| | launch-resolution engine rules | detection signature set |
| --- | --- | --- |
| Question answered | which executable holds the socket | which product is present |
| Engines | Unreal, Unity, Godot, Ren'Py | Unity, Unreal, Source, Godot, CryEngine, RE Engine |
| Ren'Py | yes | no |
| GameMaker | no | no |

An engine the launch rules can resolve a client for, but the detection set
cannot name, produces a run where the resolver silently used an engine rule
while the listing reports no engine at all. That is the drift this issue asks
to be settled in writing.

### One column is doing four jobs (#174)

```
  #  TARGET                       CAPTURE         KNOWN
  2  arc_raiders                  ready           Unreal, Steam DRM
 15  oblivion_soundtrack          ready           no online mode recorded
 21  steam                        needs a target  Unity, Unreal, Steam DRM
```

Row 2 mixes an engine with a protection product. Row 15 substitutes a sentence
about capture likelihood for absent evidence. The underlying data is already
correctly partitioned by category (engine, anti-cheat, DRM, with a declared
display order); the rendering is what flattens it.

There is a fourth state hiding in the blank: a row whose install root could not
be read, a row that was never scanned, and a row that was scanned clean all
render identically today. Two of those are claims about coverage and one is a
finding, and collapsing them is the silent loss P-4 forbids.

## Clarifications

### Session 2026-08-20

Answered under the autopilot decision policy from the constitution, the two
issues' own evidence, and the existing code, rather than raised to the operator.
Each is recorded here because it changes what gets built; the two
architecture-affecting ones (Q2, Q3) also get a dated decisions fragment.

- Q: The two Steamworks SDK signature rows, drop them or recategorize them into
  a new platform-sdk category? -> A: Drop them. A fourth category would need a
  fourth rendering bucket in a slice whose subject is that the columns already
  conflate categories, and since the library ships with essentially every Steam
  title the row would add a column of noise carrying no information. The fact
  that a title links the Steamworks SDK is already implied by its `steam:`
  anchor. #169 calls dropping defensible for exactly this reason.
- Q: The dual-detector relationship, fold the launch-resolution engine rules
  into the signature table (#168 option a) or keep them separate behind a
  mechanical check (option b)? -> A: Option b, stated as a directed subset
  invariant rather than as a two-way sync. Option a is rejected: the two
  detectors answer different questions, and the signature schema carries no
  per-engine rule for selecting a client executable, so folding them would
  either require extending the signature schema with a client-selection rule
  (a schema change larger than this slice) or drop the client selection the
  resolver cascade depends on.
- Q: Where is the three-state coverage information stored on a target? -> A: A
  new nullable field on the target entry, added by an additive schema migration
  and carried by an explicit export key, where absent means never scanned. It is
  not folded into the free-form provenance blob: #174 calls this a P-4 concern,
  and the export key set is a reviewed contract by design, so an out-of-set
  value is rejected at parse rather than read as a scan that did not happen.
- Q: What is the 80 column rule for the widened table, truncate or wrap? -> A:
  Neither. Columns size to their content as they already do, and the
  sensitivities column is last and free-running. Truncation is the silent loss
  P-4 forbids, and wrapping a row would break the alignment the columns exist to
  provide; a genuinely long value overflows visibly instead of lying. The budget
  is therefore stated over the columns the tool controls: with every bounded
  column at its widest they cost 53 of the 80, leaving 27 for a handle.
  **Corrected during implementation**: this answer originally said a
  representative row was tested to fit within 80. Rendering the real listing
  showed the operator's longest handle is 47 characters, so that machine's rows
  run to 100 and do not fit. The overflow is the declared behavior and is now
  covered by its own test; shortening handles is #166 and #173 in slice S066.
- Q: The readiness fallback sentences, move them into the readiness column or
  retire them? -> A: Retire them. "no launch data known" is a relabeling of
  "needs a target" and "no online mode recorded" is a relabeling of "ready" with
  no findings, which the two technology columns now state directly. Moving them
  would print the same fact twice in one row and cost 16 columns of an 80 column
  budget.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The evidence names what is actually there (Priority: P1)

An operator runs `fragcap targets` and reads the technology evidence for each
row. Every product named is a product whose marker was actually observed in
that title's files, and a product that is not present is not named.

**Why this priority**: This is the P-9 defect. A column that is wrong on most
rows costs the operator more than an empty column would, because they act on
it. Everything else in this slice is presentation over data that must first be
correct.

**Independent Test**: Point detection at a fixture tree carrying a wrapped
executable and at one carrying an unwrapped executable that ships the same
platform SDK library. The first reports the DRM product; the second reports no
DRM product. Neither requires a game or a platform install.

**Acceptance Scenarios**:

1. **Given** an install tree whose launch executable carries a `.bind` PE
   section, **When** detection runs over it, **Then** the DRM product is
   reported with definitive confidence.
2. **Given** an install tree whose launch executable carries no `.bind` section
   but does ship the Steamworks SDK library, **When** detection runs over it,
   **Then** no DRM product is reported.
3. **Given** a signature set containing byte-marker rows that this build cannot
   match, **When** the set is compiled, **Then** those rows are counted inert
   and the applied, inert, and skipped counts still sum to the rows loaded.

---

### User Story 2 - An installed engine is named (Priority: P1)

An operator with Ren'Py and GameMaker titles installed runs `fragcap targets`
and sees each title's engine named, the same way a Unity or Unreal title's is.

**Why this priority**: An engine that is present and unnamed is the same class
of failure as a product that is absent and named: the listing does not describe
the machine. It is P1 alongside story 1 because the two together are what make
the split columns in story 3 worth rendering.

**Independent Test**: Build fixture trees with the canonical Ren'Py and
GameMaker layouts and assert detection names each engine, with no platform
install present.

**Acceptance Scenarios**:

1. **Given** an install tree with a `renpy/` package directory, **When**
   detection runs, **Then** Ren'Py is named as the engine.
2. **Given** an install tree containing a GameMaker runtime data file, **When**
   detection runs, **Then** GameMaker is named as the engine.
3. **Given** an engine that the launch-resolution rules can resolve a client
   for, **When** the repository's checks run, **Then** a check fails if that
   engine has no detection signature naming the same product.

---

### User Story 3 - Each column reports one kind of fact (Priority: P2)

An operator reads the listing and can tell at a glance which title runs which
engine, and separately which titles carry anti-cheat or DRM. A blank in either
column means something specific, and the operator can tell which.

**Why this priority**: Presentation over data. It is only worth doing once
stories 1 and 2 make the data correct, which is the sequencing #174 states for
itself.

**Independent Test**: Render the listing over registered rows carrying known
evidence and assert the column contents and the three coverage states, without
any detection run.

**Acceptance Scenarios**:

1. **Given** a row whose evidence carries both an engine and a DRM product,
   **When** the listing renders, **Then** the engine appears in one column and
   the DRM product in another, and neither column carries both kinds.
2. **Given** a row that was scanned and matched no signature, **When** the
   listing renders, **Then** its technology columns are visibly distinct from a
   row that was never scanned and from a row whose install root could not be
   read.
3. **Given** the machine-readable target output, **When** an operator reads a
   row, **Then** the same partition and the same coverage state are recoverable
   from it, so the two surfaces cannot disagree.

---

### Edge Cases

- A launch executable that is present but cannot be opened or read: it must not
  be reported as carrying a marker, and it must not be reported as having been
  scanned clean either.
- A tree with many executables: the section-table read must be bounded, and any
  candidate not examined because of that bound must be counted rather than
  silently skipped.
- A file whose name ends in `.exe` but which is not a PE image: no marker, no
  error, no crash.
- A binary-marker pattern whose form this build does not recognize: inert and
  counted, never treated as a malformed row and never silently dropped.
- A row registered before this change, whose stored data carries no coverage
  information: it must render as never scanned, never as scanned clean.
- A two-product sensitivities value on a long handle: the line must not be
  silently truncated, even when the resulting row is wider than the terminal.

## Requirements *(mandatory)*

### Functional Requirements

#### Detection truthfulness (#169)

- **FR-001**: The two signature rows matching the Steamworks SDK client library
  MUST be removed from the shipped signature set, so no DRM product is reported
  on the basis of that library's presence.
- **FR-002**: The system MUST support a detection signature that matches a named
  PE section in an executable, with a pattern form that distinguishes a section
  name from other binary-marker forms.
- **FR-003**: The signature set MUST carry a definitive DRM signature for the
  Steam DRM wrapper matching the `.bind` section, so a wrapped title is reported
  and an unwrapped title is not.
- **FR-004**: Section matching MUST read only executables that are plausible
  launch targets, bounded in both directory depth and candidate count, and MUST
  read only a bounded prefix of each candidate rather than the whole file.
- **FR-005**: A candidate executable not examined because of a bound MUST be
  counted and the count MUST be reachable by a caller, so reduced coverage is
  visible rather than silent (P-4).
- **FR-006**: A binary-marker signature whose pattern form this build cannot
  match MUST remain inert and counted, and the applied, inert, and skipped
  counts MUST continue to sum to the number of signatures loaded.
- **FR-007**: The section reader MUST open no process handle, read no process
  memory, and call no operating-system inspection API; it reads the bytes of a
  file already on disk (P-1).

#### Engine coverage and the dual-detector question (#168)

- **FR-008**: The signature set MUST detect Ren'Py from its package directory
  and from its interpreter library, and MUST carry its archive extension as a
  weaker corroborating signal.
- **FR-009**: The signature set MUST detect GameMaker from its runtime data
  file, and MUST carry the GameMaker-specific platform extension library as a
  weaker corroborating signal, distinct from the generic platform SDK library.
- **FR-010**: The relationship between the two engine detectors MUST be settled
  in a written decision recorded with the slice, naming the option chosen and
  why the alternative was rejected.
- **FR-011**: A mechanical check MUST enforce the directed subset invariant:
  every engine the launch-resolution rules can resolve a client for MUST have at
  least one engine-category detection signature naming the same product. The
  reverse is deliberately not required, because the signature set may name
  engines for which no client-selection rule has been written. The check MUST
  derive the engine set structurally from the declaration, so that adding an
  engine to the launch rules without adding a signature fails the check; it MUST
  NOT assert a count or a hand-maintained list.

#### The column split (#174)

- **FR-012**: The listing MUST render the detected engines and the detected
  anti-cheat and DRM products in two separate columns, partitioned on the
  signature category rather than by flattening every product into one string.
- **FR-013**: The readiness fallback sentences MUST NOT appear in a technology
  column. The capture-readiness distinction MUST be reported once, in the
  readiness column, and not restated.
- **FR-014**: The listing MUST distinguish three coverage states in the
  technology columns: scanned with no match, never scanned, and scanned with
  reduced coverage because something could not be read.
- **FR-015**: The coverage state MUST be stored as its own field on the target
  entry, recorded by every source that can produce a target (the platform walk,
  the known-roots walk, the pointed-directory scan, and single-target
  registration) when that source ran detection, and left absent (meaning never
  scanned) when the producing source ran none. A source left unplumbed is a
  defect, not a blank row. A
  store written by an earlier build MUST open and read as never scanned rather
  than failing or claiming a scan. The field MUST survive an export and import
  round trip, and an out-of-set value MUST be rejected at parse.
- **FR-016**: The machine-readable target output MUST carry both the per-finding
  category and the coverage state, so the table and the machine output cannot
  disagree about what a technology is or about whether a scan happened.
- **FR-017**: No column value may be silently truncated and no row may be
  wrapped. Columns size to their content and the sensitivities column is last and
  free-running. The budget is stated over the columns the tool controls: with
  every bounded column at its widest, the columns other than the target handle
  MUST cost no more than a stated number of columns, measured from rendered
  output by test rather than recomputed from the layout. The handle is operator
  data of unbounded width, so a table whose handles exceed the remaining budget
  overflows 80 columns visibly, and that overflow MUST be covered by a test that
  asserts nothing was clipped.
- **FR-018**: The readiness column MUST keep the two labels it has today, so the
  widened table costs no width there and the readiness distinction is stated in
  exactly one place.

### Key Entities

- **Detection signature**: a row naming a category (engine, anti-cheat, DRM), a
  match kind, a pattern, a product, and a confidence. The category is the
  partition the split columns render.
- **Detection finding**: one product observed in one install directory, with the
  evidence that produced it and the fidelity it earns.
- **Coverage state**: a field on a target entry recording whether its install
  directory was scanned and whether that scan was complete. Distinct from the
  finding set, which can legitimately be empty for a complete scan.
- **Engine rule**: a launch-resolution rule that recognizes an engine's install
  layout and names the client executable. Its engine set is constrained by
  FR-011 to be nameable by the signature set.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On the operator's machine, `fragcap targets` reports no DRM
  product for ARC Raiders, Barotrauma, Shale Hill Secrets, or Trapped with Ivy
  and Piper, and still reports Steam DRM for Detroit Become Human, Palworld,
  and Enshrouded.
- **SC-002**: On the operator's machine, `fragcap targets` names Ren'Py for
  Trapped with Ivy and Piper and GameMaker for Shale Hill Secrets.
- **SC-003**: No column in the listing contains both an engine and a protection
  product.
- **SC-004**: A row with no findings is distinguishable, without opening any
  other command, from a row that was never scanned and from a row whose install
  root could not be read.
- **SC-005**: The signature accounting sums: applied plus inert plus skipped
  equals loaded, asserted for the shipped set and for a set containing an
  unrecognized marker form.
- **SC-006**: With every bounded column at its widest, the columns other than
  the target handle cost 53 columns, so a table whose handles are 27 characters
  or fewer renders within an 80 column terminal. A longer handle overflows with
  every value intact and nothing clipped. Measured on the operator's machine, the
  longest handle is 47 characters
  (`warhammer_40_000_dawn_of_war_definitive_edition`), so that machine's listing
  runs to 100 columns. That is the declared no-truncation behavior, not a defect
  of this slice; shortening the handles a target carries is #166 and #173, which
  are slice S066.
- **SC-007**: `cargo xtask ci` is green, including the new engine-coverage
  check.

## Assumptions

- The `.bind` section name is the discriminator for the Steam DRM wrapper. This
  is the operator's direct measurement on seven titles, recorded above, not an
  inference from documentation.
- A launch executable sits within a small number of directory levels of the
  install root. The bound chosen must accommodate the observed ARC Raiders
  layout, where the launch executable is four levels below the root.
- The three parked byte-marker signatures (Denuvo, Arxan, VMProtect) remain
  unimplemented in this slice. Implementing them needs marker byte sequences
  that have not been measured, and guessing them would reintroduce exactly the
  defect this slice fixes.
- The evidence a target carries is produced by the source that registered it.
  A target registered before this change carries no coverage information and is
  read as never scanned, which is the conservative reading: the tool does not
  claim a scan it cannot vouch for.
- Anti-cheat detection and machine-scope questions (#170) are a separate slice
  and are out of scope here, as are the Steam install root and target identity
  questions (#166, #167, #173).
