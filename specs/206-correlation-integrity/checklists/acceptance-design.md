# Acceptance Design Checklist: Packet Correlation Integrity

**Purpose**: Check that the S206 requirements describe independently reviewable evidence.

## Correlation authority

- [x] Does the spec distinguish route receipt from passive packet observation?
- [x] Does it name exact generation, tuple, time, and clock uncertainty as separate facts?
- [x] Does it prevent an omission on another flow from vetoing an exact flow?
- [x] Does it keep missing history and observed packet loss distinct?

## Qualification

- [x] Does each issue have an independently testable acceptance path?
- [x] Does #594 require both physical orderings before closure?
- [x] Does #596 include pre-client, post-client, helper, and unrelated traffic?
- [x] Does the spec forbid private-title and unobserved physical success claims?
