# fragcap-proxy

`fragcap-proxy` owns the native loopback runtime for fragcap Deep Capture.

S102 provides the library boundary and bounded listener, connection, task, and
shutdown ownership. S103 adds authenticated admission, upstream policy,
certificate and trust ownership, and loss-accounted raw events. S104 adds the
bounded HTTP/1.1, CONNECT, and two-boundary TLS engines and makes this crate the
sole production Deep Capture proxy path. S105 adds bounded HTTP/2 multiplexing,
protocol-faithful metadata, incremental body evidence, and bounded gzip,
zlib-deflate, and Brotli derivations. S106 adds wire-preserving WebSocket,
incremental Server-Sent Events, and schema-free gRPC envelope observation.
S114 adds session-authenticated SOCKS5 TCP CONNECT for IPv4, IPv6, and
proxy-resolved domains, with bounded byte-transparent relay and typed tunnel
metadata. S115 adds authenticated UDP ASSOCIATE with control-channel lifetime,
immutable client endpoint pinning, fixed family sockets, bounded exact peer
mappings, policy-checked proxy DNS, explicit fragmentation refusal,
metadata-only events, and complete drop accounting. S116 adds bounded
directional generic TCP chunks with plaintext, opaque TLS, and intercepted TLS
provenance while retaining the existing HTTP and refusal boundaries. Later
milestones own broader transports, launch
coverage, richer artifacts, and final completion.

S117 adds one bounded generic UDP record for each accepted ingress datagram on
the S115 association. Directional sequence, exact endpoints, timing, observed
length, retained prefix, retention outcome, queue loss, and platform-observed
socket errors remain explicit. Forwarding always uses the complete payload,
unrouted UDP remains packet-only, and no application or ICMP meaning is inferred.

S118 promotes the existing Quinn transport and adds Hyperium HTTP/3 for scoped
QUIC inspection. Each admitted connection is one immutable pair of
client-facing session-authority TLS and independently verified upstream TLS.
Negotiated `h3` selects bounded HTTP/3 streams, datagrams, metadata, bodies,
timings, and terminals. Unknown ALPN, zero round-trip application data, active
migration, changed endpoints, trust failures, and unscopable routes are refused
without transparent fallback.

This crate never configures an ambient system proxy and never reaches inside a
target process.

S119 carries one exact IPv4 or IPv6 loopback endpoint through authorization,
bind, authenticated routes, lifecycle, and evidence. Bracketed IPv6 literals
and bounded numeric scope indexes reach exact socket addresses, mapped aliases
share one canonical policy identity, and proxy-owned TCP candidates use one
finite staggered race with a sole selected peer. The controlled lab exercises
IPv6 HTTP, HTTPS, SOCKS, TCP, UDP, and QUIC without wildcard listening.

S120 leaves raw protocol authority in this crate and adds the exhaustive public
classification contract in the `fragcap` facade. Raw transport, TLS, parser,
retention, queue, and writer facts remain unchanged; facade classification and
artifact summaries derive from them without feeding policy back into proxy
forwarding.
