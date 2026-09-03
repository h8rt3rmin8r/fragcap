# Security Requirements Checklist: Scoped QUIC And HTTP/3 Inspection

**Purpose**: Review the completeness, clarity, and consistency of S118 security requirements
**Created**: 2026-09-03
**Audience**: Pull request reviewers

## Scope And Admission

- [x] CHK001 Is authenticated target-scoped route ownership explicitly required for every inspected QUIC connection? [Completeness, Spec FR-001]
- [x] CHK002 Are origin endpoint and certificate identity immutability defined for the complete connection lifetime? [Clarity, Spec FR-002]
- [x] CHK003 Are unrouted, unscopable, and unsupported address-family outcomes explicitly excluded from inspection claims? [Coverage, Spec FR-018]

## TLS And Replay Safety

- [x] CHK004 Are client-facing session-authority and upstream independent-validation requirements stated separately? [Consistency, Spec FR-003, Spec FR-004]
- [x] CHK005 Are trust, pinning, client-certificate, identity, and protocol failures required to refuse rather than downgrade? [Coverage, Spec FR-005]
- [x] CHK006 Is the zero round-trip policy unambiguous for both QUIC halves and replay-sensitive data? [Clarity, Spec FR-006]
- [x] CHK007 Does the specification avoid any target key extraction or pinning bypass authority? [Scope, Spec FR-021]

## Migration And Isolation

- [x] CHK008 Is active migration disabled while ordinary connection identifier rotation remains distinguishable? [Consistency, Spec FR-007, Spec FR-008]
- [x] CHK009 Are outer client, selected destination, and upstream path changes all covered by terminal scoped refusal requirements? [Completeness, Spec FR-007]
- [x] CHK010 Are connection, stream, datagram, task, queue, and memory owners explicitly finite? [Security, Spec FR-009, Spec FR-019]

## Evidence Integrity

- [x] CHK011 Are observation loss and forwarding independence requirements measurable for every evidence authority? [Measurability, Spec FR-013, Spec FR-014]
- [x] CHK012 Are packet truth and proxy-observed application truth kept explicitly distinct? [Consistency, Spec FR-017]
- [x] CHK013 Are key updates supported without claiming unavailable packet-key material? [Clarity, Spec FR-015]
- [x] CHK014 Are security-negative cases required in the offline controlled lab? [Coverage, Spec FR-020]

## Completion Boundary

- [x] CHK015 Is S118 prevented from claiming complete IPv6 parity, exhaustive protocol coverage, or Deep Capture completion? [Scope, Spec FR-022]
