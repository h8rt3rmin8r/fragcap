# Research: Native Deep Capture Performance Envelope

## R-1: Measurement must traverse the production proxy

**Decision**: Drive every protocol row through `NativeProxyBackend` and the bounded application artifact writer. Use four harness-owned loopback driver families for HTTP/1.1 and WebSocket, HTTP/2 and gRPC, SOCKS TCP and UDP, and scoped QUIC/HTTP/3.

**Rationale**: The existing protocol laboratory can connect peers directly and therefore cannot establish proxy throughput, queue behavior, cache ownership, or shutdown. The four driver families reuse transport setup while keeping all parsing, forwarding, retention, and accounting in production code.

**Alternatives considered**: Time the existing unit tests (rejected because setup dominates and retention modes are incomplete); use the direct protocol laboratory (rejected because it bypasses the product); copy seven independent clients (rejected because duplicated wire scaffolds would drift).

## R-2: Performance tooling is an isolated workspace

**Decision**: Place a non-published `fragcap-performance` binary in an isolated workspace under `performance/native-proxy/`, with an exact lockfile and only existing product-graph packages. Keep the registry validator in `xtask`.

**Rationale**: The harness needs client, origin, process-measurement, and report machinery that is not a shipped product capability. Isolation follows the fuzz-workspace precedent, avoids changing the product dependency graph, and still permits one explicit command to run real release-mode workloads.

**Alternatives considered**: A facade benchmark target (rejected because its large private helper surface would be mixed into an existing crate's all-target checks); a new product crate (rejected because measurement is not product behavior); Criterion (rejected because it adds a dependency and does not own the cross-protocol resource contract).

## R-3: Fresh worker processes own measurement state

**Decision**: Run each measured case in a fresh child process. Every window pairs direct and proxied work with fixed workload settings; the parent validates returned JSON Lines and applies case budgets. Windows children use a hidden, non-interactive launch with redirected I/O.

**Rationale**: Process peak working set cannot be reset, allocator state contaminates later rows, and a stalled row needs a hard owner. Per-row workers also make the report's CPU and memory boundaries explicit. The worker contains only synthetic clients, origins, artifact writers, and the proxy, so measurements are labeled whole-harness rather than falsely called proxy-only.

**Alternatives considered**: One process for the matrix (rejected because high-water values bleed between rows); opening another process to inspect it (rejected because self-measurement is simpler and avoids handle-policy ambiguity); per-test shell wrappers (rejected under P-7).

## R-4: Use paired controls and conservative hard budgets

**Decision**: Every timing case includes a same-run direct-loopback control and seven measured windows after warmup. Gate absolute safety floors plus proxy/direct ratios and added latency. A timing result fails only when its median breaches and at least five of seven windows breach; a five percent boundary band reruns once and cannot normalize to success.

**Rationale**: Shared runners vary materially. Same-run ratios remove much host-speed variance while hard floors still catch an unusable proxy. Resource, loss, cache, task, and shutdown invariants remain hard per-sample failures and receive no statistical forgiveness.

**Alternatives considered**: Compare raw throughput to a committed developer machine (rejected as non-comparable); update baselines automatically (rejected because measurement must not rewrite its own acceptance rule); average all rows (rejected because a fast protocol could conceal a failing one).

## R-5: Add narrow runtime accounting where observation is incomplete

**Decision**: Add bounded failure-detail retention, leaf-certificate cache accounting, and accepted-connection task gauges to `RuntimeObservation`. Connection task ownership increments at the runtime spawn site and decrements on completion, panic, or abort. Nested protocol work remains structurally bounded and joined by its connection owner under the existing stream and attempt limits. Queue occupancy is added to the bounded application writer accounting.

**Rationale**: S128 exposed one real defect: runtime failure details grow without a cap under failure churn. Existing reports expose accepted connections and terminal joining but not their live task gauge, while cache and writer capacities exist without observed occupancy. The accepted connection is the lifetime owner that joins or aborts nested work, so its exact gauge plus the existing finite protocol limits proves the supported ownership boundary without claiming access to Tokio internals.

**Alternatives considered**: Present the connection gauge as a count of every nested future (rejected as false); inspect Tokio internals (rejected because no stable public contract exists); assert configured cache limits without observing them (rejected as assertion without evidence).

## R-6: Measure the current process without new dependencies

**Decision**: On Windows, use `GetCurrentProcess`, `GetProcessTimes`, and `K32GetProcessMemoryInfo` from the existing `windows-sys` line. On Linux, read self-owned `/proc` CPU and resident-memory fields. Unsupported measurement environments return unavailable and cannot pass a required profile.

**Rationale**: The parent samples only its harness-owned worker through the process handle returned by creation; it opens no target handle and adds no package. Working set and peak working set are recorded; Windows private usage is the plateau authority where available. Logical CPU count comes from the standard library.

**Alternatives considered**: Add a cross-platform system-monitor crate (rejected because the required surfaces are narrow and the dependency would enter only for a test tool); omit CPU or memory on Linux (rejected because the short matrix must be comparable on both CI families).

## R-7: Disk growth is exact logical artifact growth

**Decision**: Measure logical file length only across the exact harness-owned artifact allowlist. Compare growth with retained payload bytes, record count, and the predeclared structural allowance.

**Rationale**: Filesystem allocation varies by filesystem and host. Logical bytes are reproducible and describe what fragcap actually wrote. Payload-disabled cases still write metadata, so zero total growth would be a false requirement.

**Alternatives considered**: Measure a broad temporary directory (rejected because unrelated files could enter it); require zero bytes with retention off (rejected because metadata evidence remains required); use allocated blocks (rejected as host-specific).

## R-8: Separate the pull-request gate from genuine soak evidence

**Decision**: Add a required short performance workflow on Windows and Ubuntu, plus a Windows `workflow_dispatch` and scheduled soak job whose default wall-clock duration is two hours. The short job never claims soak success. Both use the same registry and upload reports even on failure.

**Rationale**: Multi-hour work cannot fit every pull request, but a dormant manual-only recipe would not establish an operational gate. A scheduled run provides recurring long-session evidence while explicit dispatch supports release validation.

**Alternatives considered**: Time-compress two hours (rejected because real leak and drain behavior is the purpose); make every pull request wait two hours (rejected as disproportionate); commit one machine's result as a universal baseline (rejected because environment comparability is required).

## R-9: Pinned workflow change is recorded

**Decision**: Add `.github/workflows/performance.yml` and record the change in a dated S128 decisions fragment.

**Rationale**: The constitution permits pinned process changes only with a dated decision. Performance is issue #326's deliverable, so workflow automation is in scope.

**Alternatives considered**: Fold timing into the generic CI workflow (rejected because the soak trigger and report artifacts need separate lifecycle and timeout policy).

## R-10: Project-owner approval supersedes recurring soak execution

**Decision**: Remove the weekly soak trigger and accept the final reviewed S128 evidence after 1,916 completed case terminals across at least 4,316 seconds with zero case, application, queue, or storage failures. Preserve the raw campaign as interrupted and record approval only in the sanitized authority summary.

**Rationale**: The project owner explicitly judged the repeated zero-failure coverage sufficient and directed that the run stop and the requirement be marked approved. Continuing or scheduling further costly repetitions would contradict that authority. A manual 7,200-second diagnostic remains available without being required for this slice.

**Alternatives considered**: Fabricate a complete raw terminal (rejected because it would corrupt evidence); discard the completed evidence (rejected because 1,916 terminal decisions remain useful); retain the recurring trigger (rejected by explicit owner direction).
