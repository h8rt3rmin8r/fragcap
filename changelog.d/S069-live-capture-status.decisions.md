<!-- spec-impact: 17.5, 17.6 -->
**2026-08-22** The redraw is wired only into `capture_live`/`drive_live`
(`crates/fragcap-cli/src/orchestrator.rs`), the live `--launch`/attach path.
`capture_prerecorded`/`drive`, the offline two-phase driver every tier-1
test, every committed golden, and the `extcap` integration run through,
is untouched: it blocks on `rx.recv()` with no timeout and has no
multi-minute-silence problem to solve, and its own doc comment already
states its behavior is unchanged since before the live path existed.
Restricting the change to one function kept the well-tested golden-producing
path's timing semantics exactly as they were and satisfied the extcap
non-interference requirement by construction rather than by a new gate.

A new `fragcap-core::pipeline::LiveStats` handle mirrors two counters
(`sink_dropped`, the per-image holder tally) that were, before this slice,
plain local variables inside the output loop's single-threaded function,
visible to a caller only after the whole pipeline thread joined at the end
of a run. It is obtained via `Pipeline::live_stats()`, callable any time
after `Pipeline::new`, before `run(self)` consumes the pipeline by value,
directly mirroring the `SessionGate`/`GateHandle` split S10 already
established for the same class of problem. The third counter,
`buffer_dropped`, already had a live-readable path (`Consumer::evicted()`),
but mirroring it into `LiveStats` from the output loop's hot per-packet path
by calling `evicted()` there would double the lock acquisitions on the
buffer's shared mutex for the run's entire lifetime, contention the buffer
module's own documentation says its design exists to avoid. The fix is a new
`Consumer::next_and_evicted()` beside the existing, untouched `next()`,
reading the eviction count inside the same lock `next()` already holds, at
no additional lock acquisition; `next()` itself became `#[cfg(test)]` once
`output_loop` was its only production caller, since a `pub(crate)` method
with zero non-test callers is dead code under `cargo clippy --all-targets`
when `fragcap-core` builds as a plain dependency of `fragcap`/`fragcap-cli`
(found by compiling, not anticipated in planning).

`crate::color::use_color()` (`doctor`, `targets`) took no parameter and
always tested `std::io::stdout()`. Reusing it unmodified for the live
display, which renders to stderr and must never be influenced by what
stdout happens to be, would have silently gated the wrong stream. It now
takes an explicit `Stream` (`Stdout` or `Stderr`); `doctor` and `targets`
pass `Stdout` and are unaffected.

Every new rendering rule (the status block's content and layout, the
redraw's erase-then-write sequence, the heartbeat's due/reset timer) is a
pure function over a plain snapshot struct, with no ETW, socket, or platform
dependency. Only the call site that constructs a snapshot from a live run
and writes real bytes is gated to `#[cfg(all(feature = "etw", windows))]`,
which `AGENTS.md` records as never asserted as green in continuous
integration. This mirrors the codebase's standing pattern for exactly this
problem (`doctor`'s classifier over a plain `Inputs` value,
`CompletionSummary::render`), so every acceptance scenario this slice's spec
names has a test that runs on every CI platform, and the untestable-by-CI
surface is reduced to "does `drive_live` call the pure functions with the
right inputs," verified by direct tests against the wiring struct
(`LiveStatusDisplay::tick`) rather than only by a full live capture.
