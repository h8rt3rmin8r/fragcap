# Security Requirements Checklist: Durable Guided Calibration Resume

**Purpose**: Test the specification's authority, recovery, secret, and truth claims.

**Created**: 2026-09-11

**Feature**: [spec.md](../spec.md)

## Authority Boundaries

- [x] Is resume selected explicitly rather than inferred from a target?
- [x] Is each effectful attempt separately planned and confirmed?
- [x] Are prior responses and plans forbidden from the checkpoint?
- [x] Does target drift stop before any new plan or effect?
- [x] Are concurrent writers prevented from silently overwriting progress?

## Secret and Effect Safety

- [x] Does the checkpoint exclude capabilities, credentials, key material, trust
  state, proxy endpoints, and target secrets?
- [x] Is an in-flight marker evidence of interruption rather than reusable authority?
- [x] Does existing lifecycle recovery remain authoritative for unfinished effects?
- [x] Do explicit pauses perform no capture, proxy, trust, launch, cleanup, or process
  control?

## Truth and Auditability

- [x] Are workflow progress, operator intent, observations, and compatibility facts
  kept distinct?
- [x] Are every state, reason, protocol token, version, and revision validated?
- [x] Are missing evidence and warm state classified conservatively?
- [x] Are human and structured outputs required to project the same state?
- [x] Are crash, corruption, migration, concurrency, and drift tests mandatory?
