# Implementation Plan: ETW Process Watcher and Tree

**Branch**: `claude/s11-s12-parallel-dev-086100` | **Date**: 2026-08-09 |
**Spec**: [spec.md](spec.md)

**Input**: Feature specification from
`/specs/011-etw-process-watcher/spec.md`

## Summary

S11 gives fragcap its first look at the machine it runs on. A `ProcessWatcher`
backed by an ETW session it creates for itself observes every process start and
exit system wide, and a process tree in `fragcap-core` folds those observations
into the ancestry relation that sections 10.3 through 10.6 will consume.

The split is the plan's central decision and everything else follows from it.
The watcher touches the platform and lives in `fragcap-attr` behind a feature
that is off by default. The tree touches nothing and lives in `fragcap-core`, so
the whole of specification section 10.2 is a tier 1 test on any machine. A
scripted watcher replays the two launcher chains reconnaissance actually
observed, which makes the Division 2 case, where three processes share an image
name and only the last transmits, testable without Windows, elevation, or a
game.

The platform binding is `windows-sys`, not the `ferrisetw` that Appendix A
names. Research R-1 measured why: two crates against thirty, published by
Microsoft from the Windows metadata rather than wrapping it a version line
behind, and a declared minimum of 1.71 against this workspace's 1.82.

## Technical Context

**Language/Version**: Rust 2021 edition, workspace minimum 1.82

**Primary Dependencies**: `windows-sys` 0.36, already in the workspace for S10,
optional, behind the `etw` feature. No new package in `Cargo.lock`, and no new
dependency for `fragcap-core`.

**Storage**: None. The tree is in memory and lives for the session.

**Testing**: `cargo test`, tier 1 for everything in `fragcap-core::process` and
for the scripted watcher, tier 2 for the ETW watcher.

**Target Platform**: Windows 10 and later for the watcher. Any target with the
standard library for the tree.

**Project Type**: Rust workspace, library crates plus a task runner.

**Performance Goals**: None stated as a target. The relevant quantity is that
the watcher must not miss a process, which is a correctness property rather than
a performance one, and is why polling is refused.

**Constraints**: No process handle carrying memory rights. No polling. No
machine-wide singleton session. Command lines verbatim. `fragcap-core` stays
platform-neutral.

**Scale/Scope**: A working machine runs thousands of processes over a session.
Reconnaissance scanned 3,694 command lines across two sessions of about twenty
minutes each. The tree retains every node for the session and reports how many
it holds.

## Constitution Check

*GATE: passed before Phase 0 research, re-checked after Phase 1 design.*

**P-1, Passive Observation Only.** The two mechanisms are ETW kernel providers
and query-only process enumeration, which are entries one and four on the
section 19.2 allowlist. **This slice opens no handle against any target process
at all.** It first opened one with `PROCESS_QUERY_LIMITED_INFORMATION` to read a
start time, which P-1 permits, and withdrew it at integration in favour of the
stronger rule S10 had already established and lint-enforced; see D-11. The only
handle taken anywhere here is to a snapshot object. `cargo xtask lint` fails on
`openprocess` and on four memory-bearing rights. **Pass, and mechanically.**

The command line is where this principle was most at risk, because the obvious
way to obtain one for a running process is to read its process environment
block, which needs `PROCESS_VM_READ`. Research R-3 records the refusal and its
consequence: a snapshot node has no command line, and says so.

**P-2, Core Stays Platform-Neutral.** The tree is arithmetic and collections
over types `fragcap-core` already has. `windows-sys` is optional and declared by
`fragcap-attr` alone. `cargo xtask neutral` is extended to build `fragcap-attr`,
so the claim is checked rather than asserted. **Pass.**

**P-3, Capture And Attribution Stay Separate.** The watcher names neither
`PacketSource` nor `FlowAttributor`. It is a third thing beside them, which is
what section 8.6 already shows. **Pass.**

