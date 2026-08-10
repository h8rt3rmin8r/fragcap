# Attribution Requirements Quality Checklist: S10

**Purpose**: Validate that the S10 requirements are complete, unambiguous, and
consistent in the areas this slice can plausibly get wrong: producing a
confident wrong answer, conflating observed attribution with inferred
attribution, mishandling the two instants that matter, publishing a snapshot
unsafely, losing the distinction between never attempted and attempted and
unresolved, and the constitutional limits on how a process may be named.

**Created**: 2026-08-09

**Feature**: [spec.md](../spec.md)

**Depth**: Formal gate. This is the first attributor in the project that can be
wrong, it modifies a trait the architecture of record declares, and it records
three deviations. The bar is a release gate rather than a pre-commit sanity
pass.

**Audience**: The reviewing operator at pull request time, and the implementing
agent before `/speckit-plan`.

**Note**: These items test whether the requirements are written well. They do
not test whether the implementation works; that is what the slice's tests and
`cargo xtask ci` are for.

## The Join, and Answers That Are Confidently Wrong (P-9)

- [x] CHK001 Is the match rule stated separately for TCP and UDP, rather than
      as one rule with an exception, given that section 8.4 requires the
      asymmetry not be papered over? [Clarity, Spec FR-005, FR-006]
- [x] CHK002 Is the prohibition on inventing a remote endpoint for a UDP entry
      stated as a requirement, or only as a property of a type declared in
      another slice? [Completeness, Spec FR-002, FR-006]
- [x] CHK003 Is the order over competing matches total as written, meaning that
      no two distinct entries can tie under every stated rule? [Ambiguity, Spec
      FR-008 through FR-008b]
- [x] CHK004 Are the exactness ranks mutually exclusive, so that one entry
      cannot occupy two of them? [Coverage, Spec FR-008]
- [x] CHK005 Is the dual-stack allowance bounded to the protocol that takes the
      wildcard allowance at all, or could it widen TCP matching by implication?
      [Conflict, Spec FR-007]
- [x] CHK006 Are requirements defined for a flow whose local endpoint matches
      no bind and whose remote matches a bind, so that the local and remote
      positions cannot be silently swapped? [Gap]
- [x] CHK007 Is the determinism requirement verifiable by a test that does not
      depend on knowing the implementation's data structure? [Measurability,
      Spec SC-014]

## Observed Versus Inferred (Fidelity)

- [x] CHK008 Is every path that produces an `Attribution` covered by a
      requirement stating which fidelity it carries, or can a path exist with
      no stated value? [Coverage, Spec FR-019]
- [x] CHK009 Is it stated that fidelity is supplied rather than inferred from
      whether an attribution exists, given that S06 shipped exactly that bug?
      [Completeness, Spec FR-019]
- [x] CHK010 Are requirements defined for the case where a live entry and a
      retained entry both match, including which fidelity the winner carries?
      [Ambiguity, Spec FR-020]
- [x] CHK011 Is the risk that a retained answer is wrong stated in the
      requirements rather than only in the overview, so that a later change
      cannot quietly widen the window? [Clarity, Spec FR-018a]
- [x] CHK012 Is `Fidelity::None` accounted for: does this slice ever produce
      it, and if not, is that stated rather than left to inference? [Gap]

## The Two Instants

- [x] CHK013 Is it unambiguous which instant each rule uses, given that the
      packet's instant, the refresh instant, the socket's creation instant, and
      the endpoint's last-seen instant all appear? [Ambiguity, Spec FR-009,
      FR-010, FR-018a, FR-021]
- [x] CHK014 Is the retention origin stated in a way that cannot be satisfied
      by measuring from the refresh that noticed the absence? [Clarity, Spec
      FR-018a]
- [x] CHK015 Are requirements defined for a packet whose instant precedes every
      snapshot the attributor has taken, which is the ordinary case for the
      first packets of a run? [Gap, Edge Case]
- [x] CHK016 Is the creation-instant filter stated as unavailable for UDP
      rather than silently inapplicable, so that the asymmetry is a recorded
      property? [Completeness, Spec FR-003, FR-009, SC-004]
