# Feature Specification: Explicit Fresh-Start Uninstall

**Feature Branch**: `codex/137-fresh-start-uninstall`

**Created**: 2026-09-09

**Status**: Complete

**Input**: Work slice S137 implementing issue #377 after S131 established preserve-by-default package ownership and S124 established exact Doctor recovery authority.

## User Scenarios & Testing

### User Story 1 - Preserve Data Unless Explicitly Selected (Priority: P1)

As a Windows user removing fragcap, I need the ordinary uninstall path to preserve all application and user data so an uninstall, repair, upgrade, rollback, or automated maintenance action cannot unexpectedly erase my targets or evidence.

**Why this priority**: Destructive cleanup is acceptable only as an explicit exception to the existing ownership contract.

**Independent Test**: Seed every canonical data category, run each installer lifecycle path without the fresh-start opt-in, and compare every byte with its pre-operation digest.

**Acceptance Scenarios**:

1. **Given** canonical fragcap user data exists, **when** ordinary interactive or silent uninstall runs without the opt-in, **then** every user-owned item remains byte-identical.
2. **Given** the fresh-start control is displayed, **when** the user leaves it unchecked or cancels, **then** no user-data cleanup command runs.
3. **Given** repair, upgrade, rollback, or automated maintenance, **when** the lifecycle completes or fails, **then** fresh-start cleanup never runs even if a stale public property is present.

---

### User Story 2 - Reset the Initiating User Safely (Priority: P2)

As a Windows user who deliberately wants a clean reinstall, I need uninstall to show an unchecked irreversible fresh-start choice, identify my exact canonical data roots and categories, and remove them only after exact Deep Capture recovery succeeds.

**Why this priority**: This is the primary issue #377 journey and must produce a genuine clean-account state without broad path inference.

**Independent Test**: Use isolated canonical roaming and local roots containing all owned categories plus excluded neighbors, preview and confirm the current-user cleanup, uninstall, reinstall, and verify clean first-run behavior plus preservation of every excluded path.

**Acceptance Scenarios**:

1. **Given** interactive uninstall for a per-machine installation, **when** removal is selected, **then** an unchecked control names the initiating user scope, irreversible consequence, exact canonical roots, and the local database, profiles, catalog, logs, caches, and sensitive Deep Capture evidence categories.
2. **Given** the user confirms the current-user choice, **when** cleanup runs, **then** the installed fragcap executable receives the initiating user's exact roaming and local roots rather than resolving a service-account profile.
3. **Given** inactive recoverable Deep Capture obligations, **when** cleanup runs, **then** the shared Doctor authority reconciles them before the related records or session roots are removed.
4. **Given** an active, ambiguous, malformed, unsupported, or failed recovery obligation, **when** cleanup runs, **then** the affected evidence remains, the wipe is partial, and the exact retained path and Doctor guidance are reported.
5. **Given** cleanup succeeds and fragcap is reinstalled, **when** first-run storage is initialized, **then** no prior targets, profiles, calibration facts, session residue, or writable catalog state reappears.

---

### User Story 3 - Perform Auditable Administrative Cleanup (Priority: P3)

As a Windows administrator, I need a separate all-users workflow that previews every exact canonical profile root before deletion and binds confirmation to that inventory so no other profile is swept implicitly or after the inventory changes.

**Why this priority**: A per-machine package requires an administrative scope, but its broader authority must be more deliberate than a static MSI checkbox.

**Independent Test**: Build an isolated multi-profile inventory, verify the preview names each exact root, execute with its digest, mutate the inventory and verify refusal, then confirm included profiles are reset while excluded and redirected paths remain untouched.

**Acceptance Scenarios**:

1. **Given** an administrator requests all-users cleanup, **when** the preview runs, **then** it enumerates each exact canonical fragcap root and category without deleting anything.
2. **Given** the administrator supplies the exact preview identifier and confirms, **when** the inventory is unchanged, **then** only those listed roots are eligible for cleanup.
3. **Given** any root, owner, redirection, or inventory entry changes after preview, **when** execution begins, **then** cleanup refuses and requires a new preview.
4. **Given** a silent current-user uninstall, **when** both the dedicated opt-in property and current-user scope are supplied, **then** the same cleanup contract runs; absence, an unknown value, or all-users scope preserves data and reports the separate administrative command.

