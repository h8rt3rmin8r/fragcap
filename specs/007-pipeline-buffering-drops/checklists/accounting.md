# Concurrency and Accounting Correctness Checklist

**Purpose**: Validate that the S08 requirements pin the conservation of
packets, the meaning and increment rule of every named counter, the
producer-never-waits property, ordering, and every termination path well enough
that a reviewer can judge compliance without reading the pipeline

**Created**: 2026-08-08

**Feature**: [spec.md](../spec.md)

**Depth**: Formal merge gate. This is the first concurrent code in the project
and the first producer of the drop counters. Both of its failure modes are
silent: an interleaving no test reached loses packets without saying so, and a
discard path with no counter reports a clean capture that was not one. Neither
is visible in the output, which is precisely why the requirements have to carry
the weight.

**Note**: This checklist tests the requirements, not the implementation. An
item passes when the requirement is written well enough for a reviewer to judge
compliance, not when the code works.

## Conservation and the paths a packet can take

- [x] CHK001 Is the conservation identity stated as a requirement rather than
  left to be inferred from the individual counters? [Completeness, Spec §FR-022]
- [x] CHK002 Is the identity scoped per sink rather than globally, given that a
  packet can be refused by one sink and accepted by another? [Clarity, Spec
  §FR-022, §Edge Cases]
- [x] CHK003 Are all the terminal states of a packet enumerated (written,
  evicted, refused), so that a reviewer can tell whether a fourth state was
  introduced? [Coverage, Spec §FR-022]
- [x] CHK004 Is it specified that the identity must hold in every test that
  runs the pipeline, rather than in a single dedicated test? [Measurability,
  Spec §SC-004]
- [x] CHK005 Is the case of zero attached sinks addressed, where packets are
  drained and discarded and the identity has no sink to range over? [Edge Case,
  Spec §Edge Cases]
- [x] CHK006 Is a packet that produced no flow key required to reach the sink
  rather than being discarded as unusable? [Coverage, Spec §FR-039]

## Counters and their increment rules

- [x] CHK007 Is the increment for an eviction specified as exactly one, rather
  than as "counted"? [Clarity, Spec §FR-016]
- [x] CHK008 Is the multiplicity of `sink_dropped` under fan-out stated
  explicitly, given that per-packet and per-refusal are both defensible
  readings? [Ambiguity, Spec §FR-017, §Clarifications]
- [x] CHK009 Is the rationale for choosing per-refusal recorded, so that a
  later reader does not re-derive the opposite? [Traceability,
  Spec §Clarifications]
- [x] CHK010 Is the prohibition on folding backend counters into fragcap
  counters stated as a requirement rather than only as a type-level
  arrangement? [Completeness, Spec §FR-018]
- [x] CHK011 Are the attribution counters specified as mutually exclusive, with
  the no-flow-key case advancing neither? [Clarity, Spec §FR-020]
- [x] CHK012 Is the accumulation of parse counters into the run statistics
  required, given that S03 produces them and nothing currently collects them?
  [Gap closed, Spec §FR-021]
- [x] CHK013 Is there a standing requirement against adding a discard path with
  no counter, rather than only an enumeration of today's paths? [Completeness,
  Spec §FR-024]
- [x] CHK014 Is it specified that `Sink::finish` receives the run's own final
  values, including counters produced after the acquisition side ended?
  [Completeness, Spec §FR-023]
- [x] CHK015 Is the mechanism by which output-side counters reach the final
  value constrained enough to rule out reading shared state at an arbitrary
  instant? [Clarity, Spec §Clarifications]
- [x] CHK016 Is a clean run required to report zero in every drop counter,
  rather than omitting them? [Coverage, Spec §US2]

## The producer-never-waits property

- [x] CHK017 Is the property stated as "never waits for a sink to make
  progress" rather than as "never blocks", so that it is a claim the
  implementation can actually honor? [Measurability, Spec §Clarifications,
  §FR-013]
- [x] CHK018 Is the distinction between waiting on sink progress and waiting on
  a bounded critical section drawn explicitly, rather than left for a reviewer
  to supply? [Ambiguity, Spec §Clarifications]
- [x] CHK019 Is the specification's reason for the property recorded (that
  blocking capture converts a fragcap drop into a kernel drop), so that a later
  change knows what it would be trading away? [Traceability, Spec §Overview]
- [x] CHK020 Is there an observable consequence the property can be tested by,
  rather than only a structural claim about the code? [Measurability, Spec
  §US3, §SC-005]
- [x] CHK021 Is drop-oldest required rather than drop-newest, and is the reason
  recorded? [Clarity, Spec §FR-012, §Overview]
- [x] CHK022 Are the rejected alternatives for the buffer mechanism recorded
  with why each fails, so that "why not a channel" is answered once? [Assumption,
  Spec §Clarifications]
- [x] CHK023 Is the consumer required to wait without spinning when the buffer
  is empty, given that a correct-but-spinning consumer would pass every other
  requirement here? [Gap closed, Spec §FR-015]

