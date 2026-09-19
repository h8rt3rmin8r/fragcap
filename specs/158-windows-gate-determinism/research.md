# Research: S158 Windows gate determinism

## R-1: The remaining application loss is a consumer-publication race

**Decision**: Add a one-way readiness handshake from the dedicated application writer thread and do not return `ApplicationArtifactLease` until that signal arrives.

**Rationale**: Every retained Windows failure reaches the exact 4,096-event ceiling, while passing post-S150 windows peak near 125 to 150. `open_correlated` currently spawns the writer and immediately publishes its sink. Windows may therefore run the QUIC producers before it first schedules the consumer. S150 reduced steady-state storage cost but did not establish consumer readiness. A worker signal closes that exact gap without changing capacity, payload, event identity, forwarding or loss semantics.

**Alternatives considered**: Increase the queue (rejected because it hides the race and raises worst-case memory); block or retry producers (rejected because artifact pressure must not block forwarding); remove generic UDP or QUIC records (rejected because the specification requires one record per accepted datagram and protocol evidence remains distinct); add writer priority (rejected because priority is platform-specific and still not a readiness proof).

## R-2: Readiness failure must settle worker ownership

**Decision**: Treat a disconnected readiness channel as startup failure, retire the sink, close the sender and join the spawned worker before returning an error.

**Rationale**: A startup handshake that can return failure while leaving a worker or sender alive would exchange one nondeterministic defect for a resource leak. The worker sends readiness before entering its receive loop. Once thread creation succeeds, waiting for that first signal has the same liveness dependency as using the writer at all and introduces no separate product timeout claim.

**Alternatives considered**: A short wall-clock startup timeout (rejected because it recreates scheduler timing as authority and can leak or falsely fail a viable worker); detach on failure (rejected because ownership would be unverifiable); publish an unready lease with a status flag (rejected because callers could still start traffic).

## R-3: Deterministic injection tests the publication boundary

**Decision**: Keep one private construction helper that accepts a worker-start callback. Production supplies a no-op. Unit tests use a channel-controlled callback to prove lease construction remains blocked before readiness and use a panic callback to prove failure cleanup.

**Rationale**: Repeating a fast real thread start cannot deterministically reproduce scheduler starvation. A private callback holds the exact worker boundary without exposing a product switch or changing runtime behavior.

**Alternatives considered**: Sleep in a test (rejected as another timing race); rely only on hosted repetition (rejected because it does not prove the mechanism); add a public debug option (rejected because the seam is test-only and not operator behavior).

## R-4: Structured drain completion replaces the stale capture clock

**Decision**: Preserve the stable `ProxyLease::observations` method and add a defaulted `drain_observations` method that returns an `ObservationDrain` containing observations and an explicit complete or incomplete status. Session stop uses the structured method with the remaining authorized shutdown budget, trusts complete bounded adapter evidence and records exact incomplete evidence without comparing collection to the earlier capture start.

**Rationale**: `observe` correctly owns capture duration. The native proxy stop path caches a terminal stopped observation after bounded task drain. Reading that cache after shutdown is deterministic completion. Comparing it with the original capture clock makes later scheduling invalidate evidence already complete. The adapter already enforces its supplied finite budget; the facade needs the completion fact, not another unrelated clock sample.

**Alternatives considered**: Remove all deadline checks (rejected because genuine late work must fail); give every drain an unbounded wait (rejected by lifecycle requirements); add a fifth public plan deadline (rejected because the existing shutdown deadline already authorizes bounded terminal proxy work and changing the public plan would be disproportionate); accept any returned vector as complete (rejected because an explicit incomplete state must remain representable).

## R-5: Stop and drain share one displayed shutdown ceiling

**Decision**: Preserve the four-field authorization plan. Capture stop, proxy stop and observation drain consume the remaining shutdown budget from one shutdown start. Cached complete observations may be returned at zero remaining budget because no work remains; incomplete live work at zero budget fails.

**Rationale**: This preserves the plan's exact millisecond contract and prevents S158 from silently doubling terminal time. It also separates authority correctly: the observation deadline owns capture work, the shutdown deadline owns terminal stop and drain work, and structured completion distinguishes already finished evidence from work that still needs time.

**Alternatives considered**: Restart the full shutdown budget for drain (rejected because total elapsed time could exceed the displayed bound); reuse the observation budget (rejected because capture may legitimately consume it); add a hidden grace period (rejected because an undisclosed deadline violates the plan).

## R-6: Test isolation requires poison recovery and scoped environment restoration

**Decision**: Centralize controlled calibration environment acquisition in an RAII suite guard that recovers `PoisonError::into_inner`, snapshots every controlled variable and restores each exact prior value on drop.

**Rationale**: Rust mutex poisoning is a diagnostic hint, not proof that the process environment lock is unusable. The original panic remains reported by the test harness. Recovery prevents unrelated secondary failures. RAII restoration is required because a panic before `remove_var` currently leaks state into later tests.

**Alternatives considered**: Clear poison globally (rejected because recovery at acquisition is sufficient and explicit); remove serialization (rejected because process environment is shared); ignore poisoning with `unwrap_or_default` (rejected because it cannot recover the held guard); manually duplicate cleanup in every test (rejected because panic bypasses it).

## R-7: Hosted evidence remains first-attempt authority

**Decision**: Require fresh Windows native performance and platform conclusions on the final pull-request head. A rerun may diagnose but cannot satisfy acceptance.

**Rationale**: Both issues exist because identical-head retries passed after first-attempt failure. Treating a retry as acceptance would encode the defect into the process.

**Alternatives considered**: Require a workflow retry by policy (rejected because it normalizes nondeterminism); weaken hard loss or deadline assertions (rejected by P-4 and P-9); run an installed product locally (rejected by operator direction and slice scope).
