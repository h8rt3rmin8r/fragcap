# Feature Specification: Known-Roots Discovery Corrections

**Feature Branch**: `codex/077-known-roots-discovery-corrections`

**Created**: 2026-08-26

**Status**: Implemented

**Input**: User description: "Consolidate issues #209 and #210 into S077: keep known-roots path identities canonical, reject multi-engine container directories as titles, continue discovery beneath them, and account visibly for bounded descent."

## Clarifications

### Session 2026-08-26

- Q: Does S077 expand the known-roots maximum descent? A: No. The existing shallow bound remains; S077 makes a container blocked by that bound visible through a dedicated account outcome and warning.
- Q: What exactly establishes a container? A: More than one distinct canonical engine product in the classifier's observed findings. Repeated markers for one engine and non-engine findings do not count.
- Q: Does every multi-engine result become a durable target classification? A: No. Container is a discovery-control verdict. It suppresses the directory as a candidate and requests descent; it is not persisted as a target.
- Q: Where is path normalization applied? A: When the separator-neutral known-root definition is composed with a real volume mount. Fixture paths remain under fixture control, and emitted real-filesystem children inherit the host-native form.
- Q: Are existing mixed-path rows migrated? A: No. This slice corrects newly discovered candidate identities and install roots. Rewriting user-owned historical rows requires a separate migration policy.
- Q: How is privacy protected in regression coverage? A: Tests use temporary roots and synthetic title and engine labels only. No actual local title, account, endpoint, or operator path enters tracked artifacts.

## User Scenarios & Testing

### User Story 1 - Discover Titles Inside Container Directories (Priority: P1)

An operator whose known game root contains another organizational or platform directory sees the actual game directories beneath it instead of one misleading target representing the whole container.

**Why this priority**: A container falsely registered as one title hides every game below it. This is a user-visible correctness failure and an unreported loss of discovery coverage.

**Independent Test**: Build a directory tree in which a known-root child contains markers for multiple distinct engines and contains separately classifiable game directories. Run discovery and verify that the container is not emitted, its child games are emitted, and every examined directory has one named account outcome.

**Acceptance Scenarios**:

1. **Given** a known-root child with evidence for multiple distinct engine products and separately identifiable games below it, **When** known-roots discovery runs, **Then** the child is treated as a container, is not emitted as a target, and discovery descends to the games beneath it.
2. **Given** a directory with evidence for one engine product, **When** known-roots discovery runs, **Then** it remains a title candidate and stop-on-hit prevents descent into its ordinary asset subtree.
3. **Given** repeated evidence for the same engine product, **When** the directory is classified, **Then** repetition alone does not make the directory a container.

---

### User Story 2 - See Bounded Discovery Loss (Priority: P1)

An operator can distinguish a container that discovery traversed from one whose descendants were not traversed because the shallow-walk limit was reached.

**Why this priority**: Constitution P-4 prohibits silent loss. Declining a false container candidate without reporting that its descendants remained unseen would replace one silent coverage defect with another.

**Independent Test**: Place a multi-engine container at both traversable and terminal depths, run discovery, and verify separate conserved outcomes for descended containers and depth-limited containers, with a named warning for the latter.

**Acceptance Scenarios**:

1. **Given** a container above the depth limit, **When** discovery classifies it, **Then** the account records a descended-container outcome and discovery examines its immediate children.
2. **Given** a container at the maximum descent depth, **When** discovery classifies it, **Then** the account records a depth-limited-container outcome and warns that descendants may be undiscovered.
3. **Given** any mixture of titles, misses, containers, skipped volumes, and access errors, **When** discovery completes, **Then** all named outcomes reconcile exactly to the number of items considered.

---

### User Story 3 - Receive Canonical Path Identities (Priority: P2)

An operator receives known-roots candidates whose path identity and install root use one platform-consistent separator form, so listing, persistence, comparison, and export do not disagree about the same directory.

**Why this priority**: Mixed separators are more than display noise because the path is also durable target identity and install-root data.

**Independent Test**: Run the real filesystem directory lister beneath a joined known root and verify that every emitted candidate identity and install root uses only the host platform's separator convention.

**Acceptance Scenarios**:

1. **Given** a volume mount point and a separator-neutral known-root definition, **When** the root is joined and walked, **Then** the filesystem receives a platform-native path.
2. **Given** candidates returned by the real filesystem lister, **When** they become candidate identities and install roots, **Then** no path contains both slash styles.
3. **Given** a fixture directory tree using its existing normalized paths, **When** offline discovery tests run, **Then** the shared known-root definitions still drive the fixture and real walks without separate root lists.