**P-4, No Silent Loss.** Three answers, and the third is the one worth reading.
Events the kernel reports losing are counted in `WatcherReport`. Exits that
never find a start are counted. And the channel between the consumer and its
subscribers has no discard path at all, because it is unbounded: a start event's
loss costs a subtree, unlike a packet's, so the bounded drop-oldest buffer of
section 12.4 would be the wrong shape here. Adding a bound with a counter would
satisfy the letter of P-4 while introducing the loss it exists to prevent.
**Pass.**

**P-5, Compatibility Outranks Richness.** No output format changes. **Not
engaged.**

**P-6, Glossary First.** Six terms, listed in D-9, written in this change.
**Pass.**

**P-7, Wrappers Stay Thin.** No wrapper changes. **Not engaged.**

**P-8, House Standards Apply.** `CONVENTIONS.md` binds. **Pass.**

**P-9, The Instrument Does Not Lie.** Four places, and all four were decisions
rather than defaults. Command lines are verbatim. An unavailable command line is
recorded as unavailable rather than as empty. An unknown start time is recorded
as unknown rather than as the session start. And a tree that may have a hole in
it reports itself incomplete rather than presenting as whole. **Pass.**

The refusal of a polling fallback belongs here too. A degraded mode that misses
transient launchers would produce a capture that is silently about the wrong
process, which is P-9's failure at the level of the whole run.

**Licensing.** `windows-sys` 0.36.1, already in the graph for S10 and `pcap`,
is MIT OR Apache-2.0, on the allowlist. This slice adds no package to the lock
file, and nothing in the graph is a packet interception library. **Pass.**

## Decisions

### D-1: `windows-sys` supplies the platform binding

Measured in research R-1. Two crates against `ferrisetw`'s thirty, Microsoft's
own generated binding rather than a wrapper over it, and a declared minimum
comfortably under the workspace's 1.82. A probe naming every symbol the slice
needs compiles under `rustup run 1.82`.

This diverges from specification Appendix A, which names `ferrisetw` and
`sysinfo`. Appendix A's dependency column is indicative and predates any
measurement. Recorded as a deviation and promoted at the next version.

**The version is 0.36, not the 0.61 this plan first chose.** S10 merged first
and had pinned the workspace to the line `pcap` already resolves, so that its
socket table backend added no package to `Cargo.lock`. Taking 0.61 here would
have put a second complete tree in the graph for declarations that have not
changed. Every symbol the watcher names was checked against 0.36 before the
change was made.

Feature groups added by this slice, default features off:
`Win32_System_Diagnostics_Etw` and `Win32_System_Time`. `Win32_Foundation` and
`Win32_System_Diagnostics_ToolHelp` were already there for S10.

The second is not obvious and is recorded so nobody rediscovers it:
`EVENT_TRACE_LOGFILEW` and `OpenTraceW` are gated behind `Win32_System_Time`,
not behind the ETW feature group. That holds on both lines.

### D-2: One feature, `etw`, off by default

Symmetric with S09's `live`. Named for the mechanism rather than the capability,
because a feature called `process` would suggest the tree is behind it, and the
tree is behind nothing.

`cargo xtask ci` does not enable it. The `platform` workflow does.

### D-3: The tree lives in `fragcap-core`, the watcher in `fragcap-attr`

The structural decision. Section 10.2 becomes a tier 1 test on any machine, and
S12's stage matching, which is a decision over a tree, becomes testable at all.
`interface::select` established the shape in S09.

It also keeps this slice clear of S10, which is developing in `fragcap-attr` in
parallel. That is a convenience and not the reason, and it is written down as
such so that nobody later mistakes one for the other.

### D-4: A private session with the system logger mode, never the kernel logger

The classic kernel session is one instance per machine. Contending for it makes
fragcap fail whenever another tool is tracing; taking it makes fragcap the tool
that breaks the operator's other instrumentation. Both are refused by FR-005.

Windows 8 and later permit several concurrent system loggers, subject to a small
fixed limit, and the platform floor is Windows 10. Exhausting that limit is a
real condition and is relayed with the platform's own reason.

### D-5: Subscribe first, then snapshot

Settled in clarification. The two orders fail differently: subscribing first can
report a process twice, which the tree reconciles, while snapshotting first can
miss a process entirely, which nothing downstream can detect. A visible
duplicate beats an invisible gap, and an invisible gap in a launcher chain is
the failure this slice exists to prevent.

