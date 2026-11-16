# Research: Direct Packet and Process Correlation

## Current source boundary

`NativeObservationContext::direct_helper_correlation` already accepts transport-specific client transfer proof independently of passive packet presence. It emits `direct-route-client-transfer-original-packet-unavailable` for the absent case, but the result is a flat correlation reason and the original flow registry is keyed only by tuple. Reuse and incomplete history require a versioned, explicit reconciliation contract. The helper and original flow must never be conflated.

`build_process_trace` currently joins each packet observation through `instance_at(pid, packet_time)`. A process exit before a reset therefore leaves the interval's process instance null even when the socket table retained PID/image. `FlowObservation` carries timestamp and attribution but not TCP flags or socket creation instant. These facts must pass from the parser and socket attributor into the bounded flow history before the process join can prove a tail.

## Decisions

1. Carry observed TCP flags and socket creation instant as optional packet-flow metadata. Existing callers without those facts retain the old behavior and cannot qualify a tail. No general retention window changes.
2. Use a strict 100 ms tail candidate after actual exit. A reset needs a unique same-PID/image generation containing socket creation. ACK/FIN additionally needs a prior same-flow in-lifetime observation for that generation and creation time. Any event loss or competing reuse refuses tail binding.
3. Keep the route and packet dimensions independent. A selected route receipt plus demonstrated transport-specific client transfer is positive route evidence. Missing or truncated passive history remains explicit even when route reachability is positive.
4. Version the new artifact projection additively and validate reader counts. Preserve read-only compatibility with version 1 records; do not rewrite historical bundles.

## Alternatives considered

- Widening the process lifetime by 100 ms would falsify the observed exit and admit unrelated PID reuse. Explicit per-interval tail basis retains the true timestamp.
- Requiring the passive first packet to grant route reachability would repeat the physical ordering defect in #594. Route and packet authorities remain separate.
- Treating any later ACK/FIN with a matching PID as continuity would invent ownership across socket or process reuse. The prior exact same-flow packet and socket creation proof are required.
