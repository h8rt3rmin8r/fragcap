# Research: Exact Managed Shutdown and Cleanup History

## Existing boundaries

- `managed_launch.rs` currently drops the `Child` after recording its PID. The CLI's `CaptureProcessEvidence` retains the launch PID and bounded ETW events, but neither proves a live process object's identity.
- `deep_capture.rs` currently waits on declared image basenames during the 120-second route-owner release interval, asks the operator to close them, and reports failure if a name remains or inventory fails. This cannot authorize process control.
- Doctor's residue inventory treats a retained manifest as healthy whenever its journal is readable, even when the manifest's independent terminal cleanup status is failed. `manifest_cleanup_unfinished` already recognizes this condition, but does not expose the failed resource.

## Decisions and alternatives

| Decision | Chosen approach | Rejected alternative and reason |
| --- | --- | --- |
| Root ownership | Query the declared launch root when its start event arrives, retain a narrow process-object receipt, and bind it to launch PID and time window | PID alone can be reused; retaining a broad child handle does not state minimal rights |
| Descendant authority | Bind bounded ETW creation-time ancestry to the exact root and declared stage, then revalidate a live process object before control | Basename inventory and first-match iteration can select an unrelated process |
| Graceful close | Target only windows proven to belong to the exact PID and verify the process after a bounded wait | Broadcasting a title or image message can reach another application |
| Force | Only exact still-owned objects, with a named terminate right and post-call exit verification | Success return is not exit evidence |
| Steam sharing | Refuse control of the shared Steam root if unrelated title ownership cannot be excluded | Killing Steam could disrupt an unrelated game |
| Historical state | Manifest and cleanup projection govern the past; journal governs current tangible resources | A readable journal does not change the terminal outcome |

## Verification boundary

Controlled process fixtures can prove identity refusal and ordering without a real game. A real-title calibration can separately show compatibility on this machine. It cannot replace controlled same-name, PID-reuse, and failure tests.