### D-6: `CommandLine` is an enum, not an `Option`

`Option<Arc<str>>` invites `unwrap_or_default`, which converts "not observed"
into "was empty" at one call site and loses the distinction permanently. The
enum makes the substitution something an author has to write on purpose, which
is what FR-036 asks for and what P-9 is about.

The same reasoning does not apply to `started`, which stays
`Option<Timestamp>`, because there is no plausible default for a timestamp that
a careless call site would reach for. Its `None` gains a defined meaning in
resolution instead.

### D-7: The event channel is unbounded

Section 12.4's bounded drop-oldest buffer is the right shape for packets and the
wrong shape here. Packets arrive faster than they can be written and losing an
old one costs one packet. Process events arrive in the thousands over a session
and losing one costs a subtree.

There is therefore no discard path to count, which is how P-4 is satisfied for
this stream. A future reviewer who wants to bound it should read this decision
and FR-013 first, because the counter they would add would be counting the loss
the bound introduced.

### D-8: Ancestry provenance is stored, not derived

Three states rather than two, because "no parent resolved" is different from
both "observed at creation" and "read from a snapshot". S06 learned this about
attribution fidelity, which it initially derived from whether an attribution
existed, and which review caught claiming a live socket-table hit for a
resolution that came from a text file.

### D-9: Terms introduced, per P-6

Six entries, written in the same change:

- Synthetic process identifier
- Process node
- Ancestry provenance
- Startup snapshot
- Trace session
- Lost event

`ETW`, `process tree`, `PID recycling`, and `launcher chain` already have
entries and gain cross-links rather than duplicates.

### D-10: The `platform` workflow changes, with a dated decision

Three changes, all to a pinned artifact and therefore recorded as a dated
decision fragment in `changelog.d`:

1. `crates/fragcap-attr/**` is added to the path triggers, because a change
   there can now change the answer.
2. A step builds `fragcap-attr --features etw`, which is what proves the binding
   links, in the same way S09's build step proved `wpcap.lib` was acquired.
3. Tier 2 tests are gated on a runtime elevation check rather than assumed. The
   runner may or may not be elevated, and S09's lesson about
   `STATUS_DLL_NOT_FOUND` is that a workflow which assumes its precondition goes
   red for a reason that has nothing to do with the code. The check reports
   plainly which case it took, so a run that did not exercise the watcher does
   not look like one that did.

### D-11: `cargo xtask lint` gains the memory-rights check

The transmit-call check S09 added is the precedent and the argument is the same.
P-1's most important claim about this slice, that fragcap never reaches inside a
process, should be mechanical rather than remembered. The check fails if any
fragcap source names `PROCESS_VM_READ`, `PROCESS_VM_WRITE`,
`PROCESS_VM_OPERATION`, or `PROCESS_ALL_ACCESS`.

**S10 got there first and went further.** It merged with a lint forbidding
`openprocess` outright, on the ground that P-1's rule about stating access
rights exists because a handle request is a thing a reviewer has to check, and
opening nothing removes the thing to check. That entry conflicted with this
slice's startup snapshot, which opened one with the narrowest right Windows
defines. S10's own comment invited a later slice to delete the entry and argue
for it; this slice declined, dropped the call, and gave up the start time it
bought. The four constants remain as a complement rather than a duplicate: a
right can be named where the call is not, and they are what stops a future slice
that does delete the `openprocess` line from quietly asking for memory.

`cargo xtask neutral` is extended to build `fragcap-attr` for the same reason
S09 extended it to `fragcap-capture`. The check is a match arm in
`xtask/src/main.rs`, not a module of its own.

**The other P-1 claim is not mechanized, and that is a considered choice rather
than an omission.** FR-011 forbids a polling fallback, and SC-008 verifies it by
inspection. A lint could forbid the names a poller would use, but the names a
poller would use are `Duration`, `interval`, and `loop`, all of which appear
legitimately throughout the workspace. A check with that false-positive rate
gets suppressed, and a suppressed check is worse than an honest inspection
because it looks like a guarantee. The memory-rights check is mechanized because
its forbidden names are four constants that have exactly one meaning. Recorded
here so that the asymmetry is a decision on the record rather than something a
later reviewer discovers and assumes was overlooked.

