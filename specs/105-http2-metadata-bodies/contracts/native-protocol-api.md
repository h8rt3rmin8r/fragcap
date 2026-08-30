# Native Protocol API Contract

## Admission and negotiation

- A connection is accepted only through the existing session capability and destination-policy boundary.
- TLS advertises HTTP/2 and HTTP/1.1. The selected client protocol is offered exclusively to the origin.
- Missing ALPN retains the S104 HTTP/1.1 compatibility behavior. An explicit protocol mismatch is refused and observed.
- Cleartext HTTP/2 prior knowledge is recognized by its complete connection preface. Each h2c connection is bound to one authenticated authority.

## HTTP/2 bridging

- The proxy accepts multiple concurrent streams and maps each client stream to exactly one upstream stream.
- Client and upstream stream identifiers remain distinct.
- Flow-control capacity is released only after the corresponding bytes have been accepted by the downstream send path.
- Server push is disabled in SETTINGS and any attempted push is refused and observed.
- Concurrent stream, window, header-list, send-buffer, pending-reset, reset-retention, idle, and shutdown bounds are finite configuration values.
- Stream failure terminates only that stream unless the protocol or transport requires connection termination.

## Metadata

- HTTP/1.1 fields preserve wire order, name casing, duplicates, empty values, and raw values.
- HTTP/2 pseudo-fields are typed and separate from regular fields. Regular values remain binary-safe and duplicate order is retained within the boundary exposed by `h2`.
- HTTP/2 HPACK bytes and original compressed cross-name order are reported unavailable.
- Informational responses, final responses, and trailers are separate metadata blocks.
- Parsed cookies and queries reference their raw source entries and retain repeated values and decode outcomes.
- Sensitive fields are application-artifact data and never formatted into human diagnostics.

## Bodies

- Forwarding reads and writes bounded chunks and never requires an entire body in memory.
- Observation retention is selected by scope and is independent of forwarding eligibility.
- Raw segments are authoritative. Derived transfer and content representations cite their inputs.
- Supported content decoders are gzip, zlib-wrapped deflate, and Brotli.
- Decoder input, output, ratio, time, and concurrency bounds are mandatory.
- Every body direction terminates as complete, partial, cancelled, malformed, intentionally omitted, truncated, storage failed, or queue dropped, with exact byte accounting.

## Event sink

- Protocol tasks submit typed bounded events through a nonblocking interface.
- A full or retired sink returns an explicit disposition and advances accounting.
- Event sink pressure cannot await disk or indefinitely block network forwarding.
- Payload bytes are absent in metadata-only scope.

## Shutdown

- Runtime ownership includes every connection, stream, decoder, and application-writer producer handle.
- Stop first prevents new admission, then drains inside the session budget, then aborts remaining owned work and records forced terminal outcomes.
- No detached task may outlive runtime completion.
