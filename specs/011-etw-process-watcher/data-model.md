# Data Model: ETW Process Watcher and Tree (S11)

**Slice**: S11 | **Date**: 2026-08-09 |
**Spec**: [spec.md](spec.md)

What this slice adds to the vocabulary, what it changes, and what it
deliberately leaves alone. Types in `fragcap-core` are platform-neutral by P-2;
types in `fragcap-attr` are behind the `etw` feature unless stated otherwise.

## New types in `fragcap-core`

### `ProcessId`

`fragcap-core::process`. A newtype over `u32`, wrapping an operating system
process identifier.

It exists to stop a `u32` that recycles being passed where a synthetic
identifier is wanted. Both are small integers and the compiler is the only thing
that will notice the difference.

### `NodeId`

`fragcap-core::process`. A newtype over `u32`, the synthetic session-local
identity of a node. Issued by the tree, monotonic, never reused within a
session (FR-020).

Distinct from `ProcessId` by type, which is the point. Section 10.2's whole
argument is that one recycles and the other does not.

### `Ancestry`

`fragcap-core::process`. Where a node's parent came from.

```text
Observed   the start event carried it, recorded at creation
Snapshot   read from a running process at startup, may be stale or wrong
Unresolved no parent could be resolved
```

Carried on the node, never derived (FR-022). This is the `Fidelity` lesson from
S06 applied to a second field: a value that says how trustworthy another value
is has to be stored beside it, because deriving it from whether the other value
is present answers a different question.

### `CommandLine`

`fragcap-core::process`. Either the observed command line or a record that none
was available.

```text
Observed(Arc<str>)   exactly what the platform reported, unaltered
Unavailable          no command line could be obtained without a denylisted right
```

Not `Option<Arc<str>>`, and the difference is not stylistic. An `Option` invites
`unwrap_or_default`, which turns "we could not see it" into "it was empty" at
one call site and loses the distinction for good. FR-036 forbids exactly that
substitution, so the type carries the reason rather than the absence.

### `ProcessNode`

`fragcap-core::process`. One process in the tree.

| Field | Type | Source |
| --- | --- | --- |
| `id` | `NodeId` | issued by the tree |
| `pid` | `ProcessId` | event or snapshot |
| `parent` | `Option<NodeId>` | resolved by pid and time |
| `ancestry` | `Ancestry` | which source supplied the parent |
| `image` | `Arc<str>` | full image path (FR-038) |
| `command_line` | `CommandLine` | start event, or unavailable |
| `started` | `Option<Timestamp>` | `None` means unknown (FR-009) |
| `exited` | `Option<Timestamp>` | `None` means still running |
| `stage` | `Option<StageId>` | reserved for S12, always `None` here |

`image` is the full path. `ProcessNode::image_name` derives the file name from
it, because section 10.3's `exe` predicate matches the name while
`path_contains` and `path_regex` match the path, and S12 needs both from one
recorded value (FR-038).

`stage` uses the existing `StageId` from `fragcap-core::attribution`, so S12
binds a value rather than introducing a type. This slice never writes it.

### `ProcessTree`

`fragcap-core::process`. The nodes, the ancestry relation, and what is known to
be missing.

Behavior, not shape, is what matters here:

- `apply(&mut self, event: ProcessEvent)` folds one event.
- `apply_snapshot(&mut self, records: &[ProcessRecord])` folds the startup
  snapshot, reconciling against nodes already present (FR-033).
- `resolve(&self, pid: ProcessId, at: Timestamp) -> Option<NodeId>` is the
  lookup from the operating system's vocabulary into the tree. A node with an
  unknown start time is selected only when no node with a known start time
  covers `at` (FR-024).
- `ancestry(&self, id: NodeId) -> Vec<NodeId>` returns the path to the root in
  creation order, including exited nodes (FR-032).
- `node(&self, id: NodeId) -> Option<&ProcessNode>`.
- `len(&self)` reports how many nodes are retained (FR-029).
- `is_complete(&self) -> bool` is false once any loss is recorded (FR-034).
- `note_lost(&mut self, count: u64)` is how the watcher tells the tree that the
  kernel dropped events.
- `unmatched_exits(&self) -> u64` counts exits still unjoined (FR-031).

Nothing on `ProcessTree` performs I/O, opens a handle, or names a platform type.
That is what makes the whole of section 10.2 a tier 1 test.

**Pending exits.** An exit arriving before its start is held rather than
counted, because a trace consumer delivers from several buffers and does not
guarantee timestamp order (FR-031). A held exit joins its node when the start
arrives; one still held at the end of the session is the unmatched count.

