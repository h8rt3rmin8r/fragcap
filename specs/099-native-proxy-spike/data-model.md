# Data Model: Native Proxy Backend Spike

## Controlled Scenario

One deterministic local interaction.

| Field | Meaning |
| --- | --- |
| `id` | Stable scenario name: `http1`, `https-http1`, `https-http2`, or `websocket` |
| `request` | Method, authority, path, headers, and fixed body bytes |
| `response` | Expected status, protocol, headers, and fixed body bytes |
| `messages` | Ordered WebSocket direction, kind, length, and digest expectations |
| `deadline_ms` | Maximum startup, exchange, drain, and shutdown interval |

Validation: authorities resolve only to loopback, payloads contain no private data, and every expected observation has an exact byte length and digest.

## Backend Run

One execution of the complete matrix through `candidate` or `baseline`.

| Field | Meaning |
| --- | --- |
| `backend` | Stable backend kind, name, and exact version |
| `environment` | Sanitized operating-system, architecture, Rust, Cargo, and baseline versions |
| `started_at` | Relative run instant rather than a machine-identifying path |
| `observations` | Ordered normalized results by scenario and proof point |
| `lifecycle` | Bind, ready, cancellation, drain, stop, deadline, and residue results |
| `certificate` | Generation/import, trust mutation, cache, and cleanup evidence |
| `key_log` | Requested side, produced status, line count, labels, and omission reason |
| `artifacts` | Sanitized roles, sizes, sensitivity, and retention state |

## Observation Result

| Field | Meaning |
| --- | --- |
| `scenario_id` | Controlled scenario identity |
| `kind` | Request, response, WebSocket handshake, WebSocket message, HAR source, lifecycle, cache, or key log |
| `status` | `complete`, `empty`, `bounded`, `truncated`, `unsupported`, `failed`, or `not-measured` |
| `protocol` | Observed protocol version when available |
| `direction` | Client-to-server or server-to-client when applicable |
| `byte_length` | Observed bytes |
| `digest` | Stable digest for fixed payload comparison, never raw private content |
| `detail` | Bounded non-secret reason or limitation |

State rule: only `complete` may contribute to parity. Every other state remains negative or inconclusive and carries a reason.

## Dependency Audit

| Field | Meaning |
| --- | --- |
| `manifest_hash` | Hash of the isolated manifest |
| `lock_hash` | Hash of the isolated resolution |
| `packages` | Unique package identities, versions, sources, licenses, and declared Rust versions |
| `active_paths` | Normal Windows dependency paths for the candidate feature set |
| `target_conditional` | Resolved but inactive or alternate-target packages |
| `deny_result` | License, ban, advisory, and source-policy outcome |
| `toolchain_results` | Rust 1.82 and pinned check/build outcomes |
| `build_measurements` | Clean and warm elapsed times and target sizes |
| `product_graph` | Before and after root workspace package identity and lock hashes |

## Backend Decision

| Field | Meaning |
| --- | --- |
| `date` | Decision date |
| `outcome` | Exactly one of `adopt`, `patch-or-fork`, `evaluate-fallback`, or `retain-baseline` |
| `deciding_evidence` | References to normalized results and audit findings |
| `known_limits` | Failed, unsupported, and not-measured proof points |
| `follow_up` | One bounded issue title and scope |
| `shipping_state` | Must remain `mitmdump` for S099 |

Transition: `planned` to `measured` to `reviewed-decision`. Adoption or product integration is outside this slice.
