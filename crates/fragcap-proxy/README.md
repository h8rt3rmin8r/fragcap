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
Later milestones own broader transports,
transports, launch coverage, richer artifacts, and final completion.

This crate never configures an ambient system proxy and never reaches inside a
target process.
