# Internal Direct Route Contract v1

This is a session-private contract among the WinDivert adapter, packet-to-stream bridge, facade, and native proxy. It is not a public CLI or network protocol.

## Admission

`admit(session_id, original_flow_key, owner_proof, transport, deadline) -> route_receipt` succeeds only once for a live, exact final-client generation and an authorized direct component. `bind(receipt, helper_connection_id, original_tuple) -> original_owner` consumes the receipt atomically. The proxy's helper PID remains a separate diagnostic and cannot replace `original_owner`.

## Data path

TCP ingress begins at a selected initial SYN and passes ordered bytes, close and error events through a finite local stack into one bridge stream. The bridge authenticates to the native proxy before forwarding client bytes. UDP ingress retains one datagram per event and uses one authenticated association with the original peer. Destination admission and TLS verification remain owned by the native proxy.

## Outcomes

`inspected` requires proxy-observed supported application content after a successful trust boundary. `opaque` means forwarded encrypted content without inspection. `unknown` means routed transport without recognized application semantics. `refused` means a named policy, trust, protocol, capacity, or transport failure. `bypassed` means the original connection continued without a positive route, including missed first-packet proof. `ambiguous` and `unavailable` are evidence states, never a positive route.

## Accounting and release

For packets delivered by `WinDivertRecv`, `received = reinjected + consumed + lost + in_flight`, with `in_flight = 0` at confirmed release. The WinDivert driver exposes no exact count of packets lost before delivery to the adapter; this is reported as unmeasurable, never as zero. The controlled flow oracle and active sentinels require zero observed missing replies or unexplained loss. Every route receipt is consumed once or retired. Shutdown closes packet and event handles, stack sockets, bridge tasks, proxy legs, and any helper. An unconfirmed close is an exact recovery obligation, not a success result.
