# Research: Bounded Guided Calibration Sequence

## Decision: Rebuild the proposal at every boundary

**Rationale**: S139 already combines exact target topology, caller-proven cold state, current applicable facts, reachability precedence, and deterministic protocol ordering. Reopening the store, resolving the durable target, taking a process snapshot, and rebuilding S139 before every attempt makes current authority the sequence state.

**Alternatives considered**: Mutating the prior proposal in memory, which misses concurrent facts and process changes. Reading facts only after a session, which misses target and topology drift before the next plan.

## Decision: Define progression by the attempted fact becoming current-positive

**Rationale**: A session can exit cleanly while seeing no traffic or only partial traffic. The attempted case is complete only when a fresh proposal no longer requires it for the exact current launch, route, family, backend, product, and target-version dimensions.

**Alternatives considered**: Progress on exit zero, retained observations, or bundle finalization. Each can be true without a compatible current fact.

## Decision: Track exact attempted case identity

**Rationale**: A finite supported vocabulary alone does not prevent a policy or evidence bug from selecting the same case repeatedly. Recording launch case, route, address family, phase, and protocol makes repetition an explicit no-progress outcome before another authorization plan is built.

**Alternatives considered**: Count iterations only, which eventually stops but may repeat effects. Track protocol alone, which would conflate reachability and exact launch dimensions.

## Decision: Use the supported matrix as the closed upper bound

**Rationale**: S139 can select one routing case or one case for each of thirteen concrete supported protocols. The current maximum is therefore fourteen, and one-per-exact-case tracking proves the bound mechanically.

**Alternatives considered**: A configurable retry count, which invites repeated effects. A hard-coded smaller limit, which can silently truncate explicitly requested supported coverage.

## Decision: Accumulate observed candidates, not observed completion

**Rationale**: Existing final-client observation policy says which concrete protocols are worth measuring. S121 facts say which measurements are current and positive. Keeping these roles separate prevents an observation from being promoted into compatibility evidence.

**Alternatives considered**: Mark observed protocols complete or replace the requested set with observations. Both erase the request, observation, and fact distinction.

## Decision: Derive later explicit bundle paths as siblings

**Rationale**: Existing users and tests expect the supplied path to be the first session bundle. That directory becomes non-empty after completion, so later sessions need peers such as `<name>-attempt-02-tls-https`. Fixed enum vocabulary and a numeric index make names deterministic and path-safe. The existing low-level validation refuses collisions.

**Alternatives considered**: Turn the supplied path into a parent, which breaks current first-attempt layout. Reuse it, which correctly fails but prevents progression. Delete or overwrite it, which is prohibited.

## Decision: Treat a later decline as a clean bounded stop

**Rationale**: Interactive default-no already means no effects and success. The sequence should preserve completed evidence and emit current continuation without turning an operator choice into an operational failure.

**Alternatives considered**: Fail the whole invocation or roll back earlier facts. Earlier facts are independently authorized append-only observations and must remain.

## Decision: Add ordered attempt fields to existing guidance

**Rationale**: `calibration.guidance` already carries selected case and coverage. Adding one-based attempt and maximum fields lets human and JSON consumers reconstruct the sequence without a new persisted workflow or event family.

**Alternatives considered**: Infer order from event count, which is fragile because plan and session events interleave. Add a workflow identifier and schema, which belongs with later persistence and resume.
