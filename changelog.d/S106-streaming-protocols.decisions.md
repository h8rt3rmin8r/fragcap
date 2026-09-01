<!-- spec-impact: 13.7, 19.6, 25, 28.1 -->
### Decisions

- **2026-08-30:** S106 combines issues #295, #298, and #299 because all three consume the same bounded streaming observer seam and durable application record family. Splitting them would repeat forwarding, loss-accounting, and artifact review work.
- Raw WebSocket frames and HTTP bodies remain authoritative. WebSocket messages, SSE events, and gRPC envelopes are derived records that cannot delay or change forwarding.
- `flate2` 1.1.10 is promoted from the exact existing lock graph for stateful RFC 7692 raw DEFLATE. A message-oriented WebSocket dependency was rejected because it hides frame and masking evidence.
- gRPC payloads remain opaque bytes. Without an explicit protobuf schema, assigning field meaning would violate the honest-capability boundary.
- **2026-08-31:** Review separated payload omission from parse outcome, delayed HTTP/2 WebSocket semantic observation until the extended CONNECT response fixes negotiated extensions, and replaced the protocol test's timing sleep with the observed settings acknowledgement. These changes preserve failures honestly and remove platform-dependent scheduling from the RFC 8441 proof.
