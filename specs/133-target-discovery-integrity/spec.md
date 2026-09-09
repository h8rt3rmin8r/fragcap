# Feature Specification: Target Discovery Integrity

**Feature Branch**: `codex/133-target-discovery-integrity`
**Created**: 2026-09-09
**Status**: Complete
**Input**: S133 corrects issue #375 by making automatic target registration high precision, keeping broader discovery explicit and non-persistent, and reconciling only provably fragcap-owned historical residue under confirmation.

## Clarifications

### Session 2026-09-09

- Q: Which work owns slice code S133? -> A: Issue #375, superseding the stale roadmap assignment of S133 to issue #331.
- Q: What evidence may authorize automatic registration? -> A: An authoritative platform application identity or positive local title evidence; location alone is insufficient.
- Q: How may historical residue be removed? -> A: Preview first, then explicit confirmation, and only for exact fragcap-owned rows; user-authored and ambiguous rows are preserved.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run the Safe Hero Command (Priority: P1)

As an operator, I can run the no-argument targets experience without Steam client infrastructure or other location-only guesses becoming durable capture targets.

**Why this priority**: The current default silently writes false targets into the user-owned store. That corrupts the primary product inventory and compounds on every affected machine.

**Independent Test**: Use a bounded synthetic installation tree where a generic Games directory contains a Steam client, client infrastructure, a manifest-backed title with an unsupported engine, and a locally evidenced non-platform title. Run the default discovery-registration decision and inspect the resulting candidates and durable rows.

**Acceptance Scenarios**:

1. **Given** a Steam installation nested beneath a generic Games root, **When** the safe default runs, **Then** the exact Steam root and every infrastructure descendant are refused by the known-roots source and are not registered.
2. **Given** a Steam title with an authoritative application manifest but no recognized engine, **When** the safe default runs, **Then** it is registered once with its exact Steam application anchor and install root.
3. **Given** a non-platform directory with positive local title evidence, **When** the safe default runs, **Then** it remains eligible for automatic registration.
4. **Given** a directory supported only by its name or location, **When** the safe default runs, **Then** it is not registered and the refusal is counted.
5. **Given** discovery coverage is truncated or unreadable, **When** the safe default completes, **Then** the affected count and diagnostic remain visible and no success claim fills the missing evidence.

---

### User Story 2 - Explore Broad Candidates Explicitly (Priority: P1)

As an operator investigating an unsupported or proprietary title, I can explicitly request broader bounded discovery, see lower-confidence candidates and their eligibility, and leave the target store unchanged.

**Why this priority**: Removing location-only guesses from automatic registration must not make unsupported titles undiscoverable or conceal why they were withheld.

**Independent Test**: Run explicit discovery over the same synthetic tree and compare the complete candidate listing, registration-eligibility decisions, discovery account, and local-store bytes before and after the command.

**Acceptance Scenarios**:

1. **Given** a bounded known-root child with no positive title evidence, **When** explicit discovery runs, **Then** the candidate may be shown as lower confidence, is reported as ineligible for automatic registration, and is not persisted.
2. **Given** accepted and refused candidates in one run, **When** the discovery account is rendered, **Then** it separately reports produced candidates, precision-policy refusals, and truncated or failed coverage.
3. **Given** the operator takes no explicit add or scan action, **When** explicit discovery completes, **Then** the durable target inventory is byte-for-byte unchanged except for the pre-existing documented volume-eligibility effect.

---

### User Story 3 - Reconcile Historical Tool-Owned Residue (Priority: P1)

As an operator affected by earlier discovery behavior, I can preview exact obsolete rows, understand why each is removable or preserved, and explicitly confirm one atomic cleanup without risking authored or ambiguous targets.

**Why this priority**: Correcting future registration leaves already-persisted false rows in place. Cleanup must be safe because the local store also contains user-owned work.

**Independent Test**: Seed a store with a whole Steam aggregate, Steam infrastructure children, a known-roots duplicate of an anchored title, an unrelated ambiguous known-roots row, and user-authored rows under the same paths. Preview and apply reconciliation against an exact platform inventory.

**Acceptance Scenarios**:

1. **Given** rows whose provenance proves they were created by fragcap's known-roots source and whose exact paths prove they are the Steam client root, client infrastructure, a duplicate of an anchored platform title, or a multi-title aggregate, **When** reconciliation previews, **Then** every row is named with an exact identifier and reason.
2. **Given** a user-authored row, missing provenance, conflicting provenance, or insufficient evidence of ownership, **When** reconciliation evaluates it, **Then** the row is preserved and its ambiguity is reported without inferring ownership from its folder name.
3. **Given** a preview without confirmation, **When** reconciliation exits, **Then** no target row is changed.
4. **Given** explicit confirmation of a non-empty preview, **When** cleanup runs, **Then** exactly the previewed removable rows are deleted atomically and preserved rows remain byte-for-byte equivalent.
5. **Given** the store changes after preview or a requested row cannot be matched exactly, **When** cleanup runs, **Then** the entire cleanup refuses without partial deletion.

---

### User Story 4 - Validate Without Publishing Private Inventory (Priority: P2)

As a maintainer, I can validate the corrected default on a real machine and retain only aggregate candidate, accepted, refused, registered, already-present, and coverage counts.

**Why this priority**: The defect is machine-layout dependent, while installed-title names and local paths are private data that do not belong in public review evidence.

**Independent Test**: Execute the documented count-only validation procedure on an eligible machine and verify that the recorded evidence contains no title name, application identifier, local path, account name, or machine identifier.

**Acceptance Scenarios**:

1. **Given** a real installation, **When** the count-only validation is performed, **Then** the record reconciles all automatic-registration decisions without retaining names or paths.
2. **Given** private candidate details are needed for local diagnosis, **When** they are inspected, **Then** they remain local and are excluded from committed artifacts and pull-request discussion.

### Edge Cases

- The Steam installation is itself one of the fixed known roots or is nested several components beneath a generic Games root.
- Path spelling differs by separator, case, trailing separator, or Windows extended-path form while still naming the same exact directory.
- A Steam manifest identifies a title whose engine is unsupported, whose catalog entry is absent, or whose installation is missing.
- A client infrastructure subtree contains a false-positive engine marker.
- One path is represented by both an authoritative platform row and an older known-roots row.
- A historical aggregate carries findings for several distinct engines.
- A row claims platform classification but has absent, malformed, or unexpected provenance.
- A preview is empty, the operator declines cleanup, or the store changes between preview and confirmation.
- A source fails or reaches a bound after other candidates were found; all outcomes must still reconcile.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The default hero path MUST automatically register only candidates backed by an authoritative platform application identity or positive local title evidence.
- **FR-002**: Directory name, parent directory, known-root membership, or location alone MUST NOT authorize automatic registration.
- **FR-003**: When an exact Steam installation root is known, the known-roots source MUST exclude that root and its infrastructure subtree before classification or descent, including when it is nested beneath a generic Games root.
- **FR-004**: Steam-installed titles MUST come from the authoritative Steam source, retain exact application anchors and install roots, and remain eligible when no engine signature or catalog classification is available.
- **FR-005**: A locally detected title MUST earn eligibility from a positive title-class signal; unrelated security, DRM, UI, browser, or location evidence MUST NOT independently qualify it.
- **FR-006**: The default decision MUST keep automatic-registration acceptance separate from discovery production so lower-confidence candidates remain available to an explicit, bounded, non-persistent discovery flow.
- **FR-007**: The discovery account MUST separately report candidates withheld by the automatic-registration precision policy and MUST preserve all existing parse, access, volume, container, and truncation accounting.
- **FR-008**: The safe default and the doctor discovery action MUST use the same automatic-registration policy and MUST surface accepted, refused, newly registered, and already-present counts without false success.
- **FR-009**: Explicit broad discovery MUST write no target rows and MUST make each candidate's automatic-registration eligibility and refusal reason observable.
- **FR-010**: Reconciliation MUST classify stored rows as removable or preserved from exact provenance, identity, platform-root, authoritative-install, and multi-title evidence; it MUST NOT infer fragcap ownership from a folder name alone.
- **FR-011**: Reconciliation MUST preserve user-authored rows, anchored authoritative platform rows, rows with absent or conflicting ownership evidence, and all other ambiguous entries.
- **FR-012**: A reconciliation preview MUST list the exact stable identifier, handle, and reason for every removable row, summarize preserved ambiguous rows without exposing more data than the ordinary local command, and make no mutation.
- **FR-013**: Reconciliation MUST require explicit confirmation after a preview before deletion and MUST refuse non-interactive mutation without that confirmation.
- **FR-014**: Confirmed reconciliation MUST delete exactly the previewed removable rows in one atomic operation and MUST refuse the entire operation if any exact precondition changed.
- **FR-015**: No whole Steam installation, platform-client infrastructure directory, or multi-title aggregate MAY be produced or retained as one automatically managed target after safe registration and confirmed reconciliation.
- **FR-016**: Regression coverage MUST use a synthetic Games/platform-client tree containing cache, configuration, logs, packages, resources, UI assets, one manifest-backed supported title, and one manifest-backed unsupported-engine title.
- **FR-017**: Real-machine validation evidence MUST contain aggregate counts only and MUST exclude local title names, application identifiers, paths, accounts, and machine identifiers.
- **FR-018**: The master specification and outline MUST narrow the disproven claim that known roots contain only games and MUST describe the safe automatic-registration, explicit discovery, and reconciliation boundaries.
- **FR-019**: The slice MUST preserve the bounded known-roots walk, volume eligibility, idempotent target identity, target store shape, platform precedence, P-4 accounting, P-9 evidence fidelity, and P-10 single target path.
- **FR-020**: The slice MUST NOT add exhaustive system-wide executable enumeration, automatic deletion, other platform integrations, target-list presentation work owned by #376, Steam-list rendering work owned by #374, or release/completion actions.

