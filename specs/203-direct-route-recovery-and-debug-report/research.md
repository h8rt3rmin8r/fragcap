# Research: Direct Route Recovery and Debug Report

## External TCP failure boundary

**Decision**: Establish a failing non-loopback IPv4 and IPv6 client-visible handshake first, with selected-flow events for initial checksum flags, original and chosen interface indices, synthetic reply generation/send, observed client ACK, and directional bytes. Repair only a demonstrated boundary.

**Rationale**: The active release established upstream SOCKS CONNECT and zero bytes, not client SYN/ACK delivery. The current reply path copies the outbound WinDivert address and clears its outbound bit; the official WinDivert 2.2 documentation says outbound indices may be arbitrary while inbound injection requires valid interface and subinterface indices. Captured TCP checksums may also be unfinished under offload. Either may matter, but neither is yet proven as the cause. Source: https://reqrypt.org/windivert-doc.html, sections 5.3 and 5.7.

**Alternatives considered**: Hardcode the captured outbound interface, or disable checksum verification globally. Both would turn a candidate into an assumption and could hide a second failure. Preserve exact flags and use controlled failure evidence before choosing a normalization or interface-selection rule.

## Correlation authority

**Decision**: Treat selected route receipt, process generation, client-side delivery, proxy transfer, and passive packet observation as separate facts. A direct route can qualify without an Npcap first-packet record when independent exact route and client-exchange evidence proves it. Passive packet absence remains explicit.

**Rationale**: The current direct helper correlation in native.rs returns unavailable when FlowRegistry lacks the original, even though the route selected and consumed it. Requiring that observation creates a driver-order dependency. A SOCKS CONNECT alone is likewise insufficient for reachability. Existing one-use receipts and exact process ancestry are the ownership authority.

**Alternatives considered**: Invent a passive packet observation, trust a consumed count, or trust the helper PID. All violate observation truth or owner identity.

## Route stream and report

**Decision**: Add a typed route-event producer with a bounded nonblocking queue, one protected writer thread, finite records and bytes, sequence/header/trailer, and explicit queue/writer gaps. Use the existing lifecycle stream grammar and artifact access machinery where possible. Emit only after selection or a selected-flow refusal, never for broad unrelated pass-through payloads.

**Rationale**: lifecycle.rs already owns crash-prefix validation and protected proxy.jsonl writing. A separate route.jsonl remains the authority for selected route decisions; summary projections can derive from it and existing artifacts. The hot packet worker does not do JSON or filesystem work.

**Alternatives considered**: Extend stderr, write synchronously on the packet path, or add a general application log. None gives bounded durable selected-flow identity under a crash.

## Export and identity

**Decision**: Use a monotonically assigned session-local route flow ID for durable joins, retaining the secret one-use receipt only in memory. Bind that ID to proxy connection and packet-flow IDs when exact correlation exists. Exports use a deterministic bounded JSON report with opaque flow IDs, exact stages and counts, and a declared transformation for omitted endpoints and account paths. Native route.jsonl remains protected and excluded from ordinary share copies.

**Rationale**: Exported capability bytes or unprotected native endpoint details would violate existing artifact access. The report must still expose enough authority and missing fields to diagnose the route. Source bundles stay immutable.

**Alternatives considered**: Use tuple or timestamp as ID, or serialize the 32-byte receipt. Tuple reuse and clock uncertainty make the first ambiguous; the receipt is a capability and must not leave memory.

## Validation and closure

**Decision**: Use repository-controlled driver-free tests, a controlled signed-driver non-loopback matrix, and exact MSI/ZIP package qualification. Private-title runs are not required to close these implementation issues. Keep #596, #597, and #580 separate.

**Rationale**: The active-release issues describe private evidence, but their acceptance criteria allow controlled reproduction and exact regressions. The constitution disallows future field observation as a standing completion gate.
