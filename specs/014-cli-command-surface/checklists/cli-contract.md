# Requirements Quality Checklist: CLI Command Surface

**Purpose**: Validate that the command-line behavioral contract requirements are
complete, unambiguous, consistent, and testable before implementation, faithful
to specification section 17 and section 26.3.
**Created**: 2026-08-10
**Feature**: [spec.md](../spec.md)

## Command surface and dispatch

- [x] CHK001 Is the complete set of commands the tool exposes enumerated, and is
  each one classified as implemented or stubbed? [Completeness, Spec §FR-001,
  §FR-002]
- [x] CHK002 Is the behavior of a stubbed command fully specified: the message,
  that it names the delivering slice, and its exit code? [Clarity, Spec §FR-002]
- [x] CHK003 Are the requirements for top-level help, per-command help, and
  version output specified? [Completeness, Spec §FR-003]

## Exit-code contract

- [x] CHK004 Is every outcome across all four working commands mapped to exactly
  one of exit 0/1/2, with no outcome unmapped or double-mapped? [Coverage, Spec
  §FR-004]
- [x] CHK005 Is "operator interrupt during capture yields exit 0" stated
  unambiguously and distinguished from an aborted or failed run? [Clarity, Spec
  §FR-005]
- [x] CHK006 Is the exit code for an unrecoverable sink failure specified and
  distinguished from a usage error? [Clarity, Spec §FR-005a]
- [x] CHK007 Are the boundaries between class 1 (expected failure) and class 2
  (usage/configuration error) defined by example for each ambiguous case (bad
  reference vs unresolved reference, unsupported mode, driver absent)? [Ambiguity,
  Spec §FR-004, Edge Cases]

## Output streams and verbosity

- [x] CHK008 Is the routing of progress, diagnostics, capture data, and structured
  events to specific streams specified for every command? [Completeness, Spec
  §FR-019]
- [x] CHK009 Is the rule "a sink writing to standard output forces all diagnostics
  to standard error" stated as a testable requirement? [Clarity, Spec §FR-019]
- [x] CHK010 Are quiet and silent precisely defined by what each suppresses and
  retains, with "errors are never suppressed" holding under both? [Clarity,
  Consistency, Spec §FR-020]

## Structured event stream

- [x] CHK011 Is the set of lifecycle events, and the fields each carries,
  enumerated? [Completeness, Spec §FR-018]
- [x] CHK012 Is it specified that structured events go to standard error while
  capture data goes to sinks, so the two never share a stream? [Consistency, Spec
  §FR-018, §FR-019]
- [x] CHK013 Is doctor's structured output specified to carry the same content as
  its human report? [Consistency, Spec §FR-018, US1 Scenario 5]

## Doctor readiness and npcap obligation

- [x] CHK014 Are all readiness sections and their individual checks enumerated?
  [Completeness, Spec §FR-013]
- [x] CHK015 Is detect-never-install stated as an absolute obligation, with no
  path that installs, downloads, or modifies the driver? [Clarity, Spec §FR-014]
- [x] CHK016 Are the two non-default npcap options specified as separate checks,
  each naming its exact remediation when absent? [Completeness, Spec §FR-014, US1
  Scenario 2]
- [x] CHK017 Is the exact rule for when a missing tracing session is blocking
  versus a skip specified, rather than left to judgement? [Ambiguity, Spec §FR-015]
- [x] CHK018 Is the exit-0-versus-1 decision for doctor tied to a defined
  classification (blocking fail present or not), and is "optional-integration
  warnings never block" explicit? [Clarity, Spec §FR-015, §FR-016]
- [x] CHK019 Is "every failing check names a specific remediation" measurable
  (each fail has a non-empty remediation)? [Measurability, Spec §FR-016]

## Profile management

- [x] CHK020 Is "reports every diagnostic in one pass" specified as all-at-once,
  not first-failure, and tied to exit 2? [Clarity, Spec §FR-017, US3 Scenario 2]
- [x] CHK021 Are the outputs of list and show specified, including which source
  supplied a resolved reference and the not-found behavior? [Completeness, Spec
  §FR-017, US3 Scenario 4]

## Capture configuration and bounds

- [x] CHK022 Is the override rule "command line wins over profile capture
  defaults" specified, including that an option absent from both stays absent?
  [Clarity, Spec §FR-007, Key Entities]
- [x] CHK023 Is the size-literal grammar (units, base, zero-rejection) specified
  unambiguously for both `--max-bytes` and the size form of `--ring`? [Clarity,
  Spec §FR-011a]
- [x] CHK024 Is the scope of `--roles` and `--direction` for this slice
  (scoping/recording versus full filtering) stated, so a reviewer knows what is
  and is not enforced? [Ambiguity, Spec §FR-011b]
- [x] CHK025 Are the not-yet-supported modes (stream, ring), transport sinks
  (pipe, tcp), and managed launch specified as parser-accepted but rejected with a
  named slice, rather than silently ignored? [Coverage, Spec §FR-010, §FR-011,
  Edge Cases]

## Loss accounting

- [x] CHK026 Is the completion summary required to surface each existing discard
  counter by name, and prohibited from inventing new counters or fabricating
  counts? [Completeness, Spec §FR-021]
- [x] CHK027 Is the conservation property (nothing observed goes unaccounted)
  stated as a measurable success criterion? [Measurability, Spec §SC-004]

## Testability and scope boundary

- [x] CHK028 Is it specified that every capture behavior is demonstrable with no
  driver, no elevation, and no game, so the slice is verifiable in continuous
  integration? [Coverage, Spec §SC-007]
- [x] CHK029 Are the live-only behaviors (real capture, socket table, tracing)
  explicitly out of the continuous-integration path and marked as developer-machine
  only? [Consistency, Assumptions]
