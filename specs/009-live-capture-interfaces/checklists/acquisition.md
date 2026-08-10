# Acquisition Requirements Quality Checklist: S09

**Purpose**: Validate that the S09 requirements are complete, unambiguous, and
consistent in the areas this slice can plausibly get wrong: the constitutional
constraints on the acquisition path, the capture driver's distribution rules,
loss accounting once several threads produce into one buffer, the fidelity of
values fragcap relays rather than owns, the two recorded deviations, and the
tier separation that keeps the ordinary check set runnable anywhere.

**Created**: 2026-08-09

**Feature**: [spec.md](../spec.md)

**Depth**: Formal gate. This slice touches two types the architecture of record
declares and adds the project's first platform-specific acquisition path, so
the bar is a release gate rather than a pre-commit sanity pass.

**Audience**: The reviewing operator at pull request time, and the implementing
agent before `/speckit-plan`.

**Note**: These items test whether the requirements are written well. They do
not test whether the implementation works; that is what the slice's tests and
`cargo xtask ci` are for.

## Passive Observation and the Technique Denylist (P-1)

- [x] CHK001 Is the permitted acquisition mechanism named positively rather
      than only by exclusion, so that a reviewer can tell a compliant design
      from a merely unlisted one? [Clarity, Spec FR-046]
- [x] CHK002 Are requirements defined for what fragcap may do with a process
      handle in this slice, given that interface enumeration and driver
      detection may be tempted to reach for one? [Gap]
- [x] CHK003 Is it specified that the acquisition path reads no process memory
      and modifies no traffic, rather than leaving that inferred from P-1?
      [Completeness, Spec FR-046]
- [x] CHK004 Are requirements stated for how a proposed dependency is evaluated
      against the denylist, given that a capture library is exactly the class
      of dependency that could supply a prohibited capability? [Gap, Spec
      FR-051]
- [x] CHK005 Do the requirements distinguish the NDIS capture driver fragcap
      uses from the packet interception drivers P-1 forbids, in terms a
      reviewer unfamiliar with both could apply? [Clarity, Spec FR-046]

## Capture Driver Distribution Rules (constitution licensing section)

- [x] CHK006 Are all four npcap rules from the constitution's licensing section
      represented in the requirements, or is the specification silent on any of
      them? [Completeness, Spec FR-041 through FR-045]
- [x] CHK007 Is "never downloads, installs, or invokes an installer" stated as
      a property of every code path rather than of the happy path? [Clarity,
      Spec FR-043]
- [x] CHK008 Are the required non-default installation options named
      individually in the requirements, or referred to collectively in a way
      that would let a diagnostic report the wrong one? [Clarity, Spec FR-045]
- [x] CHK009 Is the mechanical check that no driver or software development kit
      file enters the repository specified, or only the prohibition? [Gap, Spec
      FR-044, SC-010]
- [x] CHK010 Are requirements defined for what the build acquires versus what
      the runtime requires, so that "the SDK is acquired at build time" and
      "the driver is detected at runtime" cannot be conflated? [Ambiguity, Spec
      FR-022, FR-041]
- [x] CHK011 Is the boundary between this slice's detection capability and
      S14's `doctor` presentation stated clearly enough that neither slice can
      assume the other owns the message text? [Consistency, Spec FR-041,
      Assumptions]

## Loss Accounting Across Several Threads (P-4)

- [x] CHK012 Are requirements defined for every discard path this slice
      introduces, including any that exist only because there is now more than
      one producer? [Coverage, Spec FR-049]
- [x] CHK013 Is it specified whether the conservation identity S08 established
      is per-interface, capture-wide, or both, given that several capture
      threads now feed one buffer? [Ambiguity, Spec FR-025, FR-029, SC-006]
- [x] CHK014 Are requirements clear on which counters are per-interface and
      which are capture-wide, rather than leaving the reader to infer it from
      where each is incremented? [Clarity, Spec FR-029]
- [x] CHK015 Is the requirement that interface retirement advances no drop
      counter stated with its reason, so that a later reviewer does not
      "correct" it into a counter? [Clarity, Spec FR-028]
