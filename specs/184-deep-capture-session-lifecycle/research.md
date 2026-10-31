# S184 research and decisions

## Decision 1: Separate verified owners from unresolved observations

`shutdown::select_owners` currently returns an error immediately when a declared-image start lacks ancestry. `release_monitored` therefore never requests graceful close of separately proven owners. Keep global evidence-integrity failures as refusals, but return an owner selection containing verified owners and independently recorded unresolved events for per-event uncertainty. Continue exact-owner release and exit verification. An unresolved plausible event prevents complete route release; it never grants control rights. Re-run selection after watcher closure so late events remain visible.

## Decision 2: Reconcile local route cleanup with inherited route lifetime

The child-environment route lease can finish its own cleanup while a child still holds inherited proxy variables. Session cleanup currently reports its local `route` result without considering the preceding `route-owner` result, and the routing lease emits `target-route` success independently. Preserve the local cleanup attempt, but report inherited route as incomplete until owner release is verified. Keep causal details in cleanup records and Doctor. Proxy and trust remain active until the bounded owner-release decision. If an unresolved plausible live owner survives that decision, retain exact trust for Doctor recovery and report that a stopped proxy cannot provide route continuity. Do not present this interim behavior as completion of #528's installation CA requirement.

## Decision 3: Mode-specific effective observation and finite return budget

`CalibrationDeadlines::from_args` currently applies a 60-second calibration cap to every mode. The facade separately caps observation at 300 seconds. The real Capture adapter reserves up to five seconds from the same observation budget, shortening an ordinary request even if the 60-second cap is removed. Keep calibration's 60-second maximum. For ordinary Deep Capture, use the existing five-minute ceiling as an explicit pre-effect limit and preserve the requested duration within it. Account for a separately visible finite Capture return allowance so the outer coordinator does not force early stop or flag normal return as overrun. Plan, runtime, and terminal identities must agree; no second silent clamp may change an accepted plan.

## Decision 4: Controlled evidence and delivery

Windows process tests use hidden controlled children and exact process-object receipts, including an unproven same-name observation. An installed-path controlled target run longer than 60 seconds checks the emitted authorization, actual elapsed observation and packet span, terminal status, and cleanup. The private title and account audit remains source evidence only. Repository gates, relevant Windows/package checks, exact-head CI, and at most two external review rounds determine PR handoff. Final merge and release publication remain owner decisions.
