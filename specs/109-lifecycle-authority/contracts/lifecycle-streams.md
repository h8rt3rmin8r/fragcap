# Contract: Proxy and Cleanup Lifecycle Streams

## Common Rules

- JSON Lines with schema version 1.
- Exactly one header.
- Monotonic sequence per stream.
- Append and flush while the session runs.
- Bounded nonblocking producers.
- Named gap records for loss or unavailable localization.
- Exactly one trailer on orderly completion.
- A missing trailer is readable and incomplete.

## Proxy Stream

`proxy.jsonl` carries listener, admission, connection, DNS/upstream, TLS, protocol, error, loss, stop, and drain records. Connection-scoped records repeat `proxy_connection_id`.

## Cleanup Stream

`cleanup.jsonl` carries obligation, attempt, retry, result, retained, recovery-link, and gap records. Resource-scoped records repeat `resource_id` and journal sequence where available.

## Derived Summary

`cleanup.json` is generated only from the parsed cleanup chronology. It contains the terminal resource rows expected by existing consumers plus `source_role: cleanup-log`. It is not an independent chronology authority.

## Reconciliation

Trailers expose accepted, written, dropped, failed, connection, resource, and terminal counts. Manifest v2 declares stream finalization and loss from parsed stream truth, not file existence alone.
