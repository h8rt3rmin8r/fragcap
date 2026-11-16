# Research: Game-first Deep Capture

## Route authority

The CLI builds a `RoutingPlan` for the authorization record, then `DeepCapture::preflight_with_direct_socket` rebuilds a child-environment route in the library. These independent decisions can diverge without an explicit mismatch check. The current constructor assigns upper- and lower-case HTTP(S) proxy, ALL_PROXY, authorization, and NO_PROXY variables to the managed root. An exact direct socket component can be appended, but it does not remove the launcher proxy effects. The selected route and scope must be passed as typed intent to library preparation, and the pre-effect plan must be generated from the resulting immutable `SessionPlan` or compared against its canonical effect identity before effects.

## Scope and ownership

Both CLI and library resolve `auto` to launch-chain today. The exact-game direct-route decider already requires socket inception and exact managed process generation. Packet admission has a separate causal ancestry path that can retain launcher packets even when the requested scope is exact-game; it must use the final-game proof rather than any causal descendant. Application scope and manifest must use the same resolved selection. Keep process ancestry observation for ownership and cleanup, without admitting its payload.

## Launcher behavior

The observed publisher updater was fast under native launch and stalled or failed when Fragcap supplied HTTP proxy variables. A controlled HTTP CONNECT handoff can identify client close before TLS; that observation is more precise than attributing it to certificates or origin. The ordinary route should apply no Fragcap proxy variables to the launcher. Explicit advanced full-chain/SOCKS routing must name every environment effect in authorization and compare them with what was applied. Inherited ambient proxy variables are not Fragcap's effect; retain the user's normal environment unless a current-session variable is explicitly being removed or replaced.

## Compatibility evidence

Ordinary capture already derives and appends route, TLS, protocol, and inspectability candidates from session observations. `policy.rs` nonetheless requires a historical `proxy-routing=reached-client` row before admission. Remove that prior-positive gate, preserve current launch and effect feasibility checks, and keep historical rows readable. Retire the live calibration command, guided/resume and embedded calibration entry without dropping store migrations or old bundle readers. Reuse assessment code still required by ordinary capture.

## SOCKS classification

The proxy currently groups timeout, EOF, read failure, and partial prefix as `OpaqueTcp`, then calls that `TcpPlaintext`. Application and compatibility layers promote it into positive plaintext facts. Introduce an explicit undetermined provenance; maintain byte-transparent forwarding and loss accounting. Positive TLS prefix is encrypted-prefix evidence, not proof of trust or decryption. A later prefix can support a later observation but cannot reclassify an earlier unknown interval as inspected.

## Alternatives rejected

- Merely changing CLI plan JSON leaves the library route authority unchanged.
- Merely setting `NO_PROXY` still mutates launcher behavior and does not mean native routing.
- Silently retrying with SOCKS after a game-only failure violates the exact authorized effect set.
- Deleting old store rows or the entire calibration implementation would break readback or ordinary assessment code.
- Treating a non-TLS first byte as plaintext proof conflates opaque binary protocols with positively identified cleartext.
