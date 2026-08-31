# Protocol Security Requirements Checklist: Streaming Application Protocols

**Purpose**: Validate security, fidelity, boundedness, and failure requirements before implementation
**Created**: 2026-08-30
**Audience**: Pull request reviewers
**Depth**: Formal slice gate

## Requirement Completeness

- [x] CHK001 Are both authorized WebSocket handshake forms and their refusal boundaries specified? [Completeness, Spec FR-001]
- [x] CHK002 Are directional masking, reserved-bit, opcode, fragmentation, control-frame, length, close, and text validation requirements defined? [Completeness, Spec FR-004, FR-005]
- [x] CHK003 Are per-message compression negotiation, raw authority, decompression, and context-state requirements specified? [Completeness, Spec FR-006]
- [x] CHK004 Are bounded WebSocket assembly and retention outcomes defined independently from forwarding? [Completeness, Spec FR-007]
- [x] CHK005 Are SSE recognition, grammar, source linkage, malformed text, reconnect state, cancellation, and incomplete termination requirements specified? [Completeness, Spec FR-009 through FR-013]
- [x] CHK006 Are gRPC recognition, metadata, message envelope, compression, trailer, cancellation, and malformed framing requirements specified? [Completeness, Spec FR-014 through FR-018]
- [x] CHK007 Are metadata-only behavior and payload omission requirements defined uniformly for every protocol family? [Consistency, Spec FR-025]
- [x] CHK008 Are shutdown obligations defined for partial frames, messages, events, and calls? [Coverage, Spec FR-024]

## Requirement Clarity

- [x] CHK009 Is raw protocol authority distinguished unambiguously from every derived message or event? [Clarity, Spec FR-003, FR-011, FR-015]
- [x] CHK010 Is the difference between protocol refusal, observation truncation, queue loss, decode failure, and writer failure explicit? [Clarity, Spec FR-022]
- [x] CHK011 Are the fields required on WebSocket frame and message observations enumerated? [Clarity, Spec FR-002, FR-003]
- [x] CHK012 Are the fields required on SSE field and event observations enumerated? [Clarity, Spec FR-010]
- [x] CHK013 Are the fields required on gRPC call and message observations enumerated? [Clarity, Spec FR-014, FR-015]
- [x] CHK014 Is gRPC protobuf decoding explicitly excluded so framing evidence cannot be mistaken for application interpretation? [Clarity, Spec FR-016]
- [x] CHK015 Is HAR ownership limited clearly to the WebSocket handshake? [Clarity, Spec FR-008]

## Requirement Consistency

- [x] CHK016 Do all three protocols retain the S105 separation between forwarding bounds and evidence-retention bounds? [Consistency, Spec FR-021 through FR-023]
- [x] CHK017 Do all protocol records use the same available correlation anchors without fabricating unavailable process or flow identity? [Consistency, Spec FR-019]
- [x] CHK018 Do binary-safety requirements agree across frame, event, message, and application-record contracts? [Consistency, Spec FR-020]
- [x] CHK019 Do protocol-specific terminal states agree with the single-terminal-outcome lifecycle rule? [Consistency, Spec FR-024]
- [x] CHK020 Does the slice retain explicit Deep Capture consent, destination policy, TLS ownership, and cleanup boundaries? [Consistency, Assumptions]

## Scenario and Edge-Case Coverage

- [x] CHK021 Are HTTP/1.1 upgrade and HTTP/2 extended CONNECT success and failure scenarios both covered? [Coverage, User Story 1]
- [x] CHK022 Are fragmented and compressed WebSocket messages, interleaved control frames, and malformed peer behavior covered? [Coverage, User Story 1, Edge Cases]
- [x] CHK023 Are arbitrary SSE segmentation, all standard fields, line endings, malformed UTF-8, long-lived idle periods, and reconnect state covered? [Coverage, User Story 2, Edge Cases]
- [x] CHK024 Are unary and all three streaming gRPC patterns plus partial, oversized, compressed, cancelled, and malformed envelopes covered? [Coverage, User Story 3, Edge Cases]
- [x] CHK025 Are queue saturation, writer retirement, session interruption, and forced cleanup covered for every long-lived protocol? [Coverage, FR-022 through FR-026]
- [x] CHK026 Is repeated cleanup with zero owned task residue a measurable acceptance gate? [Measurability, SC-008]

## Security and Evidence Boundaries

- [x] CHK027 Does the specification prohibit target instrumentation, target key extraction, pinning bypass, and silent system-wide routing expansion by retaining the existing session boundary? [Security, Assumption]
- [x] CHK028 Are payload authorization and sensitive artifact handling preserved without copying observed payloads into human diagnostics? [Security, FR-019, FR-025]
- [x] CHK029 Are invalid protocol inputs required to fail safely without corrupting unrelated streams or forwarding evidence? [Security, FR-005, FR-018, FR-023]
- [x] CHK030 Are decompression expansion, time, memory, and concurrency bounds mandatory rather than advisory? [Security, FR-006, FR-021]
- [x] CHK031 Is every intentional omission or involuntary loss required to have exact named accounting? [Fidelity, FR-022]
- [x] CHK032 Is a partial or interrupted application artifact prohibited from claiming completeness? [Fidelity, FR-020, FR-023]

## Dependencies and Scope

- [x] CHK033 Are the S105 prerequisite contracts stated and are the three GitHub issue dependencies satisfied? [Dependency, Assumptions, Included]
- [x] CHK034 Are later HAR, key-log, correlation, client-certificate, transport, and completion work excluded explicitly? [Scope, Excluded]
- [x] CHK035 Is the controlled lab required to prove protocol behavior without Internet, privileged effects, or a real target? [Dependency, FR-026]

## Notes

- All requirements-quality checks pass. Implementation evidence remains the responsibility of tasks and tests.
