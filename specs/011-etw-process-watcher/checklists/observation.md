# Observation Requirements Quality Checklist: S11

**Purpose**: Validate that the S11 requirements are complete, unambiguous, and
consistent in the areas this slice can plausibly get wrong: the constitutional
limit on how a process may be observed, the fidelity of what is recorded once
command lines are in scope, ancestry correctness under identifier recycling, the
seam between the startup snapshot and the event stream, incompleteness that no
packet counter can express, the five recorded deviations, and the boundaries
with the two slices either side of this one.

**Created**: 2026-08-09

**Feature**: [spec.md](../spec.md)

**Depth**: Formal gate. This slice is the first to observe the machine rather
than a file, it extends types the architecture of record declares, and it is the
one slice whose characteristic failure is a session that exits zero having
watched the wrong process. The bar is a release gate rather than a pre-commit
sanity pass.

**Audience**: The reviewing operator at pull request time, and the implementing
agent before `/speckit-plan`.

**Note**: These items test whether the requirements are written well. They do
not test whether the implementation works; that is what the slice's tests and
`cargo xtask ci` are for.

## Passive Observation and the Technique Denylist (P-1)

- [x] CHK001 Is the permitted observation mechanism named positively, so that a
      reviewer can tell a compliant design from a merely unlisted one? [Clarity,
      Spec FR-042]
- [x] CHK002 Are requirements defined for every process handle this slice opens,
      including the access rights each requests and where that is stated?
      [Completeness, Spec FR-008, FR-037]
- [x] CHK003 Is the prohibition on memory-read rights stated as a property of
      every code path rather than of the paths currently foreseen, given that a
      command line is the field most likely to tempt one? [Clarity, Spec FR-037]
- [x] CHK004 Are requirements stated for how a proposed telemetry dependency is
      evaluated against the denylist, given that an ETW crate is exactly the
      class of dependency that could supply a prohibited capability? [Gap, Spec
      FR-047]
- [x] CHK005 Is the mechanical check for a memory-read handle specified, or only
      the prohibition? The transmit-call check S09 added is the precedent. [Gap,
      Spec SC-012]
- [x] CHK006 Do the requirements distinguish observing a process from
      interacting with one, in terms a reviewer unfamiliar with either could
      apply to the anti-cheat launcher that appears in one focal title's
      ancestry? [Clarity, Spec FR-042]

## Fidelity of What Was Observed (P-9)

- [x] CHK007 Is "recorded verbatim" defined precisely enough to be checkable for
      a command line, including whether re-encoding, trimming, or normalizing
      path separators counts as alteration? [Ambiguity, Spec FR-035]
- [x] CHK008 Are requirements defined for a command line that is not valid
      Unicode, given that the recorded type must hold it either way? [Gap, Edge
      Case]
- [x] CHK009 Is the difference between an unavailable command line and an empty
      one specified in a way a consumer can act on, rather than left to a
      sentinel value? [Clarity, Spec FR-036]
- [x] CHK010 Are requirements stated for what fragcap reports when the platform
      supplies a field it cannot interpret, or supplies none at all? [Gap]
- [x] CHK011 Is it specified that an observed timestamp is carried without
      normalization, including when it is implausible, rather than corrected
      toward something reasonable? [Completeness, Spec FR-003]
- [x] CHK012 Can the requirement that a command line reaches the tree unaltered
      be objectively verified, or does it rest on inspection? [Measurability,
      Spec SC-013]
- [x] CHK013 Is the reasoning for recording command lines verbatim reproduced or
      referenced where an implementer will meet it, so that a later reader does
      not add redaction as an improvement? [Clarity, Spec FR-035]

## Ancestry Correctness and Identifier Recycling

- [x] CHK014 Are the identity of a node and the lookup key into the tree stated
      as separate things, or could an implementation collapse them? [Ambiguity,
      Spec FR-020, FR-023]
- [x] CHK015 Is parent resolution specified by identifier and time everywhere it
      occurs, including for the startup snapshot's parents, or only for events?
      [Coverage, Spec FR-026]
- [x] CHK016 Are requirements defined for a node whose start time is unknown
      taking part in a resolution, including the tie against a node whose start
      time is known? [Gap, Spec FR-024]
- [x] CHK017 Is the behavior specified for an event stream that arrives out of
      order, given that a trace consumer is not obliged to deliver in timestamp
      order? [Gap, Edge Case]
      **Resolved 2026-08-09, at authoring.** It was not. FR-031 was rewritten to
      forbid the fold assuming timestamp order, and the edge case now says why a
      trace consumer does not guarantee it.
- [x] CHK018 Are requirements stated for an exit event that arrives before the
      start event of the same process? [Gap, Edge Case]
      **Resolved 2026-08-09, at authoring.** FR-031 now holds such an exit
      against a start that has not arrived, and counts it unmatched only at the
      end of the session. Story 2 gained scenarios 7 and 8 for the two cases.
- [x] CHK019 Is retention of exited nodes specified as unconditional for the
      session, or could an implementation read it as best effort? [Clarity, Spec
      FR-027]
