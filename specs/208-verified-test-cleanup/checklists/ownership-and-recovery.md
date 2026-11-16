# Test Ownership and Recovery Requirements Checklist

**Purpose**: Review the written cleanup and residue requirements before design

**Created**: 2026-11-16

**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [x] CHK001 Are the artifact classes and effective roots in scope specified? [Completeness, Spec FR-001, FR-005]
- [x] CHK002 Are both normal and abnormal completion duties specified? [Completeness, Spec FR-002, FR-003]
- [x] CHK003 Are product-managed bundles explicitly distinguished from test scratch? [Completeness, Spec FR-006]

## Requirement Clarity

- [x] CHK004 Is exact fixture ownership distinguished from a matching directory prefix? [Clarity, Spec Clarifications]
- [x] CHK005 Is the result of cleanup failure explicit rather than a silent best effort? [Clarity, Spec FR-002]
- [x] CHK006 Is the historic-directory handling boundary stated without implying permission to delete unproven material? [Clarity, Spec Scope]

## Scenario Coverage

- [x] CHK007 Are success, assertion failure, interruption, and concurrent run scenarios addressed? [Coverage, Spec User Stories 1 and 2]
- [x] CHK008 Are redirected roots, open handles, and same-prefix unrelated paths addressed? [Coverage, Spec Edge Cases]
- [x] CHK009 Is the unresolved native-session recovery exception specified? [Coverage, Spec User Story 3]

## Acceptance Criteria Quality

- [x] CHK010 Can zero new residue after a successful controlled run be objectively measured? [Measurability, Spec SC-001]
- [x] CHK011 Can a forced cleanup failure be observed as a failing result with an exact residual path? [Measurability, Spec SC-002]
- [x] CHK012 Are the controlled outcomes within this slice, without a future private field run? [Consistency, Spec Scope, SC-004]

## Dependencies and Assumptions

- [x] CHK013 Are product retention and recovery kept outside test-scratch authority? [Assumption, Spec Assumptions]
- [x] CHK014 Does the audit require evidence of a leak before changing an unrelated producer? [Assumption, Spec FR-005]
