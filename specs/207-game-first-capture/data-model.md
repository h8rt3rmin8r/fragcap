# Data Model: Game-first Deep Capture

## Route selection

`SessionConfig` carries a typed launcher exposure (none, legacy HTTP/SOCKS, or SOCKS-only), an inspection scope (exact game or explicit launch chain), and the exact direct-game component request. The route constructor receives these values and the selected loopback family, and produces the immutable `RoutingPlan` stored in `SessionPlan`. The CLI's authorization document is a projection of that plan. A late listener port fills values already declared as session proxy values; it cannot add or remove effect names.

## Applied route

An applied route records each declared environment assignment or removal and the direct component actually armed. The managed launch consumes that exact set. Planned and applied effect identities must compare before trust, driver, proxy, capture, or launch effects. Empty launcher environment effects are valid only with a valid exact direct component.

## Ownership and scope

The final-game owner is a process identifier plus generation, executable/path and causal proof. The launch ancestry is an observation and cleanup authority. Default route, packet, and application admission require the proven final-game owner. Explicit advanced launch-chain scope permits proven intermediates; unresolved or unrelated owners remain excluded and counted.

## Protocol provenance

`Undetermined` records absent, partial, timed-out, EOF, or failed first-byte evidence without a plaintext claim. `EncryptedTlsPrefix` is a positive encrypted-shape observation. Positively identified cleartext protocols can carry plaintext provenance. Decrypted traffic requires a completed proxy-owned TLS stage. Retained bytes and directional totals remain independent from classification.

## Compatibility facts

One capture yields append-only observations with target/case/version identity, route reachability, TLS/protocol/inspectability outcome, evidence source, and uncertainty. Historical facts remain readable and cannot gate a new authorized capture or silently change its route. Store version 16 transactionally rebuilds only the compatibility-fact table to admit `direct-socket-v1` and `socks-only+direct-socket-v1` route identities and the `direct-socket` strategy. Existing rows retain their values.
