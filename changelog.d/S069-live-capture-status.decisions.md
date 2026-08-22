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
dependency, mirroring the codebase's standing pattern for exactly this
problem (`doctor`'s classifier over a plain `Inputs` value,
`CompletionSummary::render`).

**Correction, 2026-08-22 (the first `ci` run on `windows-latest`/`ubuntu-latest`):**
the initial commit gated only the wiring call site (`drive_live` and its
immediate glue) to `#[cfg(all(feature = "etw", windows))]`, on the claim
that the pure `live_status` module's own tests would then run on every CI
platform. That claim was false the moment it was checked against a real
`ubuntu-latest` run: `cargo clippy --all-targets --all-features` also
compiles the crate's plain (non-test) library target, which excludes
`#[cfg(test)]` code entirely, and on that target `live_status`'s items had
*no* caller at all once their one production caller stayed behind the
Windows/ETW gate, `windows` being false regardless of any feature flag on
that runner. The whole `live_status` module, the `Event::CaptureProgress`
variant, `Stream::Stderr`, and the handful of `Emitter` methods this slice
added (`format`, `verbosity`, `live_write`, `progress_written`) are now
gated the same way as `capture_live` itself, each with a comment pointing
back to this entry. The consequence is real and worth stating plainly: this
slice's pure-function test suite runs in CI only on the `windows-latest` leg
of the `check` job (which resolves `windows` true and, under
`--all-features`, `etw` true too), not on `ubuntu-latest`. That is the
existing, established posture for every other Windows/ETW-only code path in
this codebase (`capture_live`, `elapsed_ts`); this slice does not change it,
it only discovered that its own design had, incorrectly, claimed to be an
exception. Local verification of the `windows` cfg evaluating false (no
Linux cross-compiler was available in the environment that found this) used
the equivalent, cfg-identical substitute of building on Windows with the
`etw` feature simply omitted, which `#[cfg(all(feature = "etw", windows))]`
cannot distinguish from `windows` being false.

Review of PR #196 (Codex and Copilot, both independently) also found six
real defects the merge-readiness pass had not caught: the redraw and the
optional JSON tick fired once per message rather than on their own cadence,
because `rx.recv_timeout(tick)` returns immediately whenever a message is
already queued rather than only on the timeout; an ordinary progress line
written while a frame was on screen left the next redraw erasing from a
stale line count, corrupting both; `extcap` reaches the live driver too
(`assemble::components` selects it whenever the run is not an offline
`--offline` replay), so the heartbeat was reaching its stderr despite
FR-008; an attach-to-running capture never populated the bound-process
maps, since the acquisition loop that would otherwise have populated them
exits immediately when the session is already capturing; the status block always passed no terminal
width into the renderer, so the narrow-terminal truncation path (and a
resize) was never actually reached; and the live holder tally's snapshot
method held its mutex across an O(n log n) sort rather than only across the
copy. A seventh finding (ranking the live holder tally by bytes rather than
packets) was verified against the code and declined: the tally mirrors
`CaptureStats::holder_tally`'s existing per-packet count (`dominant_holder`,
used by the completion summary and the launch-and-observe promotion, ranks
the same way), and diverging the live view's ranking from it would let the
two disagree about which process dominates the same run. The spec's
Clarifications session is corrected to describe what is actually counted;
a byte-weighted tally is left as a real, separately-scoped follow-up. All
seven findings, including the declined one, are answered individually on
their PR review threads.
