# Research: Guided Protocol Calibration Attempt

## Decision: Reuse the S139 proposal for ordering

**Rationale**: S139 already enforces complete cold proof, current exact routing precedence, deterministic candidate ordering, exact applicability, and suppression of current positive protocol facts. S141 supplies candidates and executes one returned step.

**Alternatives considered**: Select the first CLI argument, which ignores current facts and stable ordering. Add a second sequencing policy in the command, which would drift from the facade contract.

## Decision: Expose one pure observed-candidate helper

**Rationale**: Candidate derivation needs the same concrete-family mapping and final-client correlation authority used by compatibility facts. A facade helper can exclude unknown and unrouted traffic, require direct final-client ownership, and return a deterministic deduplicated set.

**Alternatives considered**: Infer candidates from target metadata, emitted event strings, or launcher observations. Each confuses a hint or unrelated observation with eligible traffic evidence.

## Decision: Preserve the typed terminal report in-process

**Rationale**: The low-level command receives the authoritative report after facts and cleanup. Returning its observation projection to a crate-private guided caller avoids a second capture and keeps the existing public CLI result unchanged.

**Alternatives considered**: Capture and parse JSON output, inspect bundle files after the run, or add a global side channel. Those are lossy, duplicate artifact authority, or introduce hidden state.

## Decision: Use repeatable `--protocol`

**Rationale**: The existing closed `DeepCaptureCalibrationProtocolArg` vocabulary gives clap validation, help consistency, and direct mapping to `CompatibilityProtocol`. Repeatable values make generated continuations explicit and parseable.

**Alternatives considered**: A comma grammar adds a second parser. Automatic guessing hides why trust was requested. A single value cannot carry bounded observed coverage between invocations.

## Decision: Execute one step and carry remaining candidates

**Rationale**: One authorization plan remains one operator-visible attempt. Re-reading exact facts on the next invocation naturally suppresses newly completed work.

**Alternatives considered**: An internal loop would bundle several trust-bearing sessions under one command and require persistent progress or ambiguous partial completion.

## Decision: Do not require a candidate to report ordinary readiness

**Rationale**: Existing product readiness is route-based. No observed candidate means protocol coverage is unknown, not that ordinary Deep Capture is unusable. The event and human output state both facts.

**Alternatives considered**: Refuse ordinary Deep Capture, which changes S140 readiness semantics. Guess HTTPS, which invents evidence.
