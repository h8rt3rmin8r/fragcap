# Streaming Transports Checklist: Transports and streaming sinks

**Purpose**: Validate that the requirements for file rotation, streaming
transports, per-consumer backpressure, analyzer compatibility, and platform
gating are complete, clear, consistent, and measurable before planning.
**Created**: 2026-08-10
**Feature**: [spec.md](../spec.md)

## Analyzer Compatibility

- [x] CHK001 Is the requirement that each consumer receives its own header
  preamble (Section Header Block plus every Interface Description Block) before
  any packet data stated unambiguously and independently of connection time?
  [Clarity, Spec §FR-007]
- [x] CHK002 Is "independently valid stream" defined by an objective criterion
  (accepted in full by an unmodified pcapng parser) rather than left as a
  qualitative goal? [Measurability, Spec §SC-001]
- [x] CHK003 Are the interfaces a mid-capture consumer sees required to match
  those every other consumer declares, and is that match specified rather than
  implied? [Consistency, Spec §FR-007]
- [x] CHK004 Is the requirement that a rotated segment opens on its own (begins
  with its own Section Header Block and Interface Description Blocks) stated for
  every segment, not only the first? [Completeness, Spec §FR-002, §SC-002]
- [x] CHK005 Are compatibility requirements written to bind both formats
  (pcapng and JSON Lines) across every transport, or do they silently assume
  pcapng only? [Coverage, Spec §FR-006]

## No-Silent-Loss Accounting

- [x] CHK006 Is the per-consumer drop counter defined as distinct from the
  pipeline's capture-wide `sink_dropped`, with the distinction stated rather
  than assumed? [Clarity, Spec §FR-010, Clarifications]
- [x] CHK007 Is it specified that a streaming sink returns success to the
  pipeline for every packet it accepts (including with zero consumers), so the
  conservation invariant is preserved? [Completeness, Spec §FR-010]
- [x] CHK008 Are consumer disconnect events required to be counted and/or
  logged with the consumer's identity and reason, and is "surfaced" given a
  concrete meaning (statistics, log, or event stream)? [Measurability, Spec
  §FR-011]
- [x] CHK009 Does the spec state where per-consumer drop and disconnect figures
  appear to the operator (run statistics, log, or both), so "reported" is not
  ambiguous? [Ambiguity, Spec §FR-010, §FR-011]
- [x] CHK010 Is the boundary between "counted per-consumer drop" and "sink
  retirement" defined, so backpressure can never be mistaken for a sink failure?
  [Consistency, Spec §FR-010, Clarifications]
- [x] CHK011 Is the packets-not-delivered-to-any-consumer case (idle transport)
  specified so it is not miscounted as capture loss? [Edge Case, Spec §Edge
  Cases]

## Consumer Isolation and Non-Blocking Capture

- [x] CHK012 Is "must not stall the capture" expressed as a testable property
  (capture throughput unaffected by a stalled consumer) rather than a slogan?
  [Measurability, Spec §FR-012, §SC-003]
- [x] CHK013 Is the independence of each consumer's bounded queue stated, so one
  full queue provably cannot affect another consumer or the file sink?
  [Clarity, Spec §FR-009]
- [x] CHK014 Is the file sink's immunity to a stalled network consumer stated as
  an explicit requirement, not merely an implication of isolation? [Completeness,
  Spec §FR-012, §US4]
- [x] CHK015 Are abrupt mid-stream disconnect and reconnect behaviors specified
  (detect closed connection, stop writing, continue serving others)? [Coverage,
  Spec §Edge Cases]

## Rotation Correctness

- [x] CHK016 Are both rotation triggers (size threshold, duration threshold)
  defined with a clear measurement origin (bytes since segment open, time since
  segment open)? [Clarity, Spec §FR-002]
- [x] CHK017 Is the no-loss/no-duplication/no-reorder property across segment
  joins stated as a verifiable equality against an un-rotated capture?
  [Measurability, Spec §SC-002]
- [x] CHK018 Is the segment-numbering scheme and its ordering guarantee
  specified, so "numbered segments" is unambiguous? [Ambiguity, Spec §FR-001]
- [x] CHK019 Is the degenerate case (rotation threshold smaller than a segment's
  mandatory header) addressed so no unreadable segment can be emitted? [Edge
  Case, Spec §Edge Cases]
- [x] CHK020 Is it specified that rotation occurs only at a clean section
  boundary, and is "clean section boundary" defined? [Clarity, Spec §FR-001]

## Platform Gating

- [x] CHK021 Is the named-pipe transport's Windows-only availability stated,
  along with the required behavior when it is requested on a non-Windows target
  (refused at configuration time, naming the limitation)? [Completeness, Spec
  §FR-003, §Edge Cases]
- [x] CHK022 Is the requirement that no platform-specific dependency or
  assumption leaks into the platform-neutral core stated as a binding
  constraint with a verifiable check (core builds for a backendless target)?
  [Measurability, Spec §FR-016, Clarifications]
- [x] CHK023 Is the per-target transport set (which transports exist on which
  platform) documented, so availability is not discovered only at runtime?
  [Coverage, Spec §Assumptions]
- [x] CHK024 Are the Unix domain socket's availability conditions specified
  (present where the platform supports it, parity semantics with the named
  pipe)? [Clarity, Spec §FR-005, §US5]

## Configuration and Scheme Wiring

- [x] CHK025 Is format resolution (inference from extension, otherwise explicit
  qualifier) specified for every scheme, including those with no inferable
  extension (pipe, tcp)? [Completeness, Spec §FR-006, §Edge Cases]
- [x] CHK026 Is every accepted sink scheme required to resolve to a working
  transport or be rejected before capture, with no scheme reaching capture as an
  unimplemented stub? [Consistency, Spec §FR-013, §FR-014, §SC-005]
- [x] CHK027 Is the duplicate-bind case (two sinks on the same pipe name or TCP
  address) specified to fail at startup rather than silently? [Edge Case, Spec
  §Edge Cases]
- [x] CHK028 Is `--mode stream` (and a valid no-file, streaming-only run)
  specified, so the mode this slice enables is unambiguous? [Completeness, Spec
  §FR-017, Clarifications]

## Notes

- Items are requirements-quality checks (do the requirements read well), not
  implementation tests. They are resolved by confirming the spec answers each
  question, amending the spec where it does not.
- The plan phase inherits the numeric defaults deferred here (queue depth,
  disconnect timeout, rotation thresholds); CHK016 and CHK008 confirm the spec
  frames them as configurable-with-defaults rather than leaving them undefined.