- [x] CHK016 Are requirements defined for what happens to packets already in
      the buffer when the interface that produced them retires? [Gap, Edge
      Case]
- [x] CHK017 Is it specified whether a packet lost between the driver and
      fragcap during filter installation is counted, given that section 12.2
      says installation briefly interrupts capture on that handle? [Gap, Spec
      FR-037]
- [x] CHK018 Are `filter_gaps` requirements stated for this slice, or is that
      counter left entirely to S13 despite the bootstrap filter being installed
      here? [Gap, Consistency]

## Fidelity of Relayed Values (P-9)

- [x] CHK019 Is "relayed unaltered" defined precisely enough to be checkable,
      including whether accumulating a driver counter across polls counts as
      alteration? [Ambiguity, Spec FR-017]
- [x] CHK020 Are requirements defined for what fragcap reports when the driver
      supplies a counter it cannot interpret, or supplies none at all? [Gap,
      Edge Case]
- [x] CHK021 Is it specified that a driver-supplied timestamp is carried
      without normalization, including when it is implausible, rather than
      corrected toward something reasonable? [Completeness, Spec FR-014]
- [x] CHK022 Are requirements stated for an interface that was watched and saw
      nothing, so that zero is reported as an observation rather than the
      interface being omitted? [Coverage, Spec Edge Cases]
- [x] CHK023 Can the requirement that the driver's counts stay distinct from
      fragcap's own be objectively verified, or does it rest on inspection?
      [Measurability, Spec SC-007]

## The Two Recorded Deviations

- [x] CHK024 Are both deviations stated with the specification section they
      diverge from, the reason the divergence is necessary, and the commitment
      to promote them to section 29? [Completeness, Spec Deviations]
- [x] CHK025 Is the blast radius of adding `Send` to `PacketSource` documented,
      including which existing implementations and tests it constrains? [Gap,
      Spec FR-023]
- [x] CHK026 Are requirements defined for what the interface identifier means
      for a source that is not an interface, such as the replay source reading
      a file? [Ambiguity, Spec FR-030]
- [x] CHK027 Is the non-optional choice for the identifier stated with its
      consequence for `CapturedPacket::from_raw`, which existing tests call
      throughout the workspace? [Gap, Spec FR-030]
- [x] CHK028 Do the deviation requirements conflict with the traits module's
      own statement that its contents are intended to reach 1.0.0 unchanged,
      and if so is the conflict acknowledged rather than silent? [Conflict]

## Interface Selection and Identity

- [x] CHK029 Is the rule that classifies an interface as virtual specified, or
      only the requirement that one exist? [Gap, Spec FR-004]
- [x] CHK030 Are the three precedence steps mutually exclusive and exhaustive
      as written, including the case where no explicit names are given and
      broad capture is not requested and no default route exists? [Coverage,
      Spec FR-005, Edge Cases]
- [x] CHK031 Is "the interface carrying the default route" defined for a
      machine with several default routes or a metric tie? [Ambiguity, Spec
      FR-005]
- [x] CHK032 Are requirements defined for interface identity when the
      platform's own names collide, beyond stating that identity must remain
      unambiguous? [Clarity, Spec FR-002, Edge Cases]
- [x] CHK033 Is the format and destination of the selection record specified
      well enough to be built, given that the run report it feeds does not
      exist until S14? [Gap, Spec FR-009, FR-028]
- [x] CHK034 Are requirements consistent between "selection is a decision over
      an inventory value" and "selection must fail the run" on an unmatched
      name, given that a pure decision cannot fail a run? [Conflict, Spec
      FR-007, FR-010, FR-011]
      **Resolved 2026-08-09.** They were not. FR-007 and FR-011 were rewritten
      to produce a named error that the caller surfaces as a failed run, which
      is the only form compatible with FR-010.

## Tier Separation and Buildability

- [x] CHK035 Is the feature that gates the live source named, along with which
      crates declare it and which check commands enable it? [Gap, Spec FR-022]