- [x] CHK017 Are requirements defined for a platform that reports no creation
      instant at all, given that FR-003 requires distinguishing "not reported"
      from any instant? [Coverage, Spec FR-003, FR-008a]
- [x] CHK018 Is the injected clock's role bounded to cadence and retention, so
      that it cannot be mistaken for the source of the packet instant?
      [Ambiguity, Spec FR-012]

## Publication and Concurrency (section 11.6)

- [x] CHK019 Is "reads without locking" stated as a checkable property rather
      than an aspiration, given that a short critical section also feels
      lock-free in a test? [Measurability, Spec FR-028, FR-029, SC-006]
- [x] CHK020 Are requirements defined for what a reader sees when a refresh is
      in progress, beyond "never a partial snapshot"? [Clarity, Spec FR-027]
- [x] CHK021 Is the failure-to-read behavior specified for both the first
      refresh, where there is no previous snapshot to keep, and a later one?
      [Coverage, Spec FR-030, Edge Case]
- [x] CHK022 Are the requirements consistent between a refresh that takes
      `&mut self` and lookups that take `&self` from several threads, or does
      satisfying both require something the requirements do not state? [Conflict,
      Spec FR-028, FR-029]
- [x] CHK023 Is the rate-limit requirement stated over requests rather than
      over refreshes, so that a burst cannot drive the table read rate even if
      the owner refreshes promptly? [Ambiguity, Spec FR-015]
- [x] CHK024 Are requirements defined for the interaction between the interval
      and a triggered request, so that a trigger arriving just before the
      interval elapses does not produce two reads? [Gap]

## Loss Accounting (P-4)

- [x] CHK025 Is it stated that no discard path is introduced by this slice, or
      are the discard paths it does introduce each given a named counter?
      [Coverage, Spec FR-025]
- [x] CHK026 Is the distinction between never attempted and attempted and
      unresolved preserved in requirements, given that S07 lost it once and it
      stood for a whole slice? [Completeness, Spec FR-026]
- [x] CHK027 Are requirements defined for a lookup that fails because the
      naming seam failed, as distinct from one that fails because the table had
      no entry? [Gap, Spec FR-032]
- [x] CHK028 Is the conservation identity S08 established stated as continuing
      to hold, rather than assumed to be unaffected? [Consistency, Spec SC-007]

## Naming a Process (P-1)

- [x] CHK029 Is the permitted enumeration mechanism named positively, so that a
      reviewer can distinguish a compliant design from a merely unlisted one?
      [Clarity, Spec FR-033]
- [x] CHK030 Are requirements defined for what access rights any process handle
      in this slice may request, given that naming a process is the classic
      reason to open one? [Gap, Spec FR-033, SC-012]
- [x] CHK031 Is it stated that the socket table read itself requires no handle
      against any target process? [Completeness, Spec FR-034]
- [x] CHK032 Are the requirements clear that the table interface and the
      object-model projection are different mechanisms with the same data,
      rather than an optimization preference? [Clarity, Spec FR-034]
- [x] CHK033 Is the boundary between this slice's naming seam and S11's process
      tree stated in terms of what lands here, rather than only what does not?
      [Consistency, Spec FR-031, Assumptions]

## The Three Recorded Deviations

- [x] CHK034 Is each deviation stated with the specification section it
      diverges from, why the divergence is necessary, and the commitment to
      promote it to section 29? [Completeness, Spec Deviations]
- [x] CHK035 Is the blast radius of adding `Sync` to `FlowAttributor`
      documented, including which existing implementations and call sites it
      constrains? [Gap, Spec Deviations]
- [x] CHK036 Does adding `Sync` conflict with the traits module's own statement
      that its contents are intended to reach 1.0.0 unchanged, and if so is the
      conflict acknowledged rather than silent? [Conflict]
- [x] CHK037 Is the injected clock deviation bounded, so that it does not
      become a general clock abstraction the rest of the workspace must adopt?
      [Ambiguity, Spec Deviations]
