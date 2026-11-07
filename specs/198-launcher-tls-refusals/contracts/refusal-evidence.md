# Contract: Launcher TLS Refusal Evidence

## Stable outcome vocabulary

| Evidence at TLS boundary | Terminal code | Transport category | Claim limit |
| --- | --- | --- | --- |
| EOF, reset, aborted connection, or broken pipe with no structured rustls cause | `tls-transport-ended` | Exact observed category | No deliberate closer, trust, or pinning inference. |
| `rustls::Error::InvalidMessage` in the I/O source chain | `tls-invalid-message` | Existing `invalid-data` or observed I/O category | Malformed TLS input only, not a certificate decision. |
| `rustls::Error::PeerIncompatible`, no common ALPN, or explicit protocol alert | `protocol-mismatch` | Existing category and alert if present | Exact observed incompatibility, not a universal title verdict. |
| Existing structured certificate, pinning-signal, or client-auth alert | Existing class | `tls-alert` and alert name | Only the emitted alert's supported class. |
| Generic `InvalidData` without structured rustls cause | `tls-refusal-unknown` | `invalid-data` | No invented TLS or certificate cause. |

The classifier's structured-error cases take precedence over broad I/O kinds. The existing `ConnectionFailureCategory` derives `transport` for `tls-transport-ended` and `protocol` for `tls-invalid-message`; detailed terminal code remains independently visible.

## Artifact join

The existing bounded connection diagnostic in `proxy.jsonl`, facade proxy summary, and calibration JSON carries `connection_id`, `terminal`, `code`, reached/completed `stages`, `boundary`, `tls_alert`, `transport_category`, `os_error_code`, `header_timeout_phase`, and `completed_exchanges`. `application.jsonl` correlation uses the same accepted connection ID and exact endpoint/time evidence to name a process generation or an explicit unavailable/ambiguous reason. Raw authority, proxy capability, client TLS flight, certificate private key, and HTTP body do not enter the compact diagnostic.

An `Inspect` connection that fails TLS ends as failed. There is no transition from failed inspection to uninspected relay. The separate game-only blind-relay policy is chosen before connection effects and remains outside this failure transition.

## Controlled fixture matrix

| Row | Client action | Required observation |
| --- | --- | --- |
| L1 | Launcher-like client authenticates, CONNECTs, trusts session CA, exchanges HTTP through verified origin TLS | Both TLS stages and HTTP complete; exact launcher owner join. |
| L2 | Updater-like client independently repeats L1 | Distinct ID and exact updater owner join. |
| F1 | Client closes after CONNECT before TLS completes | Client-TLS boundary, transport-ended code, exact EOF category, no alert or inspected HTTP. |
| F2 | Client sends malformed TLS after CONNECT | Invalid-message code only with structured rustls cause; no inspected HTTP. |
| F3 | Client sends or provokes a supported trust/pinning/client-auth/protocol signal | The corresponding existing class, alert when observed, no fallback. |
| H1 | HTTP/1 keep-alive request completes and next head times out | After-completed-exchange phase and positive count, distinct from F1/F2. |
| H2 | First HTTP request head times out | Before-first-request phase and zero count. |
| C1 | Packet-flow owner is absent or ambiguous | Explicit unavailable/ambiguous owner with no final-client fact. |

The source slice does not reproduce a private launcher. Its controlled compatible rows establish the native route capability; installed v0.10.14 failures retain their original observed, unclassified cause until a later owner-controlled field run supplies more evidence.
