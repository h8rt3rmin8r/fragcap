# Causal Coverage Checklist: Causal Launch-Chain Coverage

**Purpose**: Review the completeness and clarity of packet, process, scope, and recovery requirements

**Created**: 2026-11-06

**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [x] CHK001 Are launch, client observation, and route-release packet windows separately defined? [Completeness, Spec FR-003/FR-004]
- [x] CHK002 Are packet and application inspection scopes defined independently for default, explicit game-only, and fallback attempts? [Completeness, Spec FR-001/FR-002/FR-009]
- [x] CHK003 Are undeclared descendants and unrelated same-name processes both addressed? [Coverage, Spec FR-003/FR-010]
- [x] CHK004 Are all ownership proof components specified for packet eligibility and process control? [Completeness, Spec FR-005/FR-011]
- [x] CHK005 Are source, capacity, identity, and intentional-window failures given distinct reportable outcomes? [Coverage, Spec FR-004/FR-005/FR-006]

## Requirement Clarity and Consistency

- [x] CHK006 Is a process generation distinguished from a PID, executable name, and profile role? [Clarity, Spec Key Entities]
- [x] CHK007 Is the relation between client observation duration and packet collection lifetime unambiguous? [Clarity, Spec FR-004]
- [x] CHK008 Is each gate-rejection reason defined as an actual packet decision rather than a retrospective guess? [Consistency, Spec FR-006]
- [x] CHK009 Do the plan and recovery requirements agree that process-control authorization precedes every session effect? [Consistency, Spec FR-002/FR-011]
- [x] CHK010 Do compatibility and resume requirements preserve historical facts while preventing old game-only qualification? [Consistency, Spec FR-008]

## Scenario and Edge Coverage

- [x] CHK011 Are delayed process proof, packet ordering, and finite staging exhaustion addressed? [Edge Case, Spec FR-005]
- [x] CHK012 Are late children, same-image siblings, partial exits, reuse, and missing proof covered after successful client acquisition? [Coverage, Spec FR-010/FR-012]
- [x] CHK013 Are route and listener outcomes distinct from process exit and packet-file completion? [Coverage, Spec FR-007/FR-012]
- [x] CHK014 Are direct sockets and TLS refusal boundaries excluded from false success claims without narrowing the requested packet scope? [Consistency, Spec Scope/FR-007]
- [x] CHK015 Are ordinary Capture and warm or unowned processes outside new shutdown authority? [Coverage, Spec FR-011/FR-015]

## Acceptance Quality

- [x] CHK016 Can independent controlled cases distinguish the default change from the packet-gate repair? [Measurability, Spec User Stories 1/2]
- [x] CHK017 Can controlled cases measure zero unrelated admission and zero unowned control actions? [Measurability, Spec SC-001/SC-003]
- [x] CHK018 Are issue closure and exact-head verification defined without requiring a future private-title run? [Measurability, Spec Done when/SC-005]
