# Contract: Exact Shutdown and Historical Cleanup

## Pre-effect authorization

The accepted plan includes `shutdown.mode = exact-owned-managed-launch`, declared direct/Steam/publisher owner chain, `grace_seconds = 60`, `force_by_seconds = 90`, `release_ceiling_seconds = 120`, and a plain unsaved-progress warning. The plan identity covers these fields. Approval covers the later automatic close; there is no second prompt.

## Route-owner release

Each candidate requires the session launch receipt, exact process-object identity, creation instant, canonical image path, user/session match, and retained causal ancestry. A mismatch or missing observation records a refusal, with no control call. A process-bound graceful request precedes any force when available. Force uses only named minimal rights and only after revalidation. A return from a close or terminate call is an attempt result; exit verification is a separate transition. Proxy and trust cleanup follow verified release or the finite failure deadline.

The original ETW watcher remains live after packet Capture ends and throughout the release interval. Every release poll folds new process events and rechecks descendants; a graceful-close handler that starts a helper therefore changes the owner decision. Only after all known route-capable processes exit may the watcher stop. Its queued events are then consumed and the complete descendant set is checked once more before a released result. Watcher loss, missing receipts, an undeclared live descendant, or a late live owner produces an unresolved result.

## Doctor projection

Terminal manifest and cleanup projection preserve `historical_state`, `historical_cleanup_status`, failed resource and reason. Current journal and process state retain `current_health`, `current_readiness`, and `recovery_eligible`. A historical failed route owner with no current process may have `current_readiness = true` and no fix action. Human, JSON, and history detail text must agree; no retained file is rewritten.
