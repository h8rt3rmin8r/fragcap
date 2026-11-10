# Direct Route Isolation Requirements Checklist

**Purpose**: Review S201 owner, unrelated-traffic, loss, and recovery requirements before implementation.

## Requirement Completeness

- [x] CHK001 - Does the spec distinguish packet receipt, socket-event arrival, process proof, and decision timing?
- [x] CHK002 - Does it require exact process generation and original destination before direct selection?
- [x] CHK003 - Does it require prompt pass-through for a proven nonselected owner?
- [x] CHK004 - Does it preserve explicit uncertainty for a missing or ambiguous owner?
- [x] CHK005 - Does it state finite candidate capacity, worker stall, cancellation, and cleanup outcomes?

## Acceptance Quality

- [x] CHK006 - Are same-destination unrelated TCP and UDP sentinels included under parallel load?
- [x] CHK007 - Are selected UDP reply ownership and reused tuple cases included?
- [x] CHK008 - Is WinDivert pre-receive loss identified as unmeasurable rather than zero?
- [x] CHK009 - Is an exact final-client receipt plus independent packet and proxy evidence required for a positive fact?
- [x] CHK010 - Are MSI and ZIP package identity and cleanup included in the controlled gate?
