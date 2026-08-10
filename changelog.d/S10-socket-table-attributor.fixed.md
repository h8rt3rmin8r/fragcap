### Fixed

Found by automated review of pull request 13, before the slice merged. All five
are in code this slice introduced, so nothing shipped with them; they are
recorded because the reasoning is worth keeping and because three of them were
invisible to the tests as written.

- **A retained wildcard bind stopped matching.** Retention was keyed by
  endpoint and looked up the packet's own local address, while a socket bound
  to `0.0.0.0:30000` was retained under the wildcard. Every flow that resolved
  through the section 8.4 wildcard allowance while live therefore became
  unattributable the instant its socket closed, which is the whole class of UDP
  game sockets and the tail of every one of their flows. Retention now resolves
  through the same matcher as the live path.
- **Retention kept one socket per local endpoint.** Several sockets can occupy
  one: a server holds a row per client on a single port, and a reused port is
  two sockets in sequence. The map overwrote, so only whichever row the platform
  reported last survived, the others' tails were lost, and the creation-time
  ordering of FR-008a did not reach retention at all. Retention is now keyed by
  socket identity rather than by endpoint.
- **Retained attributions lost their image name, or gained the wrong one.**
  Every refresh re-resolved every retained process identifier. A process that
  had exited was no longer in an enumeration, so a name once known was dropped;
  worse, an identifier the platform had reused resolved to a different
  process, attaching its name to a connection it never opened. That is a
  confidently wrong report of the kind constitution P-9 exists to prevent. A
  retained record now carries the name it was captured with and is never
  re-resolved.
- **A refresh could erase a request made against the index it had just
  published.** The index was published before the schedule was marked
  refreshed, leaving a window in which a capture thread could read the new
  index, find an endpoint it still did not carry, and record a request that the
  mark then cleared. Because recording it also consumed the rate-limit window,
  nothing could re-arm for two hundred milliseconds and a short-lived flow
  would stay unattributed until the next periodic refresh. The order is
  reversed: an extra table read is cheap, a missed one loses attribution.
- **Two capture threads could both claim one rate-limit window.** The trigger
  loaded and then stored, so two callers could pass the same check before either
  wrote. The window is now claimed with a compare-and-exchange.

The first three share a cause worth naming. Retention was written as a lookup
of its own rather than as the live path with a different fidelity and an expiry,
and every one of the divergences followed from that. It is now the same code
path, which is what the specification meant by a grace period all along.
