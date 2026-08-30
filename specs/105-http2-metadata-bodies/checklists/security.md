# Security and Protocol Requirements Checklist: HTTP/2, Metadata, and Streaming Bodies

**Purpose**: Review the completeness, clarity, consistency, and measurability of S105 protocol, security, bounded-resource, and loss requirements before implementation

**Created**: 2026-08-30

**Feature**: [spec.md](../spec.md)

**Audience**: Pull-request reviewers

## Authorization and Sensitive Data

- [x] CHK001 Are payload-retention requirements explicitly conditional on operator-authorized scope? [Completeness, Spec §FR-011, §FR-016]
- [x] CHK002 Are metadata-only omissions distinguished from accidental loss and failed retention? [Clarity, Spec §FR-016]
- [x] CHK003 Are sensitive header and body values excluded from human logs and diagnostics while remaining available only in classified artifacts? [Completeness, Spec §FR-010]
- [x] CHK004 Does the scope prohibit system-wide interception, target-process access, target key extraction, and fabricated protocol support? [Consistency, Spec §Assumptions, §FR-028]

## HTTP/2 Fidelity

- [x] CHK005 Are connection and stream identities defined separately for every multiplexed observation? [Clarity, Spec §FR-002]
- [x] CHK006 Are stream reset, refusal, cancellation, completion, GOAWAY, protocol error, and connection shutdown specified as distinct outcomes? [Completeness, Spec §FR-004]
- [x] CHK007 Are unrelated-stream progress requirements stated for backpressure and per-stream failure? [Coverage, Spec §FR-005]
- [x] CHK008 Is server push explicitly refused rather than silently discarded or claimed as supported? [Consistency, Spec §FR-006]
- [x] CHK009 Are out-of-order completion, overlapping streams, trailers, and connection-level failure scenarios all covered? [Coverage, Spec §User Story 1, §Edge Cases]

## Metadata Authority

- [x] CHK010 Are ordered raw names, raw values, duplicates, empty values, trailers, informational responses, protocol versions, and reason details all named? [Completeness, Spec §FR-007]
- [x] CHK011 Are HTTP/2 pseudo-headers protected from fabricated HTTP/1.1 casing, ordering, lines, and reason phrases? [Consistency, Spec §FR-008]
- [x] CHK012 Are query, cookie, and convenience projections required to trace to retained raw observations and expose decode uncertainty? [Traceability, Spec §FR-009]
- [x] CHK013 Are binary-safe and repeated metadata cases measurable for both supported HTTP versions? [Acceptance Criteria, Spec §SC-003]

## Body Authority and Bounds

- [x] CHK014 Are raw body bytes identified as the authority over every decoded convenience? [Consistency, Spec §FR-012]
- [x] CHK015 Are transfer decoding and content decoding separate transformations with explicit provenance and terminal outcomes? [Clarity, Spec §FR-012]
- [x] CHK016 Are fixed-length, chunked, connection-delimited, indefinite, partial, malformed, cancelled, and rejected-before-body cases addressed? [Coverage, Spec §FR-013, §Edge Cases]
- [x] CHK017 Are gzip, deflate, Brotli, unsupported encoding, malformed content, truncated content, and expansion-limit outcomes all specified? [Completeness, Spec §FR-014]
- [x] CHK018 Are message, session, disk, queue, expansion, idle, and shutdown bounds required to be finite and testable? [Measurability, Spec §FR-003, §FR-015]
- [x] CHK019 Is forwarding integrity required to survive observation saturation, storage failure, and decoding failure? [Reliability, Spec §FR-018]

## Application Stream and Loss Truth

- [x] CHK020 Is the application stream defined as live, append-only, versioned, and crash-readable rather than final-only? [Clarity, Spec §FR-022]
- [x] CHK021 Are all in-scope record families and deferred-family non-export reasons defined? [Completeness, Spec §FR-023]
- [x] CHK022 Are correlation, scope, protocol, truncation, timing, and loss anchors required without inventing unavailable values? [Consistency, Spec §FR-024]
- [x] CHK023 Are schema version, deterministic ordering, prefix framing, product-version distinction, and terminal reconciliation requirements explicit? [Completeness, Spec §FR-025]
- [x] CHK024 Is writer retirement required to preserve forwarding and already written evidence while preventing a complete claim? [Recovery, Spec §FR-026]
- [x] CHK025 Does every accepted protocol object reconcile to retained, omitted, truncated, failed, or dropped evidence with no unnamed remainder? [Measurability, Spec §FR-017, §SC-005]
- [x] CHK026 Are orderly completion and forced interruption both measurable through trailer presence and parseable-prefix rules? [Acceptance Criteria, Spec §SC-009]

## Dependencies and Completion Boundary

- [x] CHK027 Is compatibility with S104 HTTP/1.1 behavior an explicit regression requirement? [Dependency, Spec §FR-021]
- [x] CHK028 Does the done gate require every acceptance criterion from #294, #296, #297, and #301 without implying closure of deferred issues? [Traceability, Spec §Scope and Traceability]
- [x] CHK029 Are WebSocket frames, Server-Sent Events, gRPC semantics, HAR, client certificates, generic transports, and feature-completion claims explicitly excluded? [Scope, Spec §FR-028, §Excluded]
- [x] CHK030 Are controlled verification requirements independent of Internet access, games, elevation, trust mutation, and capture drivers? [Clarity, Spec §FR-020, §SC-007]

## Notes

- All requirements-quality items passed during checklist generation.
- The #301 scope clarification is load-bearing because it gives #297 body segments one bounded durable authority rather than a temporary artifact contract.