## Ordering

- [x] CHK024 Is order preservation required of the buffer itself, separately
  from the end-to-end golden comparison that would incidentally detect a
  violation? [Completeness, Spec §FR-014, §FR-038]
- [x] CHK025 Is ordering required to survive eviction, so that a buffer that
  reorders only when full is not compliant? [Edge Case, Spec §US3]
- [x] CHK026 Is the prohibition on reordering tied to its constitution
  principle rather than stated as a bare preference? [Traceability, Spec
  §FR-038]

## Termination

- [x] CHK027 Are all four ordinary end reasons enumerated and required to be
  distinguishable in the report? [Completeness, Spec §FR-032]
- [x] CHK028 Is draining the buffer required before any sink is finished, on
  every ending rather than only on clean exhaustion? [Coverage, Spec §FR-030]
- [x] CHK029 Is "flushed and then finished exactly once" specified, rather than
  leaving double-finish or finish-without-flush open? [Clarity, Spec §FR-031]
- [x] CHK030 Is the behavior when a sink fails non-countably specified for the
  other sinks, rather than only for the failing one? [Coverage, Spec §FR-028]
- [x] CHK031 Is the acquisition-side panic path specified, including how the
  output side learns the run ended? [Gap closed, Spec §FR-033a, §Edge Cases]
- [x] CHK032 Is it specified that a panic is re-raised rather than converted
  into an end reason, so that a defect is not filed under an accounting
  category? [Clarity, Spec §FR-033b]
- [x] CHK033 Is the loss of acquisition-side statistics during a panic named as
  a known gap rather than left to be discovered? [Assumption, Spec §Edge Cases]
- [x] CHK034 Is stop latency bounded by something stated, rather than described
  only as cooperative? [Measurability, Spec §Clarifications]
- [x] CHK035 Is a recoverable source error required not to end the run and not
  to be counted as loss, given that both mistakes are easy and opposite?
  [Coverage, Spec §FR-033]
- [x] CHK036 Are the degenerate startings specified: a source closed on the
  first call, and a stop requested before the run begins? [Edge Case, Spec
  §Edge Cases]
- [x] CHK037 Is the report required to carry the statistics on the failure path
  as well as the success path? [Completeness, Spec §FR-035]

## Constitution obligations as they bind this slice

- [x] CHK038 P-2: Is the prohibition on a platform dependency, an I/O crate,
  and an asynchronous runtime stated for the pipeline specifically, rather than
  inherited silently from the crate? [Traceability, Spec §FR-001]
- [x] CHK039 P-2: Is the no-new-runtime-dependency requirement stated, and is
  the alternative analysis recorded so that adding one later is a visible
  reversal? [Completeness, Spec §FR-009, §Clarifications]
- [x] CHK040 P-3: Is it required that the pipeline compose the source and the
  attributor without either naming the other? [Traceability, Spec §FR-003,
  §US6]
- [x] CHK041 P-3: Is the test placement requirement stated, given that
  `cargo xtask deps` ignores dev-dependencies and would not catch the
  violation? [Gap closed, Spec §FR-042]
- [x] CHK042 P-4: Is every discard path in this slice matched to a named
  counter in the requirements, with none left implied? [Completeness, Spec
  §FR-016 to §FR-024]
- [x] CHK043 P-6: Are the terms needing glossary entries named individually,
  rather than left as "every new term"? [Clarity, Spec §FR-041]
- [x] CHK044 P-9: Is the prohibition on altering, reordering, and withholding
  stated as three separate requirements, given that an implementation can
  satisfy one and not the others? [Completeness, Spec §FR-037 to §FR-040]
- [x] CHK045 P-9: Is drop-oldest characterized as a declared and counted
  omission rather than as an exception to P-9? [Consistency, Spec §Overview]

## Scope discipline

- [x] CHK046 Is the control thread's absence specified as a seam to leave
  unfilled, rather than as an omission? [Clarity, Spec §FR-007]
- [x] CHK047 Is the one-interface restriction on both writers required to
  survive this slice, so that a convenient workaround is a visible violation?
  [Consistency, Spec §FR-043]
- [x] CHK048 Is the absence of a throughput target stated with its reason,
  rather than left looking like an oversight? [Assumption, Spec §Clarifications,
  §SC-012]
- [x] CHK049 Is the absence of logging stated as a requirement, so that adding
  a logging call is a visible scope change? [Gap closed, Spec §FR-041a]
- [x] CHK050 Is the blocking shape of the run entry point required, so that a
  spawning variant is a visible addition rather than a judgement call?
  [Clarity, Spec §FR-033c]

## Outstanding

None. Every item above was reachable from the specification as written, which
is the result the clarify pass was for. Five items (CHK012, CHK023, CHK031,
CHK041, CHK049) are marked "Gap closed" because the requirement they check was
added during clarification rather than present in the first draft.
