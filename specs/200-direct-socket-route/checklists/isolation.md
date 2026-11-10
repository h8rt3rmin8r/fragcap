# Route and Isolation Requirements Checklist

**Purpose**: Check whether S200's route and isolation requirements are precise enough to implement and verify.

## Ownership

- [x] Does the spec require generation and ancestry proof before a positive route claim? FR-002 and FR-003.
- [x] Does the spec define missed, delayed, reused, and ambiguous owner outcomes? FR-003 and Edge Cases.
- [x] Does the spec distinguish helper ownership from original-client ownership? FR-007.

## Isolation and accounting

- [x] Does the spec name active same-destination unrelated sentinels? Story 2 and FR-005.
- [x] Does the spec quantify sentinel loss and latency? SC-002 and Clarifications.
- [x] Does every packet delivered to the adapter have an exclusive disposition, with pre-receive driver loss explicitly unmeasurable? FR-006.
- [x] Are failed starts, stops, and recoveries covered? FR-010 and Story 2.

## Protocol truth

- [x] Are raw TCP and UDP independently accepted without target proxy handshakes? Stories 1 and 3.
- [x] Are opacity and trust failure distinct from successful inspection? FR-008 and FR-009.
- [x] Are packet, proxy, and application evidence distinct? FR-009.