- [x] CHK038 Is the creation-instant deviation stated as a change to the
      snapshot's contents rather than to the trait, so that its blast radius is
      not overstated? [Clarity, Spec Deviations]

## Tier Separation and Buildability

- [x] CHK039 Is the feature gating the platform backend named, along with which
      crates declare it and which check commands enable it? [Gap, Spec FR-035]
- [x] CHK040 Is the distinction between "compiled out on a non-Windows target"
      and "off by default everywhere" specified, or are the two conflated?
      [Ambiguity, Spec FR-035, FR-036]
- [x] CHK041 Are requirements stated for how a tier 2 test declares its need
      for the platform, so that it is skipped rather than failed elsewhere?
      [Gap, Spec SC-010]
- [x] CHK042 Is it specified whether `cargo xtask neutral` must cover
      `fragcap-attr`, given that S09 extended it to cover `fragcap-capture` for
      the same reason? [Gap, Consistency]
- [x] CHK043 Are requirements defined for whether the platform backend adds a
      dependency, and if so how it is evaluated against the licence allowlist
      and the denylist? [Gap]

## Scope Boundaries and Vocabulary

- [x] CHK044 Is the boundary against S12 stated, so that role and stage
      remaining absent is a decision rather than an omission? [Clarity, Spec
      Assumptions]
- [x] CHK045 Are the requirements consistent with `ScriptedAttributor`
      remaining the tier 1 attributor for corpus and pipeline tests, rather
      than being superseded here? [Consistency, Spec Dependencies, SC-013]
- [x] CHK046 Is the decision to keep the cadence configuration out of the
      profile schema stated with its reason, so that a later slice does not
      "fix" it by adding keys? [Clarity, Spec FR-011a]
- [x] CHK047 Are the glossary terms this slice introduces identified, so that
      P-6 can be satisfied in the same change rather than discovered at review?
      [Gap, Spec FR-039]

## Notes

- Check items off as resolved: `[x]`. An item resolved by a specification edit
  should name the requirement it added or changed.
- An item that turns out not to apply is struck with a one-line reason rather
  than silently checked. A checklist that only ever gets ticked is not
  measuring anything.
- CHK003, CHK022, and CHK036 are the three most likely to be real. Each names a
  possible internal contradiction rather than a gap, and a contradiction
  survives review more easily than an omission does. CHK022 in particular
  asks whether the requirements as written are satisfiable at all: a trait with
  `&mut self` refresh and `&self` lookups, shared across threads, is not
  obviously expressible without something the requirements have not named.

## Resolution pass, 2026-08-09

Worked through after implementation. Every item is checked, and the ones whose
answer is not obvious from the diff are recorded here, because a checklist that
was only ever ticked would not have been worth writing.

**CHK003 and CHK022 were both real, and both were caught before implementation.**

CHK003 asked whether the order over competing matches is total. It was not:
the specification said "prefer the more exact match" and stopped, which leaves
a wildcard bind against a specific bind, and two sockets on a reused port, both
decided by whatever order the platform reported its rows in. The clarify session
replaced it with a four-rank ladder and two tiebreaks, FR-008 through FR-008b,
and SC-014 turned it into a property under test. `index.rs` resolves the same
flow against the same entries in every rotation and every reversal.

CHK022 asked whether `&mut self` refresh and `&self` lookups from several
threads are jointly satisfiable. They are, and only because the published index
is a separately shareable value rather than a private field. That is not an
implementation preference: a test of concurrent resolution across a publication
has to publish from one thread while others read, which no `&mut self` method on
a shared object can express. Had the index stayed private, SC-006 would have
been unwritable and the requirement would have shipped unverified.

**CHK016 was answered the opposite way from how it was asked.** The item assumed
the creation-instant filter is unavailable for UDP, which is what Appendix D
records. Reading the platform bindings during planning found `liCreateTimestamp`
on `MIB_UDPROW_OWNER_MODULE` as well as on the TCP row, when each table is
requested by owning module. The filter therefore applies to both, and it matters
more for UDP: section 8.4 keys UDP on the local endpoint alone, so it is the
weaker join. Recorded as a deviation and promoted to Appendix D.

