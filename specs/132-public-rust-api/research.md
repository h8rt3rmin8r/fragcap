# S132 Research: Stable Public Rust API

## Decision 1: Stabilize a curated facade module instead of every visible export

The current `fragcap::deep_capture` module uses wildcard re-exports from implementation modules. Rust visibility therefore exposes artifact writers, journal parsers, lifecycle helpers, controlled-test seams, and concrete backend support values beside actual product contracts. Treating that entire accidental surface as stable would prevent safe internal refactoring after the completion release.

S132 introduces `fragcap::deep_capture::api` as the explicit stable compatibility boundary. It re-exports session configuration, preparation, authorization, lifecycle control, adapter seams, routing contracts, observations, classification, facts, artifacts, recovery values, terminal reports, and the production native backend entry points required by consumers. Legacy top-level re-exports remain available during the pre-1.0 transition but do not receive the curated stability promise.

Removing all legacy re-exports was rejected because it creates a broad source break immediately before release without adding consumer value. Stabilizing every existing re-export was rejected because many are implementation authorities rather than integration contracts.

## Decision 2: Make the shipped CLI a compile-time coverage consumer

The Deep Capture command currently references the facade through `fragcap::deep_capture`, but that path does not distinguish curated contracts from incidental exports. S132 aliases and imports the new stable API module in the command implementation. A repository test scans command references and the stable inventory so a shipped CLI capability cannot silently become command-only or bypass the compatibility boundary.

Moving effect adapters into a new crate or deleting all command adapters was rejected. The architecture permits the CLI to provide production effect bridges, interactive consent, presentation, and exit mapping. Issue #330 forbids CLI-only business rules, not CLI-owned operating-system bridges. Existing session classification, ordering, fact selection, artifact authority, and cleanup policy already live in the facade.

## Decision 3: Record thread confinement instead of adding unjustified trait bounds

The coordinator owns mutable adapter objects and executes their calls serially. Requiring every injected adapter to be `Send` or `Sync` would be a source-breaking restriction with no execution requirement: the coordinator does not share adapters concurrently, and arbitrary UI or embedded adapters may legitimately be thread-confined. The stable contract therefore guarantees that immutable plans, configuration, cancellation handles, and terminal values are transferable when their fields are, while `DeepCaptureSession`, `AdapterSet`, and adapter trait objects are thread-confined unless a future API explicitly adds a movable runner.

Adding `Send + Sync` to every adapter trait was rejected because it would falsely imply concurrent invocation and would exclude valid single-thread integrations. Claiming no concurrency contract was rejected because issue #330 requires consumers to know the boundary.

## Decision 4: Add cooperative cancellation without unsafe preemption

Rust cannot safely interrupt an arbitrary blocking trait call. The existing finite `Budget` contract remains the in-call guarantee. S132 adds a cloneable cancellation handle owned independently from a session, passes it into the coordinator through an explicit construction path, and checks it before each later non-cleanup lifecycle effect. Cancellation becomes a typed interrupted terminal outcome and still drives every safe cleanup and finalization obligation. Adapters that block must continue to honor their finite budget and may share the same handle when they need faster cooperative response.

Thread termination, process termination, async-runtime adoption, and hidden global interrupt state were rejected. They either violate ownership, expand dependencies, or cannot safely preserve partial evidence and cleanup truth.

## Decision 5: Use builders for extensible inputs and non-exhaustive output policy

`SessionConfig` and `AdapterSet` are the principal external inputs. Public exhaustive field literals make adding an optional capability source-breaking. S132 adds supported builder construction while retaining current public fields for source compatibility. The compatibility contract directs new consumers to builders and reserves field-literal compatibility only through the current pre-1.0 line.

Evolvable public enums in the stable inventory are non-exhaustive. Small values whose closed set is itself the contract may remain exhaustive only when named in the API contract. Repository tests assert the policy against the reviewed inventory. Making every public struct private in one slice was rejected because it would force a large mechanical migration with no release-critical behavioral gain.

## Decision 6: Version the curated contract independently from wire schemas

`DEEP_CAPTURE_API_VERSION` begins at 1 and identifies the reviewed Rust integration surface. It does not replace crate semantic versioning, manifest versions, JSON Lines versions, classification schema versions, or resource-journal versions. A compatible release may add documented builder methods, optional capabilities, non-exhaustive variants, and new unstable implementation helpers without changing this API version. Removal or semantic change to a stable item increments the contract version and the crate version under the recorded pre-1.0 policy.

A dependency on an external semantic-version checker was rejected. An exact repository-owned inventory, external-consumer compile test, CLI coverage test, and docs example provide deterministic offline evidence with no new package.

## Decision 7: Provide a no-CLI production native example over the controlled lab

The example uses public facade contracts, the production `NativeProxyAdapter`, an exact loopback reservation, and the committed controlled local origin path. It performs no real trust mutation, Internet access, capture-driver access, game launch, or target inspection. An integration test executes the same library entry path and proves cleanup before the example is published as guidance.

A prose-only example was rejected because issue #330 requires compilation and execution. A real target example was rejected because it would depend on local accounts, trust effects, routing compatibility, and privileged Capture setup.

## Decision 8: Change no dependency, workflow, release command, or scheduled work

The contract, builders, cancellation handle, example, tests, and documentation fit the existing workspace. S132 adds no dependency or lockfile package, changes no workflow or release script, starts no soak, and does not cut or publish a release. S131's merged spec status is corrected as narrow housekeeping metadata.