- [x] CHK020 Does the requirement that ancestry be answerable after the whole
      parent chain has exited follow from the stated node fields, or does it
      need a requirement of its own? [Consistency, Spec FR-032, SC-005]

## The Startup Snapshot Seam

- [x] CHK021 Is the ordering of subscription and snapshot stated with its
      reason, so that a later reviewer does not reorder it for tidiness?
      [Clarity, Spec FR-007]
- [x] CHK022 Are requirements defined for reconciling a duplicate in both
      arrival orders, rather than only in the order the implementation happens
      to produce? [Coverage, Spec FR-033, SC-014]
- [x] CHK023 Is it specified how a duplicate is recognized as the same process,
      given that the two sources supply different fields? [Gap, Spec FR-033]
- [x] CHK024 Are requirements stated for a process that exits during the
      startup sequence, so that the snapshot's view and the event stream's view
      do not disagree permanently? [Gap, Edge Case]
- [x] CHK025 Is ancestry provenance specified as carried rather than derived,
      with the consequence for a snapshot node whose parent happens to resolve?
      [Clarity, Spec FR-022]
- [x] CHK026 Are requirements defined for whether a snapshot node's ancestry may
      later be upgraded to creation-time provenance, and if not, why not? [Gap,
      Spec FR-033]

## Incompleteness and Loss (P-4)

- [x] CHK027 Are requirements defined for every path by which an observation can
      be lost, including ones inside the platform that fragcap only learns about
      after the fact? [Coverage, Spec FR-014, FR-045]
- [x] CHK028 Is the decision that the event channel is unbounded stated with its
      reason, so that a later reviewer does not "correct" it into a bounded
      buffer with a counter? [Clarity, Spec FR-013]
- [x] CHK029 Is it specified that a tree which may have a hole in it says so,
      and that this is distinct from the count of lost events? [Ambiguity, Spec
      FR-034, SC-007]
- [x] CHK030 Are requirements clear on whether an unmatched exit event is a
      defect, a normal consequence of starting mid-session, or both? [Ambiguity,
      Spec FR-031]
- [x] CHK031 Is the requirement that the watcher's counters stay out of
      `CaptureStats` stated with the conservation identity as its reason?
      [Clarity, Spec FR-015, SC-016]
- [x] CHK032 Are requirements defined for what the operator sees when the
      watcher fails after the session has started, as opposed to failing to
      start? [Gap, Spec User Story 4]
- [x] CHK033 Is the growth of the tree over a long session bounded by something
      an operator can observe, rather than by an estimate in the specification?
      [Measurability, Spec FR-029, SC-017]

## The Five Recorded Deviations

- [x] CHK034 Is each deviation stated with the specification section it diverges
      from, the reason the divergence is necessary, and the commitment to
      promote it to section 29? [Completeness, Spec Deviations]
- [x] CHK035 Is the blast radius of adding a command line to
      `ProcessEvent::Started` documented, including which existing tests and
      match sites it breaks? [Gap, Spec Deviations]
- [x] CHK036 Is the interaction between `#[non_exhaustive]` on the enum and a
      new field on an existing variant stated correctly, rather than assumed to
      be covered? [Conflict, Spec Deviations]
- [x] CHK037 Are requirements defined for what `image` means now that it is
      settled as a path, including for the S02 tests that pass a bare file name?
      [Gap, Spec FR-038]
- [x] CHK038 Is the watcher-owned report's relationship to section 26.2's list
      of runtime statistics stated, given that section 26.2 is the architecture
      of record for what an operator sees? [Consistency, Spec Deviations]
- [x] CHK039 Do the deviation requirements conflict with the traits module's own
      statement that its contents are intended to reach 1.0.0 unchanged, and if
      so is the conflict acknowledged rather than silent? [Conflict]

## Tier Separation and Buildability

- [x] CHK040 Is the feature that gates the ETW watcher named, along with which
      crates declare it and which check commands enable it? [Gap, Spec FR-017]
- [x] CHK041 Is the distinction between "compiled out on a non-Windows target"
      and "off by default everywhere" specified, or are the two conflated?
      [Ambiguity, Spec FR-017, FR-018]
- [x] CHK042 Are requirements stated for how a tier 2 test declares its need for
      elevation, so that it is skipped rather than failed on an unelevated
      machine? [Gap, Spec SC-009]
- [x] CHK043 Is it specified whether `cargo xtask neutral` must build
      `fragcap-attr` as well, given that S09 extended it for `fragcap-capture`
      for the same reason? [Gap, Spec SC-010]
- [x] CHK044 Are requirements defined for what the `platform` workflow must
      change, given that it is a pinned artifact requiring a dated decision?
      [Gap, Spec Assumptions]
- [x] CHK045 Is the requirement that the whole of section 10.2 runs at tier 1
      checkable, or does it rest on the claim that the tree holds no platform
      interface? [Measurability, Spec FR-019, SC-011]

## Scope Boundaries with S10 and S12

- [x] CHK046 Is the boundary with S12 stated in terms of what code lands here,
      rather than only what does not? [Clarity, Spec FR-049]
