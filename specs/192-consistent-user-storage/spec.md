# Feature Specification: Consistent User Storage

**Slice**: S192
**Branch**: `codex/s192-consistent-user-storage`
**Created**: 2026-11-04
**Status**: In progress
**Issue**: #556
**Architecture of record**: `docs/fragcap-specification.md` sections 15.3, 19.2, 24, and 27

## User Scenarios

### User Story 1: Find the same current-user data from either launch context (P1)

The same Windows user starts Fragcap directly or through a packaged application and sees the same targets, catalog, local database, profiles, session registry, and retained sessions. New persistent data resides under that user's profile in `.fragcap`, outside AppData virtualization. A different user's data is never selected.

**Independent check**: Independent controlled processes under one account write and read the same synthetic target and retained session while one process uses altered AppData variables. Their opened-file destinations agree. Actual arbitrary packaged-host behavior remains a post-release field check.

### User Story 2: Preserve existing state during the change (P1)

The first data operation inventories the current user's existing ordinary and redirected Fragcap roots before creating a new store. One unambiguous legacy store is transferred with SQLite consistency; conflicting or active state remains intact and receives an actionable diagnostic. Session evidence with unresolved cleanup obligations remains discoverable and recoverable at its original location unless an exact relocation can preserve its authority.

**Independent check**: Synthetic legacy databases, active journals, retained bundles, and conflicting roots exercise migration, refusal, and repeated startup without missing or overwritten records.

### User Story 3: Report the actual destination and cleanup scope (P2)

Doctor and terminal session reports distinguish the selected path from the path that backs actual file operations, including missing paths, redirection, and access failure. Recovery, retention, and fresh-start use the same current-user inventory and never broaden cleanup to another user's or host application's files.

**Independent check**: Read-only Doctor output and a controlled session report agree with independently opened-file destination evidence; fresh-start preview and execution select the same exact product roots.

## Edge Cases

- A packaged host redirects legacy AppData reads and writes while the ordinary process sees another root.
- Both ordinary and one or more package-private legacy roots contain different databases or sessions.
- SQLite is in WAL mode or has an active transaction during transition.
- An old bundle contains an unresolved route owner or path-bound recovery reference.
- A storage root is absent, inaccessible, redirected, a file instead of a directory, or reached through a junction.
- The initiating user elevates through a supported same-user path; a different administrator account must not silently take ownership of that user's store.
- Folder names contain non-ASCII characters or the profile is redirected to another volume.
- Explicit database, profile, session, and bundle overrides retain their current precedence and do not trigger default migration.
- Help, version, and read-only Doctor do not create a store.

## Functional Requirements

- **FR-001**: The default persistent root for Windows MUST be the initiating user's `%USERPROFILE%\.fragcap`, outside the AppData virtualization area, with separate non-overlapping `roaming` and `local` subdirectories and documented assignments for catalog, local database, profiles, sessions, registry, and local capture data.
- **FR-002**: Ordinary and supported packaged-host launches for one user MUST resolve the same root and observe the same persistent state; another account, SYSTEM, or another session MUST NOT be substituted silently.
- **FR-003**: Default path resolution MUST be centralized across target bootstrap and commands, Capture, Deep Capture, calibration, Doctor, recovery, retention, and fresh-start. Existing explicit path overrides MUST keep their precedence and semantics.
- **FR-004**: Before the first default write, Fragcap MUST inventory exact relevant current-user ordinary and redirected legacy roots and MUST NOT initialize a replacement database over unexamined populated state.
- **FR-005**: A single unambiguous legacy database MUST transfer through a SQLite-consistent operation. Active transactions, conflicting stores, and transfer failures MUST retain original state and report a specific outcome without silent fallback.
- **FR-006**: Retained session bundles, registries, manifests, journals, and unresolved cleanup obligations MUST remain discoverable and recoverable. Moving one requires exact path-bound references and recipient access to remain valid; otherwise Fragcap MUST retain and index the old location.
- **FR-007**: Doctor human and JSON output MUST distinguish selected location, selection source, existence, verified effective destination, redirection, and inability to verify. It MUST include both stores, session root, and registry.
- **FR-008**: Session success and failure reports MUST name the effective retained bundle destination. Path-bound recovery records MUST agree with file operations.
- **FR-009**: Retention and fresh-start preview and execution MUST use the same exact current-user root inventory as normal commands, preserving explicit custom paths, other users, and unrelated package data. Confirmed fresh-start MUST record durable suppression of later automatic import from retained legacy sources before removing the new data roots, and preview MUST disclose the marker path.
- **FR-010**: Default initialization MUST remain lazy. Help and version MUST not create state, and read-only Doctor MUST not create or migrate state.
- **FR-011**: Missing profile identity, denied access, unresolved destination, and conflicting state MUST have distinct actionable diagnostics. No alternative persistent root may be silently selected.
- **FR-012**: Controlled Windows checks MUST include independent-process destination readback under one account with altered AppData variables, legacy conflict and journal cases, explicit overrides, recovery, and cleanup. Actual packaged-host and elevation behavior MUST be recorded as post-release field verification.
- **FR-013**: The master specification, default-path and installed-test guidance, Doctor schema, package certification, and changelog MUST describe the delivered storage contract and compatibility behavior.

## Key Entities

- **Storage identity**: The initiating user's account and profile location used for all default data operations.
- **Selected root**: The configured default or explicit path before a file operation.
- **Effective destination**: The independently verified path backing a file or directory handle, or a named unavailable state.
- **Legacy root**: An exact current-user ordinary or package-private Fragcap root containing pre-S192 state.
- **Transition record**: Durable disposition of each legacy database and session root, with conflict, transfer, retention, and recovery status.

## Success Criteria

- **SC-001**: Independent controlled processes under one account read the same synthetic target identity from one opened-file destination despite different AppData variables. Actual packaged-host readback is verified after release and any defect returns as a GitHub issue.
- **SC-002**: Every synthetic legacy state case either transfers without SQLite or session loss or retains the original with a specific conflict or failure outcome; repeated startup is idempotent.
- **SC-003**: Doctor and session reports distinguish selected and effective paths in every required case, and independent-process readback confirms actual bytes at the reported destination.
- **SC-004**: Fresh-start and retention select only the exact current-user product inventory, with matching preview and execution.
- **SC-005**: Applicable repository checks and hosted CI pass. Later installed-product field failures become new GitHub issues after release.

## Clarifications (2026-11-04)

- The owner selected `%USERPROFILE%\.fragcap` as the new shared default root. The path is outside AppData to avoid arbitrary host-package write virtualization, and existing AppData data must be preserved through a careful transition.
- An explicit override remains authoritative and does not silently migrate default state into or out of a custom location.
- An ambiguous or actively modified legacy source is preserved with a visible diagnostic rather than guessed or overwritten.

## Scope

- #556 owns the default-storage correction, transition, effective-path reporting, and exact cleanup scope. It does not change proxy TLS diagnosis or game-only inspection semantics.
- Source and controlled package checks use synthetic identities and data. Private installed-product observation occurs after release and is not an implementation completion condition.
