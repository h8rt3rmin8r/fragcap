# Contract: The Process Surface After S11

**Slice**: S11 | **Date**: 2026-08-09 |
**Spec**: [spec.md](../spec.md)

The public surface this slice leaves behind, stated as signatures so that S12
can be planned against it and so that a reviewer can tell an accidental change
from an intended one. Everything here is `pub` unless marked otherwise.

## 1. The behavioral seam, `fragcap-core::traits`

Unchanged by this slice.

```rust
pub trait ProcessWatcher: Send {
    fn subscribe(&self) -> Receiver<ProcessEvent>;
    fn snapshot(&self) -> Vec<ProcessRecord>;
}
```

`subscribe` returns an independent receiver on each call. Each observes every
event published after that call and none published before it. A watcher with no
subscribers discards nothing; it has nowhere to publish and the events are gone,
which is a property of the seam rather than a discard path, because an event
nobody asked for was never received.

## 2. The process vocabulary, `fragcap-core::process`

```rust
pub struct ProcessId(u32);
pub struct NodeId(u32);

pub enum Ancestry { Observed, Snapshot, Unresolved }

pub enum CommandLine {
    Observed(Arc<str>),
    Unavailable,
}

#[non_exhaustive]
pub enum ProcessEvent {
    Started {
        pid: u32,
        parent: u32,
        image: Arc<str>,          // full image path
        command_line: CommandLine,
        at: Timestamp,
    },
    Exited { pid: u32, at: Timestamp },
}

pub struct ProcessRecord {
    pub pid: u32,
    pub parent: u32,
    pub image: Arc<str>,
    pub command_line: CommandLine,
    pub started: Option<Timestamp>,
}

pub struct ProcessNode { /* fields per data-model.md */ }

impl ProcessNode {
    pub fn id(&self) -> NodeId;
    pub fn pid(&self) -> ProcessId;
    pub fn parent(&self) -> Option<NodeId>;
    pub fn ancestry(&self) -> Ancestry;
    pub fn image(&self) -> &str;          // full path
    pub fn image_name(&self) -> &str;     // derived file name
    pub fn command_line(&self) -> &CommandLine;
    pub fn started(&self) -> Option<Timestamp>;
    pub fn exited(&self) -> Option<Timestamp>;
    pub fn stage(&self) -> Option<&StageId>;   // always None until S12
    pub fn is_live(&self) -> bool;
}
```

## 3. The tree, `fragcap-core::process`

```rust
pub struct ProcessTree { /* private */ }

impl ProcessTree {
    pub fn new() -> Self;

    pub fn apply(&mut self, event: ProcessEvent);
    pub fn apply_snapshot(&mut self, records: &[ProcessRecord]);
    pub fn note_lost(&mut self, events: u64);

    pub fn resolve(&self, pid: ProcessId, at: Timestamp) -> Option<NodeId>;
    pub fn node(&self, id: NodeId) -> Option<&ProcessNode>;
    pub fn ancestry(&self, id: NodeId) -> Vec<NodeId>;
    pub fn descends_from(&self, id: NodeId, ancestor: NodeId) -> bool;
    pub fn nodes(&self) -> impl Iterator<Item = &ProcessNode>;

    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn is_complete(&self) -> bool;
    pub fn unmatched_exits(&self) -> u64;
}
```

`descends_from` takes node identifiers rather than stage names. Section 10.3's
predicate resolves a stage name to a node and then asks this question, and the
name half is S12's. Providing the relation here and the naming there is what
keeps the profile schema out of `fragcap-core`.

### Invariants the tests assert

1. `NodeId` values issued in one tree are distinct, always. Recycling a
   `ProcessId` never recycles a `NodeId`.
2. `resolve(pid, at)` returns the node whose lifetime contains `at`, or `None`.
   Where two nodes share a `pid`, their lifetimes do not overlap, so at most one
   matches.
3. A node with `started == None` is selected only when no node with a known
   start time contains `at`.
4. `ancestry(id)` terminates. A cycle is impossible because a parent is always
   resolved against nodes that already exist.