- [x] CHK047 Is the reserved place for a matched stage specified well enough to
      be built without the profile schema, and inert enough that S12 is not
      constrained by a guess made here? [Ambiguity, Spec FR-049]
- [x] CHK048 Are requirements defined for what this slice may add to
      `fragcap-attr` given that S10 is developing in the same crate in parallel,
      or is the non-collision left to chance? [Gap, Spec Assumptions]
- [x] CHK049 Is the claim that S10 and S11 have no dependency in either
      direction stated, and does anything in the requirements contradict it?
      [Consistency, Spec Dependencies]
- [x] CHK050 Are the glossary terms this slice introduces identified, so that
      P-6 can be satisfied in the same change rather than discovered at review?
      [Gap, Spec FR-046]

## Notes

- Check items off as resolved: `[x]`. An item that is resolved by a
  specification edit should name the requirement it added or changed.
- An item that turns out not to apply is struck with a one-line reason rather
  than silently checked. A checklist that only ever gets ticked is not
  measuring anything.
- CHK017, CHK018, and CHK036 are the three most likely to be real. The first two
  name orderings the specification assumes away and a trace consumer does not
  guarantee, and the third names a Rust rule that is easy to state backwards.
  CHK039 names a possible internal contradiction, and a contradiction survives
  review more easily than an omission does.

## Resolution pass, 2026-08-09

Worked through after implementation. The items whose answer is not obvious from
the diff are recorded here, because a checklist that was only ever ticked would
not have been worth writing.

**CHK017 and CHK018 were real, and were closed at authoring** rather than at
implementation. The specification assumed timestamp-ordered delivery and a trace
consumer reading from several buffers does not guarantee it. FR-031 was
rewritten to hold an exit against a start that has not arrived, and to count it
unmatched only at the end of the session. Both cases have tests.

**CHK002 and CHK003, the process handle.** There is none. The slice first opened
one in `etw/snapshot.rs` with `PROCESS_QUERY_LIMITED_INFORMATION`, which
complies, and withdrew it at integration in favour of the stronger rule S10 had
already lint-enforced. The only handle taken anywhere is to a snapshot object.
CHK005 is satisfied mechanically: `cargo xtask lint` fails on `openprocess` and
on four memory-bearing rights, and the check was confirmed to fail by adding
`PROCESS_VM_READ` to a source file and watching it report before removing it
again.

**CHK007 through CHK013, fidelity.** `CommandLine` is an enum rather than an
`Option` so that `unwrap_or_default` cannot silently convert "not observed" into
"was empty". A command line with characters outside ASCII and one of 60,000
characters both reach the tree byte for byte, asserted on bytes rather than on a
display form. CHK008 is answered by a decision rather than a test: an unpaired
surrogate is converted lossily rather than refused, because a command line
Windows accepted is one fragcap has to record, and the lossy form is what the
platform itself displays.

**CHK014 through CHK020, ancestry.** `ProcessId` and `NodeId` are distinct
newtypes, which is what stops a caller confusing an identifier that recycles
with one that does not. A cycle is impossible for an ordinary node by
construction, and the one path that could build one, the snapshot upgrade,
refuses a candidate whose chain reaches the node; there is a test for the
absurd case.

**CHK021 through CHK026, the snapshot seam.** Reconciliation is tested in both
arrival orders. CHK026 was answered by implementing it: a snapshot node whose
creation event later arrives is upgraded in place to `Ancestry::Observed`, which
is the second arrival order and the reason a duplicate is preferable to a gap.

**CHK027 through CHK033, loss.** Two discard paths exist and both are counted:
records that do not parse, and rundown events that are deliberately not
published. The channel to subscribers has no discard path at all, which is the
correct way to satisfy P-4 for this stream and is written down in three places
so that a later reviewer does not add a bound.

**CHK034 through CHK039, the deviations.** Seven rather than five by the end.
Implementation added the observed parent identifier surviving an unresolved
parent, and found a defect in the specification: section 5.4's prose says the
Division 2 chain is six levels while its own diagram and Appendix D.3 both list
seven. CHK039 is answered in the negative: `ProcessWatcher` is unchanged, so
nothing here contradicts the traits module's own statement about reaching 1.0.0.

**CHK040 through CHK045, tier separation.** The feature is `etw`, off by
default. `cargo xtask ci` passed on this machine with no elevation. `cargo xtask
neutral` was extended and reports `fragcap-attr` building for the neutral
target. CHK042 is answered structurally rather than by a helper: tier 2 tests
are `#[ignore]`, so they are skipped by name rather than by a runtime probe.

**CHK046 through CHK050, boundaries.** `stage` is reserved on the node, always
`None`, and typed as the existing `StageId` so S12 binds a value rather than
introducing a type. CHK048 is answered by placement: this slice adds two modules
beside S10's work and modifies no line of `script.rs` or `scripted.rs`. Six
glossary entries were written, per P-6.

**Struck, with reasons.** CHK044 does not apply as written: the `platform`
workflow did need changing, and the dated decision is in the changelog fragment,
so the item is satisfied rather than not applicable. Nothing else was struck.