**CHK006, the swapped positions.** Not applicable in the form asked, and
provably so. `FlowKey` normalizes the local endpoint at parse time and
`AttributionKey` is derived from it, so a matcher never sees an unnormalized
pair and cannot swap them. Struck rather than ticked would be wrong too: the
requirement is satisfied structurally by a type from S02.

**CHK012, `Fidelity::None`.** This slice never produces it. An unresolved lookup
returns no attribution rather than one carrying `None`, which is what
`AttributionState` distinguishes and what the `attribution.rs` doc comment
already said. Stated in the code rather than left to inference.

**CHK015, a packet older than every snapshot.** Resolvable, and the tests cover
it. The live path does not consult `taken_at` at all, only the entry's creation
instant, so a packet from before the first refresh attributes normally provided
the socket predates it. That is correct: the socket table reports sockets, not
history, and a socket that existed then and still exists now owned the flow.

**CHK019, "reads without locking".** Settled by mechanism rather than by
wording. `arc-swap` load is an atomic pointer read with no lock to wait on, and
research R-3 records why `RwLock<Arc<T>>` was rejected: a reader can block
behind a writer, which would satisfy a test and not the requirement. That is the
distinction the item was worried about, and it is the one that decided the
dependency.

**CHK024, the interval against a triggered request.** `mark_refreshed` clears
any pending request, so a trigger arriving just before the interval elapses is
satisfied by the refresh the interval was about to cause rather than producing a
second read. Tested directly.

**CHK027, a naming failure versus an empty table.** They are not the same and
neither is a lookup failure. `ProcessNamer::names` returns a map rather than a
`Result`, so a name that cannot be resolved produces an attribution carrying the
observed identifier and an empty image name. Reporting nothing would discard an
observation, which P-9 forbids.

**CHK030 and CHK031, process handles.** Nothing in this slice opens one.
`CreateToolhelp32Snapshot` returns a handle to a snapshot object rather than to
any process, and the image name is in the enumeration result. The socket table
read opens nothing at all. `cargo xtask lint` now fails on `OpenProcess`,
`ReadProcessMemory`, and `WriteProcessMemory`, and the assertion was verified to
fire by temporarily introducing a violation and watching lint report it.

**CHK035 and CHK036, the `Sync` blast radius.** Both existing implementors,
`ScriptedAttributor` and the pipeline's test stubs, were already `Sync` and
neither changed. `Pipeline::new` kept its `Box` parameter, so no caller changed
either. The conflict CHK036 names is real and acknowledged rather than silent:
`traits.rs` states the surface is intended to reach 1.0.0 unchanged, and this is
the second slice to change it. The mitigating argument, recorded there, is that
a bound every implementor already satisfies is a far smaller commitment than a
method, which is the same argument S09 made.

**CHK039 through CHK043, tier separation.** The feature is `socket-table`, off
by default, and deliberately not `live`; the analyze gate caught that collision
and the reasoning is in the S10 decisions fragment. `cargo xtask neutral` was
extended to build `fragcap-attr`, and it passes for
`x86_64-unknown-linux-gnu`. `cargo xtask msrv` passes at 1.82 with both new
dependencies. Tier 2 tests are `#[ignore]`, and they were run: see below.

**CHK041, and the thing worth saying loudest.** The tier 2 tests are not merely
skippable, they have been executed. A real socket was opened on this machine,
found in the machine's real socket table, attributed to the process that opened
it with `Fidelity::Live`, then closed and observed to resolve as
`Fidelity::Retained`. S09 could not make that claim about its live source and
still cannot. The difference is that this backend needs no capture driver and no
elevation, which is also why its workflow step is the first in `platform.yml`
that can go green on a bare runner.

**CHK047, the glossary.** Five entries written in this change, and the existing
socket table entry extended with a primary-source reference and the
owning-module distinction, per P-6.
