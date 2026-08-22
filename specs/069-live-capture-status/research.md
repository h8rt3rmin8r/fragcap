# Phase 0 Research: Live capture status display

## Decision R-1: Wire the redraw into `drive_live` only, not `drive`

**Decision**: The live status block and the non-terminal heartbeat are wired
into `capture_live`/`drive_live` in `crates/fragcap-cli/src/orchestrator.rs`
only. `capture_prerecorded`/`drive` (the offline, two-phase driver used by
every tier-1 test, every committed golden, and the `extcap` path) is not
touched beyond, at most, threading an unused-by-it handle through a shared
constructor.

**Rationale**: `drive` blocks on `rx.recv()` with no timeout; it wakes only
when a packet arrives and processes a bounded, pre-collected event/packet
timeline. Its own doc comment states its "observable behavior is unchanged
from before the live path existed." There is no multi-minute silent stretch
on this path: the run this issue was filed against, and every scenario the
issue describes, is the live `--launch`/attach path (`drive_live`), which
already runs a `recv_timeout(tick)` loop with `tick = Duration::from_millis(200)`
regardless of whether a packet or process event arrives. That existing tick is
also, incidentally, inside the 4-10 Hz range the issue itself suggests for a
redraw rate, so the redraw reuses it rather than adding a second timer.
Restricting the change to `drive_live` also directly satisfies FR-008 (extcap
untouched) and keeps the well-tested golden-producing path's timing semantics
exactly as they are, avoiding the class of defect recurring-bug memory #2
warns about ("building a mechanism and not wiring it to the loop") by putting
the display inside the one loop that actually has the silence problem, not a
shared helper both loops must remember to call.

**Alternatives considered**:
- *Wire both drivers identically.* Rejected: `drive` would need a new
  timeout-based receive loop where none exists today, for a silence problem
  it does not have, and it would put churn into the exact function the
  project's own comments call out as the golden-stability anchor.
- *Add a driver-agnostic wrapper that both loops call periodically.* Rejected:
  `drive`'s blocking `rx.recv()` has no periodic wakeup to call it from
  without the same restructuring the previous option requires; a
  driver-agnostic helper would still have to be built once and wired into two
  different loop shapes, which is more surface for the same "not actually
  wired" failure mode than doing it once where it is needed.

## Decision R-2: A new live-readable stats handle, mirroring `SessionGate`/`GateHandle`

**Decision**: Add `LiveStats`, a small `Clone`-able handle owned by
`Pipeline` (constructed in `Pipeline::new`, returned by a new `pub fn
live_stats(&self) -> LiveStats` callable any time after construction,
including before `run(self)` consumes the pipeline by value), exposing three
atomics: `sink_dropped` (`Arc<AtomicU64>`, incremented at the same site
`crates/fragcap-core/src/pipeline/mod.rs:1042/1048/1053` today updates a
local `sink_dropped: u64`), a snapshot of the holder tally
(`Arc<Mutex<BTreeMap<Arc<str>, u64>>>`, updated at the same site
`pipeline/mod.rs:1034-1035`), and `buffer_dropped` (`Arc<AtomicU64>`,
described below).

