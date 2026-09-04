# Research: Native Deep Capture Failure Injection

## Decision 1: Generate the matrix from owned boundaries

**Decision**: Store effect and lifecycle boundary definitions once, then generate exactly two scenario cells per boundary: before and after.

**Rationale**: Hand-authored scenario lists can omit one side while still looking comprehensive. Cartesian generation makes the coverage count and missing side mechanically decidable.

**Alternatives considered**:

- Maintain an informal test checklist. Rejected because it cannot detect production inventory drift.
- Store every expanded row manually. Rejected because duplicate and missing sides become review problems rather than schema violations.

## Decision 2: Use production adapters and authorities

**Decision**: Inject external effect failures through the existing facade adapters and assert actual coordinator, artifact, fact, event, cleanup, journal, and recovery types.

**Rationale**: The adapters already define the effect boundary and keep tests portable. A separate lifecycle simulator could prove its own behavior while the coordinator remained wrong.

**Alternatives considered**:

- Add environment-variable fault switches to production. Rejected because ambient mutable state is concurrency-unsafe and creates a shipped hidden control surface.
- Trigger destructive host failures directly. Rejected because disk exhaustion, trust denial, and port theft would be nondeterministic and could affect unrelated resources.

## Decision 3: Treat post-effect bookkeeping failure as uncertain acquisition

**Decision**: An after-side injection retains the durable pending obligation and requires bounded cleanup or exact recovery, even when the applied transition is unavailable.

**Rationale**: Failure to record success cannot prove the effect did not happen. Treating it as non-acquisition would create the exact cleanup gap S109's pending state was designed to prevent.

**Alternatives considered**:

- Mark the effect not applied when the success record fails. Rejected because this invents evidence.
- Retry the effect to determine state. Rejected because repeated trust, launch, or listener effects are not generally idempotent.

## Decision 4: Keep failure dimensions independent

**Decision**: Each executable case asserts a vector of terminal outcome, artifact, fact, event, cleanup, journal, and recovery dispositions.

**Rationale**: One aggregate success bit permits a successful cleanup to hide a corrupt artifact or permits an event failure to suppress evidence. Independent assertions protect authority boundaries.

**Alternatives considered**:

- Assert only the terminal outcome. Rejected because several valid outcomes are partial for different reasons.
- Assert only journal state. Rejected because the journal does not own artifact or fact truth.

## Decision 5: Bind review currency to source inventories

**Decision**: The task-runner gate extracts resource kinds, lifecycle states, the coordinator's executable lifecycle-edge table, and coordinator resource identifiers from their owning sources and compares them with the registry. Every coordinator state mutation passes through the same checked edge table.

**Rationale**: A new state or effect changes the matrix obligation. Source-derived comparison forces the same pull request to supply failure evidence.

**Alternatives considered**:

- Keep a second hard-coded Rust inventory. Rejected because two stale lists can agree.
- Depend on reviewer memory. Rejected because completeness is a mechanical property.

## Decision 6: Add no dependency

**Decision**: Reuse serde_json, the task runner, existing integration-test dependencies, and controlled adapters.

**Rationale**: S127 needs orchestration evidence, not a mocking framework. The existing seams can represent every required failure family with stable typed results.

**Alternatives considered**:

- Add a fault-injection crate. Rejected because it expands the product graph without supplying authority the current adapters lack.
