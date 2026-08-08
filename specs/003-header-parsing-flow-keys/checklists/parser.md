# Parser Correctness and Observation Fidelity Checklist

**Purpose**: Validate that the S03 requirements specify parsing coverage,
rejection accounting, and observation fidelity completely, unambiguously, and
measurably, before any code is written

**Created**: 2026-08-08

**Feature**: [spec.md](../spec.md)

**Depth**: Formal merge gate. This slice is on the critical path and every
slice after it inherits whatever this parser gets wrong, so an item passes only
when a reviewer could judge compliance from the requirement alone.

**Note**: This checklist tests the requirements, not the implementation. Each
item asks whether the spec says enough. An item passes when the requirement is
written well, not when the code works.

## Combination Coverage

- [x] CHK001 Are the supported link types enumerated exhaustively, rather than
  described by a category name that a reader would have to interpret?
  [Completeness, Spec §FR-006, §FR-007, §FR-008]
- [x] CHK002 Is the BSD loopback link type's byte order ambiguity addressed,
  given that the field is host ordered and a fixture may have been recorded on
  a host of the opposite order? [Edge Case, Spec §FR-008]
- [x] CHK003 Are the supported network protocols specified together with what
  must be validated in each header, rather than only named?
  [Clarity, Spec §FR-012, §FR-013]
- [x] CHK004 Is the IPv6 extension header set that must be walked enumerated,
  so that "walk the chain" is checkable rather than open-ended?
  [Measurability, Spec §FR-014]
- [x] CHK005 Are the supported transport protocols and the fields read from
  each stated, so that reading more than the ports would be a visible scope
  change? [Clarity, Spec §FR-018]
- [x] CHK006 Is coverage of every supported combination required as an outcome,
  rather than left to the discretion of whoever writes the tests?
  [Measurability, Spec §SC-001]
- [x] CHK007 Are the encapsulations deliberately excluded named, so that their
  absence reads as a decision rather than an oversight?
  [Completeness, Spec §FR-009, §Out of Scope]

## Rejection Cause Completeness

- [x] CHK008 Is every distinct rejection cause named individually, rather than
  grouped under a general parse failure? [Completeness, Spec §FR-033]
- [x] CHK009 Is a header truncated by a snapshot length required to be counted
  separately from a header that is malformed, given that the two have different
  remedies? [Clarity, Spec §FR-017]
- [x] CHK010 Is an unsupported transport protocol required to be counted
  separately from an IPv6 chain that legitimately ends with no transport?
  [Clarity, Spec §FR-016, §FR-019]
- [x] CHK011 Is an unsupported link type required to be counted separately from
  an unsupported EtherType, given that one indicates a backend surprise and the
  other a traffic surprise? [Consistency, Spec §FR-009, §FR-010]
- [x] CHK012 Is the rejection cause set required to be closed and enumerated,
  so that a later contributor adding a path must name it?
  [Completeness, Spec §Key Entities]
- [x] CHK013 Is reachability of every enumerated cause required as an outcome,
  so that a cause nothing can trigger would be caught?
  [Measurability, Spec §SC-002]
- [x] CHK014 Is "exactly one counter advances" specified, rather than "a
  counter advances", so that double counting is a defined failure?
  [Clarity, Spec §FR-033]

## Direction Rules

- [x] CHK015 Are all four combinations of source and destination membership in
  the interface address set assigned a defined outcome?
  [Coverage, Spec §FR-028, §FR-029, §FR-030]
- [x] CHK016 Is the loopback case required to produce no direction rather than
  a chosen one, and is the reason recorded so a later contributor does not
  resolve it locally? [Clarity, Spec §FR-029, §User Story 3]
- [x] CHK017 Is the absent-local-endpoint case distinguished from the loopback
  case in both its flow key outcome and its counter?
  [Consistency, Spec §FR-030, §SC-004]
- [x] CHK018 Is the prohibition on placing a non-local endpoint in the local
  position stated as a requirement, rather than implied by the field's name?
  [Completeness, Spec §FR-030]
- [x] CHK019 Can "both halves of one conversation produce one key" be
  objectively verified, given that the ordering rule itself is deferred to the
  plan? [Measurability, Spec §SC-005]
- [x] CHK020 Are the ownership and refresh requirements for the interface
  address set specified, including that the parser never queries the platform
  for one? [Completeness, Spec §FR-027, §FR-032]
- [x] CHK021 Is the behavior with an empty or stale address set defined, given
  that it is the most likely misconfiguration?
  [Edge Case, Spec §Edge Cases]

## Fragment Handling Without Reassembly

- [x] CHK022 Is the refusal to reassemble stated as a requirement rather than
  only as a rationale in the source specification?
  [Completeness, Spec §FR-025]
- [x] CHK023 Is fragment identity defined precisely for both address families,
  given that the two standards define different reassembly keys?
  [Clarity, Spec §FR-022]
- [x] CHK024 Is the fragment table's capacity a stated number rather than a
  requirement that one exist? [Measurability, Spec §FR-024]
- [x] CHK025 Are both the eviction policy and its counter specified, so that a
  full table is a visible condition rather than a silent one?
  [Completeness, Spec §FR-024]
- [x] CHK026 Is entry removal on the last fragment specified, so that entries do
  not outlive the datagrams they describe? [Coverage, Spec §FR-024a]
- [x] CHK027 Is the outcome defined for a non-initial fragment whose first
  fragment was never seen, including the out-of-order arrival case?
  [Edge Case, Spec §FR-023]
- [x] CHK028 Is the outcome defined for a first fragment whose own transport
  header could not be parsed? [Edge Case, Spec §FR-021a]
