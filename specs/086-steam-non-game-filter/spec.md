# Feature Specification: Steam Non-Game Filter

**Feature Branch**: `codex/086-steam-non-game-filter`

**Created**: 2026-08-27

**Status**: Draft

**Input**: User description: "Spec out S086 and run it end-to-end like usual"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Exclude Non-Capturable Steam App Types (Priority: P1)

An operator running target discovery does not see Steam utility entries, redistributable bundles, configuration entries, or video entries offered as capture-ready targets.

**Why this priority**: Discovery is the path that turns installed software into capture candidates. Offering Steamworks Common Redistributables or other non-game Steam records as ready targets violates P-9 because the tool is presenting something that cannot produce game network behavior as a target.

**Independent Test**: Can be fully tested with a fixture Steam library whose appinfo cache includes `Music`, `Tool`, `Application`, `Config`, and `Video` records, then asserting none become candidates and all are counted through the existing not-a-game account bucket.

**Acceptance Scenarios**:

1. **Given** an installed Steam app whose app type is `Tool`, **When** Steam discovery runs, **Then** the app is not emitted as a candidate and the discovery account remains conserved.
2. **Given** installed Steam apps whose app types are `Music`, `Application`, `Config`, or `Video`, **When** Steam discovery runs, **Then** none are emitted as candidates and each increments the not-a-game outcome.
3. **Given** one or more excluded non-game apps beside valid games, **When** discovery prints or registers candidates downstream, **Then** the valid games are unchanged and the excluded apps do not reach registration.
4. **Given** a Steam non-game install under a known-root Steam `common` directory, **When** composed discovery runs, **Then** the lower-authority known-roots pass does not reintroduce that exact install directory as a path candidate.
5. **Given** a platform-created target row already stored from a previously discovered Steam non-game app, **When** the hero listing runs after this slice, **Then** the row is hidden from the listing and row-index snapshot without deleting or hiding user-authored rows.

---

### User Story 2 - Preserve Game-Like Steam Entries (Priority: P2)

An operator still sees real game-like Steam entries, including demos and entries whose app type could not be determined, instead of losing possible capture targets to an over-broad filter.

**Why this priority**: The filter must correct known false positives without inventing a new false-negative class. `Demo` can be a playable game, and unknown app types are an observation gap rather than proof that an app is not a game.

**Independent Test**: Can be fully tested with fixture Steam appinfo entries for `Demo`, `Game`, and a title with no appinfo type, then asserting each remains eligible under the same candidate and account rules as before.

**Acceptance Scenarios**:

1. **Given** an installed Steam app whose app type is `Demo`, **When** Steam discovery runs, **Then** it remains eligible as a candidate.
2. **Given** an installed Steam app whose app type is absent because appinfo is missing or unreadable, **When** Steam discovery runs, **Then** it remains eligible rather than being filtered by name or folder shape.
3. **Given** a valid game beside excluded non-game entries, **When** discovery completes, **Then** the game keeps its existing identity, fidelity, evidence, classification, and install-root behavior.

### Edge Cases

- App type comparisons must be case-insensitive so `tool`, `TOOL`, and `Tool` behave identically.
- A title with a non-numeric app id and an excluded app type must still count through the not-a-game bucket before numeric parsing, because its app type is enough to explain why it is not a candidate.
- Name or folder tokens such as `redist` are not a primary Steam app filter when app type is known; app type is the reliable signal for this slice.
- A missing app type is not evidence that the title is non-game and must not be excluded.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Steam discovery MUST exclude app types `Music`, `Tool`, `Application`, `Config`, and `Video` from candidate output.
- **FR-002**: Each excluded Steam app type MUST increment the existing `considered_not_a_game` discovery-account outcome.
- **FR-003**: Excluding non-game Steam app types MUST preserve discovery-account conservation.
- **FR-004**: Steam discovery MUST keep `Demo` app types eligible for candidate output.
- **FR-005**: Steam discovery MUST keep `Game` app types eligible for candidate output.
- **FR-006**: Steam discovery MUST keep titles with absent or unreadable app type eligible for candidate output.
- **FR-007**: Steam app type matching MUST be case-insensitive.
- **FR-008**: Composed discovery MUST pass the current Steam non-game install roots to the known-roots source and the known-roots source MUST suppress only exact matching child directories.
- **FR-009**: The hero target listing MUST hide already stored platform-created rows whose current Steam app id or install root matches an excluded Steam non-game install.
- **FR-010**: The hero target listing MUST NOT hide user-authored rows solely because they share a Steam app id or install root with an excluded Steam app type.
- **FR-011**: The change MUST NOT add a new storage field, CLI flag, dependency, network access, process access, capture behavior, or registration path.
- **FR-012**: The master specification MUST describe the widened non-game app type filter, lower-tier coordination, and the preserved eligibility of demos and unknown app types.

### Key Entities

- **Steam App Type**: A local Steam appinfo value describing the installed app class. This slice treats `Music`, `Tool`, `Application`, `Config`, and `Video` as non-capturable, keeps `Demo` eligible, and treats a missing value as unknown rather than non-game.
- **Discovery Account**: The conservation record for discovery. This slice reuses the existing `considered_not_a_game` outcome and adds no account field.
- **Steam Non-Game Install**: The current Steam app id and resolved install directory for an installed app whose appinfo type is excluded. The value is observed at runtime and is not stored by this slice.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A fixture library containing five excluded app types emits zero candidates for those five app ids.
- **SC-002**: The same fixture increments `considered_not_a_game` by five and remains conserved.
- **SC-003**: Fixture titles with `Demo`, `Game`, and absent app types are still emitted as candidates.
- **SC-004**: A known-roots fixture excludes a Steam non-game install root and emits a sibling game unchanged.
- **SC-005**: A listing filter fixture hides matching platform-created rows while preserving a user-authored row with the same anchor and install root.
- **SC-006**: Focused Steam discovery tests pass without requiring a live Steam installation.
- **SC-007**: `cargo xtask ci` passes after implementation.

## Assumptions

- Steam appinfo `common.type` is the authoritative local signal for app class when present.
- `Demo` is a playable title class and remains a possible capture target.
- Unknown app type is an incomplete local observation, not proof that the app is non-game.
- Future name-based fallback filtering, if needed for appinfo-less non-game records, is out of scope for S086 and must be specified separately.