`buffer_dropped` needs one more step than the other two because
`Consumer::evicted()` (`crates/fragcap-core/src/pipeline/buffer.rs:169`)
takes its own lock on the buffer's shared `Mutex<Shared>`, the same mutex
the acquisition thread's `Producer::push` locks on every single packet.
Calling `rx.evicted()` from `output_loop` once per packet, as a naive mirror
would, doubles the lock/unlock traffic on that mutex for the whole run's
lifetime, which is exactly the contention this buffer's own module
documentation says the design exists to avoid ("a producer that never waits
for the consumer to make progress"). The correct zero-added-contention
design instead adds one new method beside the existing `next()`, reusing its critical
section rather than replacing it: `Consumer::next_and_evicted(&self) ->
(Option<Item>, u64)` holds the same lock `next()` already takes, pops the
same way, and additionally reads `shared.evicted` before releasing the lock,
at no extra lock acquisition. `next()`'s 7 existing test call sites are untouched; `next()` itself becomes
a thin `#[cfg(test)]` wrapper around `next_and_evicted().0` once `output_loop`
(its only production caller) switches to `next_and_evicted()` directly,
because a `pub(crate)` method with zero non-test callers is dead code under
`cargo clippy --all-targets` when `fragcap-core` builds as a plain dependency
of `fragcap`/`fragcap-cli` (discovered by actually compiling, not anticipated
during planning; recorded here per `AGENTS.md`'s deviation-recording rule).
`output_loop` stores the count `next_and_evicted()` returns into
`live.buffer_dropped` (`Ordering::Relaxed`) every time it returns, whether or
not the call also yielded a new packet. The CLI's redraw
tick then reads `live.buffer_dropped` a few times a second with a plain
atomic load, contending with nothing.

**Rationale**: Everything else FR-001 needs is already live-readable without
a new seam:
- Packets/bytes written so far, and every scope-related discard counter
  (watching, out-of-window, out-of-scope, scope-unresolved): already on
  `GateHandle`'s existing atomics (`crates/fragcap/src/session.rs`'s
  `GateShared`), the same handle `drive_live` already holds as
  `gate_handle`.
- Filter narrowing / active endpoint count: already read live via
  `stamper_reader.active_endpoints()`, the same call `FilterNarration::poll`
  already makes every tick.
- Elapsed time: trivially `started.elapsed()`, already in scope in
  `drive_live`.
- The bound process's pid/role/stage: already folded into the `bound:
  HashMap<u32, String>` and `session.role_bindings()` that `apply_event`
  already maintains in the same function.

Only `sink_dropped` and `holder_tally` are, today, plain local variables
inside the pipeline's single-threaded output loop
(`crates/fragcap-core/src/pipeline/mod.rs`'s output-loop function), never
exposed until the whole pipeline thread is joined at the end of the run
(`handle.join()` in both drivers). `buffer_dropped` reads `rx.evicted()` from
the bounded buffer, which the buffer module already keeps behind a shared
`Arc<Mutex<Shared>>` reachable independent of the single `Consumer`; no new
counter is needed there, only a cheap accessor.

This is a direct, minimal repetition of the pattern S10 already established
for exactly this problem (`SessionGate` owns atomics behind an `Arc`; the
driver keeps a cheap `GateHandle` clone and reads it without contending with
the packet path). A plain `Mutex` for the holder tally (not a lock-free
structure) is deliberate and consistent with precedent: `arc-swap` exists in
this codebase specifically because the attribution snapshot is read on the
hot per-packet path and must never block a capture thread (S10's own stated
argument). The holder-tally lock here is taken once per admitted packet by
the single output-loop thread that already owns the data (no new contention
on that side) and read a few times a second by the CLI thread; that is the
coarse, infrequent read pattern a plain `Mutex` is the right, minimal tool
for, not the case `arc-swap` was introduced to solve.

**Alternatives considered**:
- *Restructure the pipeline to publish a full `CaptureStats` snapshot
  periodically (e.g. via `arc-swap`, matching the attribution index).*
  Rejected as more machinery than the three counters justify; the attribution
  index needed `arc-swap` because it is read per packet, while this handle is
  read a few times a second from a different thread entirely.
- *Have the CLI poll `report.stats` after the run instead of adding a live
  handle.* Rejected outright: that is exactly the status quo the issue is
  filed against: the data already exists, but only after the run ends.
- *Move `holder_tally` construction out of the output loop into something
  externally driven.* Rejected: the output loop is the only place that "sees
  every admitted packet from every interface" (the existing code comment's
  own justification for where it lives); moving it would touch conservation
  accounting this slice has no reason to touch.
- *Call `rx.evicted()` directly from `output_loop` once per packet.*
  Rejected per the contention argument above; a second lock acquisition on
  the shared buffer mutex, once per packet, for the run's entire lifetime, to
  serve a value read a few times a second, is disproportionate and exactly
  the class of avoidable contention the buffer module's own documentation
  warns against.
- *Construct the buffer channel earlier, in `Pipeline::new`, so a reader
  handle could be captured before `run()`.* Rejected: `run(self)` currently
  owns the entire acquisition/output-thread lifecycle and creates the channel
  at the point it is needed; moving channel construction earlier is a larger
  structural change to `Pipeline`'s lifecycle than this slice's two missing
  counters justify, for no benefit the new `next_and_evicted()` method does
  not already provide more cheaply.

## Decision R-3: Terminal detection gates on stderr, not stdout, and needs its own predicate

**Decision**: A new predicate in the CLI crate (not a reuse of
`crate::color::use_color()`) tests `std::io::stderr().is_terminal()` and
`NO_COLOR`. The existing `WARN`/`RESET` ANSI constants in
`crates/fragcap-cli/src/color.rs` are reused for coloring the status block
(and any additional palette constants this slice needs are added to that same
module), but `use_color()` itself is not reused as-is.

**Rationale**: `crate::color::use_color()` is hard-coded to test
`std::io::stdout().is_terminal()` (`color.rs:19`), which is correct for
`doctor` (whose report renders to stdout) but wrong for this feature, whose
entire design constraint is that it renders to stderr only and must never be
influenced by, or influence, what stdout is doing (`--mode stream --out -`
puts capture bytes on stdout; the live display must not care whether stdout
is a pipe). Reusing `use_color()` unmodified would silently gate the capture
status block on the wrong stream. The fix is a second, stderr-gated predicate
placed beside the existing one (in `color.rs`, since that module's own stated
purpose is being "the one place both read from instead" of duplicating
escape-code logic), not a fork of the ANSI constants themselves.

**Alternatives considered**:
- *Generalize `use_color()` to take a stream parameter.* Considered and
  preferred at the call-site level: the plan makes `use_color()` accept which
  stream to test, updating its two existing callers (`doctor.rs`'s two call
  sites) to pass `Stdout` explicitly, and this feature passes `Stderr`. This
  keeps one predicate and one set of constants rather than two predicates,
  and avoids the drift the module's own doc comment says it exists to
  prevent.

## Decision R-4: Redraw mechanism is hand-rolled cursor movement, no new dependency

**Decision**: The redraw counts the number of lines the previous frame wrote,
then before writing the next frame emits `\x1b[<n>A` (cursor up `n` lines)
followed by `\x1b[0J` (erase from cursor to end of screen) to standard error,
matching the issue's own proposed mechanism and FR-011's ban on a new
dependency (`indicatif`, `crossterm`, or similar).

**Rationale**: This workspace has refused terminal-UI and HTTP-client crates
repeatedly on graph-size grounds recorded throughout `AGENTS.md`'s dependency
inventory (most directly, the `indicatif`/`crossterm` refusal is the issue's
own explicit design constraint). Two escape sequences are the entire
mechanism needed; a crate would buy nothing this slice cannot already do by
hand with the same rigor `doctor`'s color handling already demonstrates.

**Alternatives considered**: none seriously; the issue itself forecloses the
dependency option and the existing `doctor` precedent already proves the
hand-rolled approach works in this codebase.

## Decision R-5: Redraw payload is a pure function over a plain data struct

**Decision**: The status block's content (what text and colors the block
contains, and how many lines it occupies) is computed by a pure function
`render_status(&LiveStatusSnapshot, use_color: bool, width: Option<usize>) ->
String` taking a plain, cloneable snapshot struct with no dependency on ETW,
live sockets, or any platform type. The only code gated behind
`#[cfg(all(feature = "etw", windows))]` is the call site inside `drive_live`
that constructs a fresh snapshot each tick from the live handles and writes
the rendered bytes to real stderr.

**Rationale**: `capture_live` (and therefore `drive_live`) is compiled and
runnable only on Windows with the `etw` feature, and per `AGENTS.md`'s own
recorded state, that tier is "never asserted as run in CI." A feature whose
only tests live inside that gate would be exactly the untestable-by-CI defect
class `AGENTS.md` calls out for `MachineAntiCheatProbe` and warns against
repeating: "cfg-gate the data source, not the call site... so the caller and
the called function are both unconditional." This mirrors the codebase's
standing pattern for exactly this problem: `doctor`'s whole classifier is "a
function from an `Inputs` of raw environment facts to a `Report`," fully
testable with hand-built inputs on any target, while only the thin `probe`
module that gathers real inputs is platform-gated and untested.
`CompletionSummary::render` is tested the same way, over a hand-built struct.
This design gives every acceptance scenario in User Story 1 and User Story 2
(the redraw's content, the erase-line accounting, the truncation to 5 rows
plus a trailing count, the heartbeat line's text) a real, CI-running unit
test, leaving only "does `drive_live` call this every tick with the right
inputs" as the part that needs the Tier 2 discipline already documented in
`AGENTS.md` (manual verification, read and recorded, not claimed from a green
CI run alone).

**Alternatives considered**:
- *Render directly against the live handles inline in `drive_live`.*
  Rejected: this is precisely the untestable-by-CI shape recurring-bug memory
  item 6 already names and the codebase already has a working alternative
  pattern for.

## Decision R-6: The `--json` `capture.progress` event reuses the same snapshot

**Decision**: FR-009's optional `capture.progress` event is populated from
the same `LiveStatusSnapshot` the human renderer consumes, serialized through
the existing hand-rolled JSON writer (`fragcap::write_json_string`,
consistent with every other event in `crates/fragcap-cli/src/events.rs`).
Emitted from the same `drive_live` tick, gated on `Format::Json` exactly the
way `Emitter::event` already gates every other structured event, so the human
and machine paths never fire on the same tick for the same run.

**Rationale**: One snapshot type serving both consumers means the two
surfaces cannot silently diverge on what "the live counters" means, the
mistake recurring-bug memory item 3 ("specifying two surfaces and
implementing one") already flags as this campaign's most repeated failure
mode. Since this event is explicitly optional (FR-009 says MAY), and adding
it costs nothing once the snapshot type exists, it is included to close that
gap up front rather than leaving it for a follow-up issue.

**Alternatives considered**:
- *Skip the JSON event entirely, deferring it to a follow-up issue.*
  Considered; not chosen because the marginal cost given the shared snapshot
  type is one `Emitter::event` call already following an established pattern,
  and leaving it out would recreate exactly the two-surfaces gap this
  project has hit twice before.

## Summary of resolved unknowns

No `NEEDS CLARIFICATION` markers remain from the spec. Every "Technical
Context" field in `plan.md` is filled from this research: Rust (workspace
MSRV 1.82, unchanged), no new dependency, target platform Windows for the
live-wired half and any platform for the pure rendering half, and the
existing `cargo xtask ci` gate set as the verification method.