### D-12: No file format for process scripts

S04's attribution script has a text format because a committed fixture corpus
needed one. A process script has two users, both of them Appendix D chains in
this slice's own tests, and a file format for two in-code callers would be
speculative. S12 will show what a matcher needs to be tested against; if a
format is warranted, it lands there with a reason.

## Project Structure

### Documentation (this feature)

```text
specs/011-etw-process-watcher/
├── plan.md               # This file
├── spec.md               # The feature specification
├── research.md           # Phase 0: seven questions, six measured
├── data-model.md         # Phase 1: types added, changed, and left alone
├── quickstart.md         # Phase 1: how to check this slice
├── contracts/
│   └── process-api.md    # Phase 1: the surface S12 plans against
├── checklists/
│   ├── requirements.md   # Specification quality
│   └── observation.md    # Domain checklist, P-1 and P-9 focused
└── tasks.md              # Phase 2, from /speckit-tasks
```

### Source Code (repository root)

```text
crates/fragcap-core/src/
├── process/
│   ├── mod.rs            # moved from process.rs, then extended: ProcessId,
│   │                     #   NodeId, Ancestry, CommandLine, WatcherReport,
│   │                     #   and the changed ProcessEvent
│   └── tree.rs           # new: ProcessNode, ProcessTree, the fold
└── lib.rs                # re-exports

crates/fragcap-core/tests/
└── process_tree.rs       # new: the section 10.2 invariants at tier 1

crates/fragcap-attr/src/
├── lib.rs                # extended: two module declarations, two re-exports
├── proc_script.rs        # new: ProcessScript, ScriptedWatcher (no feature)
└── etw/                  # new, behind the `etw` feature
    ├── mod.rs            #   EtwWatcher, WatcherError
    ├── session.rs        #   StartTraceW, EnableTraceEx2, teardown
    ├── consumer.rs       #   OpenTraceW, ProcessTrace, the callback
    ├── record.rs         #   the process event layout, FILETIME conversion
    └── snapshot.rs       #   Toolhelp enumeration, no process handle

crates/fragcap-attr/tests/
├── chains.rs             # new: the two Appendix D chains, tier 1
└── etw_live.rs           # new: tier 2, ignored by default

xtask/src/
├── lint.rs               # extended: the memory-rights check
└── main.rs               # extended: the `neutral` arm builds fragcap-attr

.github/workflows/
└── platform.yml          # extended: triggers, build step, elevation gate

docs/glossary.md          # six entries
changelog.d/
├── S11-etw-process-watcher.added.md
└── S11-etw-process-watcher.decisions.md
```

**Structure Decision**: The existing workspace layout, extended along the seam
section 8.2 already draws. `fragcap-core` gains the tree because it is a value;
`fragcap-attr` gains the watcher because it touches the platform. The `etw`
directory is a module rather than a file because the four concerns inside it,
session lifetime, consumption, record layout, and enumeration, fail differently
and are worth reviewing separately.

`AGENTS.md` is deliberately not in the tree above. S10 is developing in parallel
and its "Current state" narrative is the file both slices would rewrite. This
slice's narrative goes into its decisions fragment and is folded into
`AGENTS.md` by whichever pull request merges second.

## Complexity Tracking

No constitution violations, so this table records the two places where the
simpler option was rejected on grounds a reviewer should be able to check.

| Choice | Why | Simpler alternative rejected because |
| --- | --- | --- |
| Raw `windows-sys` over `ferrisetw` | The layout guarantee lives in Microsoft's generated binding, which is what S09's argument for taking `pcap` actually points at | A wrapper adds 28 crates, including a bignum stack and a random number generator, to obtain schema parsing the kernel process provider does not use |
| Tree split from watcher across two crates | Section 10.2 becomes tier 1 testable, and S12 becomes testable at all | Keeping them together needs elevated Windows for every test of ancestry, retention, and recycling, and leaves S12 with nowhere to be tested |
