# Research: Scope-Aware Calibration and Operator Journey

## Decision 1: One exact scope vocabulary across stores, facade, and CLI

**Decision**: Public input uses `auto|game|launch-chain`. Durable requested scope records `auto|exact-game|launch-chain`; effective scope is always `exact-game|launch-chain` before effects. The existing S191 policy identifiers `exact-game-v1` and `launch-chain-v1` remain authoritative.

**Rationale**: S191 already supplies the pre-effect resolution boundary and S193 consumes its frozen effective decision. Reusing it prevents a second policy model. The shorter public `game` token remains discoverable while durable and artifact vocabulary stays exact.

**Alternatives considered**: `exact-game` as the public token was precise but inconsistent with the issue's operator vocabulary. Destination-based bypass was rejected because shared-hostname fixtures require process ownership.

## Decision 2: Add scope to the exact case and facts, preserve legacy fact absence

**Decision**: Add nullable requested scope, effective scope, middleware handling, and policy identity columns to compatibility facts. Add non-null immutable equivalents to durable workflows. Existing fact rows remain null and therefore `legacy-incomplete`; existing workflows migrate to their actual historical launch-chain behavior with retry and fallback disabled.

**Rationale**: Facts with missing dimensions cannot safely qualify game scope. Existing workflows were created under a launch-chain-only CLI, so making that workflow intent explicit preserves meaning rather than inventing a new choice.

**Alternatives considered**: Rewriting all facts to launch-chain would erase the distinction between observed dimensions and inferred history. A new parallel fact table would violate the one-store and one-target path.

## Decision 3: Store successful preference as observed scoped evidence

**Decision**: Add a closed compatibility fact key for scope recommendation, written only from a qualifying completed exact attempt. Topology-derived game scope remains an in-memory proposal and is never stored as an observation.

**Rationale**: The operator needs later ordinary Deep Capture to reuse a measured preference while P-9 requires topology inference and observed compatibility to remain distinct.

**Alternatives considered**: A mutable target column would hide history and conflict. A separate recommendation store would add a second precedence path.

## Decision 4: One linked game-scope retry, separately declared packet fallback

**Decision**: Workflow intent contains independent booleans for one linked game-scope retry and packet-only fallback. A successor gets a fresh attempt identity and predecessor link after exact cleanup reconciliation. Passive fallback records effective Capture mode and its real observation window.

**Rationale**: Independent authority keeps normal users on a short path, preserves the original failure, and prevents resume from widening effects. One retry fits the existing finite attempt model without an unbounded loop.

**Alternatives considered**: Automatic fallback on every failure was rejected because it widens the reviewed plan. Midstream TLS downgrade was rejected because it falsifies the frozen connection decision.

## Decision 5: Integrate operator surfaces in the same slice

**Decision**: Calibration and Deep Capture share the same parser enum and facade conversion. Authorization, normal/verbose/JSON output, targets, Doctor, help, public schema, and documentation project the same durable values.

**Rationale**: #560 creates the data and execution contract consumed directly by #561. Splitting them would temporarily ship a persistent contract with no complete operator path and would duplicate cross-surface verification.

**Alternatives considered**: A separate presentation slice was rejected because the two issues share parser, plan, workflow, event, schema, and regression boundaries.

## Decision 6: Package evidence stops at the artifact boundary

**Decision**: Existing Windows CI package certification must parse and exercise the controlled scope contract against exact packaged bytes and record build identity. Installed-product behavior and private-title checks happen only after release.

**Rationale**: The constitution closes implementation issues on repository-controlled evidence and converts active-release failures into defects. This also follows the owner's explicit testing policy.

**Alternatives considered**: A pre-release installed-product prerequisite was rejected because it would create a standing completion condition outside repository control.
