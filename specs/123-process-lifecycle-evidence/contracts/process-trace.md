# Process Trace JSON Lines Contract

## Framing

`process-trace.jsonl` is UTF-8 JSON Lines. Every complete line is independently readable. The first record is `process-trace.header`; orderly completion appends exactly one `process-trace.trailer` last.

## Header

Required fields:

- `type`: `process-trace.header`
- `schema_version`: `1`
- `session_id`
- `target_id`
- `target_handle`
- `launch_case`
- `event_limit`
- `snapshot_authority`
- `watcher_authority`

## Evidence records

Closed record kinds:

- `launch.receipt`
- `process.snapshot`
- `process.started`
- `stage.matched`
- `socket-owner.interval`
- `stage.exited`
- `process.exited`
- `process-trace.limitation`
- `session.terminal`

Every record carries `session_id`, `sequence`, and its observed or derived event interval. Process-bearing records carry `pid` and a nullable `process_instance_id`. Flow-bearing records carry the existing `flow_id` string.

## Stable limitation reasons

- `launch-pid-unavailable`
- `launch-generation-unavailable`
- `snapshot-creation-unavailable`
- `parent-instance-unavailable`
- `process-instance-unavailable`
- `process-exit-unobserved`
- `stage-instance-unavailable`
- `flow-owner-unavailable`
- `flow-owner-ambiguous`
- `packet-evidence-unretained`
- `watcher-event-loss`
- `watcher-unparseable-event`
- `watcher-buffer-loss`
- `watcher-ended`
- `event-retention-overflow`
- `stage-transition-retention-overflow`
- `unsupported-launch-authority`

## Trailer

Required fields:

- `type`: `process-trace.trailer`
- `schema_version`: `1`
- `session_id`
- `records`
- `process_instances`
- `flow_owner_intervals`
- `limitations`
- `events_lost`
- `unparseable_events`
- `buffers_lost`
- `rundown_ignored`
- `events_unretained`
- `stage_transitions_unretained`
- `unresolved_flow_owners`
- `terminal_state`
- `stop_reason`
- `completeness`: `complete`, `partial`, `unavailable`, or `failed`
- `finalization`: `complete`

A reader treats a missing trailer, duplicate trailer, malformed line, or record after the trailer as not orderly finalized.

Manifest version 2 preserves the trailer's `unavailable` completeness value
verbatim for a produced trace with no lifecycle authority.
