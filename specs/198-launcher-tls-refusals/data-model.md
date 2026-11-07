# Data Model: Launcher TLS Refusal Diagnosis

## Existing entities and ownership

| Entity | Authority | S198 treatment |
| --- | --- | --- |
| Accepted proxy connection | Native runtime | Retain the existing numeric connection ID, endpoint tuple, and finite stage chronology. |
| TLS refusal class | Proxy TLS classifier | Add `tls-transport-ended` for handshake EOF/reset/abort/broken pipe and `tls-invalid-message` only for structured rustls invalid-message evidence. Keep existing alert, client-auth, validation, protocol-mismatch, and unknown classes. |
| Connection diagnostic | Bounded runtime record, schema 1 | Retain terminal code, boundary, reached/completed stages, optional alert, transport category, OS code, HTTP timeout phase, and completed exchange count. No new field or schema. |
| Proxy lifecycle record | `proxy.jsonl` | Publish the existing diagnostic projection under the same connection ID, with existing loss accounting when the bounded detail population overflows. |
| Application correlation | `application.jsonl` | Join the accepted connection ID to exact packet-flow process generation after collection; owner may be proven, ambiguous, or unavailable. |
| Calibration fact | Existing target store | Only final-client evidence may qualify final-client routing or inspection. Launcher and updater success remain their own observed outcomes. |

## Refusal transitions

```text
accepted -> authenticated -> CONNECT admitted -> tunnel response complete
    -> client TLS complete -> upstream TLS complete -> protocol exchange complete
    -> terminal complete

At any reached stage: exact failure -> terminal failed with that stage and boundary.
For a failed client TLS handshake: no upstream TLS completion, no inspected HTTP fact, no blind-relay fallback.
For a later HTTP header timeout: retain the completed-exchange count and phase independently of TLS completion.
```

## Invariants

- A stable refusal code is selected from structured evidence, never from `std::io::Error` display text or private TLS bytes.
- `tls-transport-ended` has a transport category that names the observed EOF, reset, abort, or broken-pipe kind; it implies neither trust rejection nor intentional client closure.
- `tls-invalid-message` requires a `rustls::Error::InvalidMessage` in the I/O source chain. An unrelated `InvalidData` remains `tls-refusal-unknown` with `invalid-data` category.
- The accepted connection ID is the only join key between proxy diagnostic and independently proven application owner. A missing or ambiguous packet-flow match never becomes an exact owner.
- Diagnostic detail loss is counted by the existing capacity counter, while aggregate terminal counts remain authoritative.
