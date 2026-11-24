# Research: Evidence and Diagnostic Debt Reconciliation

## Decision 1: Correct the public work queue against observed product behavior

**Decision**: Review the active epic and affected child issues against merged source, released behavior, and retained controlled evidence. Preserve historical observations, label completed comparisons as completed, and leave only controllable product outcomes as implementation acceptance criteria.

**Rationale**: Earlier issues mixed diagnostic hypotheses with product defects. The constitution closes scoped implementation work on automated or controlled proof and treats a later active-release observation as a new concrete defect. A tracker that retains an unobservable client decision as a completion gate cannot be completed by product code.

**Alternatives considered**: Leave all issues unchanged (continues misleading scope); close all issues at once (erases independent defects); publish private target evidence (violates the public boundary).

## Decision 2: Extract one bounded general diagnostic

**Decision**: Carry `http_head_bytes_before_failure` from the existing HTTP/1.1 head reader through the proxy trace, facade connection diagnostic, native `proxy.jsonl`, terminal diagnostics JSON, and the separately generated bundle debug report. It is an optional unsigned count. Keep the existing error code and boundary as the terminal explanation.

**Rationale**: `read_head` already buffers no more than the configured header limit and knows the exact length at read-error, partial EOF, and size-refusal branches. Those branches currently discard that length. A byte count is directly observed and does not require retention of credentials or HTTP content. The reader may operate on a plain or TLS stream; the count describes observed HTTP-head bytes without making a universal decryption claim.

**Alternatives considered**: Retain the partial head (unnecessary content exposure); infer count from a later exchange (false for failed heads); add a separate log stream (duplicates native connection identity); change the proxy route on failure (unrelated behavior change).

## Decision 3: Make absence and zero distinct

**Decision**: Clean EOF with zero bytes remains `Ok(None)` and has no failure-only count. A real read error before any byte has count `0`. Partial EOF, partial read error, and header-limit refusal have their observed positive count. A complete head and non-head error omit the field. Native and terminal diagnostics omit the key entirely when absent; the bundle debug report omits its `head_read_failures` array when no valid counted failure exists.

**Rationale**: Zero is a measured outcome on an actual error, not a placeholder for no failure. An emitted `null` key would falsely suggest that a count was considered and unavailable on ordinary successful records. The current terminal codes already distinguish EOF, I/O error, and limit.

**Alternatives considered**: Emit `0` for clean EOF (turns ordinary no-head completion into a failure); emit `null` everywhere (creates false diagnostic noise); count a timeout from a separate task (requires new synchronization and is outside this slice).

## Decision 4: Preserve the existing artifact and dependency boundary

**Decision**: Add one optional field to the version-one connection diagnostic without changing its identity, failure categories, or artifact role. Project it separately to terminal diagnostics JSON and, only for valid counted failures, to `proxy.head_read_failures[]` in the bundle debug report. Read back both newly emitted native records and older records without the field. The diagnostic adds no crate or product workspace lock package; the separately justified CI repair updates two isolated spike lockfiles.

**Rationale**: Terminal diagnostics JSON already projects a typed in-memory connection record. The existing bundle debug report separately reads the native lifecycle stream but did not project connection diagnostics. A narrow, allowlisted native-to-bundle failure summary closes that evidence gap without exposing endpoint or head content. An additive optional native key preserves previous output and avoids a second source of truth.

**Alternatives considered**: Bump the artifact schema for one optional count (unnecessary reader churn); create an untyped free-form log line (loses cross-layer reconciliation); import experimental TLS dependencies (outside the approved product slice).

## Decision 5: Inventory before retiring disposable work

**Decision**: Record every original modified tracked path and substantive hunk in a sanitized ledger, then compare the final public diff to it before any cleanup. Retain excluded experiments privately until the extraction is verified, without implying that they are shipped or accepted.

**Rationale**: Whole-file copying would pull in unrelated changes. Immediate cleanup could lose investigatory work. A per-file disposition makes that separation reviewable.

**Alternatives considered**: Ship the entire dirty tree (uncontrolled scope); discard everything (loses useful work); keep a vague note without file coverage (cannot prove no hunk was lost).

## Decision 6: Preserve the audit gate with a finite, reviewable exception

**Decision**: Pin the audit workflow and governed tool record to locked `cargo-deny 0.21.0`. Allow only advisory IDs `RUSTSEC-2026-0574` through `RUSTSEC-2026-0582` when reached through exact parent `smoltcp@0.12.0`, with expiry 2026-12-15 UTC and `unsound = "all"` unchanged. Mirror the exact exceptions in offline policy and validate policy-to-denial drift. Update `url` to 2.5.9 and `form_urlencoded` to 1.2.3 in the two isolated spike locks. Renew the due `rustls 0.23.46` maintenance review only against verified unchanged source, checksum, features, and graph identity. Keep #640 open for removal of the affected dependency path and all exceptions.

**Rationale**: The all-features audit failed on main before S212. A bounded review of the enabled `smoltcp 0.12.0` source found uses of `heapless::Vec` and `LinearMap`, but not the affected SortedLinkedList, Spsc, IndexMap, Deque, or pool APIs. Its owned `Vec` iterator on the enabled IPv6 path contains Copy address/CIDR values, so the known panicking-Drop precondition was not found there. This review is specific to that source path and does not establish that `heapless 0.8.0` is generally safe. The current `smoltcp 0.13` replacement raises MSRV beyond the workspace's 1.89, so it requires separate direct-route and compatibility qualification. Positive locked audits, offline mirror validation, and negative direct-parent, expired-waiver, and policy-drift canaries test the exception boundary. The due rustls review is a routine verification of the existing pinned package, not an independent security audit. Issue #640 records the permanent repair.

**Alternatives considered**: Disable `unsound` audits or ignore the nine IDs globally (conceals future vulnerable paths); leave CI blocked indefinitely (prevents the authorized S212 PR from reaching green); upgrade `smoltcp` and MSRV inside this diagnostic slice (expands runtime behavior and qualification scope); silently extend the exception later (defeats its finite authority).
