# Native Proxy Foundation API Contract

## Listener authentication

Starting `NativeProxyBackend` creates one `SessionCapability` and returns it only to the selected-session adapter. `NativeProxyLease` authenticates the protocol authorization value before upstream allocation. Runtime observations expose admitted and refused counts but no secret. Cleanup invalidates the capability.

## Upstream connection

`DestinationAuthority::parse` is effect-free. `UpstreamConnector` accepts an authority, `DestinationPolicy`, stage budgets, and cancellation signal. It returns an owned plain or verified secure stream plus typed attempt facts, or a stage-specific error. Resolver, root loader, and connector effects are injectable for deterministic tests.

## Certificate ownership

`SessionCertificateAuthority::generate` creates one non-reusable session authority. Protected persistence is explicit and returns an inventory; it is not trust. `LeafCache::certificate_for` validates one DNS name or IP, issues or reuses a bounded entry, and reports evictions. Rotation invalidates old generations.

## Trust ownership

`CertificateStore` supports observe, add-exact, and remove-exact for `CurrentUser/Root`. The native Windows implementation never widens scope. The facade exposes the same implementation to Deep Capture and doctor. Non-Windows native effects return a typed unavailable result.

## Raw observations

`ObservationStream::push` assigns order and applies payload/queue bounds. `drain` preserves remaining order. `snapshot` exposes occupancy, monotonic counters, and completeness. Raw unknown and malformed records remain valid event variants. Projection APIs can report gaps but cannot delete raw events.

## Protocol lab

The test-only runner consumes `ProtocolScenario` and returns `TruthLedger` plus independent artifact expectations. A protocol family may be negotiated-wire, framed-wire, or reference-vector fidelity. Unsupported proxy output is a successful explicit expectation, not a produced observation.

## Refusals

- Authentication failure creates no upstream attempt and retains no application payload.
- Listener recursion, default-private destinations, rebinding outside an exact grant, malformed authority, and empty root sets fail closed.
- Trust requires explicit authorization and exact DER/thumbprint ownership.
- Cache, queue, payload, time, and task bounds never expand implicitly.
- No API in this slice selects the native backend for the production CLI.
