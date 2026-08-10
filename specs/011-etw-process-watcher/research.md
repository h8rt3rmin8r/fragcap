# Research: ETW Process Watcher and Tree (S11)

**Slice**: S11 | **Date**: 2026-08-09 |
**Spec**: [spec.md](spec.md)

Seven questions had to be answered before the plan could be written. Six were
answered by measurement on this machine and one by reading the architecture of
record. Every claim below that says a thing compiles or resolves was checked by
running the command, and the commands are reproduced so a reviewer can run them
again.

The probes live outside the repository, in the session scratchpad, so nothing
here adds a crate to the workspace before the plan says it should.

## R-1: How fragcap consumes process telemetry

**Decision.** `windows-sys`, with default features off. Not `ferrisetw`, which
specification Appendix A names, and not `windows`.

**Amended during integration, 2026-08-09.** This research selected version 0.61
and the slice shipped on 0.36. S10 merged first and had pinned the workspace to
the 0.36 line, matching what `pcap` already resolves, so that the socket table
backend added no package to `Cargo.lock` at all. Taking 0.61 for the watcher
would have put a second complete `windows-sys` tree in the graph. Every symbol
this slice names was checked against 0.36 and resolves there, with
`Win32_System_Diagnostics_Etw` and `Win32_System_Time` as the feature groups.
Two differences are real and are handled in the code: the 0.36 line predates the
handle newtypes, so a trace handle is a plain `u64`, and it has no
`GUID::from_u128`, so both provider identifiers are written out field by field.
The comparison below is left as it was measured, because the argument against
`ferrisetw` does not depend on which `windows-sys` line is taken.

### What was measured

Three candidates were resolved and their graphs counted. All three are MIT OR
Apache-2.0, which the constitution's licensing section permits.

| Candidate | Version | Crates in graph | Declared minimum | Publisher |
| --- | --- | --- | --- | --- |
| `windows-sys` | 0.61.2 | **2** | 1.71 | Microsoft |
| `windows` | 0.62.2 | 14 | 1.82 | Microsoft |
| `ferrisetw` | 1.2.0 | 30 | not declared | third party |

The `windows-sys` graph is `windows-sys` itself and `windows-link` 0.2.1.
Nothing else, and neither has a build script.

The `ferrisetw` graph carries `rand`, `getrandom`, `ppv-lite86`, `zerocopy`,
`num-bigint`, `num-complex`, `num-rational`, `num-iter`, `bitflags` 1.3,
`widestring`, `memoffset`, `once_cell`, `byteorder`, a proc-macro derive, and
`windows` 0.57, which is five releases behind the current line.

Reproduce with:

```sh
cargo new --lib probe && cd probe
cargo add windows-sys --no-default-features \
  --features Win32_Foundation,Win32_System_Diagnostics_Etw,Win32_System_Threading,Win32_System_Diagnostics_ToolHelp,Win32_System_Time
cargo tree -e normal --prefix none | sort -u
```

### That the surface exists was checked rather than assumed

A probe naming every symbol S11 needs was compiled against `windows-sys`
0.61.2 under `rustup run 1.82`, which is the workspace minimum. It builds
clean. The symbols are:

- Session control: `StartTraceW`, `ControlTraceW`, `EnableTraceEx2`
- Consumption: `OpenTraceW`, `ProcessTrace`, `CloseTrace`
- Types: `EVENT_TRACE_PROPERTIES`, `EVENT_TRACE_LOGFILEW`, `EVENT_RECORD`
- Modes: `EVENT_TRACE_REAL_TIME_MODE`, `EVENT_TRACE_SYSTEM_LOGGER_MODE`,
  `PROCESS_TRACE_MODE_REAL_TIME`, `PROCESS_TRACE_MODE_EVENT_RECORD`
- Snapshot: `CreateToolhelp32Snapshot`, `PROCESSENTRY32W`, `Process32FirstW`,
  `Process32NextW`
- Start time: `OpenProcess`, `GetProcessTimes`,
  `PROCESS_QUERY_LIMITED_INFORMATION`, `FILETIME`, `CloseHandle`

One finding is worth carrying into implementation, because it costs an hour to
rediscover: **`EVENT_TRACE_LOGFILEW` and `OpenTraceW` are gated behind the
`Win32_System_Time` feature**, not behind `Win32_System_Diagnostics_Etw`. The
first probe failed to compile for exactly that reason and the compiler named the
feature, which is the kind of error worth having early.

