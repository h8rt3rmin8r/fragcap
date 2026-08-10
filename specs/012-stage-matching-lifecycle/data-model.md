# Data Model: Stage Matching and Session Lifecycle

## `fragcap-core` additions

### `ProcessTree::bind_stage`

```rust
/// Bind a node to a profile stage, writing the stage field reserved in S11.
///
/// Returns true when the node existed and was previously unbound, false when
/// the node is unknown or already bound. Idempotent per node: a node binds to
/// at most one stage.
pub fn bind_stage(&mut self, id: NodeId, stage: StageId) -> bool
```

The only mutation S12 adds to `fragcap-core`. No new field: it sets the existing
private `ProcessNode::stage`.

## `fragcap-profile::matching`

Pure functions over `&Profile` and `&ProcessTree`. No mutation of core except
through `bind_stage`, called by `bind_stages`.

```rust
/// The first stage, in declaration order, all of whose specified predicates
/// hold for the node. None when no stage matches.
pub fn stage_for<'p>(profile: &'p Profile, tree: &ProcessTree, node: NodeId) -> Option<&'p Stage>

/// Whether every specified predicate of `pred` holds for `node` in `tree`.
fn predicates_hold(pred: &MatchPredicates, tree: &ProcessTree, node: NodeId) -> bool

/// Walk nodes in creation order; bind each unbound node to its stage_for result,
/// so descends_from over an already-bound ancestor resolves. Models the
/// per-event binding the session performs live.
pub fn bind_stages(profile: &Profile, tree: &mut ProcessTree)
```

Predicate semantics (all specified must hold):

| Predicate | Evaluation |
| --- | --- |
| `exe` | `ImagePattern::matches(node.image_name())` (case-insensitive glob) |
| `path_contains` | case-insensitive substring of `node.image()` |
| `path_regex` | `PathRegex::regex().is_match(node.image())` (reused, not recompiled) |
| `cmdline_contains` | `node.command_line().as_str()` is `Some(s)` and `s.contains(sub)`; `Unavailable` never matches |
| `descends_from` | some strict ancestor of `node` is bound to the named role |

## `fragcap` facade: `session` module

```rust
pub enum SessionState { Arming, Watching, Capturing, Draining, Complete }

pub enum StopReason {
    DurationReached, VolumeReached, TerminalStageExited,
    AllProcessesExited, Interrupt, SinkError, AcquisitionTimeout,
}

pub enum PacketDisposition { Discarded, Retained }

pub struct SessionConfig {
    pub acquisition_timeout: Option<Duration>,
    pub duration: Option<Duration>,
    pub packet_bound: Option<u64>,
    pub byte_bound: Option<u64>,
}

pub struct SessionStats {
    pub watching_discarded: u64,        // P-4 named counter
    pub retained: u64,
    pub retained_bytes: u64,
    pub discarded_out_of_window: u64,   // packets offered outside Watching/Capturing
}
// observed() == watching_discarded + retained + discarded_out_of_window
// (session conservation: every on_packet call increments exactly one counter)

pub struct CaptureSession { /* state, profile, tree, config, stats, bookkeeping */ }
```

Session API (event-driven, tier-1 testable):

| Method | Effect |
| --- | --- |
| `new(profile, config)` | constructs in `Arming` |
| `attach(at)` | Arming to Watching (watcher attached, handle open) |
| `on_process_event(event)` | apply to tree, match and bind on Started (a binding already exited is honored as an exit), handle terminal/all-exited on Exited, the first non-service match moves Watching to Capturing (the event carries its own timestamp) |
| `on_packet(len) -> PacketDisposition` | Watching discards and counts; Capturing retains, counts, and may hit a volume bound; any other state discards into the out-of-window counter |
| `on_tick(now)` | acquisition timeout (from Watching) and duration bound |
| `on_interrupt()` / `on_sink_error()` | stop with the matching reason |
| `finalize()` | Draining to Complete (flush, finish) |
| `state()`, `stats()`, `stop_reason()`, `tree()` | inspection |

State transitions:

- `Arming --attach--> Watching`
- `Watching --first stage match--> Capturing`
- `Watching --acquisition timeout--> Complete` (nothing captured; no drain)
- `Capturing --any stop condition--> Draining --finalize--> Complete`

## Key entities recap

- **Stage binding**: `ProcessNode::stage: Option<StageId>`, written by `bind_stage`.
- **Watching-discard counter**: `SessionStats::watching_discarded`.
- **Stop condition**: `StopReason` variants, first to occur wins.
- **Acquisition timeout**: `SessionConfig::acquisition_timeout`, optional, from arm.
