# Data Model: Complete Process Lifecycle Evidence

## Capture Process Evidence

- Startup snapshot records and snapshot instant
- Bounded observed process-event sequence
- Count of events not retained at the bound
- Managed launch case and optional receipt PID
- Observed role and stage transitions
- Watcher report: event loss, buffer loss, running state, rundown ignored
- Session terminal reason and whether the watcher ended unexpectedly

The report is returned by the shared capture orchestrator. Ordinary Capture may ignore it; Deep Capture reconciles it into its sensitive sidecar.

## Process Instance

- `instance_id`: stable session-local identifier
- `pid`: observed operating-system identifier
- `created_at`: creation timestamp when ETW observed it
- `parent_pid`: creating parent from ETW, or query parent from a snapshot
- `parent_instance_id`: present only when the parent lifetime is resolvable
- `image`: observed image path or name
- `command_line`: observed value or explicit unavailable state
- `ancestry_authority`: `creation-event`, `query-snapshot`, or `unresolved`
- `exit_at`: observed exit time when it belongs to this lifetime
- `relevance`: launch root, declared stage, flow owner, ancestor, or a combination

PID reuse creates a new instance whenever a later start event has a distinct creation timestamp. An exit applies only to the latest non-exited instance whose creation is not after the exit.

## Stage Transition

- Event time
- Kind: matched or exited
- PID and process instance when resolvable
- Role and stage
- Process image when observed
- Authority reason

## Socket Owner Transition

- Flow identifier
- Observation interval start and end
- Process instance when resolvable
- PID, process, role, stage, and fidelity from packet attribution
- Correlation state and limitation reason

Adjacent observations with identical ownership collapse into one interval. A change in PID, process instance, role, stage, fidelity, or attribution availability begins a new interval.

## Process Trace Header

- Record type and schema version
- Session and target anchors
- Launch case
- Collection bound
- Snapshot and watcher authority availability

## Process Trace Trailer

- Finalization and completeness state
- Terminal session state and stop reason
- Counts by record and limitation kind
- Watcher, retention, writer, and unresolved-join loss
- Shared flow-anchor reconciliation counts

Only the trailer can declare orderly completion. Any nonzero unaccounted or unavailable evidence class weakens completeness according to its typed reason.

## State Transitions

```text
header -> evidence records* -> terminal record -> trailer
header -> evidence records* -> interrupted/crash prefix
```

Process instance:

```text
snapshot-limited -> start-reconciled -> exited
observed-start -> stage-bound -> socket-owner* -> exited
observed-start -> terminal-without-exit
```