### `WatcherReport`

`fragcap-core::process`. What the watcher observed about its own operation.

| Field | Type | Meaning |
| --- | --- | --- |
| `events_lost` | `u64` | events the kernel itself reported dropping |
| `buffers_lost` | `u64` | trace buffers the kernel reported dropping |
| `running` | `bool` | whether the session is still consuming |

Separate from `CaptureStats` by FR-015. `CaptureStats` carries the conservation
identity that every pipeline test asserts, and a quantity that is not a packet
must not enter it. This mirrors `SourceStats`, which is a value a source
produces rather than a field the capture owns.

## New types in `fragcap-attr`

### `EtwWatcher`

`fragcap-attr::etw`, behind the `etw` feature. The `ProcessWatcher` of
FR-001.

Owns the trace session, the consumer thread, and the fan-out to subscribers. On
construction it subscribes and then snapshots, in that order (FR-007), so a
process created during startup is reported twice rather than not at all.

`subscribe` returns an independent receiver on each call (FR-012). The channel
is unbounded (FR-013).

### `WatcherError`

`fragcap-attr::etw`. Why a watcher could not start or could not continue.

```text
NotElevated             the trace session needs a privilege this session lacks
SessionUnavailable      the platform refused, with its own code and message
ProviderUnavailable     the process provider could not be enabled
Stopped                 the session ended after having started
```

`NotElevated` is separate from `SessionUnavailable` because they have different
remedies and section 26.4 requires an error to say what to do next. The
platform's own reason is carried in `SessionUnavailable` rather than replaced
(FR-016).

There is no variant for "falling back to polling", because there is no fallback
(FR-011).

### `ScriptedWatcher` and `ProcessScript`

`fragcap-attr::proc_script`, not behind any feature. The offline half, mirroring
`ScriptedAttributor` and `AttributionScript` from S04.

`ProcessScript` is a declared sequence of `ProcessEvent` values plus an optional
startup snapshot. `ScriptedWatcher` publishes them in order. A tree built from a
script is indistinguishable from a tree built from the same events arriving from
ETW (FR-041), because both go through `ProcessTree::apply`.

The script is built in code rather than parsed from a file. S04's attribution
script has a text format because a fixture corpus needed one; a process script
has exactly two users so far, both of them the Appendix D chains in this slice's
tests, and inventing a file format for two callers would be speculative until
S12 shows what a matcher needs.

## Changed types

### `ProcessEvent::Started` gains `command_line`

`fragcap-core::process`. Section 10.1 states the start event carries a command
line and section 10.2 makes it a tree field, so the variant carries it.

The enum is `#[non_exhaustive]`, which permits new variants but does not permit
new fields on an existing variant without breaking every pattern match that
names the fields exhaustively. This is therefore a breaking change to the
variant, recorded as a deviation. S02 anticipated it in the module's own
documentation, which is why this is a scoped edit rather than a surprise.

Blast radius: the module's own tests, and the traits module's test doubles.
Nothing outside `fragcap-core` constructs a `ProcessEvent` today.

### `ProcessEvent::Started::image` is settled as a path

`fragcap-core::process`. The field was ambiguous between a file name and a full
path, and S02's tests pass a bare file name. Sections 10.2 and 10.3 need the
full path, with the file name derived. The documentation says so and the tests
change to use paths.

### `ProcessRecord` gains `command_line`

`fragcap-core::process`. Always `CommandLine::Unavailable` from the Windows
snapshot, for the reason research R-3 records: obtaining one for a running
process needs a memory-read right that P-1 forbids. The field exists so that a
platform that can supply one without a denylisted technique is not blocked by
the type.

`ProcessRecord::started` stays `Option<Timestamp>`, and its `None` now has a
defined meaning in resolution (FR-024) rather than being merely permitted.

## What does not change

- `ProcessWatcher`. The trait as section 8.5 declares it and S02 transcribed it
  is sufficient. `subscribe(&self)` already implies fan-out, and `snapshot`
  already returns records. No deviation is needed here, which is worth stating
  because S09 needed one on `PacketSource` and the symmetry invites the
  assumption.
- `CaptureStats`. FR-015 keeps the watcher out of it.
- `Timestamp`. Nanoseconds since the Unix epoch, `i64`, unchanged. The
  `FILETIME` conversion happens at the boundary in `fragcap-attr`.
- The pipeline. FR-050 keeps the watcher out of it until the control thread has
  its other occupants.
- `fragcap-core`'s dependency set. The tree is arithmetic and collections over
  types core already has (FR-043).
