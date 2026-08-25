# Contract: `deep_capture_facts`

The compatibility fact table is local-only storage in the targets database.

## Required columns

- `target_id`: foreign key to `targets(id)`, cascade delete.
- `fact_key`: closed compatibility fact key.
- `fact_value`: key-specific value token.
- `evidence_source`: closed evidence source.
- `final_owner_handoff`: boolean integer, default `0`.
- `stale`: boolean integer, default `0`.

## Optional columns

- `launch_case`
- `observed_at`
- `fragcap_version`
- `target_version`
- `proxy_backend`
- `proxy_backend_version`
- `proxy_mode`
- `final_owner_executable`
- `note`

## Required behavior

- Opening a v8 store creates the table and stamps schema version 9.
- Opening a v8 store never inserts compatibility facts.
- Invalid fact keys, launch cases, evidence sources, boolean values, empty required values, and invalid key/value pairs are rejected.
- Deleting a target deletes that target's compatibility facts.
