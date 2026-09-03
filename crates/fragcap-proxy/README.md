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

This crate never configures an ambient system proxy and never reaches inside a
target process.
