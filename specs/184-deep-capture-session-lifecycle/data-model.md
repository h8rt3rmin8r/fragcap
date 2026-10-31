# S184 data model

## Owner selection

- `ProvenOwner`: existing exact process identity, ancestry depth, action state, and verified exit.
- `UnresolvedDeclaredEvent`: PID and event time plus a stable reason; no control handle or control authority.
- `OwnerSelection`: proven owners plus unresolved observations. Global event-authority failure remains a separate error.
- Transition: observed -> proven or unresolved; proven -> graceful requested -> forced if still exact and live -> exit verified or incomplete. Unresolved never transitions to controllable without new exact proof.

## Timing

- `RequestedObservation`: parsed operator duration, absent default, or invalid input.
- `EffectiveObservation`: mode-specific accepted duration bound to the plan.
- `ReturnAllowance`: finite separately visible time for Capture return and artifact drain; it does not shorten `EffectiveObservation`.
- Transition: parsed -> validated before effects -> plan authorized -> armed -> stopped -> terminal evidence. A duration refused at validation reaches no session effect.

## Cleanup

- `RouteOwnerResult`: complete only after exact and plausible owner exits are verified; otherwise partial failure with causal reason.
- `RouteLeaseResult`: local lease cleanup result distinct from inherited route lifetime.
- `TrustObligation`: exact installed session root identity retained for Doctor when plausible owners remain live. It is not an installation-owned CA.
- `ProxyContinuity`: whether a live routed owner still has a running proxy. Finite proxy shutdown may make continuity unavailable even while trust remains.
