# Research: Guided Calibration Case Proposals

## Decision: Place proposal policy in the Deep Capture facade

**Rationale**: The decision joins existing target topology, S121 compatibility dimensions, warm-to-cold semantics, and calibration phases. The facade is already the only layer above all four authorities and is the S132 stable integration boundary.

**Alternatives considered**: `fragcap-targets` would make session phases and warm-launch behavior flow down into storage vocabulary. The CLI would create a second policy path inaccessible to library users. `fragcap-core` cannot depend on target or I/O vocabulary under P-2.

## Decision: Reuse stored launch-entry parsing and add stricter proposal validation

**Rationale**: `entry_windows_launch_entries` already applies the Windows filter, carries arguments and roles, and preserves first-seen order. Proposal validation then applies the shipped direct and publisher structural rules without touching the filesystem. This keeps P-10's one stored form while refusing malformed or ambiguous declarations.

**Alternatives considered**: Reading raw JSON independently would duplicate filtering and entry semantics. Calling `prepare_managed_launch` would canonicalize paths and read the filesystem, violating the pure proposal boundary.

## Decision: Model process authority as complete or unavailable

**Rationale**: A list of observed images proves absence only when enumeration completed. An unavailable snapshot must block cold readiness rather than behave like an empty list. Case-insensitive deduplication matches Windows image identity while preserving the declared spelling used in guidance.

**Alternatives considered**: A bare image list cannot distinguish a cold machine from failed enumeration. A per-image unknown state adds complexity without more authority because any unknown required image blocks the same conclusion.

## Decision: Use existing exact applicability and a conservative conflict layer

**Rationale**: S121 already defines applicable, stale, legacy-incomplete, and named mismatch outcomes over launch, routing, family, protocol, backend, product, and target versions. Proposal policy should consume that authority. It additionally treats differing current exact values as conflict so a guided retest does not hide disagreement behind chronology.

**Alternatives considered**: Reimplementing all applicability comparison would create drift. Selecting only the latest row would be valid for eligibility but would fail #380's requirement to explain conflicts and propose useful retests.

## Decision: Use routing and inspectability as proposal completion signals

**Rationale**: `proxy-routing = reached-client` directly proves the trust-free prerequisite. Per-protocol `inspectability = full` directly proves the useful protocol outcome. Other current values are retained observations but remain negative or partial for guided completion, so a retest remains explainable.

**Alternatives considered**: Treating any current row as complete would suppress negative and partial retests. Requiring every possible fact class would produce redundant attempts and turn the proposal engine into an aggregate verdict.

## Decision: Preserve deterministic output independently of input ordering

**Rationale**: Protocols and present images are case-insensitively deduplicated and sorted by closed token where ordering is semantic. Fact assessment uses sets of current values and applicability categories, not caller vector order. Declared launch stages retain stored order because that order is part of publisher topology.

**Alternatives considered**: Trusting caller ordering would make identical evidence produce different workflow plans. Sorting publisher stages would destroy ancestry meaning.

## Decision: No new dependency or storage version

**Rationale**: Existing enums, target entries, fact applicability, and standard collections are sufficient. Proposal generation is a pure in-memory projection.

**Alternatives considered**: A workflow database and serialization layer belong to later interruption and resume work in #380, not to S139.
