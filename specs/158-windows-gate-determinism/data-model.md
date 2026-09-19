# Data Model: S158 Windows gate determinism

## Writer readiness

`WriterReady` is an internal one-shot synchronization fact owned by the spawned application writer. It has no serialized representation and no public API. The lease constructor receives success only after the worker reaches the receive-ready boundary. Channel disconnection before that signal is startup failure.

State progression is `allocated -> worker-spawned -> ready -> lease-published -> draining -> joined`. Failure before `ready` progresses through `retired -> sender-closed -> joined -> refused`.

## Observation drain

`ObservationDrain` contains an ordered vector of compatibility observations and one `ObservationDrainStatus`.

`Complete` means the adapter finished collecting all terminal observations available from its bounded stop result within the supplied budget. `Incomplete` carries one bounded stable code and detail and may accompany partial observations that remain truthful.

The facade always retains returned partial observations. Only `Complete` can authorize successful collection. Adapter error remains a separate failed collection result.

## Deadline ownership

The existing `Deadlines` value remains unchanged.

- `launch` owns proxy, trust, route and managed-launch acquisition.
- `observation` owns ordinary Capture execution.
- `shutdown` owns capture stop, proxy stop and terminal observation drain as one total ceiling.
- `cleanup` owns final effect cleanup and recovery settlement.

No later stage reuses an earlier stage's clock origin.

## Controlled environment scope

`ControlledEnvironment` owns the recovered suite mutex guard and any environment restoration guards created beneath it. `EnvironmentValueGuard` stores the variable name plus its exact optional prior value. Drop restores the prior value or removes the variable when it was originally absent.

Poison recovery does not clear, rewrite or suppress the test panic that caused poisoning. It only permits the next test to serialize and restore shared process state.

## Invariants

- No sink exists outside the constructor before writer readiness.
- Queue capacity, current and peak remain event counts under the existing bound.
- Accepted events equal written plus dropped after terminal reconciliation.
- Complete drain is adapter evidence, not inferred from elapsed time or observation count.
- Incomplete drain never becomes successful because some observations exist.
- Shutdown work and drain together do not receive more than the authorized shutdown duration.
- Every environment mutation is restored on ordinary return and unwind.
