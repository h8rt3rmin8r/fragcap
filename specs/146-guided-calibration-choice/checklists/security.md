# Security Requirements Checklist: Explicit Guided Calibration Choice

**Purpose**: Test S146's selection, persistence, authorization, and no-effect refusal boundaries.

**Created**: 2026-09-11

**Feature**: [spec.md](../spec.md)

## Selection Authority

- [x] Is every ambiguous selection explicit and content-bound rather than positional?
- [x] Does canonical identity cover every candidate field that can affect registration or Steam client authoring?
- [x] Are unknown, duplicate, stale, malformed, and unused choices refused?
- [x] Is a supplied choice consumed at exactly one permitted boundary?

## Workflow and Effect Authority

- [x] Are exact-case values immutable after workflow creation?
- [x] Does resume forbid candidate and case mutation?
- [x] Is a launch override validated against current inferred topology and process truth?
- [x] Do unsupported routing values stop before plan or effect?
- [x] Does every attempt still require current recovery readiness, a fresh plan, and exact confirmation?

## Secrets and Truth

- [x] Does the workflow continue to exclude responses, credentials, capabilities, trust state, and effect obligations?
- [x] Are selection, authorization, compatibility evidence, and topology authoring kept distinct?
- [x] Are human and JSON outputs required to expose the same selected dimensions and provenance?
- [x] Are all refusal tests required to demonstrate zero target, workflow, bundle, launch, trust, and fact effects?
