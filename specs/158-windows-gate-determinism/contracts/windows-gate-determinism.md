# Contract: Windows gate determinism

## Application writer publication

The application artifact constructor MUST NOT return its lease until the dedicated writer has signaled readiness immediately before entering its receive loop.

The signal is internal and one-shot. It does not appear in application JSON Lines, lifecycle streams, manifests or operator output. Queue capacity and every serialized artifact contract remain unchanged.

If the worker ends before readiness, construction returns an I/O error whose stable public-facing meaning is `application writer stopped before readiness`. The constructor retires the sink, closes its sender and joins the worker before returning.

## Proxy observation drain

The stable proxy adapter keeps its existing observation method and gains a defaulted terminal drain method that returns one of these statuses with its ordered observations:

- `complete`: all terminal observations represented by the bounded proxy stop result were collected.
- `incomplete`: terminal collection is not complete, with a stable code and bounded detail.

The native adapter may return `complete` from a cached clean stop observation even when the remaining shutdown budget is zero because that path performs no waiting. A clean stop report, released listener, zero incomplete tasks, no residue, stopped observation state, zero live connections and zero current connection tasks are all required. Live collection with no remaining budget, owner-thread failure or any failed predicate is incomplete.

The session retains all returned observations. It records an observation-stage failure for `incomplete` or adapter error. It MUST NOT compare a complete drain with the capture observation clock.

## Deadline predicates

Capture succeeds only when its run returns within `observation`.

Capture stop, proxy stop and observation drain share the remaining `shutdown` budget from one monotonic origin. Crossing that total boundary records `shutdown-deadline-exceeded` even when partial evidence exists.

Complete cached drain returned within the shutdown stage does not fail because the earlier capture duration has elapsed.

## Calibration test isolation

All tests that mutate any controlled calibration variable acquire the same controlled-environment guard through poison recovery and use scope-owned exact restoration.

A panic may poison the mutex and fail its own test. The next test recovers the guard and runs. No later test may call `lock().unwrap()` on this mutex or depend on manual environment cleanup after its last assertion.

## Hosted acceptance

The unchanged S128 performance registry remains the authority. Windows acceptance requires fourteen passing cases, seven windows per case, zero hard invariant failure, zero application-event loss, zero queue-byte loss, zero storage-byte loss, zero failure-detail loss and clean shutdown.

The Windows platform workflow must run the exact #429 regression and complete without secondary poison failures. Only first-attempt green conclusions on the final pull-request head satisfy S158.
