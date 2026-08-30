# S105 Data Model

## ProxyConnectionId

A session-local monotonically assigned identifier for one admitted proxy connection. It is never reused within a session.

## HttpStreamId

A protocol-aware identifier scoped to `ProxyConnectionId`. HTTP/2 uses the peer-visible stream number for the observed leg and records the independently assigned upstream stream number when available. HTTP/1.1 uses a monotonically assigned exchange number.

## ProtocolVersion

`http_1_1` or `http_2`. Cleartext prior knowledge and TLS ALPN are retained as negotiation provenance rather than separate versions.

## MetadataBlock

- connection and stream identity
- direction and role: request, informational response, final response, or trailers
- protocol version
- typed pseudo-fields where HTTP/2 supplies them
- ordered `MetadataField` collection
- ordering and representation fidelity
- observed and retained byte counts
- terminal parsing outcome

## MetadataField

- raw name bytes when the protocol boundary supplies them
- raw value bytes
- original name casing when HTTP/1.1 supplies it
- duplicate ordinal within the block or within the exposed HTTP/2 name group
- sensitivity marker
- source index for derived conveniences

## BodySegment

- connection and stream identity
- request or response direction
- representation: raw, transfer-decoded, or content-decoded
- monotonically increasing byte offset within that representation
- bounded payload bytes when authorized and retained
- observed length and retained length
- scope, truncation, queue, and storage outcome
- event time

## TransformationRecord

- source representation and byte interval
- destination representation
- algorithm: chunked, gzip, zlib-deflate, Brotli, or identity
- input and output byte counts
- completion outcome: complete, unsupported, malformed, truncated input, output limit, ratio limit, time limit, or cancelled

## StreamTerminal

Exactly one terminal outcome per accepted HTTP stream: complete, reset, cancelled, refused, protocol error, transport error, peer GOAWAY boundary, idle timeout, or session shutdown. Request and response body completion states remain separately available.

## ConnectionTerminal

Exactly one terminal outcome per accepted proxy connection with accepted, completed, refused, reset, failed, and forcibly terminated stream counts.

## ObservationAccount

Monotonic counters for accepted and emitted events, raw bytes observed and retained, bytes intentionally omitted, truncated, decode-failed, storage-failed, or queue-dropped, decoder output, writer attempts, writer records, writer gaps, and writer failures. Categories are mutually attributable and terminal reconciliation permits no unnamed remainder.

## ApplicationEvent

A typed proxy-owned value containing protocol truth and optional bounded payload bytes. It carries available session-local correlation anchors but no serialized schema fields.

## ApplicationRecordV2

- `type`
- `schema_version` fixed at 2
- `session_id`
- monotonic `sequence`
- `event_time_ns`
- available target, connection, stream, flow, process, role, and attribution anchors
- record-specific protocol evidence
- scope, truncation, loss, and unavailable-value provenance

Record families are `application.header`, `connection.open`, `connection.terminal`, `tls.negotiation`, `tls.terminal`, `http.stream.open`, `http.stream.terminal`, `http.metadata`, `http.body_segment`, `http.transformation`, `application.error`, `application.gap`, and `application.trailer`.

## ApplicationArtifactLease

Owns the writer queue, writer thread, artifact path, accounting, retirement state, and single-finalization transition. States are `open`, `retired`, and `finalized`. Only `open` can accept data. `retired` counts later attempts. `finalized` cannot emit a second trailer.

## State and Ordering Rules

1. The application header is sequence zero and the first complete line.
2. Data sequence is assigned at bounded-queue admission, giving one deterministic total order across producers.
3. A stream-open record precedes that stream's metadata, body, transformation, error, and terminal records.
4. Byte offsets are contiguous in the observed representation even when retention is omitted or lost.
5. Every accepted stream and connection receives exactly one terminal outcome during orderly or forced runtime cleanup.
6. Exactly one trailer closes an orderly healthy writer. Its absence means incomplete. Writer retirement cannot be relabeled complete.