5. `len()` equals the number of distinct processes observed. No fold discards a
   node.
6. `is_complete()` is true until `note_lost` is called with a non-zero count,
   and false forever after.
7. Applying the same events in a different order, where the platform could have
   delivered them in that order, yields the same tree.

## 4. The watcher report, `fragcap-core::process`

```rust
pub struct WatcherReport {
    pub events_lost: u64,
    pub buffers_lost: u64,
    pub running: bool,
}
```

Not part of `CaptureStats`, and the pipeline does not carry it. S14 assembles a
run report from both.

## 5. The ETW watcher, `fragcap-attr::etw`

Behind the `etw` feature, Windows only.

```rust
pub struct EtwWatcher { /* private */ }

impl EtwWatcher {
    pub fn start(session_name: &str) -> Result<Self, WatcherError>;
    pub fn report(&self) -> WatcherReport;
    pub fn stop(self) -> WatcherReport;
}

impl ProcessWatcher for EtwWatcher { /* per section 2 */ }

pub enum WatcherError {
    NotElevated,
    SessionUnavailable { code: u32, detail: String },
    ProviderUnavailable { code: u32 },
    Stopped { code: u32 },
}
```

### Behavioral requirements

- `start` subscribes before it snapshots (FR-007).
- `start` creates its own session with the system logger mode set. It never
  names the machine-wide kernel logger, and never stops a session it did not
  create (FR-005).
- The session's client context is set to system time, so event timestamps are
  `FILETIME` and convert exactly into `Timestamp` (research R-6).
- `Drop` stops the session it created. A session left running after the process
  exits is a resource leak the operator cannot see.

### Error mapping

| Platform condition | Variant |
| --- | --- |
| `ERROR_ACCESS_DENIED` from `StartTraceW` | `NotElevated` |
| any other failure from `StartTraceW` | `SessionUnavailable` with the code |
| failure from `EnableTraceEx2` | `ProviderUnavailable` with the code |
| `ProcessTrace` returning after having run | `Stopped` with the code |

`ERROR_ALREADY_EXISTS` maps to `SessionUnavailable` rather than to a retry. A
session by that name exists and is not fragcap's to reuse.

## 6. The scripted watcher, `fragcap-attr::proc_script`

Not behind a feature. Available on every target, which is the point.

```rust
pub struct ProcessScript { /* private */ }

impl ProcessScript {
    pub fn new() -> Self;
    pub fn with_snapshot(self, records: Vec<ProcessRecord>) -> Self;
    pub fn started(self, pid: u32, parent: u32, image: &str, cmdline: &str, at: i64) -> Self;
    pub fn started_without_cmdline(self, pid: u32, parent: u32, image: &str, at: i64) -> Self;
    pub fn exited(self, pid: u32, at: i64) -> Self;
    pub fn events(&self) -> &[ProcessEvent];
}

pub struct ScriptedWatcher { /* private */ }

impl ScriptedWatcher {
    pub fn new(script: ProcessScript) -> Self;
    pub fn play(&self);          // publishes every event to every subscriber
    pub fn script(&self) -> &ProcessScript;
}

impl ProcessWatcher for ScriptedWatcher { /* per section 2 */ }
```

The builder is deliberately blunt. Its two real users are the Appendix D chains,
and a chain reads as a sequence of `.started(...)` calls that a reviewer can
check against the specification's own diagram line by line.

## 7. Repository checks

- `cargo xtask deps`: `fragcap-attr` continues to depend only on
  `fragcap-core` and its own external crates. No sibling edge.
- `cargo xtask lint`: gains a check that no fragcap source requests a process
  access right carrying memory rights, alongside the transmit-call check S09
  added. The forbidden names are the memory-bearing rights: `PROCESS_VM_READ`,
  `PROCESS_VM_WRITE`, `PROCESS_VM_OPERATION`, and `PROCESS_ALL_ACCESS`.
- `cargo xtask neutral`: extended to build `fragcap-attr`, so that the crate
  that must build without its backend is checked rather than assumed.
- `cargo xtask ci`: unchanged in shape, and still passes with no elevation and
  no Windows-only feature enabled.