- [x] CHK029 Is a non-initial fragment's direction specified as derived from its
  own addresses rather than inherited, given that the address set may have
  changed? [Gap, Spec §FR-022a]
- [x] CHK030 Is the residual mis-attribution risk from identifier reuse stated,
  including why it cannot be counted?
  [Assumption, Spec §Known limitation]

## Adversarial and Malformed Input

- [x] CHK031 Is the extension header walk required to terminate on a bounded
  number of headers, with the bound stated?
  [Measurability, Spec §FR-015]
- [x] CHK032 Is a declared header length that would not advance the cursor
  addressed, given that it is the shape a non-terminating walk actually takes?
  [Edge Case, Spec §FR-015]
- [x] CHK033 Are the cases where a header's own fields contradict each other
  distinguished from the cases where they point outside the captured bytes?
  [Clarity, Spec §FR-017]
- [x] CHK034 Is a declared length exceeding the captured bytes specified as a
  legitimate condition rather than an error, given that a snapshot length
  produces exactly that? [Consistency, Spec §Edge Cases]
- [x] CHK035 Is reading beyond the captured bytes prohibited, rather than left
  as an implementation concern? [Completeness, Spec §Edge Cases]
- [x] CHK036 Is the prohibition on inferring an absent port stated, given that
  defaulting to zero is the natural shortcut on a truncated header?
  [Clarity, Spec §FR-020]

## P-4 Counting Obligations

- [x] CHK037 Is it required that no packet is dropped for any parse reason,
  rather than only that packets are usually retained?
  [Completeness, Spec §FR-036]
- [x] CHK038 Is the absence of drop-counter movement across the whole rejection
  corpus required as a measurable outcome?
  [Measurability, Spec §SC-008]
- [x] CHK039 Are the parse counters required to live in their own type held by
  value, consistent with how backend counters are already held?
  [Consistency, Spec §FR-034]
- [x] CHK040 Is any total over the parse counters required to be derived rather
  than stored, so it cannot drift from its parts?
  [Measurability, Spec §FR-035]
- [x] CHK041 Are the two undetermined-direction outcomes counted, given that
  neither is a rejection and so neither falls under the rejection counter rule?
  [Coverage, Spec §FR-029, §FR-030]

## P-9 Observation Fidelity

- [x] CHK042 Is modification of the input bytes prohibited as a requirement
  rather than assumed from the word "parsing"?
  [Completeness, Spec §FR-005]
- [x] CHK043 Is the prohibition on guessing stated positively somewhere, so
  that "return nothing and say why" is the specified behavior rather than the
  residual one? [Clarity, Spec §User Story 2, §FR-020]
- [x] CHK044 Is the decision not to store a per-packet rejection cause recorded
  with its reasoning, so a later slice that needs one knows it was considered?
  [Traceability, Spec §FR-036a]
- [x] CHK045 Is the fragment bytes-unchanged property stated, given that
  "does not reassemble" and "does not alter" are separate claims?
  [Consistency, Spec §FR-025, §User Story 4]

## Placement, Shape, and Traceability

- [x] CHK046 Is the owning crate stated with its architectural justification,
  rather than left to the implementer?
  [Completeness, Spec §FR-001, §Clarifications]
- [x] CHK047 Is the allocation-free property required with named evidence,
  rather than stated as an intention? [Measurability, Spec §FR-004, §SC-003]
- [x] CHK048 Is the parser's caller-facing shape specified, including that it
  needs no interior mutability? [Clarity, Spec §FR-031]
- [x] CHK049 Is the parse result required not to borrow from the input, so the
  caller's lifetime obligations are stated rather than discovered?
  [Completeness, Spec §FR-003]
- [x] CHK050 Are the divergences from the architecture of record required to be
  recorded for promotion, rather than left in the slice?
  [Traceability, Spec §FR-039]
- [x] CHK051 Is the correction to the existing link type documentation carried
  as a requirement, so it cannot be dropped as incidental?
  [Completeness, Spec §FR-011]

## Notes

All fifty-one items pass against the spec as it now stands. Three were failing
when the checklist was first drafted and are recorded here for traceability,
because each was a real gap rather than a wording problem:

- **CHK023** failed because FR-022 defined fragment identity as the address
  pair, protocol number, and identifier for both address families. That is the
  IPv4 reassembly key. The IPv6 fragment extension header carries a thirty two
  bit identification and no protocol number, so the single definition was wrong
  for half the cases it covered. FR-022 now defines each separately.
- **CHK028** failed because nothing said what happens when a first fragment's
  own transport header cannot be parsed. Recording an identity with no flow key
  is meaningless and matching against it later would be worse. FR-021a now
  states that no identity is recorded and the datagram's later fragments are
  counted as unmatched.
- **CHK029** failed because direction for a non-initial fragment was unstated.
  Inheriting it from the recorded entry is the tempting shortcut and is wrong,
  because every fragment carries its own address pair and the interface address
  set may have changed in between. FR-022a now requires it be recomputed.

Two items pass on a reading worth restating, so the analyze gate does not have
to rediscover the reasoning:

- **CHK019** passes even though FR-029 requires a deterministic ordering rule
  without stating what it is. The rule is a mechanism and belongs in `plan.md`;
  what the spec must pin is the observable property, and SC-005 pins it in a
  form a test can assert. If `plan.md` does not name the rule, this item
  regresses.
- **CHK041** covers the two undetermined-direction counters, which are not
  rejection counters and so are not governed by FR-033. There is deliberately
  no counter for a successful parse: the count of flow keys produced is the
  captured count less the rejection counters, and FR-035's rule against stored
  totals applies to it for the same reason it applies to the others.
