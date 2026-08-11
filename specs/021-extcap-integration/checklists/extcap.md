# Extcap Checklist: Extcap analyzer integration

**Purpose**: Validate that the requirements for the four-invocation extcap
contract, the FIFO stream, the dialog-to-capture option mapping, the doctor
install report, the passive-observation guardrails, conservation accounting, and
the terminology are complete, clear, consistent, and measurable before planning.

**Created**: 2026-08-11

**Feature**: [spec.md](../spec.md)

## The Extcap Contract

- [x] CHK001 Are all four invocations specified with an objective output
  criterion (accepted by an extcap control-grammar check), not only named?
  [Measurability, Spec FR-001, FR-002, FR-003, FR-004, SC-001]
- [x] CHK002 Is the extcap interface model pinned (a single `fragcap` interface
  keyed by profile and role options, not one per host adapter) rather than left
  ambiguous? [Clarity, Spec Clarifications, Key Entities]
- [x] CHK003 Is the declared link type specified (Ethernet default, per-packet
  link types carried by the stream's Interface Description Blocks)? [Clarity,
  Spec FR-002, Key Entities]
- [x] CHK004 Is the standard protocol surface (the version query and the
  `--extcap-interface` selector) required to be accepted rather than rejected as
  unknown? [Completeness, Spec FR-007, Edge Cases]

## The FIFO Stream and Compatibility

- [x] CHK005 Is the FIFO stream required to be a single valid pcapng an
  unmodified analyzer reads in full (constitution P-5), by an objective parser
  criterion? [Measurability, Spec FR-004, SC-002]
- [x] CHK006 Is the extcap stream required to be the same bytes the file sink
  produces (same headers, same attribution comments), giving a record-comparable
  regression anchor to a plain file capture? [Consistency, Spec FR-005]
- [x] CHK007 Is the FIFO sink required to reuse the existing `SinkFactory` seam
  and add no new format code, so the transport is orthogonal to the format?
  [Clarity, Spec Clarifications, Key Entities, Assumptions]
- [x] CHK008 Is a FIFO that cannot be opened required to fail before a started
  capture is reported, naming the path? [Edge case, Spec Edge Cases]
- [x] CHK009 Is an analyzer that closes the FIFO mid-capture required to end the
  capture as a clean stop with conservation preserved, not a silent loss? [Edge
  case, Spec Edge Cases, FR-011]

## Dialog Options Select the Capture

- [x] CHK010 Are the configurable options pinned to exactly four (profile, roles,
  direction, loopback), matching specification 14.5, rather than an open set?
  [Completeness, Spec FR-003, SC-003]
- [x] CHK011 Is each option required to be applied through the same resolution
  the `run` command uses, so the dialog and the flags select capture
  identically? [Consistency, Spec FR-006, SC-003]
- [x] CHK012 Is a profile resolution failure at capture required to be a
  configuration error rather than a started-but-empty capture? [Edge case, Spec
  User Story 2, Acceptance]

## Configuration and Refusals

- [x] CHK013 Is `--capture` without `--fifo`, and a declaration invocation
  without its required `--extcap-interface`, required to be a usage error (exit
  2) naming the missing argument, before any capture starts? [Completeness, Spec
  FR-008, SC-005]
- [x] CHK014 Is an unknown `--extcap-interface` required to be a usage error
  rather than an empty, inert declaration? [Edge case, Spec Edge Cases, SC-005]

## Passive Observation and Accounting

- [x] CHK015 Is extcap required to introduce no new capture or attribution
  technique (the capture is the existing pipeline with a FIFO sink), holding P-1,
  P-3, and P-9? [Consistency, Spec FR-010, Clarifications]
- [x] CHK016 Is the pipeline conservation invariant required to hold on the
  extcap path exactly as for a file capture, with the FIFO sink advancing no
  uncounted discard? [Measurability, Spec FR-011, SC-006]
- [x] CHK017 Is the doctor probe required to read the extcap directory read-only
  and install, download, or copy nothing (the Licensing rule, P-1)? [Consistency,
  Spec FR-009, User Story 3]

## Doctor Install Report

- [x] CHK018 Is `doctor` required to report both the installed and not-installed
  states and to name the extcap directory in each? [Completeness, Spec FR-009,
  SC-004]

## Terminology

- [x] CHK019 Are glossary entries for extcap, DLT and link type, and named pipe
  and FIFO required in the same change (constitution P-6)? [Consistency, Spec
  FR-012]

## Notes

- Every item resolves against the current spec; none is outstanding. The
  checklist keeps the analyze gate anchored to the extcap-specific risks
  (contract conformance, analyzer readability, dialog-to-capture fidelity,
  passive-observation guardrails, conservation, and terminology) rather than only
  the generic requirements-quality set.
