# Feature Specification: Targets Discover Listing

**Feature Branch**: `codex/085-targets-discover-listing`

**Created**: 2026-08-27

**Status**: Draft

**Input**: User description: "Spec out S085 and roll it out with autopilot. You know the drill"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read Discovery Results As A Listing (Priority: P1)

An operator running `fragcap targets discover` can read the discovered candidates as a headed, aligned listing instead of decoding an unlabeled tab-separated dump.

**Why this priority**: `targets discover` is a read-only human inspection command. Its current output is neither a readable listing nor an explicit machine contract, which makes a normal discovery run hard to interpret.

**Independent Test**: Can be fully tested by running discovery against a fixture Steam library and asserting the result has labelled store paths, column headings, aligned candidate rows, no tab characters, and the expected title identity.

**Acceptance Scenarios**:

1. **Given** a discovery run that finds a Steam title, **When** the command prints results, **Then** it prints labelled catalog and local store paths before the candidate listing.
2. **Given** one or more discovered candidates, **When** the listing prints them, **Then** it includes a header row naming the visible fields and prints each row with spacing computed from the rendered content.
3. **Given** a candidate whose identity is wider than another candidate, **When** both rows print, **Then** the later columns remain aligned and no value is truncated.
4. **Given** a stock barebones catalog that cannot classify titles beyond the source prior, **When** discovery prints candidates, **Then** it avoids a low-value all-`unknown` classification column.

---

### User Story 2 - Keep Candidate Evidence Attached (Priority: P2)

An operator can see each candidate's detected evidence directly under that candidate, with the evidence category, product, and fidelity preserved.

**Why this priority**: Evidence is the reason a candidate was classified or enriched. Printing it near the row preserves the observation without forcing the operator to infer which row a technology line belongs to.

**Independent Test**: Can be fully tested by printing a discovery result with evidence and checking the evidence line is indented under the row and includes category, product, and fidelity.

**Acceptance Scenarios**:

1. **Given** a discovered candidate with engine evidence, **When** the listing prints, **Then** the engine line is indented under that candidate row.
2. **Given** evidence with a below-verified or verified fidelity, **When** the evidence line prints, **Then** it preserves the exact fidelity token rather than promoting, hiding, or shortening it.
3. **Given** a candidate with no evidence, **When** the listing prints, **Then** the row remains readable and no placeholder evidence line is invented.

---

### User Story 3 - Read The Discovery Account (Priority: P3)

An operator can read the discovery conservation account as labelled lines, with zero-valued outcomes grouped instead of buried in one long run of key-value pairs.

**Why this priority**: The account is the P-4 conservation record for discovery. It needs to be visible enough that skipped, declined, and failed outcomes are not lost in a 120-character sentence.

**Independent Test**: Can be fully tested by printing a discovery account with both non-zero and zero outcomes and asserting non-zero values have individual labelled lines while zero outcomes are grouped.

**Acceptance Scenarios**:

1. **Given** a discovery account with non-zero outcomes, **When** the account prints, **Then** each non-zero outcome gets its own labelled line.
2. **Given** a discovery account with zero-valued outcomes, **When** the account prints, **Then** zero outcomes are grouped in a compact `zero:` line or omitted only when every outcome is zero and the required totals are still visible.
3. **Given** container descent and container descent truncation outcomes, **When** the account prints, **Then** both names remain distinct so the two loss classes cannot collapse.

### Edge Cases

- An empty discovery result must print a clear empty result and still print the discovery account.
- Paths containing spaces must stay readable in the store-path block and candidate identity column.
- Warnings remain diagnostics through the existing emitter and must not move into the command-result stream.
- `targets scan`, which reuses the same discovery printer, must receive the improved listing without changing its registration behavior.
- This slice must not add an implicit machine-readable format. A future machine-readable discovery listing requires an explicit flag and contract.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `targets discover` MUST print catalog and local store paths as labelled lines instead of a lowercase sentence fragment.
- **FR-002**: Discovery candidate output MUST use a headed table with at least source, identity, fidelity, and name fields.
- **FR-003**: Discovery candidate output MUST NOT contain tab-separated rows.
- **FR-004**: Discovery candidate columns except the final name column MUST size to the widest rendered value or heading, whichever is wider.
- **FR-005**: Discovery candidate rendering MUST NOT truncate or wrap source names, identities, fidelities, display names, or evidence.
- **FR-006**: Discovery candidate output MUST omit the existing classification column unless a future explicit requirement defines how that field is useful to operators.
- **FR-007**: Candidate evidence MUST print under its candidate row, indented, preserving category, product, and fidelity.
- **FR-008**: The discovery account MUST render as a labelled block with `considered` and `produced` always visible.
- **FR-009**: Non-zero discovery account outcomes MUST render as individual labelled lines.
- **FR-010**: Zero-valued discovery account outcomes MUST be grouped or omitted in a way that keeps non-zero outcomes easy to scan.
- **FR-011**: Container-descended and container-descent-truncated account outcomes MUST remain separately named in human output.
- **FR-012**: Existing warning stream behavior MUST be preserved.
- **FR-013**: The change MUST NOT add a new CLI flag, storage migration, runtime dependency, capture behavior, proxy behavior, process access, or network access.
- **FR-014**: The master specification MUST describe the human rendering contract for `targets discover`.

### Slice-Local Data Values

This slice does not introduce durable product entities. It renders existing discovery values:

- **Discovery Store Block**: The labelled catalog and local store paths used by a discovery run.
- **Discovery Candidate Row**: A visible row containing source, identity, fidelity, and display name.
- **Discovery Evidence Line**: An indented line attached to a candidate row containing evidence category, product, and fidelity.
- **Discovery Account Block**: A labelled block that reconciles every considered item through produced and non-produced outcomes.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A fixture discovery run emits no tab characters in stdout.
- **SC-002**: A fixture discovery run emits a header row before the first candidate.
- **SC-003**: A discovery result with evidence prints that evidence under the owning row with its fidelity token intact.
- **SC-004**: A discovery account with at least three zero-valued outcomes renders those zero names in no more than one grouped line.
- **SC-005**: `cargo xtask ci` passes after implementation.

## Assumptions

- Human `targets discover` remains a read-only inspection command and not a machine interface in this slice.
- Classification stays available inside the discovery data model and registration path, but the human discovery table does not spend a column on values that are commonly all `unknown`.
- `width_of` remains the shared helper for content-width calculations, matching the hero listing's no-truncation policy.
