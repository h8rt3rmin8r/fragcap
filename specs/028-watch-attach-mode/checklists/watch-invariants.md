# Correctness Checklist: Watch / Attach Mode

**Purpose**: Guard the launch-agnostic, honesty, and no-silent-loss invariants of
watch mode. These map to required tests and the analyze gate.
**Created**: 2026-08-12
**Feature**: [spec.md](../spec.md)

## Launch-agnostic capture

- [ ] CHK001 `watch` captures a target that starts from an arbitrary parent with
  no `steam://` and no authored profile (FR-002, SC-001).
- [ ] CHK002 The identity is exe plus an optional path anchor; the path anchor
  disambiguates two processes sharing an exe name (FR-001, FR-004, SC-006).
- [ ] CHK003 A watch identity carries no `descends_from`; ancestry is not part of
  it (FR-004).

## Attach vs wait (both are runtime observation)

- [ ] CHK004 A process already present at arm (in the startup snapshot, no later
  start event) is acquired at arm, not left waiting (FR-003, SC-002).
- [ ] CHK005 A process that starts after arm is acquired by wait-for-start; attach
  and wait compose in one run (US2 scenario 3).
- [ ] CHK006 The attach-to-running decision is produced by the S027
  ObservationProvider over the snapshot; the session is the single acquisition
  authority (no competing acquisition path).

## Fidelity honesty (P-9, two axes)

- [ ] CHK007 The synthesized watch identity is `authored`, never `observed`; a
  profile declaring `observed` is refused (S027), so the synthesized document
  does not claim it (FR-006).
- [ ] CHK008 The ObservationProvider's answer carries the `observed` tier and is
  not conflated with the definition's authored fidelity (FR-006, US2 scenario 2).

## No silent loss (P-4)

- [ ] CHK009 A watch that never acquires ends with
  `StopReason::AcquisitionTimeout`, surfaces the watch-time discard accounting,
  and exits a failure (FR-005, SC-003).
- [ ] CHK010 An operator interrupt during the watch is a clean stop, exit zero
  (FR-005, SC-003).
- [ ] CHK011 A watch that captured nothing states it acquired no target rather
  than an empty success (US3 scenario 3).

## Passivity and construction (P-1)

- [ ] CHK012 Attach-to-running reads only the image name and path from the
  startup snapshot (toolhelp, no handle); `cargo xtask lint` stays green (FR-006).
- [ ] CHK013 An identity with no predicate is refused at construction; a
  non-compiling path regex is refused with the profile's diagnostic (FR-008,
  SC-005).

## Reuse and output

- [ ] CHK014 `watch` reuses the shared capture engine; output is byte-identical to
  an equivalent single-stage profile capture (FR-007, SC-004).
- [ ] CHK015 The spec (7.1, 10.5) names watch mode as the default launch-agnostic
  path and the glossary gains a `watch mode` entry referencing `launcher_mediated`
  (FR-009, P-6); docs linter green.

## Notes

- CHK004/CHK006 are the subtle ones: the snapshot exists (watcher takes it) but
  the capture path never applied it before this slice; the offline `ProcessScript`
  models it via `with_snapshot`.
