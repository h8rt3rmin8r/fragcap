### Added

- **The filter manager commits an install only when the capture thread confirms
  it.** `FilterManager::poll` no longer marks a handle's program as installed
  optimistically; it records a pending install (one in flight per handle) and a
  new `FilterManager::acknowledge` commits the program, and clears the handle's
  gap set, only on a success acknowledgement. A rejected maintenance `set_filter`
  is not treated as installed: the handle keeps its prior program and the install
  is retried, rate limited, rather than the manager's model silently diverging
  from the real handle. Resolves issue #20 (the deferred half of the S13 review
  finding P2).
- **The acknowledgement flows over the reverse of the S13 filter channel.** Each
  capture thread reports the result of its `set_filter` calls to the control
  thread over a shared `std::sync::mpsc` channel tagged with its handle index,
  mirroring the forward per-source filter-program channel; the control thread
  applies each acknowledgement to the manager before it polls. No new dependency,
  and `PacketSource` gains no bound (constitution P-3). A rejecting handle is
  retried, never retired: retirement stays reserved for a capture thread that has
  ended, because correctness never depends on the kernel filter being fresh
  (section 12.3).
