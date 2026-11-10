# Data Model: Direct Socket Route

## Process generation

An existing managed process instance supplies PID, creation instant, canonical path, account/session, launch receipt, role, and causal ancestry. A final-client route requires a matched terminal role and a proven, currently live generation. PID equality without creation and ancestry evidence is insufficient.

## Original flow key

`(transport, address family, original local IP and port, original remote IP and port, process generation, session)` uniquely names a selected connection or UDP association. Endpoint reuse creates a new generation and receipt. The key is immutable after first positive admission.

## Socket event and packet observation

A socket event has its WinDivert event, PID, original tuple, and monotonic QueryPerformanceCounter timestamp. A packet has tuple, direction, capture timestamp in the same clock domain, length, and exact WinDivert handle identity. A bounded sampled conversion connects a socket event to Unix-time process-generation evidence; uncertain conversion cannot grant ownership. A decision requires one matching event and the managed-generation proof. A missing or tied match yields `unavailable` or `ambiguous`; it never creates a selected route.

## Route receipt

A route receipt contains an unpredictable session-scoped identifier, immutable original flow key, route decision, issue instant, finite deadline, and state `issued`, `consumed`, or `retired`. Consumption is atomic and binds exactly one helper-side proxy connection ID. Replay, cross-session use, changed generation or tuple, and expired receipt are refused. The token is never exported into application artifacts.

## Packet disposition and flow result

Each packet delivered to the adapter advances exactly one of `reinjected`, `consumed`, or `lost`. `consumed` means the local stack accepted it; it does not imply proxy success. Pre-receive driver queue loss is an unmeasurable field, never a zero count. Per-flow results distinguish `inspected`, `opaque`, `unknown`, `refused`, `bypassed`, `ambiguous`, and `unavailable`, with packet, route, proxy, and application evidence links retained separately. Saturation, send failure, timeout, cancellation, and cleanup failure carry stable reason codes.

## Lifecycle

`planned -> authorized -> event-observing -> packet-handle-open -> routing -> stopping -> released` is the successful path. Any pre-route failure returns to `released` before the managed root starts. A post-start error enters `stopping`; an unconfirmed effect enters `recovery-required` with its exact handle or task identity. No terminal result may claim released while a driver handle or route task remains live.
