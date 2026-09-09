# Requirements Quality Checklist: Fresh-Start Security and Ownership

**Purpose**: Challenge whether S137 defines sufficient destructive-action authority, containment, and truthful recovery behavior.

**Created**: 2026-09-09

**Feature**: `specs/137-fresh-start-uninstall/spec.md`

## Consent and Lifecycle

- [x] CHK001 Is preserve-by-default behavior required for interactive and silent uninstall? [Completeness, Spec FR-001]
- [x] CHK002 Are repair, upgrade, rollback, and maintenance excluded even with stale properties? [Coverage, Spec FR-002]
- [x] CHK003 Is the interactive choice unchecked, irreversible, scoped, and inventory-bearing? [Clarity, Spec FR-003]
- [x] CHK004 Does silent cleanup require two exact opt-in values? [Clarity, Spec FR-004]
- [x] CHK005 Is initiating-user identity separated from elevated execution identity? [Security, Spec FR-005]

## Ownership and Containment

- [x] CHK006 Is one product-owned contract required across every invocation path? [Consistency, Spec FR-006]
- [x] CHK007 Are all current canonical categories included? [Completeness, Spec FR-007]
- [x] CHK008 Are third-party, custom, exported, and independently managed paths excluded? [Boundary, Spec FR-008]
- [x] CHK009 Are weak ownership signals prohibited? [Security, Spec FR-009]
- [x] CHK010 Are roots, aliases, overlaps, links, junctions, mount points, and reparses refused? [Security, Spec FR-010]

## Recovery and Failure Truth

- [x] CHK011 Must Doctor authority run before Deep Capture records are removed? [Dependency, Spec FR-011]
- [x] CHK012 Do unsafe recovery states retain evidence and prevent complete status? [Security, Spec FR-012]
- [x] CHK013 Is the report versioned, bounded, exhaustive, actionable, and public-safe? [Completeness, Spec FR-013]
- [x] CHK014 Can independent owned items continue after a failure without hiding retained paths? [Clarity, Spec FR-014]
- [x] CHK015 Is clean-account success defined by absence of every canonical state class? [Measurability, Spec FR-015]

## Administrative Scope

- [x] CHK016 Does all-users scope require a complete exact preview and bound identifier? [Security, Spec FR-016]
- [x] CHK017 Does changed inventory or insufficient authority fail before deletion? [Security, Spec FR-017]
- [x] CHK018 Does the MSI avoid a misleading static all-users confirmation? [Truthfulness, Spec FR-018]

## Verification and Boundaries

- [x] CHK019 Are every required Windows certification scenario and finite execution constraint named? [Coverage, Spec FR-019]
- [x] CHK020 Are tests prohibited from touching real profile data? [Safety, Spec FR-020]
- [x] CHK021 Are operator documentation and partial-recovery guidance required? [Completeness, Spec FR-021]
- [x] CHK022 Are pinned-artifact decisions and architecture synchronization required? [Governance, Spec FR-022]
- [x] CHK023 Are broad deletion, hidden effects, dependency growth, and completion claims excluded? [Boundary, Spec FR-023]

## Traceability Summary

- [x] All 23 functional requirements have at least one checklist challenge.
- [x] Every issue #377 destructive-action boundary is represented.
- [x] Requirement traceability is 100 percent.
