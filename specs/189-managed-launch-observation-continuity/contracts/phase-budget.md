# Phase Budget Contract

The pre-effect human and JSON plan must carry the same effective values:

| Phase | Start | Authorized end |
| --- | --- | --- |
| Acquisition | Capture arm | Verified current client, explicit cancellation, or finite wait expiry; resumes for a proven provisional handoff within the same finite acquisition allowance |
| Eligible observation | Verified current client binding | Requested duration, explicit cancellation, true terminal condition, or reported failure |
| Owner release | Effective observation end | Complete proven owner exit or finite release expiry |
| Proxy drain/shutdown | Owner-release decision | Drained/stopped or finite shutdown expiry |
| Cleanup | Proxy shutdown decision | Exact obligations settled or finite cleanup expiry |

The plan names the launch topology, default source or exact operator override, each duration in milliseconds, guaranteed post-acquisition budget, worst-case post-acquisition budget under normal running conditions, early terminal exceptions, and worst-case attempt span. A successful acquisition at any accepted time gets the full requested eligible observation budget. A proven successor following provisional handoff gets a new full eligible interval within the finite attempt ceiling. Planning rejects a request that cannot be honored; runtime never clamps it after authorization.

The terminal result and manifest report effective phase values and the observed stop reason. Actual elapsed and remaining time for each phase remains open in #553. A budget is a time allowance, not a promise of target activity or captured traffic. Packet arming and pre-client evidence occur before eligible observation and are counted independently.
