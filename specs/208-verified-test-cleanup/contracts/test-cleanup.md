# Test Cleanup Contract

## Contributor-visible result

A test handoff states the effective canonical roots used, exactly owned outputs, cleanup operations, absence verification, and any residual paths with causes. A test assertion result and a cleanup result are separate facts. A clean claim requires both to pass.

## Runner outcomes

| Test assertion | Exact cleanup | Reported outcome |
| --- | --- | --- |
| Pass | Absent | Pass |
| Pass | Residual or error | Fail with exact path and cause |
| Fail | Absent | Fail with original assertion |
| Fail | Residual or error | Fail with original assertion and residual detail |
| Interrupted | Unknown | Unresolved with exact owner record; no clean claim |

## Reconciliation authority

Normal cleanup may remove only the exact root created and retained by the current run after its child has exited and artifacts have been consumed. Later reconciliation must validate the recorded canonical root, owner identity, inactivity, and absence of conflicting recovery obligations. An unknown or ambiguous path is reported and preserved. A prefix match alone never authorizes deletion.
