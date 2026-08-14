# Feature Specification: correct the schema $id host to fragcap.com

**Feature Branch**: `047-schema-id-host`

**Created**: 2026-08-14

**Status**: Draft

**Input**: GitHub issue #117. The published JSON schema declares
`"$id": "https://fragcap.dev/schema/target/v1.json"`, but `fragcap.dev` is not a
domain the project owns or uses. The project's real domain is `fragcap.com`
(the docs site) and the repo is `h8rt3rmin8r/fragcap`. Replace the host with the
canonical `fragcap.com` in all four locations, keeping the two schema copies
byte-identical, and record the deliberate pre-1.0 identity correction.

## Clarifications

### Session 2026-08-14

Resolved from the issue and the operator's decision on the roadmap plan:

- Q: What is the canonical replacement host? -> A: `fragcap.com`, giving
  `https://fragcap.com/schema/target/v1.json`. It is the real, owned domain the
  docs site already serves, and leaves room to actually host the schema there
  later. (Operator decision.)
- Q: Is changing a "published" $id allowed? -> A: Yes, as a deliberate pre-1.0
  identity correction. Nothing dereferences the `$id`; it is an opaque stable
  identifier, and v1 is embedded-only, not registry-published. The S025 contract
  states the host is "fixed and never changed for a published version", so this
  is recorded as a dated decision rather than a silent edit.
- Q: How many places change, and what keeps them consistent? -> A: Four: the
  published schema, the embedded asset (byte-identical to it, enforced by a
  drift test), the CLI test asserting the exact `$id`, and the identity contract
  example. All change in one commit.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The schema identifies itself with a real host (Priority: P1)

Anyone reading the schema (in the repo, or emitted by `fragcap schema print`)
sees an `$id` under `fragcap.com`, the domain the project actually owns, rather
than a nonexistent `fragcap.dev`. The two committed schema copies remain
byte-identical, and the tooling that asserts the identity is updated in the same
change so the gate stays green.

**Why this priority**: It is the whole issue (#117): the schema misrepresents its
identity with a domain the project does not control.

**Independent Test**: `grep -rn "fragcap.dev"` over the repo returns nothing;
`fragcap schema print` emits the `fragcap.com` `$id`; the schema drift test and
the CLI schema test pass.

**Acceptance Scenarios**:

1. **Given** the published schema and the embedded asset, **When** read, **Then**
   both carry `"$id": "https://fragcap.com/schema/target/v1.json"` and are
   byte-identical.
2. **Given** `fragcap schema print`, **When** run, **Then** its output contains
   the `fragcap.com` `$id` (the CLI test asserts this).
3. **Given** the whole repo, **When** searched for `fragcap.dev`, **Then** there
   are zero matches.

### Edge Cases

- The two schema JSON files must stay byte-identical; a drift test fails if they
  diverge, so both are edited identically in one change.
- The `$id` is not dereferenced over the network anywhere; this is an identifier
  string change, not a hosting change (no route is added).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The published schema `docs/schema/target-schema.v1.json` and the
  embedded asset `crates/fragcap-profile/assets/target-schema.v1.json` MUST both
  declare `"$id": "https://fragcap.com/schema/target/v1.json"`, and MUST remain
  byte-identical.
- **FR-002**: The CLI test that asserts the exact `$id`
  (`crates/fragcap-cli/tests/cli_schema.rs`) MUST be updated to the `fragcap.com`
  string so the gate passes.
- **FR-003**: The identity contract example
  (`specs/025-master-json-schema/contracts/master-schema.contract.md`) MUST use
  the `fragcap.com` host, and its "fixed and never changed for a published
  version" note MUST be annotated with the dated correction so the contract does
  not contradict the change.
- **FR-004**: The deliberate identity change MUST be recorded as a dated decision
  fragment under `changelog.d/`.
- **FR-005**: No occurrence of `fragcap.dev` remains as a live identifier: not
  as the schema `$id` (either copy), the asserted string in the CLI test, or the
  canonical example in the identity contract. Historical references that document
  this correction (the decision fragment, the contract's dated correction note,
  and this slice's own spec artifacts) name the old host deliberately and are
  permitted.
- **FR-006**: All edited text MUST be UTF-8, LF, and free of em and en dashes.

### Key Entities

- **Schema `$id`**: the opaque stable identifier of schema version 1; its host is
  the only thing this slice changes, from `fragcap.dev` to `fragcap.com`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: No `fragcap.dev` remains as a live identifier (schema `$id` in
  both copies, the CLI assertion, the contract's canonical example all read
  `fragcap.com`); the only remaining `fragcap.dev` strings are the deliberate
  historical references documenting the correction.
- **SC-002**: The schema drift test (embedded asset equals the published copy)
  and the CLI schema test pass with the new host.
- **SC-003**: `cargo xtask ci` is green.

## Assumptions

- `fragcap.com` is the correct canonical host (operator decision); no schema
  route is served at that URL today and none is added here.
- `docs/schema/**` is not on the pinned-artifact list, so no pinned-artifact gate
  applies; the recorded decision exists because the identity contract declared
  the host immutable, not because the file is mechanically pinned.
