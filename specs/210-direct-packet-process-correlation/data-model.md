# Data Model: Direct Packet and Process Correlation

## Packet-flow observation

Each retained observation has timestamp, exact flow key and session flow ID, optional resolved PID/image/role/fidelity, optional socket creation instant, and optional TCP flags. Missing fields remain null. Omission runs retain cause, count, first and last times, and precision-loss state.

## Direct-route reconciliation

A selected route has one original tuple, one process-generation ID, first-packet time and clock uncertainty, one helper acceptance window, selected packet disposition, and transport-specific transfer proof. Packet state is `observed`, `absent`, `incomplete`, or `conflicting`; route state is independently `transferred`, `unproven`, `refused`, or `ambiguous`. A tuple reused for another generation cannot share the first receipt's proof.

## Process-tail interval

One flow-owner interval retains actual packet time range, PID/image, optional process-instance ID, optional socket creation instant, observed TCP flags, and a basis: `in-lifetime`, `post-exit-reset-tail`, `post-exit-contiguous-tail`, or `unresolved`. A tail basis does not alter `process.exited.at_nanos`. The process-trace trailer counts resolved and unresolved intervals and limitations.