- [x] CHK036 Are requirements defined for what the `platform` workflow must
      change, given that it is a pinned artifact requiring a dated decision?
      [Gap, Spec Assumptions]
- [x] CHK037 Is the distinction between "compiled out on a non-Windows target"
      and "off by default everywhere" specified, or are the two conflated?
      [Ambiguity, Spec FR-021, FR-022]
- [x] CHK038 Are requirements stated for how a tier 2 test declares its need
      for a driver, so that it is skipped rather than failed on a machine
      without one? [Gap, Spec SC-011]
- [x] CHK039 Is it specified whether `cargo xtask neutral` and `cargo xtask
      msrv` must continue to pass with the feature enabled, or only with it
      off? [Gap, Consistency]

## Dependencies, Assumptions, and Scope Boundaries

- [x] CHK040 Is the boundary between this slice's bootstrap filter and S13's
      narrowing stated in terms of what code lands here, rather than only what
      does not? [Clarity, Spec FR-036, FR-039]
- [x] CHK041 Are the assumptions about where loopback and broad capture
      settings come from validated against the dependency direction the
      specification fixes? [Assumption, Spec FR-012]
- [x] CHK042 Is the exclusion of the section 12.7 session anchor justified in
      the requirements, given that this is the first slice with a real capture
      start to anchor? [Assumption, Spec Assumptions]
- [x] CHK043 Are the glossary terms this slice introduces identified, so that
      P-6 can be satisfied in the same change rather than discovered at review?
      [Gap, Spec FR-050]

## Notes

- Check items off as resolved: `[x]`. An item that is resolved by a
  specification edit should name the requirement it added or changed.
- An item that turns out not to apply is struck with a one-line reason rather
  than silently checked. A checklist that only ever gets ticked is not
  measuring anything.
- CHK034 and CHK028 are the two most likely to be real. Both name a possible
  internal contradiction rather than a gap, and a contradiction survives review
  more easily than an omission does.

## Resolution pass, 2026-08-09

Worked through after implementation. Every item is checked, and the ones whose
answer is not obvious from the diff are recorded here, because a checklist that
was only ever ticked would not have been worth writing.

**CHK034 found a real contradiction** and is the item that justified the
exercise. FR-010 made selection a pure decision while FR-007 and FR-011 had it
"fail the run", which a pure function cannot do. Both were rewritten to produce
a named error the caller surfaces.

**CHK002, the process handle.** Nothing in `crates/fragcap-capture/src/live/`
opens one. Enumeration, detection, and the route lookup all go through
`pcap::Device::list`, `std::net`, or nothing at all.

**CHK017 and CHK018, filter gaps.** Not applicable to this slice, and now
provably so: the bootstrap filter is installed inside `LiveSource::open` before
the handle delivers its first packet, so there is no window in which a packet
could pass an uninstalled filter. Reinstallation, which is where a gap becomes
possible, is S13's.

**CHK019, "relayed unaltered".** Settled by measurement rather than by wording.
`pcap::Stat` is documented as counting from the start of the run to the moment
of the call, so fragcap copies a cumulative value and never accumulates one, and
there is no arithmetic in which an alteration could hide.

**CHK026 and CHK027, the identifier on a source that is not an interface.** A
replay source is bound to an `InterfaceId` by the caller exactly as a live
source is, because a capture file was recorded from somewhere too.
`from_raw` gained a required parameter with no default; the compiler enumerated
the roughly forty call sites.

**CHK029 and CHK031, the two heuristics.** The virtual-interface pattern list is
data in one place with a comment saying it is a heuristic. The default route
comes from the operating system's own choice of source address, so a machine
with several default routes or a metric tie is answered by the routing table
rather than by fragcap guessing.

**CHK035 through CHK039, tier separation.** The feature is `live`, off by
default, and `cargo xtask ci` was run on a machine with neither npcap nor its
software development kit. `cargo xtask neutral` was extended to build
`fragcap-capture` as well as `fragcap-core`, which CHK-adjacent analyze finding
C1 showed nothing had ever checked.

**CHK043, the glossary.** Six entries written in this change, per P-6.
