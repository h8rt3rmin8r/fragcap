# Collection Contract: S171

## CLI

`fragcap bundle collect` previews the configured session root. `--include-retained` explicitly selects historically retained contents; custom output requires `--bundle <exact-path>`. `--purge-empty` separately proposes eligible empty-container removal. `--authorize <proposal-id>` applies only an unchanged exact proposal; preview alone is read-only. Structured output includes policies, eligible/retained/unresolved sessions, recoverable and actual removed logical bytes, container outcomes and visible scan limits.

`--retain-bundle` on Deep Capture selects explicit retained evidence for generated output. Explicit `--bundle` and guided subdirectories inherit explicit retention. Shared authorization, metadata and terminal presentation expose the selected policy. Automatic maintenance runs only after an authorized effectful session and excludes the current returned bundle.

## Facade

Collection inspection validates a bounded whole-session population and returns exact proposal identity, session identity, logical bytes and recognition/limitations. Applying recomputes/pins the complete population, synchronizes an external retirement transaction before any removal and performs exact object deletion. Retry reads validated transaction authority when former manifest/journal files are absent. Purge independently checks exact root ownership and emptiness. CLI additionally supplies inactive-generation authority and retention selection.

Agreed APIs: `inspect_session_collection(bundle, retirement_store) -> CollectionInspection`; `collect_session_contents(bundle, retirement_store, authorization_id) -> CollectionReport`; `purge_session_container(bundle, retirement_store, authorization_id) -> CollectionReport`; `collected_session_container(bundle, retirement_store) -> bool`. Each returns an I/O result. Inspection exposes `id`, `bundle`, `session_id`, `bytes`, `files`, `empty`, `resumed`, `owned`, `completed_at_unix_ms` and `json()`. Report exposes proposal/session identities, actual removed files/bytes, preserved/purged container, completion, object results, limitations and `json()`.

CLI orchestration APIs: `preview(root, bundle, include_retained, purge_empty)`, `apply(root, bundle, include_retained, purge_empty, authorize)`, `maintenance(root, exclude)` return I/O results with JSON values. `declare_retention(bundle, managed)` writes protected `.session-retention.json` after authorized bundle preparation. Metadata carries exact integer creation time, class, age/count/byte limits and preserved-container semantics; terminal manifest publication time supplies policy age origin. Retirement store is the root-level `.session-retirements` directory.

Aggregate `recoverable_bytes`, `removed_bytes` and `removed_files` describe bundle contents. Owner-registry metadata retirement is reported separately as an exact outcome; its metadata bytes and retained external transaction bytes are not included in those content totals. Per-session reports expose `removed_bytes` plus `removed_logical_bytes`; per-path results expose `bytes` plus `logical_bytes` for the same logical quantity.

## Doctor

Read-only inventory recognizes exact completed retirement records and empty containers without false malformed findings. Healthy and empty history do not consume actionable finding capacity. Any deadline, record/population bound, unknown or unreadable state remains visible.
