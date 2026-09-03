# Data Model: Native Deep Capture Doctor Readiness

## ModeScope

- `Capture`: applies only to ordinary Capture.
- `DeepCapture`: applies only to Deep Capture.
- `Shared`: applies to both.

Deep Capture additionally consumes Capture-scoped prerequisites because its
bundle includes packet capture.

## ModeVerdict

- `mode`: stable mode identifier.
- `ready`: true only when every applicable check is non-failing.
- `blocking_checks`: ordered stable check identifiers. Native identifiers use
  session and resource identity rather than an ordinal.

## SessionOwnerRecord

- `version`: owner-record schema version.
- `bundle`: canonical path beneath the configured session root.
- `owner_pid`: diagnostic process identifier, never liveness authority alone.
- `lease_id`: bounded opaque identifier for the exact session generation.

The live named lease plus matching canonical record proves `active`. PID alone
never does.

## NativeResidueInventory

- `sessions`: bounded ordered `SessionObservation` values.
- `findings`: bounded ordered `ResourceFinding` values.
- `limitations`: bounded ordered `InventoryLimitation` values.
- `truncated`: exact omitted counts by bound.

## SessionObservation

- `session_id` and canonical `bundle`.
- owner-record state and lease state.
- journal status and manifest status.
- exact journaled listener endpoints.
- declared artifact observations.

## ResourceFinding

- `session_id`, `bundle`, resource identity, kind, and latest lifecycle state.
- `ownership`: lease, journal, manifest, or insufficient.
- `health`: absent, healthy, active, stale, cleanup-failed, unknown, unsupported.
- `recovery`: none, exact action, explicit-confirmation legacy action, or stable
  refusal.
- `detail`: bounded non-secret operator explanation.

## InventoryLimitation

- stable reason code.
- affected authority or path.
- observed and omitted counts where known.

Any limitation that prevents a required authority from being evaluated blocks
Deep Capture readiness. It is never normalized to absence.

## State transitions

Doctor does not define resource transitions. It consumes the S109 journal state
machine and its `recovery_plan()`. A repair appends only transitions already
authorized by that implementation, then a fresh inventory derives the new
health state.
