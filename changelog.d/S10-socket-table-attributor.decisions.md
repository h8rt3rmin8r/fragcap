### Decisions

**2026-08-09: four deviations from the architecture of record, for promotion to
specification section 29.**

- **`FlowAttributor` requires `Sync`.** Section 8.5 declares it with neither
  bound. Section 11.6 requires several capture threads to read one published
  attribution snapshot without locking, and there is no arrangement of a
  `Send`-only trait that they share without a lock somewhere. S09 changed
  `PacketSource` by the same route and for the same kind of reason, and the size
  of the change is the same: a bound that every existing implementor already
  satisfies, rather than a method on a surface intended to reach 1.0.0. Both
  implementors in the workspace were already `Sync` and neither changed.
- **A socket creation instant on the socket table entry.** Section 11 describes
  a snapshot as a map from endpoint to owning process identifier and says
  nothing about creation time. Appendix D found the platform exposes it and
  states that it narrows the section 11.3 race window; carrying it is what makes
  that true rather than merely available.
- **The UDP table also reports a socket creation instant.** Appendix D D.1
  records the timestamp as a property of the TCP table. Both tables carry it,
  when each is requested by owning module rather than by owning process
  identifier, which is the class distinction the reconnaissance session did not
  have reason to draw. This matters more for UDP than for TCP: section 8.4 keys
  UDP attribution on the local endpoint alone, because the table reports no
  remote for a datagram socket, so it is the weaker join and the one where a
  reused port is least distinguishable. For promotion to Appendix D as well as
  section 29.
- **An injected clock on the attributor.** Section 11.2 states a cadence without
  saying where time comes from. A one second interval, a two hundred millisecond
  rate limit, and a thirty second retention window are otherwise untestable at
  tier 1, which section 25.1 requires. Scoped to `fragcap-attr` rather than
  introduced as a workspace-wide abstraction.

**2026-08-09: `arc-swap` is taken as a runtime dependency, for lock-free
publication.** Section 11.6 requires that the capture thread read the current
snapshot without locking while the control thread replaces it. The tempting
alternative, `RwLock<Arc<Index>>`, is a lock: a reader can block behind a
writer, and the reader here is the acquisition path that section 11.6 exists to
keep unblocked. It would satisfy a test and not the requirement, which is worse
than failing both, because it looks like the requirement was met. A hand-rolled
`AtomicPtr` is correct and needs a reclamation scheme written in `unsafe`, in a
workspace that has none outside a platform binding.

MIT or Apache-2.0, edition 2018 with no declared minimum toolchain, so it
cannot move `cargo xtask msrv`. It adds two packages to `Cargo.lock`, not one:
it has a build dependency on `rustversion`, a proc macro that contributes
nothing to the built artifact and is also MIT or Apache-2.0. The planning
research predicted one package, from reading an empty `[dependencies]` table
and not looking at `[build-dependencies]`. Recorded because a dependency audit
that makes that mistake under-reports every proc macro in the graph.

Anyone proposing to remove this dependency should answer whether a reader may be
blocked by a writer at all, which section 11.6 answers no. Whether a read lock
is fast enough is a different question and not the one being asked.

**2026-08-09: `windows-sys` is pinned to the 0.36 line, which `pcap` already
resolves.** Taking the current line would put a second complete `windows-sys`
tree in the graph for declarations that have not changed. Matching the resolved
version adds no package to `Cargo.lock` at all, so `cargo deny` has no new
subject and the licence position is unchanged. If `pcap` later requires a newer
line the graph gains a second copy, which is Cargo working correctly; the
alternative is guaranteeing the duplicate today.

The alternative to a binding crate here is the same one S09 rejected: a C ABI
whose struct layouts must be transcribed by hand with nothing checking them
against the header. A wrong offset in `MIB_TCPROW_OWNER_MODULE` yields a
plausible process identifier that is wrong, which is the P-9 failure no test
over synthetic data catches.

**2026-08-09: the feature is `socket-table` and not `live`.** The analyze gate
caught the collision. `fragcap-capture`'s `live` feature means "links against
the npcap import library"; this backend links against nothing of the sort,
because the IP Helper API and the toolhelp snapshot ship with the operating
system. Folding them into one feature would have made attribution unavailable to
anyone without a capture driver software development kit it never calls, and
would have made the workflow step that builds it fail for a reason that has
nothing to do with it. S09's own rule gives the answer: a feature is named for
the capability it gates, and these are two capabilities.

**2026-08-09: image names come from toolhelp enumeration, which opens no process
handle.** Constitution P-1 requires any process handle to state its access
rights explicitly at the call site. `OpenProcess` with
`PROCESS_QUERY_LIMITED_INFORMATION` followed by `QueryFullProcessImageNameW`
would comply: those rights carry no memory access. But P-1's requirement exists
because a handle request is a thing a reviewer has to check, and
`CreateToolhelp32Snapshot` removes the thing to check rather than documenting
it. The image name is already in the enumeration result. `cargo xtask lint` now
asserts that no fragcap source names a process-opening call at all, which is a
stronger and cheaper guarantee than asserting that every one it does name
requests the right rights.

**2026-08-09: `.github/workflows/platform.yml` gains a step and a path filter.**
A pinned artifact, changed because nothing would otherwise ever compile the new
backend: that workflow's filters named only `fragcap-capture/**` and
`fragcap-core/**`, and its only build step was the capture crate. The new step
is placed before the npcap software development kit is acquired and is not gated
on the capture driver being present, because this backend needs neither. It is
therefore the first step in that workflow which can go green on a bare Windows
runner, and the first that does not depend on an external download succeeding.

**2026-08-09: the cadence configuration is not a profile key.** `fragcap-profile`
accepts a closed set of five capture keys and refuses unknown ones, and S05
refused them deliberately: a key with no consumer is a key whose behavior is
untested and whose meaning is set by whoever first reads it. S14 owns adding
keys when it owns a command line that can set them. The interval, the retention
period, and the rate limit are plain values on `AttributorConfig` until then.

**2026-08-09: `resolve` reads the injected clock, and only on the path that
records a refresh request.** Found unspecified by the analyze gate. The rate
limit bounds how often fragcap reads the platform's table, which is a
wall-clock cost, so it cannot be measured in capture time: replaying an hour of
traffic in one second would otherwise request thousands of refreshes and a quiet
interface would request none. The clock is injected, so this costs no
determinism in a test. It does mean `resolve` is not a pure function of the
index and the packet, which is the honest reading of section 11.2 rather than a
compromise of it.
