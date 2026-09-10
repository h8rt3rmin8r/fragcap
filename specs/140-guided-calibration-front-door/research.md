# Research: Guided Reachability Calibration Front Door

## Decision: Use S139 as the only proposal policy

**Rationale**: `propose_calibration` already owns topology, complete-snapshot cold proof, exact S121 applicability, retest reasons, reachability-first ordering, and defaults. The CLI supplies observations and renders the result.

**Alternatives considered**: Reuse the older launch-case helper alone, which loses S139 fact reasons and limitations. Reimplement proposal logic in the command, which creates a second policy path.

## Decision: Reuse current target authorities

**Rationale**: The low-level command already refuses an absent store, resolves positional or durable identifiers, preserves row-index meaning, reports ambiguity, and reads target-scoped facts. Crate-private visibility lets the front door reuse those outcomes.

**Alternatives considered**: Synthesize a Capture profile through `target_resolve`, which does work the proposal does not need. Duplicate resolver logic, which risks precedence drift.

## Decision: Extract a complete process-image snapshot helper

**Rationale**: S139 needs the complete image list. Existing Tool Help enumeration is the permitted query-only mechanism. A crate-private helper returns raw names and keeps the current Boolean helper as a projection.

**Alternatives considered**: Enumerate only declared images, open target processes for paths, or treat failure as cold. Each violates the S139 or P-1/P-9 boundary.

## Decision: Project one proposal into four guided actions

**Rationale**: `run-reachability`, `operator-action`, `ready`, and `refused` cover S139 output without exposing its Rust representation or losing topology, case, reason, and limitation facts.

**Alternatives considered**: Ad hoc progress gives JSON no stable contract. Direct proposal serialization couples CLI compatibility to additive library fields.

## Decision: Run only routing reachability

**Rationale**: No protocol candidates yield either the required routing step or no protocol work. Current exact positive routing evidence authorizes the ordinary Deep Capture handoff without guessing TLS support.

**Alternatives considered**: Guess HTTPS, infer protocols from bundle files, or run TLS immediately. Each invents evidence or crosses later #380 scope.

## Decision: Preserve existing events and add one guided envelope

**Rationale**: One `calibration.guidance` event states decision, outcome, and next command. Existing authorization, routing, lifecycle, calibration, cleanup, and artifact events remain authoritative during execution.

**Alternatives considered**: Rename existing events or add one event per internal field, both of which increase compatibility surface without value.
