# Launch Safety Checklist: Managed Direct-Executable Launch

**Purpose**: Verify the safety and truth requirements that govern child creation

**Created**: 2026-08-30

**Feature**: [spec.md](../spec.md)

## Preparation Boundary

- [x] Exact stored target remains authoritative
- [x] One direct client and one stored or safely derived install root are required
- [x] Missing, ambiguous, stale, and escaping paths have named refusals
- [x] Prepared path, working directory, and arguments cannot be re-resolved after effects

## Execution Boundary

- [x] Direct launch has no raw shell command representation
- [x] Argument boundaries and values are explicit
- [x] Environment changes are child-only
- [x] Warm direct launch is refused for Deep Capture
- [x] Steam remains an independent unchanged variant

## Observation and Cleanup

- [x] Existing watcher and attribution paths remain authoritative
- [x] No target inspection or memory access is introduced
- [x] Spawn failure after session effects requires non-complete terminal truth
- [x] Every acquired resource receives a bounded cleanup attempt
