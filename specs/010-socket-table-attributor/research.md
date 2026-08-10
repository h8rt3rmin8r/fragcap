# Phase 0 Research: Socket Table Attributor

**Slice**: S10 | **Date**: 2026-08-09 |
**Spec**: [spec.md](spec.md)

Five questions had to be answered before the design could be fixed. Each is
recorded with what was checked, not only what was concluded, because three of
them contradict something the repository currently believes.

## R-1. How is the socket table read, and what does it carry?

**Checked** against the `windows-sys` 0.36.1 bindings already present in
`Cargo.lock`, at
`src/Windows/Win32/NetworkManagement/IpHelper/mod.rs`.

**Finding.** The IP Helper API exposes `GetExtendedTcpTable` and
`GetExtendedUdpTable`, each taking a table class that selects the row shape.
Two classes matter.

| Class | Row | Carries |
| --- | --- | --- |
| `TCP_TABLE_OWNER_PID_ALL` | `MIB_TCPROW_OWNER_PID` | state, local, remote, owning pid |
| `TCP_TABLE_OWNER_MODULE_ALL` | `MIB_TCPROW_OWNER_MODULE` | the above plus `liCreateTimestamp` and module info |
| `UDP_TABLE_OWNER_PID` | `MIB_UDPROW_OWNER_PID` | local, owning pid |
| `UDP_TABLE_OWNER_MODULE` | `MIB_UDPROW_OWNER_MODULE` | the above plus `liCreateTimestamp` and module info |

**Decision.** Request both tables by owning module. The creation instant is
what FR-009 is built on, and the owning-module class is the only one that
carries it.

**Consequence, and a correction to Appendix D.** The specification's Appendix
D.1 records the creation timestamp as a property of the TCP table and derives
the narrowing of the section 11.3 race window from that. `MIB_UDPROW_OWNER_MODULE`
carries `liCreateTimestamp` too, at offset three in the struct. The reconnaissance
session presumably read the table by owning process identifier, which is the
common example in the platform's own documentation and the class where the
asymmetry is real.

This is worth more than a footnote. A UDP attribution key is the local endpoint
alone, per section 8.4, because the table reports no remote for a datagram
socket. That makes UDP the weaker of the two joins and the one where a reused
port is least distinguishable, so it is exactly the protocol that benefits most
from being able to reject a socket created after the packet. Recorded as a
deviation and promoted to Appendix D.

**Cost.** An owning-module row is roughly 150 bytes larger than an
owning-process-identifier row, because `OwningModuleInfo` is sixteen `u64`
values. Against Appendix D's roughly 1800 sockets that is about 270 kilobytes
of additional copying per snapshot, once per second. Set against a measured one
to three milliseconds per snapshot and a budget of one second, this is not a
consideration. The module info itself is ignored; resolving it would require
`GetOwnerModuleFromTcpEntry`, which is a separate call per row and is not
needed, because process naming is answered more cheaply below.

**Alternatives rejected.** Declaring the functions and structs by hand with
`extern "system"`. Rejected on the same grounds S09 rejected it for `pcap`: a
transcribed C ABI has nothing checking its field offsets against the header,
and a wrong offset here yields a plausible process identifier that is wrong,
which is the P-9 failure class that no test over synthetic data catches. The
object-model projection of the same data is rejected by section 11.2 on
measured grounds, and the specification says so explicitly to stop an
implementation reaching for it.

## R-2. How is a process identifier turned into an image name, without a handle?

**Checked** against `windows-sys` 0.36.1 at
`src/Windows/Win32/System/Diagnostics/ToolHelp/mod.rs`.

**Finding.** `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)` followed by
`Process32FirstW` and `Process32NextW` yields a `PROCESSENTRY32W` per process
carrying `th32ProcessID`, `th32ParentProcessID`, and `szExeFile`.

**Decision.** Use it. The image name is in the enumeration result, so no handle
against any target process is opened at any point.

**Why this is the P-1 answer rather than merely a P-1-compatible one.** The
obvious alternative, `OpenProcess` followed by `QueryFullProcessImageNameW`,
would also comply: `PROCESS_QUERY_LIMITED_INFORMATION` carries no memory
rights, and stating it at the call site is what the constitution requires.
But the constitution's requirement exists because a handle request is the thing
a reviewer has to check, and the toolhelp path removes the thing to check
rather than documenting it. `cargo xtask lint` can then assert that this slice
opens no process handle at all, which is a stronger and cheaper guarantee than
asserting that every handle it opens requests the right rights.

**Consequence for S11.** The parent identifier is right there in the same
struct, and creation-time ancestry is what P-1 directs S11 to use. This slice
does not use it, because ancestry without the ETW creation events is a snapshot
of a tree rather than a record of how it was built, and section 10 wants the
latter. Noted so S11 does not have to rediscover the enumeration.

## R-3. How is the snapshot published so that lookups do not lock?

**The requirement.** Section 11.6: the control thread builds a new map per
refresh and publishes it atomically, and the capture thread reads the current
snapshot without locking. FR-028 and FR-029 restate it.

**Options.**