### Why not `ferrisetw`, which Appendix A names

Appendix A's dependency column was written before any of this was measured and
is indicative rather than binding. Three things decide against it.

**The S09 argument points the other way here.** S09 took `pcap` rather than
hand-rolling a binding, and the reason it gave was precise: the alternative was
"a C ABI whose struct layouts must be transcribed by hand with nothing checking
them against the header", and a wrong offset yields plausible values that are
wrong. That argument is about who guarantees the layout. `windows-sys` is
generated from the Windows metadata by Microsoft, so it is the binding that
carries the guarantee. `ferrisetw` is a convenience layer over the same
guarantee, held one version line behind it.

**The graph is disproportionate to the job.** A passive observer that reads
process start and exit events does not need a bignum stack or a random number
generator. Every crate in a graph is a crate `cargo deny` must clear, a crate
whose minimum toolchain can rise, and a crate whose maintenance is somebody
else's. Thirty against two is not a close call at this size of task.

**What it would save is small.** `ferrisetw`'s value is its schema parsing over
manifest-based providers, and the kernel process provider is not one. Its events
are fixed MOF layouts, so the parsing S11 needs is field offsets into one
structure whose shape is documented and stable, not a general property walk.

**What was not established.** `ferrisetw` was not shown to build under 1.82. The
build failed at `windows_x86_64_msvc` 0.52.6's build script with `LNK1104`,
which reproduced identically under the stable toolchain, so it is an artifact of
this machine's scratch directory rather than a fact about the crate. It is
recorded as unknown rather than as a mark against the candidate, because the
decision does not rest on it.

### Why not `windows`

`windows` 0.62.2 is the same metadata with COM plumbing, `Result` wrappers, and
a proc-macro layer on top. Seven times the graph for ergonomics that an ETW
consumer, which is a raw callback with a raw record pointer, does not use. Its
declared minimum is 1.82, exactly the workspace floor, which leaves no headroom;
`windows-sys` declares 1.71.

### Consequences accepted

The code is `unsafe` at the boundary, and there is more of it than a wrapper
would leave. That is confined to one module, every call is checked against its
documented failure return, and the alternative is not less unsafe code but the
same unsafe code inside a dependency, one version line behind, with a bignum
stack attached.

`sysinfo`, the other crate Appendix A names, is not taken either. Its process
enumeration would serve, but at the cost of a graph carrying `ntapi`, `libc`,
`memchr`, and the Apple object-model crates, to obtain what four calls already
in the chosen dependency supply.

## R-2: How the session avoids the machine-wide singleton

**Decision.** A private session named by fragcap, started with
`EVENT_TRACE_SYSTEM_LOGGER_MODE` set, carrying the kernel process provider.
Never `NT Kernel Logger`.

The classic kernel session is one instance per machine. Taking it makes fragcap
fail whenever any other tool is tracing, and taking it by force makes fragcap
the tool that breaks the operator's other instrumentation. FR-005 forbids both.

Windows 8 introduced the system logger mode, under which several sessions may
each carry system providers concurrently, subject to a small fixed limit. The
mode constant resolves in `windows-sys` and is in the probe above.
Specification section 6.1's platform floor is Windows 10, so the mode is always
available on a supported target.

Two consequences follow. The session gets a name unlikely to collide, and the
limit on concurrent system loggers is a real exhaustion condition that FR-016
already covers: the platform's own reason is relayed rather than replaced.

## R-3: How the snapshot obtains its fields without memory rights

**Decision.** `CreateToolhelp32Snapshot` for the enumeration, and nothing else.
No start time and no command line.

**Amended during integration, 2026-08-09.** This research chose
`OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` with `GetProcessTimes` for a
start time, and it was withdrawn. The right carries no memory access and naming
it at the call site is exactly what P-1 asks for, so the original choice
complied. S10 merged first with the stronger version of the same argument, made
in its own `platform::toolhelp` module and enforced by a lint entry forbidding
`openprocess` anywhere in fragcap: P-1's requirement exists because a handle
request is a thing a reviewer has to check, and opening nothing removes the
thing to check rather than documenting it. S11 kept that rule rather than
deleting the lint entry, which S10 explicitly invited a later slice to do. A
process found already running therefore has no start time, which FR-009 records
as unknown and FR-024 already gave a defined meaning in resolution.