### Edge Cases

- A container can contain repeated markers for one engine and one marker for another; distinct engine products, not marker count, determine the outcome.
- Engine product matching follows the classifier's canonical product identity and does not infer equivalence from spelling.
- Anti-cheat and DRM findings do not contribute to the multi-engine container rule.
- A detection scan can be incomplete while still observing multiple distinct engines; observed evidence is not discarded, and incomplete coverage remains reported.
- A container can be empty, inaccessible, or disappear between classification and descent; the existing absence and access-error behavior still applies to the attempted child listing.
- A genuine title can contain nested directories with other engine-like files. If its own bounded classification reports only one distinct engine, stop-on-hit remains unchanged.
- A mount point can already end in either separator or be a filesystem root; joining must not duplicate or mix separators.

## Requirements

### Functional Requirements

- **FR-001**: Known-roots classification MUST distinguish a title hit, a container directory, and a miss.
- **FR-002**: A directory MUST receive the container outcome when its classification evidence contains more than one distinct engine product.
- **FR-003**: Repeated findings for one engine product and findings outside the engine category MUST NOT trigger the container outcome.
- **FR-004**: A container directory MUST NOT be emitted as a candidate target.
- **FR-005**: A container above the shallow-walk depth limit MUST be descended using the same immediate-child discipline as a miss.
- **FR-006**: A title hit MUST retain the existing stop-on-hit behavior and MUST NOT be descended.
- **FR-007**: The discovery account MUST carry separate named outcomes for a container that was descended and a container whose descendants were not examined because the depth limit was reached.
- **FR-008**: Both container outcomes MUST participate in the account conservation invariant.
- **FR-009**: A depth-limited container MUST produce a diagnostic naming the affected directory and stating that descendants may remain undiscovered.
- **FR-010**: Detection coverage warnings produced while classifying a container MUST remain visible.
- **FR-011**: Known-root paths passed to the real filesystem boundary MUST use the host platform's native path composition.
- **FR-012**: Candidate path identities and install roots emitted from the real filesystem walk MUST NOT contain mixed slash styles.
- **FR-013**: The separator-neutral `KNOWN_ROOTS` definitions MUST remain the single root list used by fixture and real filesystem discovery.
- **FR-014**: Existing behavior for missing roots, unreadable roots, ineligible volumes, title fidelity, technology evidence, and non-container misses MUST remain unchanged.
- **FR-015**: The master specification and glossary MUST describe the container-aware exception to stop-on-hit and the expanded discovery-account outcomes.
- **FR-016**: Tests MUST use synthetic placeholder directories and engine names and MUST NOT commit local paths, personal data, or actual locally installed game titles gathered during discovery.

### Key Entities

- **Classifier Verdict**: The decision for one directory: title hit, container, or miss.
- **Container Directory**: A directory whose observed classification evidence contains more than one distinct engine product, indicating that it organizes multiple titles rather than representing one title.
- **Discovery Account**: The conserved per-run tally, expanded to distinguish descended containers from depth-limited containers.
- **Canonical Candidate Path**: The platform-consistent filesystem path used for both candidate identity and install root.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A synthetic container with two distinct engine products and two child titles yields exactly the two child candidates and never yields the container as a candidate.
- **SC-002**: A synthetic title with repeated findings for one engine retains title classification and stop-on-hit behavior.
- **SC-003**: Traversable and depth-limited containers increment different named counters, and every affected test reports a conserved discovery account.
- **SC-004**: Every depth-limited container produces one diagnostic naming its synthetic path and the reduced discovery coverage.
- **SC-005**: Real-filesystem tests on each supported host verify that emitted candidate identities and install roots contain no mixed separators.
- **SC-006**: All existing target-discovery, signature-detection, target-listing, persistence, and export tests pass without changing unrelated source behavior.
- **SC-007**: Repository privacy, formatting, lint, documentation, and test gates pass with no real local game title or personally identifying path added to tracked files.

## Assumptions

- Multiple distinct engine products are the approved observable signal for a container in this slice. General container inference from folder names or arbitrary layout heuristics is out of scope.
- The existing shallow maximum descent remains unchanged. S077 makes truncation visible rather than expanding into deep filesystem scanning.
- Path canonicalization is applied at the real filesystem boundary. It does not rewrite platform identities or introduce case, symlink, or long-path canonicalization.
- Existing public discovery types can be extended because the project is in alpha, but all in-repository consumers and renderers must expose the new account outcomes truthfully.
- No database schema migration is required. Corrected candidate paths are persisted only when the user next acts on newly discovered candidates; S077 does not rewrite existing local target rows.
