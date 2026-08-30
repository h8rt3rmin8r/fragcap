# Application JSON Lines Version 2 Contract

## Framing

- UTF-8 without BOM, one JSON object followed by LF per record.
- `application.header` is the first record and has sequence zero.
- Every line carries `schema_version: 2`, `session_id`, `sequence`, `event_time_ns`, and `type`.
- Sequence strictly increases with no duplicate. A gap in emitted evidence is represented by `application.gap`, not silently closed.
- The writer flushes complete records during the session. A torn final line is ignored by prefix readers.
- Exactly one valid `application.trailer` marks orderly completeness. No trailer, an invalid trailer, an unknown schema, or writer retirement means incomplete.

## Header

The header declares schema version, session identity, selected capture scope, exported record families, unavailable representations, and explicit non-export reasons for deferred protocol families.

## Evidence records

- `connection.open` and `connection.terminal`
- `tls.negotiation` and `tls.terminal`
- `http.stream.open` and `http.stream.terminal`
- `http.metadata`
- `http.body_segment`
- `http.transformation`
- `application.error`
- `application.gap`

Each record includes only anchors that are actually known. Missing process, flow, role, attribution, protocol, reason, ordering, or raw-representation data is omitted with record-level provenance when the absence changes interpretation.

Binary field names, values, and payload bytes use standard Base64 with an explicit encoding field. Human-readable projections may be additive but cannot replace binary authority.

## Trailer reconciliation

The trailer contains record counts by family, total observed, retained, and retention-truncated body bytes, accepted and written records, serialized event bytes, queue or retirement drops, writer failures, and writer status. A preceding `application.gap` carries any dropped-record count. Per-body omission, decoding, cancellation, and storage outcomes remain on their typed evidence records. Readers classify the stream complete only when the trailer, gap, sequence, session, and recomputed record counts agree.

## Compatibility

- A first record with `type: application.header`, no `schema_version`, and `manifest_version: 1` is recognized as legacy application schema 1.
- Readers accept schema 1 and 2 and explicitly reject every unknown schema version.
- Writers emit only schema 2 after S105.
- The separate `manifest.json` remains at manifest version 1.

## Sensitivity

The entire file inherits the existing sensitive Deep Capture artifact classification. Metadata and payload values are never copied into human logs or error messages.
