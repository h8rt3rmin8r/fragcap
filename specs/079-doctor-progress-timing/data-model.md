# Data Model: Doctor Progress And Timing

## Probe Progress Item

Represents one visible unit of doctor work.

Fields:

- `name`: Stable human label for the probe group.
- `state`: `running`, `complete`, `failed`, or `unknown`.
- `detail`: Optional short diagnostic detail when a probe cannot make a normal
  determination.
- `elapsed`: Optional elapsed duration once the probe has left `running`.

Validation:

- `name` must be one of the labels named in the contract.
- `running` items must not carry `elapsed`.
- Terminal items must carry `elapsed` when timings are enabled.
- A terminal item may not imply readiness success; readiness remains determined
  only by the final doctor report.

## Probe Timing

Represents measured cost for one completed probe group.

Fields:

- `name`: Same stable label used by the progress item.
- `elapsed_ms`: Elapsed wall-clock milliseconds measured around the real probe.

Validation:

- `elapsed_ms` is non-negative.
- Timings are ordered in the same sequence probes ran.
- Timings are diagnostic output only; they are not part of the stable JSON
  schema or final human report.

## Progress Session

Represents one doctor invocation's progress stream.

Fields:

- `enabled`: Whether progress may be emitted.
- `timings`: Whether terminal progress lines include elapsed durations.
- `items`: Ordered progress items seen so far.

State transitions:

- Session starts disabled unless the command is an interactive human doctor run.
- Each item transitions from absent to `running`, then to a terminal state.
- The session ends after report rendering finishes or the command returns an
  error.
