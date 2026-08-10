### Added

- **fragcap attributes flows to processes.** `fragcap-attr` gains
  `SocketTableAttributor`, the production `FlowAttributor` of specification
  section 11: a socket table snapshot joined against captured flows by 5-tuple,
  resolving each to the process that owns it. Every attributor before this one
  answered from a text file a test wrote.
- **The join is total and documented.** Competing table entries are ranked by
  exactness, then by the latest socket creation instant at or before the
  packet, then by a declared tiebreak that exists only to make the order total.
  A test resolves the same flow against the same entries in every rotation and
  reversal and asserts one answer, so an implementation that iterates the
  platform's rows and takes the first hit fails rather than producing results
  that change between runs over identical traffic.
- **A socket created after a packet cannot own it.** Both socket tables are read
  by owning module, which carries a creation instant, and an entry that
  postdates the packet is not a candidate. This is what tells the previous owner
  of a reused port from the current one.
- **Dual-stack sockets resolve.** An IPv6 wildcard bind matches IPv4 traffic on
  the same port, for UDP, which is the protocol that takes the wildcard
  allowance at all. `AttributionKey::local_matches_bind` has named this slice as
  the owner of that case since S02.
- **The tail of a connection stays attributed.** An endpoint that leaves the
  table remains resolvable for a grace period defaulting to thirty seconds,
  measured from the instant it was last observed present. Answers resolved that
  way carry `Fidelity::Retained`, so a consumer can see which attributions are
  inference and which are observation. A live entry always beats a retained one.
- **The refresh cadence, with both triggers.** A one second interval, an
  immediate refresh on a process start matching a profile stage, and a refresh
  on an unattributed packet from a previously unseen endpoint, rate limited to
  one per two hundred milliseconds. The whole of it is driven by an injected
  clock, so it is exercised in microseconds and no test in the slice sleeps.
- **Attribution lookup no longer takes a lock.** The attributor publishes an
  immutable index atomically and every capture thread reads it without blocking,
  which is specification section 11.6. S08 held the attributor behind a mutex
  taken once per packet and deferred the mechanism to this slice by name.
- **The Windows socket table backend.** `IpHelperTable` reads the extended TCP
  and UDP tables over both address families, and `ToolhelpNamer` resolves image
  names by query-only enumeration. Both are behind a `socket-table` feature that
  is off by default, so `cargo xtask ci` still passes on any machine.
- **The backend has actually run.** Unlike the live capture source added in S09,
  which has linked but never executed, this one was driven end to end on a
  Windows machine: a real socket opened, found in the machine's real socket
  table, attributed to the process that opened it, and then closed and observed
  to survive as a retained attribution. It needs no capture driver and no
  elevation, which is why it could be.
- **`cargo xtask lint` refuses process handles.** Naming a process is the
  classic reason to open one, and this slice opens none. The linter now fails on
  any fragcap source naming `OpenProcess`, `ReadProcessMemory`, or
  `WriteProcessMemory`, and its matching became case-insensitive so a Pascal
  case platform call cannot slip past a lowercase list.
- **`cargo xtask neutral` covers `fragcap-attr`.** For the same reason S09
  extended it to `fragcap-capture`: the crate now has a platform backend, and
  nothing otherwise checked that it still builds where that backend does not
  exist.
