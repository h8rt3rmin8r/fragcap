# Research: Verified Test Cleanup

## D1. Per-test fixture lifetime

**Decision**: Own unique temporary roots through each test's assertions and explicitly close them afterward, including when the assertion body panics. If cleanup cannot complete, keep an exact ownership marker within the residual root. `deep_capture_session.rs` keeps the owner in a test-local context because `config()` must return a plain `SessionConfig` that production code consumes by value and a single matrix test creates many configurations.

**Rationale**: Creating a temporary owner inside `config()` would delete the root before the session uses it. The current raw path creates persistent directories. A test-local wrapper can catch the original panic, close every owned root with a fallible operation, verify absence, and then preserve the original failure.

**Alternatives considered**: A destructor-only `TempDir` would hide cleanup errors and skip abnormal process exit. A new shared test crate would spread a test-only contract across the workspace for a small number of proven writers.

## D2. Aggregate residue authority

**Decision**: The test runner creates one unique, canonical run root and gives it only to the test child as its effective temp root. On every normal child exit, including test failure, it inventories only this root. An empty root is closed and verified absent. Unexpected contents fail the gate and are reported for exact reconciliation; uncertain native recovery evidence is preserved. A surviving marker after runner interruption is reported with its exact path for bounded manual reconciliation; no background prefix sweep deletes it automatically.

**Rationale**: A before/after check scoped to one run can detect writers missed by the source audit without searching the machine or treating historic same-prefix paths as disposable. The child inherits no shell interaction; the existing xtask launcher sets `CREATE_NO_WINDOW` on Windows.

**Alternatives considered**: Scanning every `fragcap-*` path in the machine temp root cannot prove ownership. Relying only on per-test destructors cannot detect a leaked writer or an abnormal runner exit.

## D3. Proven writer audit

**Decision**: Repair the raw `deep_capture_session.rs` bundle paths, fixed `xtask/src/skills.rs` directory, CLI default stores and session roots created through `tests/common/mod.rs`, and the raw `technologies.rs` test tree. Audit remaining ad hoc paths and classify separately owned or cleanup-on-Drop producers without treating all as proven leaks. The first full workspace `test-clean` run passed Cargo assertions but failed the residue gate on the CLI harness's PID-named local database and session directory. That observation expands the demonstrated CLI writer beyond the initially selected call sites.

**Rationale**: The four named writers have source-visible success paths that leave a directory or use an unsafe fixed shared path. Existing `tempfile::TempDir` sites normally clean on drop, but their destructor limitation belongs in the aggregate residue gate and guidance.

**Alternatives considered**: Refactoring every temp-producing test in one slice would increase regression surface without evidence that each producer leaks. Ignoring the additional two writers would leave a known success-path residue after #606 closes.

## D4. Governance and native evidence

**Decision**: Add a versioned constitution section for verified end-of-test cleanup and align agent, contributor, reviewer, and testing guidance. State the distinction between test scratch and product-managed native evidence, including an unresolved recovery obligation.

**Rationale**: #605 explicitly requires the rule to cover automated, certification, controlled application, and installed-product tests, including failed and interrupted runs. The existing installed-product section covers default placement but not the full end-of-test obligation.

**Alternatives considered**: A contributor-only note would leave the constitutional and agent instructions inconsistent. Deleting native bundles by temp-name pattern would violate their retention and recovery contract.

## D5. Shared CLI test environment

**Decision**: Give the in-process CLI test harness a unique owned root for its default catalog path, local store, and session directory. A test-thread owner persists across related CLI calls and closes its exact root when the harness thread exits, including after assertion panic. Each CLI call holds the environment lock while setting and restoring the three variables, so parallel in-process calls do not race. Explicit multi-call scopes use the same routing and perform fallible close and absence verification before returning. The aggregate `test-clean` gate detects any abnormal process exit or failed destructor cleanup.

**Rationale**: The PID-named paths were shared by tests in a binary and leaked after successful assertions, while at least one test needs state to persist across calls. A process-static temporary-directory owner would never run a fallible close. Changing process environment per call without a lock would race with parallel tests. Holding one lock across a complete assertion body would deadlock if that body called the ordinary helper.

**Alternatives considered**: A shared fixed directory leaks and collides. Per-call roots would break the deliberate multi-call state tests. Serializing the entire workspace is unnecessary when the affected in-process binary can lock its own environment.

The first hosted Ubuntu run found a specialized authorization-output test that called the CLI library directly after an ordinary helper call. It had depended on the old helper leaving scratch variables set for the entire process. With per-call restoration, that direct invocation reached an earlier Linux refusal and poisoned the controlled-test lock. The specialized call now enters the same owned environment through an explicit context method; no process-wide scratch setting is retained after it returns.

## D6. Other output producers and evidence boundary

| Producer | Effective location and owner | S208 disposition |
| --- | --- | --- |
| CLI bootstrap tests | Windows temp through `std::env::temp_dir`; the prior PID and tag paths were pre-deleted and ended with ignored cleanup errors | Changed to unique owned roots with fallible close and absence verification on success and panic |
| CLI fresh-start and most other CLI integration fixtures | Synthetic AppData and local stores beneath a test-owned `TempDir` | Existing owner remains; the aggregate run catches residue if a destructor fails or a child exits abnormally |
| Package certification unit fixtures | Windows temp report and transferred-artifact trees, with explicit success-path removal | No success-path residue observed in the full S208 run; the aggregate run now reports a failed cleanup rather than silently passing |
| Windows integration unit fixtures | Windows temp synthetic summaries and trees; ignored cleanup errors on some tests | No success-path residue observed in the full S208 run; exact aggregate ownership catches any remaining child |
| Hosted Windows integration matrix | Explicit `target/windows-integration` scratch and report authority, with `prepare_scratch`, effect snapshots, bounded child exit, `remove_owned_dir`, and retained trust-obligation refusal | Separately owned; S208 does not delete its native or recovery material through the temp runner. The hosted workflow now uses `test-clean` for its Cargo test commands, while its existing matrix inventory remains the authority for explicit scratch |
| Installed product, package downloads, and packaged-host `LocalCache` redirection | Product default stores or certification harness authority, outside a synthetic unit-test root | Guidance requires effective canonical readback and supported product cleanup. No installed-product or redirected package-host run was performed in S208, so those physical locations are not claimed clean by the local unit-test gate |

This audit records concrete source owners and the observed S208 aggregate run. It does not infer that every `tempfile::TempDir` destructor succeeded in every possible interrupted run, or authorize sweeping historical `fragcap-*` paths.
