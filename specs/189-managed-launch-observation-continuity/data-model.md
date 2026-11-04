# Data Model: Managed Launch Observation Continuity

## Managed client binding

One binding identifies a process instance by PID, observed creation instant, canonical executable path, account, session, and causal parent chain. It references the declared target stage and one launch receipt. A PID or image name alone is never a binding key. Exited ancestors stay available for subsequent causal joins. A replacement binding is permitted only under the managed handoff policy and only when the predecessor has entered provisional handoff.

## Handoff state

`AwaitingClient` permits a verified first client before the launch wait expires. `ObservingClient` retains packets for that exact instance. `ProvisionalHandoff` records the exited predecessor, already-proven owned descendants, finite remaining acquisition/handoff budget, and candidate successors. `ObservingSuccessor` begins a full eligible interval for the verified replacement. `IncompleteHandoff` records the actual chain and refusal/timeout reason. `EndedObservation` begins the separate owner-release phase. No transition from provisional handoff directly authorizes process shutdown.

## Lifecycle budget ledger

The immutable authorized plan has startup, capture preparation, acquisition, eligible observation, owner-release, proxy-drain/shutdown, and cleanup durations, phase-start rules, and their cumulative finite bound. The runtime uses a monotonic clock for phase decisions and records the actual terminal reason. A post-acquisition *budget* equals the requested observation duration under normal running conditions; interruption, real terminal exit, and resource failure remain named early endings. Pre-client, provisional, final observation, owner release, and cleanup windows never share a success verdict by timestamp alone.

## Retained evidence

Stage transitions preserve original event text and separately verified path/instance authority. Packet role stamps are valid for the exact instance and event interval. Proxy observations retain their own connection and process-correlation authority. Compatibility facts require eligible final-client evidence. A partial historical result remains immutable when current Doctor recovery later observes that an unresolved helper exited.
