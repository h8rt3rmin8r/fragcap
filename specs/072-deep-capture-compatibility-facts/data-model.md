# Phase 1 Data Model: Deep Capture compatibility facts

## `CompatibilityFact`

One local Deep Capture compatibility observation tied to a target row.

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | `Option<i64>` | no | SQLite row id, assigned on insert |
| `target_id` | `i64` | yes | Foreign key to `targets(id)`, cascade delete |
| `key` | `CompatibilityFactKey` | yes | Closed fact family |
| `value` | `String` | yes | Closed per-key token where the value domain is known |
| `launch_case` | `Option<CompatibilityLaunchCase>` | no | Mutually exclusive launch path, never used for handoff |
| `evidence_source` | `CompatibilityEvidenceSource` | yes | Observed run, user confirmation, imported catalog, or stale observation |
| `observed_at` | `Option<String>` | no | Text timestamp or equivalent source time |
| `fragcap_version` | `Option<String>` | no | fragcap version or commit that produced the observation |
| `target_version` | `Option<String>` | no | Target build/version clue, when known |
| `proxy_backend` | `Option<String>` | no | Backend identity, for example a native candidate or external proxy name |
| `proxy_backend_version` | `Option<String>` | no | Backend version, if known |
| `proxy_mode` | `Option<String>` | no | Configuration family or mode used to collect the observation |
| `final_owner_executable` | `Option<String>` | no | Local executable image that held final sockets, if observed |
| `final_owner_handoff` | `bool` | yes | Whether final socket ownership moved away from the initially launched executable |
| `stale` | `bool` | yes | Whether the row is retained as stale context |
| `note` | `Option<String>` | no | Local operator note, must be scrubbed before any export |

## `CompatibilityFactKey`

Closed fact families:

- `proxy-environment-honored`
- `proxy-routing`
- `proxy-propagation`
- `launch-case`
- `final-socket-owner-role`
- `publisher-launcher-present`
- `requires-platform-cold-start-for-proxy`
- `direct-exe-supported`
- `steam-protocol-supported`
- `tls-trust-behavior`
- `protocol-behavior`
- `inspectability`
- `proxy-variable-tested`

## `CompatibilityLaunchCase`

Closed launch cases:

- `steam-protocol-warm`
- `steam-protocol-cold`
- `direct-exe-warm`
- `direct-exe-cold`
- `publisher-launcher`
- `publisher-launcher-warm`
- `publisher-launcher-game-start-clean-warm`
- `publisher-launcher-cold`

Owner handoff is deliberately not a launch case. It is stored in `final_owner_handoff`.

## `CompatibilityEvidenceSource`

Closed evidence sources:

- `observed-run`
- `user-confirmed`
- `imported-catalog`
- `stale-observation`

## `deep_capture_facts`

SQLite table added in schema version 9. The table is created for fresh stores and by the v8-to-v9 migration. Existing stores receive an empty table only.

CHECK constraints enforce fact keys, launch cases, evidence sources, boolean fields, non-empty optional string fields where empty strings would be meaningless, and per-key fact-value vocabularies.
