# Data Model: Doctor ETW Session Probe

## Probe-Only ETW Session Check

Runtime check exposed through the ETW watcher surface.

- `session_name`: trace session name to start.
- `result`: `Ok(())` when the session starts and provider enables, `Err(WatcherError)` otherwise.
- `cleanup`: owned `Session` drop disables the provider and stops the trace session.

## Full ETW Watcher

Existing capture watcher.

- `session`: running ETW session.
- `consumer`: thread-backed ETW consumer reading `ProcessTrace`.
- `snapshot`: startup process snapshot used by the process tree.
- `fanout`: event distributor and counters.

## Tracing Availability Verdict

Existing doctor input.

- `None`: ETW backend not linked for this binary or platform.
- `Some(true)`: ETW backend linked and the probe-only session opened.
- `Some(false)`: ETW backend linked and the probe-only session did not open.

## State Transitions

```text
backend unavailable -> None
probe session start fails -> Some(false)
probe provider enable fails -> Some(false), started session drops
probe session start succeeds -> Some(true), started session drops
full watcher start succeeds -> capture watcher owns session, consumer, snapshot
```