| Option | Reads block? | Cost | New packages |
| --- | --- | --- | --- |
| `Mutex<Arc<Index>>` | Yes, readers serialize with each other | Uncontended lock per packet | 0 |
| `RwLock<Arc<Index>>` | Only during a publish | Read lock per packet | 0 |
| `AtomicPtr` by hand | No | Reclamation must be written, with `unsafe` | 0 |
| `arc-swap` | No | An atomic load per packet | 1 |

**Decision.** `arc-swap`.

**Reasoning.** The first option is what S08 shipped and what section 11.6
forbids. The second is the tempting one and it is a lock: a `std::sync::RwLock`
read acquisition can block behind a writer, and the reader here is the
acquisition path that section 11.6 exists to keep unblocked. It would satisfy a
test and not the requirement, which is the worst of the four outcomes because
it looks like the requirement was met. The third is correct and requires
writing a reclamation scheme with `unsafe`, in a repository that currently
contains none outside a platform binding.

`arc-swap` is dual MIT and Apache-2.0 and is edition 2018 with no declared
minimum toolchain, so it cannot move `cargo xtask msrv`. It supplies exactly
the one property the specification
asks for and nothing else, which is the test AGENTS.md applies to a proposed
dependency: name the property the dependency supplies that arithmetic does not.
Here it is wait-free reads across an atomic pointer swap.

**A note for whoever proposes removing it.** The question to answer is not
whether `RwLock<Arc<T>>` is fast enough. It is whether a reader may be blocked
by a writer at all, and section 11.6 answers no.

**Corrected during implementation.** This section first claimed `arc-swap` has
no dependencies, from reading an empty `[dependencies]` table in its manifest.
It has one, `rustversion`, which is a build-time proc macro used to gate code
on the compiler version. Resolving the manifest added two packages to
`Cargo.lock` rather than the predicted one. `rustversion` is MIT or Apache-2.0,
declares Rust 1.31, and contributes nothing to the built artifact. The
conclusion is unchanged and the arithmetic that produced it was wrong, which is
worth recording rather than quietly fixing: reading only `[dependencies]`
misses `[build-dependencies]`, and a dependency audit that does that will
under-report every proc macro in the graph.

## R-4. What version of the platform bindings?

**Checked** `Cargo.lock`. `windows-sys` 0.36.1 is already present, pulled in by
`pcap`, along with its five architecture-import crates. `windows-link` 0.2.1 is
present separately.

**Decision.** Declare `windows-sys = "0.36"`, matching what is already
resolved, with only the features this slice needs.

**Reasoning.** Taking the current 0.61 line would put a second, complete
`windows-sys` tree in the graph alongside the one `pcap` pins, for bindings
that have not changed: `GetExtendedTcpTable` and `MIB_TCPROW_OWNER_MODULE` are
the same declarations in both. Matching the resolved version adds no package to
`Cargo.lock` at all, which means `cargo deny` has no new subject and the
licence position is unchanged. Its declared minimum toolchain is 1.64, well
under this workspace's 1.82.

**The risk, stated.** If `pcap` later requires a newer `windows-sys`, the graph
gains a second copy. That is Cargo working correctly rather than a defect, and
the alternative is guaranteeing the duplicate today.

**Features.** `Win32_Foundation`, `Win32_NetworkManagement_IpHelper`,
`Win32_Networking_WinSock`, and `Win32_System_Diagnostics_ToolHelp`. Requested
individually rather than by a parent, because `windows-sys` compiles what is
requested and a parent feature would compile most of the platform.

## R-5. Where does the refresh loop live?

**Finding.** Nowhere yet, and deliberately. `Pipeline::new` documents that the
control thread of section 8.6 does not exist until S11 and S13, that the
attributor is therefore owned outright by the acquisition side, and that
building the section 11.6 publication mechanism early would fix the snapshot's
shape before this slice knew what one costs to publish.

**Decision.** This slice supplies the publication cell and an attributor that
refreshes into it. It does not spawn a control thread.

The published index is a separately shareable value rather than a private field
of the attributor. That is what makes SC-006 testable at all: a test that
demonstrates concurrent resolution across a publication needs to publish from
one thread while several others resolve, and it cannot do that through a
`&mut self` method on a shared object. It is also the seam S13 needs, so the
shape is settled by a requirement rather than by anticipation.

**The pipeline change that follows.** `Pipeline` holds
`Arc<Mutex<Box<dyn FlowAttributor>>>` today and locks it per packet. With
`Sync` on the trait it holds `Arc<dyn FlowAttributor>` and locks nothing. The
public signature of `Pipeline::new` does not change: `Arc<dyn T>` is
constructible from `Box<dyn T>`, so every existing caller and test is untouched.
The blast radius of the `Sync` bound is therefore the implementors, and both
existing ones, `ScriptedAttributor` and the pipeline's test stubs, are already
`Sync` because they hold plain data.

## Summary of decisions

| Id | Decision | Recorded as |
| --- | --- | --- |
| R-1 | Read both tables by owning module, for the creation instant | Deviation, Appendix D correction |
| R-2 | Name processes by toolhelp enumeration, opening no process handle | Plan, and a lint assertion |
| R-3 | Publish through `arc-swap` | New runtime dependency |
| R-4 | `windows-sys = "0.36"`, matching the resolved version | No new package |
| R-5 | Publication cell now, control thread in S13 | Plan |