### Key Entities

- **Automatic Registration Decision**: One candidate plus its accepted or refused result and stable reason.
- **Registration Account**: Aggregate produced, accepted, refused, newly registered, already-present, and coverage outcomes for one run.
- **Excluded Platform Subtree**: An exact normalized platform-client root that the generic known-roots source may neither classify nor descend through.
- **Reconciliation Preview**: An immutable set of exact removable row identities plus preserved ambiguity counts and reasons.
- **Reconciliation Fingerprint**: The exact row facts that must still match when a confirmed preview is applied.
- **Count-Only Validation Record**: Privacy-preserving aggregate evidence from one real-machine run.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In the required synthetic platform tree, zero Steam client or infrastructure directories are produced by known-roots discovery or written to the target store.
- **SC-002**: One hundred percent of manifest-backed eligible Steam titles in the fixture are registered with exact application anchors, including the unsupported-engine title.
- **SC-003**: One hundred percent of location-only candidates are withheld from automatic registration and counted, while explicit discovery can still report them without target-row mutation.
- **SC-004**: For every tested discovery run, produced candidate totals, precision-policy decisions, registration outcomes, and existing coverage counters reconcile with zero unaccounted candidate.
- **SC-005**: Reconciliation removes 100 percent of previewed exact fragcap-owned residue and zero authored, anchored, ambiguous, or changed rows.
- **SC-006**: A declined, unconfirmed, stale, or failed reconciliation changes zero rows; a confirmed valid cleanup is all-or-nothing.
- **SC-007**: Focused regression tests and the complete repository gate pass with zero ignored required checks and zero new silent outcome path.
- **SC-008**: The committed real-machine validation record contains counts only and zero private inventory or machine-identifying values.

## Assumptions

- Issue #375 is the scope authority. The operator's explicit S133 selection supersedes the stale roadmap sentence that assigned S133 to issue #331; final documentation remains open under #331 and receives a later slice code.
- An authoritative Steam application manifest is sufficient title evidence even when its app type is unknown, preserving the existing P-9 rule that unknown is not silently treated as non-game.
- Positive local title evidence means an engine or future equivalent signal explicitly classified as title evidence; anti-cheat, DRM, file location, and directory naming remain supporting facts only.
- Existing `targets discover` is the explicit broad inspection surface. S133 may extend its output and accounting but does not make it persist candidates.
- Reconciliation is an operator-invoked repair, not an automatic migration. Rows that cannot be proven safe to remove remain visible for manual action.
- Repository tests provide the publishable path-level evidence. Any real-machine validation retains only aggregate counts.
