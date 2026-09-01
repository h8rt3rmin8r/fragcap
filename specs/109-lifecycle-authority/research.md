# Research: Crash-Safe Lifecycle Authority

## Typed Routing Boundary

**Decision**: Put a non-secret `RoutingPlan` in the authorized `SessionPlan`, then apply it through a `RoutingAdapter` after the proxy has produced session secrets. The applied lease exposes only the exact launch-scoped route to `LaunchAdapter` and owns verification and cleanup.

**Rationale**: The operator must authorize effect shape before proxy startup, but proxy credentials do not exist until the exact proxy session exists and must not enter printable or clonable plan data. Symbolic secret sources let the plan declare destinations without retaining secret values.

**Alternatives considered**:

- Keep routing inside `LaunchAdapter`: rejected because route effects and cleanup remain implicit and cannot be journaled independently.
- Put proxy credentials in `SessionPlan`: rejected because plan events, debug output, and equality would retain secrets.
- Implement all future routing strategies now: rejected because issues #307 through #318 own their behavior and evidence.

## Journal Format and Durability

**Decision**: Use protected bundle-local JSON Lines with a schema-versioned header, monotonically sequenced transition records, and optional reconciling trailer. Flush each complete transition with `sync_all` before the effect proceeds. Treat a partial last line as an incomplete prefix and refuse uncertain recovery.

**Rationale**: The repository already uses JSON Lines for crash-readable evidence, serde_json is present, and standard file synchronization provides the required process and restart durability without another package. Whole-file JSON cannot preserve a useful prefix after a torn append.

**Alternatives considered**:

- SQLite: rejected because the facade does not depend on rusqlite and one ordered session log does not need a relational store.
- Reuse the S107 sensitive-action journal directly: rejected because its delete/share grammar cannot represent proxy, trust, routing, launch, or retained evidence without turning a narrow format into an ambiguous union.
- Write only a final recovery manifest: rejected because a crash between effect and finalization loses the obligation.

## Ownership and Safe Recovery

**Decision**: Journal stable resource kinds and ownership evidence, then separate inspection from execution. A recovery planner returns `execute`, `already-terminal`, `retain`, or `refuse` for each obligation. Platform adapters execute only exact actions whose current identity matches the journal.

**Rationale**: Recovery is security-sensitive. A data parser must never directly remove a resource, and a stale listener, path, or trust thumbprint can be reused by another owner.

**Alternatives considered**:

- Best-effort cleanup by resource name: rejected because it can remove unrelated state.
- Delete the whole bundle directory: rejected because evidence is retained by policy and P-9 forbids silent destruction.
- Treat process-owned resources as always dirty: rejected because listeners and tasks cease with process termination and can be verified without mutation.

## Cleanup Artifact Compatibility

**Decision**: Add `cleanup.jsonl` as the authoritative chronology and keep `cleanup.json` as a derived final compatibility summary whose source role is `cleanup-log`. Add both to manifest version 2 with distinct authority kinds.

**Rationale**: Replacing `cleanup.json` would break current readers immediately after S108 established manifest v2. Keeping both as independent authorities would create contradictory cleanup truth. An explicit projection preserves compatibility and one source of truth.

**Alternatives considered**:

- Rename `cleanup.json` in place: rejected because existing manifests, doctor, exports, tests, and users expect it.
- Keep only `cleanup.json` and append multiple JSON values: rejected because the file would stop being valid JSON and its extension would mislead consumers.
- Version the whole manifest again: rejected because additive artifact roles fit the open v2 contract and no existing field changes meaning.

## Proxy Lifecycle Source

**Decision**: Reuse the nonblocking application event sink as the connection/protocol source through a bounded fan-out sink, supplement it with facade-owned listener, start, stop, drain, and runtime terminal records. Gaps explicitly cover lifecycle facts the protocol engine could not retain.

**Rationale**: Application events already carry connection, stream, TLS, protocol, terminal, error, and timestamp identity. A second blocking callback inside forwarding would duplicate hot-path synchronization and could diverge from application truth.

**Alternatives considered**:

- Parse human logs: rejected by #336 and P-9.
- Derive everything at finalization from aggregate runtime reports: rejected because there is no crash-readable chronology.
- Add a second unbounded proxy event bus: rejected because forwarding must remain bounded and independent from evidence consumers.

## Bounded Localized Loss

**Decision**: Cap localized body-loss identities at the same finite connection-history class used by application correlation and retain exact aggregate overflow counters by records, observed bytes, and retained bytes.

**Rationale**: The S108 map can grow with unbounded HTTP/2 stream churn even though its event queue is bounded. Exact global loss does not require retaining every identity.

**Alternatives considered**:

- Evict an arbitrary identity: rejected because the emitted localization would become schedule-dependent.
- Drop overflow accounting: rejected by P-4.
- Raise the limit without an overflow state: rejected because any finite limit can still be exceeded.