### Edge Cases

- The roaming or local canonical root is missing, locked, read-only, replaced after inventory, or contains an unreadable child.
- A listed root, ancestor, or descendant is a symlink, junction, mount point, or other reparse point.
- The roaming and local roots resolve to the same object, overlap, leave the expected profile, or name a filesystem root.
- Deep Capture ownership proves an active session, legacy ambiguous owner, wrong certificate store, unsupported journal, failed exact action, or concurrent recovery lock.
- A custom `FRAGCAP_*` override or command-line path points inside or outside a canonical root.
- Packet captures, exported bundles, custom profiles, custom databases, Wireshark configuration, Npcap, or independently managed extcap copies exist beside canonical data.
- Cleanup removes some ordinary data and then encounters a locked file or failed recovery.
- A stale opt-in property reaches install, repair, upgrade, rollback, or maintenance that is not explicit product removal.
- The initiating interactive user's exact data roots cannot be determined.
- The all-users inventory changes between preview and execution.

## Requirements

### Functional Requirements

- **FR-001**: Ordinary interactive and silent uninstall MUST preserve all user-owned data unless the dedicated fresh-start opt-in is explicitly selected.
- **FR-002**: Install, repair, reinstall, upgrade, rollback, downgrade refusal, and automated maintenance MUST never execute fresh-start cleanup.
- **FR-003**: Interactive uninstall MUST present an unchecked current-user fresh-start option that states the operation is irreversible and identifies the exact canonical roaming and local roots plus every data category in scope.
- **FR-004**: Silent uninstall MUST require `FRAGCAP_FRESH_START=1` together with `FRAGCAP_FRESH_START_SCOPE=current-user`; any absent or unrecognized value MUST preserve data.
- **FR-005**: The installer MUST pass the initiating interactive user's exact canonical roots to the product cleanup contract and MUST NOT resolve roots from an elevated service account.
- **FR-006**: One product-owned inventory and cleanup contract MUST be shared by installer invocation, direct preview, direct execution, and automated tests; MSI path guesses and name-based filesystem search are prohibited.
- **FR-007**: The current owned inventory MUST include canonical `%APPDATA%\fragcap` and `%LOCALAPPDATA%\fragcap` content, including catalog bootstrap state, `local.db`, profiles, target and calibration state, settings, caches, logs, session bundles, sensitive artifacts, journals, and session-owner records.
- **FR-008**: The inventory MUST exclude Npcap, Wireshark configuration, independently managed extcap registration, captures or exports outside canonical roots, custom databases, custom profiles, custom session roots, `FRAGCAP_*` overrides, and unrelated neighboring data.
- **FR-009**: Cleanup authority MUST be based on exact approved roots and discovered entries, never a filename substring, display name, process identifier, certificate label, or broad profile search.
- **FR-010**: Cleanup MUST refuse filesystem roots, overlapping or aliased roots, roots outside the selected profile, and any root or traversed entry whose link or reparse status could redirect deletion outside the approved root.
- **FR-011**: Before deleting Deep Capture sessions, journals, or owner records, cleanup MUST invoke the same exact recovery authority Doctor uses.
- **FR-012**: Active, ambiguous, malformed, unsupported, wrong-store, mismatched, concurrently owned, or failed Deep Capture recovery MUST retain the evidence needed for later recovery and MUST prevent a complete-wipe result.
- **FR-013**: Cleanup MUST produce a bounded versioned report that lists each approved root and category, each removed or retained item, every refusal or failure, the overall complete or partial status, and actionable Doctor guidance without secrets or payload content.
- **FR-014**: Partial ordinary deletion MUST continue only where independent ownership remains proven, MUST retain every failed path in the report, and MUST never report success when any approved item remains.
- **FR-015**: A successful fresh start MUST leave no canonical local targets, profiles, calibration facts, session residue, catalog bootstrap state, settings, caches, or logs from the prior installation.
- **FR-016**: All-users cleanup MUST be a separate administrator workflow that first emits every exact profile identity and canonical root, then requires a confirmation identifier bound to the complete inventory. Because Windows CurrentUser certificate authority belongs to the profile identity rather than the elevated administrator, another profile's Deep Capture sessions MUST remain until current-user cleanup in that profile has reconciled them.
- **FR-017**: All-users execution MUST refuse when the preview identifier is absent, any inventory fact changed, any profile cannot be classified safely, administrator authority is unavailable, or cross-profile CurrentUser recovery would be required; independently owned ordinary state MAY still be removed under a truthful partial report. Other profiles MUST never be included by current-user cleanup.
- **FR-018**: The static MSI UI MUST NOT claim to display a dynamic all-users inventory it cannot verify; it MUST direct administrators to the separate preview-bound all-users workflow.
- **FR-019**: Package certification MUST cover preserve-by-default, confirmed current-user cleanup, explicit all-users handling, custom-path exclusion, reparse containment, Deep Capture recovery failure, partial failure truth, and clean reinstall.
- **FR-020**: Automated cleanup tests MUST use isolated roots and test seams and MUST NOT delete the developer, runner, or system account's real data.
- **FR-021**: User and maintainer documentation MUST describe both scopes, exact inclusions and exclusions, irreversible consequences, silent opt-in, preview binding, partial-wipe recovery, and clean-reinstall expectations.
- **FR-022**: S137 MUST update the master specification, outline, roadmap, package certification guidance, changelog, and a dated decision fragment because it changes pinned installer and certification artifacts.
- **FR-023**: S137 MUST add no target instrumentation, broad profile deletion, automatic external-state cleanup, hidden trust action, new runtime dependency, release execution, or final Deep Capture completion claim.

