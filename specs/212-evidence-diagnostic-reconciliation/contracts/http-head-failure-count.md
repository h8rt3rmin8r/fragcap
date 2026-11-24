# Contract: HTTP/1.1 Head-Read Failure Count

## Field

`http_head_bytes_before_failure` is an optional, nonnegative integer on a connection diagnostic. It reports how many bytes the bounded HTTP/1.1 head reader actually accepted before a read error, partial EOF, or head-size refusal. It never contains any part of the head itself. Existing `connection_id`, `code`, `boundary`, and stage facts remain authoritative for the terminal outcome.

## Presence rules

| Controlled observation | Existing outcome | Native and terminal count | Bundle report row |
| --- | --- | --- | --- |
| Clean EOF before any byte | No head, no head-read failure | Omitted | Omitted |
| Read error before any byte | `http-head-read-failed` | `0` | Count `0` |
| Read error after `n` bytes | `http-head-read-failed` | `n` | Count `n` |
| EOF after `n` bytes | `http-head-early-eof` | `n` | Count `n` |
| Limit reached after `n` bytes | `http-header-limit-exceeded` | `n` | Count `n` |
| Complete head | Success | Omitted | Omitted |
| Other connection failure | Existing code | Omitted | Omitted |

For positive partial cases, `n` is the exact length observed before the terminal branch and is at most the configured head limit. The key is omitted, never serialized as `null`, whenever the count does not apply. It may be zero only for an actual read error before any head byte.

## Boundaries and compatibility

The value travels from `ProtocolError` through the existing proxy connection trace and facade diagnostic. The native `proxy.jsonl` version-one connection record and terminal diagnostics JSON emit the key only for `Some(count)`. The separately requested bundle debug report reads the bounded native stream and emits `proxy.head_read_failures[]` only when it finds valid counted HTTP-head failures. Each row contains numeric `connection_id`, allowlisted `code` and `boundary`, and the exact count, with no raw detail, endpoint, head, or payload. No row or empty array appears when none qualifies. Older records without the key remain valid. The new field does not imply a completed HTTP request, an inspected body, a successful connection, or a known peer decision. It changes no read timeout, forwarding, response, routing, or cleanup path. The reader may consume plain or TLS-backed HTTP streams; this field makes no claim about the transport's encryption state.

## Controlled acceptance

One fixture for each table row checks terminal code and count presence/value. Terminal diagnostics JSON is checked separately. A bundle fixture builds the debug report from the same native `proxy.jsonl` record and verifies connection ID, code, boundary, and count agreement, plus absence for uncounted records. Tests also assert that no partial head content appears in any projection and that existing records without the new key still parse. A timeout is an unchanged non-head-read case and must not acquire a fabricated byte count.
