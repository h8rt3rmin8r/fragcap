# Phase 1 Data Model: Watch / Attach Mode

Signatures are indicative; the implement phase settles exact types. No new
dependency, no core edge.

## WatchArgs (crates/fragcap-cli/src/cli.rs)

```rust
pub struct WatchArgs {
    exe: String,                 // --exe: the target image-name glob (required)
    path: Option<String>,        // --path: path_contains anchor
    path_regex: Option<String>,  // --path-regex: path_regex anchor
    wait: Option<Duration>,      // --wait: acquisition timeout
    duration: Option<Duration>,  // --duration
    out: Option<PathBuf>,        // --out
    sink: Vec<SinkSpec>,         // --sink (repeatable)
    no_payload: bool,            // --no-payload
    offline: OfflineArgs,        // hidden substrate (flattened)
}
```

Added to the command enum as `Command::Watch(WatchArgs)` and dispatched to
`commands::watch::run`.

## The synthesized identity (commands/watch.rs)

A one-stage profile, validated through `Profile::parse` (no unvalidated
construction), authored fidelity:

```json
{ "schema": 1, "kind": "profile", "fidelity": "authored",
  "game": { "id": "watch", "name": "ad hoc watch" },
  "stage": [ { "role": "target", "lifecycle": "session", "terminal": true,
               "match": { "exe": "<exe>", "path_contains": "<path?>",
                          "path_regex": "<path_regex?>" } } ] }
```

Absent anchors are omitted. `Profile::parse` enforces the non-empty match
(`EmptyMatch`) and compiles the regex (`InvalidRegex`), so an unusable identity is
refused before capture.

## CaptureSession::apply_snapshot (crates/fragcap/src/session.rs)

```rust
impl CaptureSession {
    /// Fold a startup snapshot of already-running processes into the session
    /// tree and run matching, so a process already present at arm can acquire the
    /// target without a later start event. Delegates to
    /// ProcessTree::apply_snapshot_at; runs the same match/bind on_process_event
    /// runs; transitions Watching -> Capturing if a non-service stage binds.
    pub fn apply_snapshot(&mut self, records: &[ProcessRecord], at: Timestamp);
}
```

The session stays the single acquisition authority: attach-to-running is this
method binding an already-present process, exactly as wait-for-start is
`on_process_event` binding a started one.

## CaptureComponents snapshot fields (crates/fragcap-cli/src/assemble.rs)

```rust
pub struct CaptureComponents {
    // ...existing...
    pub startup_snapshot: Vec<ProcessRecord>,   // empty when none
    pub snapshot_at: Option<Timestamp>,
}
```

Filled by the offline builder from `ScriptedWatcher::snapshot`, by the live
builder from `EtwWatcher::snapshot` + `snapshot_taken_at`.

## ObservationProvider wiring (commands/watch.rs)

At arm, build a `ProcessTree` from the startup snapshot, then:

```rust
let resolver = TargetResolver::new(vec![
    Box::new(ProfileProvider::new()),
    Box::new(HintProvider::new()),
    Box::new(EngineRuleProvider::new()),
    Box::new(PlatformWalkerProvider::new()),
    Box::new(ObservationProvider::new()),
]).expect("distinct precedences");
let req = ResolutionRequest::for_observation(&identity, &snapshot_tree, &search, &bundled);
match resolver.resolve(&req) {
    Ok(target) => /* already running: report the observed attach naming target.origin() */,
    Err(ResolutionError::Unresolved(_)) => /* not present at arm: wait-for-start */,
}
```

The provider's `observed` `Target` is the honest answer that names the
already-running process; the session's `apply_snapshot` performs the acquisition.

## Orchestrator (crates/fragcap-cli/src/orchestrator.rs)

Both `capture_prerecorded` and `capture_live` apply the snapshot at arm before the
acquisition loop:

```rust
if !components.startup_snapshot.is_empty() {
    session.apply_snapshot(&components.startup_snapshot,
                           components.snapshot_at.unwrap_or(ARMED_AT));
}
```

So an already-running match reaches `Capturing` before the loop; the loop then
handles wait-for-start and the timeout exactly as today.

## Invariants

- The identity is `authored`; `observed` is only the provider's answer (FR-006).
- The session is the single acquisition authority (D-2).
- `watch` output is byte-identical to an equivalent single-stage profile capture
  (FR-007): the snapshot application changes when acquisition happens, not what is
  written.