### Key Entities

- **Fresh-Start Scope**: Current-user or all-users authority selected through distinct consent surfaces.
- **Owned Root**: One exact canonical roaming or local fragcap data root with profile identity and category set.
- **Inventory Entry**: One exact child classified as owned, excluded, refused, or unavailable without following redirections.
- **Inventory Identifier**: A stable digest over version, scope, profile identities, roots, categories, and entry facts used to bind confirmation to preview.
- **Recovery Outcome**: The shared Doctor authority result for each Deep Capture obligation before evidence removal.
- **Cleanup Report**: The bounded versioned record of inventory, recovery, deletion, retention, failures, and overall status.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every unopted uninstall, repair, reinstall, upgrade, rollback, and maintenance test preserves 100 percent of seeded user-data bytes and invokes cleanup zero times.
- **SC-002**: Interactive fresh-start presentation is unchecked by default and names 100 percent of approved roots and required categories before confirmation.
- **SC-003**: Confirmed current-user cleanup removes 100 percent of approved ordinary data after 100 percent of required recovery actions reach safe terminal states, while modifying zero excluded or custom-location items.
- **SC-004**: Reparse, alias, overlap, root, custom-path, active-session, ambiguous-recovery, changed-inventory, and insufficient-authority cases each refuse deletion with the exact retained path and reason.
- **SC-005**: Every partial failure produces a non-success result and report containing 100 percent of retained approved paths plus recovery guidance.
- **SC-006**: A successful wipe and reinstall produces the same observable first-run stores and empty user state as a clean test account.
- **SC-007**: All-users execution modifies zero profiles until an administrator supplies the exact identifier from a complete preview, and any changed fact causes zero deletion.
- **SC-008**: The ordinary CI gate plus Windows package certification cover every FR-019 scenario with finite, hidden, non-interactive child processes and no real-profile cleanup.

## Assumptions

- The supported MSI remains per-machine and WiX v3; its static UI can safely identify the initiating user's roots but cannot truthfully render a dynamic multi-profile inventory.
- The existing Doctor recovery planner and adapters remain the sole authority for reversing Deep Capture obligations.
- Canonical roots are product-owned containers, but custom paths and overrides remain operator-owned and outside implicit cleanup authority.
- The MSI current-user choice is sufficient for the common fresh-start journey; administrators use the separately documented preview-bound command for all-users scope.
- Installer-owned program files, product registration, PATH, and exact Defender ownership remain governed by S131's ordinary uninstall contract.
