# Feature Specification: Deep Capture compatibility facts

**Feature Branch**: `072-deep-capture-compatibility-facts`

**Created**: 2026-08-25

**Status**: Draft

**Input**: User description: "Issue #217. Store local Deep Capture compatibility facts in the targets database so launch behavior, proxy inheritance, supported traffic types, pinning observations, and freshness can be reused without re-running fragile game launch experiments. Do not commit PII, local paths, endpoints, account material, or real local title names from the fact-finding process."

## Clarifications

### Session 2026-08-25

- Q: Should compatibility facts create a second target-resolution path? A: No. Every compatibility fact belongs to an existing `targets(id)` row; target discovery and target identity remain owned by the target store.
- Q: Should unknown compatibility be omitted until known? A: No. Unknown is a real value where the product must preserve uncertainty, but it must be a closed token rather than free prose.
- Q: Should launch case also carry final-owner handoff? A: No. Launch case is mutually exclusive. Final-owner handoff is recorded separately so `direct-exe-cold` or `steam-protocol-cold` is not lost when a later executable owns sockets.
- Q: Should proxy backend details live in notes? A: No. Backend name, backend version, and proxy mode are structured provenance because observations can differ across dummy listeners, mitmproxy, and future native backends.
- Q: Can local observations store executable identity? A: Yes, as local target compatibility facts. Public repo artifacts and exported community evidence must remain scrubbed before publication.

## User Scenarios & Testing

### User Story 1 - Preserve local Deep Capture observations (Priority: P1)

An operator runs a Deep Capture compatibility experiment for a locally registered target. fragcap records the observation against the target row, including what was observed, how the target was launched, which proxy backend and mode produced the evidence, which executable finally owned sockets if known, and whether the fact is current or stale.

**Why this priority**: The launch and proxy behavior under study is fragile, time-consuming, and sometimes changes between cold and warm platform states. Losing the observation forces the operator to re-run tests and invites false assumptions.

**Independent Test**: Insert a compatibility fact into an in-memory target store and read it back, asserting every field round-trips exactly.

**Acceptance Scenarios**:

1. **Given** a registered target, **When** a compatibility fact is inserted, **Then** the row is keyed to that target and can be read back through a target-oriented store API.
2. **Given** the observation includes proxy backend name, backend version, and mode, **When** the fact is read back, **Then** those fields are preserved separately from the optional note.
3. **Given** the observation includes a final owner executable and handoff marker, **When** the fact is read back, **Then** the launch case remains unchanged and the handoff is preserved separately.

### User Story 2 - Reject invented or malformed compatibility values (Priority: P1)

A developer or importer attempts to record a compatibility value outside the supported vocabulary. fragcap rejects the row before it can become durable local advice.

**Why this priority**: Deep Capture compatibility facts will influence future operator choices. A fact store that accepts misspelled or improvised tokens silently becomes a source of false guidance.

**Independent Test**: Attempt to insert invalid key/value combinations through both the Rust model and direct SQLite insertions.

**Acceptance Scenarios**:

1. **Given** `proxy-routing`, **When** the value is `confirmed`, **Then** model construction fails because that token belongs to a different fact family.
2. **Given** direct SQLite insertion bypasses the Rust constructor, **When** a key/value pair violates the closed vocabulary, **Then** the table rejects it with a constraint failure.
3. **Given** the fact value is intentionally unknown, **When** the fact family supports `unknown`, **Then** the store accepts it as an explicit observation state.

### User Story 3 - Migrate existing stores without inventing facts (Priority: P1)

An existing targets database is opened after the feature lands. It gains the compatibility fact table, but no fact rows are backfilled from platform metadata or local guesses.

**Why this priority**: Compatibility is observed behavior, not metadata. A migration that fabricates rows would violate the instrument's truthfulness contract.

**Independent Test**: Simulate a v8 store with existing target rows, open it with the new build, and verify the schema migrates to v9 while the fact table remains empty.

**Acceptance Scenarios**:

1. **Given** a v8 target store with target rows, **When** the new build opens it, **Then** the schema version becomes v9 and target rows remain intact.
2. **Given** the migrated store, **When** facts are queried for an existing target, **Then** the result is empty until an observed run, user confirmation, or import writes a fact.
3. **Given** a target is deleted, **When** compatibility facts exist for that target, **Then** the facts are deleted by target-keyed cascade.

## Edge Cases

- Different proxy backends produce different observations for the same target and launch case: each row preserves backend and mode provenance so downstream consumers can compare them.
- The final socket owner is not known: `final_owner_executable` remains null and `final_owner_handoff` remains false unless observed.
- A fact is retained only as context after target or proxy changes: the row remains queryable with `stale = true`.
- The operator has only partial information: optional provenance fields may be null, while the fact key, value, target id, and evidence source remain required.
- Free-form notes can contain local details if written by the operator; any export path must scrub before publication. This slice stores local facts only and adds no export surface.

## Requirements

### Functional Requirements

- **FR-001**: The targets database MUST store Deep Capture compatibility facts in a table keyed to `targets(id)`.
- **FR-002**: Each fact MUST carry a closed fact key and a key-specific value. Known closed value domains MUST be enforced by the Rust model and SQLite CHECK constraints.
- **FR-003**: Each fact MUST carry an evidence source token identifying whether it came from an observed run, user confirmation, imported catalog, or stale observation.
- **FR-004**: Each fact MUST be able to carry an optional launch case without using launch case to encode final-owner handoff.
- **FR-005**: Each fact MUST be able to carry proxy backend name, proxy backend version, and proxy mode as structured provenance fields.
- **FR-006**: Each fact MUST be able to carry the observed final-owner executable and a final-owner handoff marker.
- **FR-007**: Existing v8 stores MUST migrate to v9 by adding the compatibility table only; migration MUST NOT backfill or infer compatibility facts.
- **FR-008**: Deleting a target MUST delete that target's compatibility facts.
- **FR-009**: The feature MUST add no new runtime dependency.
- **FR-010**: Public repository artifacts for this slice MUST NOT contain PII, local filesystem paths, account material, endpoints, or real local title names from fact-finding.

### Key Entities

- **Compatibility fact**: A single local observation or imported compatibility datum tied to one target row. It records fact key, value, launch case, evidence source, freshness, proxy provenance, final-owner information, stale state, and optional note.
- **Launch case**: A mutually exclusive launch path such as warm Steam protocol launch, cold Steam protocol launch, warm direct executable launch, cold direct executable launch, or publisher-launcher launch.
- **Proxy provenance**: Structured backend identity, backend version, and proxy mode fields that distinguish observations collected through different proxy implementations or configurations.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A compatibility fact inserted through the store API round-trips with all provenance, freshness, proxy, final-owner, stale, and note fields intact.
- **SC-002**: Invalid key/value combinations fail before durable storage through both model validation and SQLite constraints.
- **SC-003**: A simulated v8 store migrates to v9 with zero compatibility facts invented.
- **SC-004**: The dependency graph is unchanged, verified by `cargo xtask deps`.
- **SC-005**: A scan of new public artifacts contains no local title names, local paths, endpoints, account material, or other fact-finding PII.

## Assumptions

- This slice creates the local model and persistence layer only. It does not implement the proxy, live collection, CLI display, export, or community synchronization.
- The compatibility fact vocabulary is intentionally narrow for the first storage slice. Additional traffic types or proxy modes can be added by a later schema/model change when there is a concrete producer.
- `observed_at` and version fields are stored as text in this slice to avoid introducing a datetime dependency and to match existing store conventions for imported catalog data.
