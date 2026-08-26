# Contract: ETW Session Probe

## Scope

This contract governs the readiness-only ETW session probe used by `fragcap doctor`. It is not the capture watcher contract and it does not add a public CLI output field.

## Inputs

- `session_name`: ETW session name.
- `platform`: Windows with the `etw` feature, or any other build.

## Outputs

- `None`: ETW backend is not linked or platform does not support it.
- `Some(true)`: probe-only session opened and provider enabled.
- `Some(false)`: linked probe could not open or enable the session.

## Rules

1. Non-Windows or non-`etw` builds do not call ETW and return `None`.
2. Windows `etw` builds call a probe-only `EtwWatcher` entry point.
3. The probe-only entry point starts and owns only the ETW session.
4. The probe-only entry point must not open a consumer, spawn `ProcessTrace`, or take a process snapshot.
5. A successful probe drops the session before returning.
6. A failed provider-enable attempt drops any session that started before returning the error.
7. Full capture startup continues to use `EtwWatcher::start`.