The enumeration yields the identifier, the parent identifier, and the executable
file name for every process, and needs no handle on any target. That satisfies
FR-008's query-only requirement trivially, because it opens nothing.

A start time would need a handle, and the narrowest right Windows defines,
`PROCESS_QUERY_LIMITED_INFORMATION`, carries no memory access, so obtaining one
would comply with P-1. It is not obtained, per the amendment above: a process
found already running has no start time, the node records that, and FR-024
defines what an unknown start time means in a resolution. Nothing here opens a
handle against a target, so there are no access rights to state.

**The command line is not obtainable this way, and that is why FR-036 exists.**
Reading a running process's command line means reading its process environment
block, which requires `PROCESS_VM_READ`. That right is on the denylist. So a
process the snapshot finds has no command line, records that it has none, and
gains one only if it is still running when its start event would have arrived,
which it is not. The alternative routes were considered and rejected: WMI's
process class supplies command lines without a memory right, but brings a COM
dependency and a query costing over a second, which Appendix D.1 measured for
the analogous socket case and which is not worth paying for a field that only
ever applies to processes that started before fragcap did.

This is the honest shape of the constraint rather than a limitation to work
around. A process that was already running is not a member of the launcher chain
fragcap was started to watch, because that chain begins after fragcap does.

## R-4: Where the tree lives, and what it is

**Decision.** `fragcap-core::process::tree`, a value with no I/O.

Settled in the clarification session and restated here because it is the
structural decision the slice rests on. The tree is a fold over `ProcessEvent`,
so section 10.2 is testable at tier 1 on any machine. `interface::select`
established the shape in S09 for the same reason, and section 8.2 places types
and pure logic in core.

The alternative, keeping the tree beside the watcher in `fragcap-attr`, would
gate every test of ancestry, retention, and identifier recycling behind an
elevated Windows session, and would leave S12's stage matching with nowhere to
be tested either.

It also happens to keep this slice out of S10's way, which is developing in
`fragcap-attr` in parallel. That is a convenience rather than a reason, and it
is recorded as such so that nobody later mistakes it for the argument.

## R-5: What the feature gate is called

**Decision.** `etw`, declared by `fragcap-attr`, off by default.

Symmetric with S09's `live` on `fragcap-capture`, and named for the mechanism
rather than the capability for the same reason: a feature called `process` would
suggest the tree is behind it, and the tree is not behind anything.

`cargo xtask ci` does not enable it, so the ordinary check set builds and runs
on a machine with no elevation and no Windows. The `platform` workflow enables
it, which is where the tier 2 tests live.

`cargo xtask neutral` currently builds `fragcap-core` and, since S09,
`fragcap-capture`. It should build `fragcap-attr` too, for the reason S09
extended it: a crate that must build without its backend is a crate nothing was
checking.

## R-6: How an event timestamp becomes a `Timestamp`

**Decision.** Set the session's client context to system time, and convert the
`FILETIME` exactly.

`EVENT_RECORD`'s header timestamp means whatever the session's client context
says it means: query performance counter, system time, or CPU cycle counter. The
default for a real-time session is the performance counter, which is monotonic
but has no relationship to the wall clock, and therefore no relationship to the
packet timestamps a capture driver supplies. Leaving it at the default would
produce process events that cannot be placed against packets, which is the whole
point of having them.

Setting the client context to system time yields a `FILETIME`, which is 100
nanosecond intervals since 1601-01-01 UTC. `Timestamp` is nanoseconds since the
Unix epoch as an `i64`. The conversion is
`(filetime - 116_444_736_000_000_000) * 100`, exact in integer arithmetic with
no rounding and no floating point anywhere.

The offset is a named constant with the epoch difference spelled out, because a
magic number here is the kind of wrong that produces plausible timestamps.

## R-7: What is deferred, and to where

- **Stage matching, lifecycle classes, session lifecycle, stop conditions.**
  Sections 10.3 through 10.6, all S12. The tree reserves a place for a matched
  stage and puts nothing in it.
- **The control thread.** Section 8.6 puts the watcher, the tree, the
  attributor, and the filter manager on one thread. Two of those four do not
  exist. S13 and S14 assemble it.
- **Presentation.** What an operator sees of the watcher's report is S14's
  `doctor` and run reporting. This slice supplies the values.
- **A process fixture corpus.** The two Appendix D chains are expressed as
  scripts in tests rather than as committed fixture files. A file format for
  process scripts would be speculative until S12 shows what a matcher needs to
  be tested against.
