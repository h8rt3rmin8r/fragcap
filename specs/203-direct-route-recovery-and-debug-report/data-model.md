# Data Model: Direct Route Recovery and Debug Report

## SelectedRouteFlow

One session-local ID uniquely names a selected route attempt. Fields: route flow ID, one-use receipt reference in memory only, protocol, original local and remote endpoint, process ID and generation, ancestry proof, socket endpoint ID, original first-packet time and clock error, admission state, proxy leg ID, packet evidence state, client delivery state, directional transfer counts, terminal state, and named gaps. The ID does not identify an unrelated pass-through flow.

The flow moves from admitted to proxy-pending, proxy-connected or refused, client-reply-sent, client-response-observed, transferring, and terminal. A route may terminate before any later stage. No-byte closure is distinct from transferred closure. A secret receipt is retired once its exact helper binding completes or the route ends.

## RouteEvent

One typed event belongs to a selected route flow or a stream-wide gap. Fields: schema version, monotonically increasing sequence, session ID, route flow ID, event kind, monotonic and calibrated wall time with uncertainty, bounded metadata, and optional exact proxy connection or packet-flow ID. The stream starts with a header and ends with a reconciling trailer; a missing trailer means crash prefix. A queue gap or writer failure makes completeness partial or unavailable.

## RouteProof

Independent dimensions: ownership (exact, ambiguous, unavailable), client-side delivery (ACK and transfer, datagram reply, failed, unknown), proxy (connected, refused, transferred, empty, unknown), passive packet (retained, absent due to diversion order, incomplete, unknown), and route accounting (complete, gap, failed). Positive reachability requires exact final-client generation, exact selected receipt, and proven two-way client and proxy traffic. Packet observation contributes fidelity but is not invented when absent. A selected flow with no application payload remains non-positive.

## DebugReport

A bounded derived document with schema and product version, source manifest identity, artifact completeness and authority, route and launch scopes, driver version when known, per-flow milestones, transfer and packet counts, route/proxy/packet/process joins, calibration verdict, cleanup state, report omissions, and transformation manifest. Every optional field has a present, unavailable, partial, or ambiguous status. The export contains no secret receipt or raw application payload; omitted exact endpoint/account values are declared. It is written atomically to a distinct destination and leaves the source bytes unchanged.

## Invariants

- Every delivered WinDivert packet settles as reinjected, consumed, or lost, while pre-receive loss remains unmeasurable.
- Every accepted route event is written once or appears in counted queue/writer loss. A malformed or truncated prefix cannot claim complete status.
- A proxy helper cannot become the target owner. Tuple match without generation and one-use receipt does not establish identity.
- Manifest, terminal, compatibility, calibration, and export projections state the same authority and completeness.
