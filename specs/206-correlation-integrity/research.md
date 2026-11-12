# Research: Packet Correlation Integrity

## Baseline findings

- S205's `direct_helper_correlation` source already allows an exact route proof with client response and two-way transfer to yield a positive direct-route observation when the passive original packet is absent. It keeps `packet-history-incomplete` separate. The S205 installed candidate nonetheless ended inconclusive and wrote no positive fact because the direct-root `target` stage was omitted from its observation window; that source correction still needs installed requalification. The remaining #594 gates include positive installed calibration, durable fact, later admission, and packaged original-present observation. No new positive route rule is established as necessary.
- The normal pipeline gives a pending head up to 250 milliseconds for refreshed socket attribution and causal binding, then applies a 100 millisecond per-PID cooldown; `WriteGate::admit` has a separate ten millisecond fallback. A packet without socket attribution never enters that wait. Both cases previously shared `ScopeUnresolved`, obscuring whether a socket owner was absent or its causal generation remained unproved. A changed timeout without timing evidence could affect capture continuity.
- `correlate_native_observation` rejects a proxy interval if a same-flow omission overlaps it; a prior out-of-window omission does not veto a later interval. Bounded omission history and capture-wide unlocated loss are distinct uncertainty classes.
- #596's private field report names 69 unresolved proxy observations across 24 flow IDs, with 55 retaining packet observations. The public issue cannot by itself distinguish late causal proof from genuinely ambiguous ancestry. Controlled reproduction must decide the cause before implementation.

## Alternatives

1. Increase the pending wait and treat the field count as a race. Rejected without a controlled timing reproduction because it can stall the writer and may not resolve ancestry.
2. Ignore `ScopeUnresolved` whenever some packet for the flow is retained. Rejected because an overlapping withheld packet may contain the missing ownership evidence.
3. Localize decisions to exact flow and interval, preserve bounded unknowns, and correct only the demonstrated gate or join defect. Selected because it keeps packet truth and process proof auditable.

## Demonstrated correction

The deterministic gate regression first failed because no distinct missing-socket, expired-causal-proof, or cooldown result existed. The process-trace regression first failed because the bounded flow-local omission intervals were not exported, even though `FlowRegistry` retained their cause and first/last instants. S206 now carries those distinctions through the gate, registry, native correlation reason, and protected process trace. It does not retroactively classify the older private field run, change the pending timeout, or admit a packet with weaker proof.
