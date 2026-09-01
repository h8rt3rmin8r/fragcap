# Research: Managed Publisher-Launcher Chains

## R-1: Keep one shared preparation authority

**Decision**: Extend ordinary stored-target resolution and `ManagedLaunch` so Capture prepares the publisher profile and exact root launch once, before Deep Capture effects.

**Rationale**: Capture already resolves the exact stored row, validates a profile, prepares a launch, arms process observation before execution, and is consumed by Deep Capture. Extending this path satisfies P-10 and issue #307's shared-preparation requirement.

**Alternatives considered**:

- Reconstruct the publisher chain in the Deep Capture CLI adapter. Rejected because Capture and Deep Capture could disagree and the public library would not own the capability.
- Add a separate publisher target table. Rejected because it violates P-10 and introduces a second precedence system.

## R-2: Represent a publisher launch as root execution plus declared stages

**Decision**: Add a publisher-chain variant to the existing immutable managed-launch value. It contains the exact direct root launch and an ordered, validated stage declaration. Execution starts only the root; descendants inherit its child environment and are observed by the existing process session.

**Rationale**: A publisher chain is not a sequence fragcap manually executes. The launcher owns its descendant creation. fragcap owns preparation, the root effect, and external observation.

**Alternatives considered**:

- Execute every stage directly. Rejected because it would bypass publisher behavior and fabricate a topology that was not observed.
- Treat the launcher as an ordinary direct client. Rejected because it loses terminal-client identity and can end capture on the wrong process.

## R-3: Reuse profile stage matching for ancestry and lifecycle

**Decision**: Synthesize one validated profile from stored chain roles. Each non-root stage carries its exact executable identity plus `descends_from` the prior declared role. The client is the sole terminal session stage.

**Rationale**: The existing process tree records creation-time ancestry with reusable process identifier safety. `CaptureSession` already binds stages, publishes roles, ignores intermediate exits for session termination, and stops on the terminal stage.

**Alternatives considered**:

- Add a second chain reconciler beside `CaptureSession`. Rejected because two ancestry authorities could disagree and double lifecycle state.
- Match descendants only by image name. Rejected because same-named unrelated and repeated-depth processes are a known P-9 failure.

**Review correction**: Exact executable identity means an anchored, case-insensitive canonical-path predicate in addition to the basename. Generated profile stages use descendant-first matching precedence, while the immutable publisher plan retains stored root-to-client order, so consecutive roles may name the same executable without collapsing into the root role. Only sessions explicitly prepared from a publisher chain permit one binding per declared role; a competing match stops with `ambiguous-stage-match` and cannot become terminal ownership. Only those publisher sessions keep the acquisition deadline active after launcher acquisition until the exact terminal stage binds. Publisher launches with no explicit `--wait` receive a finite two-minute default. Ordinary profiles retain their prior multi-process role and watching-only timeout behavior.

## R-4: Preserve role metadata in the existing launch-entry parser

**Decision**: Extend the value parser for stored Windows launch entries to retain an optional role while preserving all current executable, argument, filter, and dedup behavior.

**Rationale**: The JSON already carries `role` for authored and promoted targets, but the current `LaunchEntry` value drops it. Keeping it in the shared parser avoids reparsing raw JSON in each consumer.

**Alternatives considered**:

- Parse roles only inside managed launch. Rejected because target resolution would need a second parser and could disagree about the same row.
- Infer roles from array position. Rejected because order alone does not prove which executable owns sockets.

## R-5: Support only fully cold publisher chains

**Decision**: Preflight checks every declared publisher-stage image against the query-only process snapshot. Any existing root produces `publisher-launcher-warm`; an existing launcher with no client remains `publisher-launcher-game-start-clean-warm`; any existing descendant makes the chain non-cold. Only `publisher-launcher-cold` proceeds.

**Rationale**: Child-only environment cannot be applied retroactively to a running launcher. Game absence does not prove routing can reach the later client. Issue #309 owns the confirmed restart workflow.

**Alternatives considered**:

- Support game-start-clean warm launchers. Rejected because the route would stop at the already-running launcher unless independent evidence proves another target-scoped strategy, which S111 does not add.
- Close and restart a launcher automatically. Rejected because that expands into issue #309 and requires additional operator consent and identity proof.

## R-6: Refuse ambiguous declarations before effects

**Decision**: Require one launcher root, zero or more uniquely named intermediate roles, one client terminal, exact Windows executable entries, unique roles, and an order whose first role is launcher and last role is client. Report all structural diagnostics before effects.

**Rationale**: The stored chain is the authorization boundary. Guessing among multiple roots, clients, or role orders would widen scope silently.

**Alternatives considered**:

- Pick the first launcher and last client. Rejected because store order is not proof of identity.
- Defer ambiguity to runtime. Rejected when the defect is knowable before proxy, trust, capture, or launch effects.

## R-7: Bound evidence through existing authorities

**Decision**: Reuse the process watcher's loss report, CaptureSession's finite bindings, observation deadlines, and S109 artifact and cleanup journals. Do not add an unbounded publisher event mirror.

**Rationale**: The source process stream and the session already account for their lifecycle and loss. Another retained event list would add memory growth without new authority.

**Alternatives considered**:

- Persist every process event in a new publisher log. Rejected as duplicate authority and unbounded evidence.
- Record only the final client. Rejected because issue #307 requires launcher and intermediate lifecycle evidence.

## R-8: Add no dependency or lockfile package

**Decision**: Implement S111 with existing workspace crates and standard library APIs.

**Rationale**: Exact path handling, argument parsing, process creation, tree matching, and controlled timelines already exist. A process-supervision or graph dependency would not provide a missing property.

**Alternatives considered**:

- Add a process-tree library. Rejected because the project already owns the creation-time tree and platform watcher.
- Add a command-line parser crate. Rejected because Windows argument parsing already uses the platform parser and preserves the exact semantics required.

## R-9: Preserve exact external publisher paths

**Decision**: Canonicalize and retain an explicitly stored absolute stage path even when it is outside the selected game's install root. Resolve a relative stage path only beneath the canonical game install root and refuse any escape. The publisher root works from its own canonical parent directory when it is absolute.

**Rationale**: Publisher launchers commonly live in a publisher-wide installation separate from the game directory. Requiring every stage beneath the game root would reject the exact real-world chain issue #307 exists to support. The stored absolute path is itself the authorization boundary; no search, fallback, or discovery is added.

**Alternatives considered**:

- Require all publisher stages beneath the game install root. Rejected because that reproduces the direct-client assumption for a topology where it is normally false.
- Search standard publisher install directories. Rejected because search would introduce a second identity authority and could silently widen the selected target.
