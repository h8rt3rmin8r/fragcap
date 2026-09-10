# Security Checklist: Guided Target Discovery and Registration

**Purpose**: Validate the requirements that guard durable target registration and preserve Deep Capture authorization boundaries.
**Created**: 2026-09-10
**Feature**: [spec.md](../spec.md)

## Authority and Consent

- [x] CHK001 Is the one-candidate registration authority fully bound before input is requested? [Security, Spec §FR-008-010]
- [x] CHK002 Does every non-affirmative interactive response and every non-exact structured response default to no write? [Security, Spec §FR-011-012]
- [x] CHK003 Is registration confirmation explicitly prevented from authorizing any session, trust, launch, proxy, capture, artifact, or fact effect? [Security, Spec §FR-013]
- [x] CHK004 Must discovery authority be reacquired and compared after confirmation? [TOCTOU, Spec §FR-014]

## Identity and Persistence

- [x] CHK005 Is the plan identifier domain-separated, deterministic, versioned, and sensitive to every bound field? [Integrity, Spec §FR-009]
- [x] CHK006 Is constant-time exact-line comparison required for structured authorization? [Integrity, Spec §FR-012]
- [x] CHK007 Is persistence restricted to the shared idempotent single-candidate operation? [Least Authority, Spec §FR-015-016]
- [x] CHK008 Are ambiguity and identity conflicts required to fail closed without a guessed candidate? [Integrity, Spec §FR-006, FR-016]

## Safety Boundaries

- [x] CHK009 Are no-effect guarantees stated for ambiguity, decline, invalid input, output failure, drift, and unsupported topology? [Safety, Spec §FR-006, FR-010-018]
- [x] CHK010 Are all constitution P-1 prohibited capabilities outside scope? [Safety, Spec §FR-022]
- [x] CHK011 Are discovery observations prevented from becoming launch authority or compatibility evidence? [Honesty, Spec §FR-018, Assumptions]
- [x] CHK012 Is the existing one-session limit preserved after registration? [Bounded Effects, Spec §User Story 3, FR-022]

## Notes

- Completed during the security-focused specification review. The plan must retain these as implementation gates.
