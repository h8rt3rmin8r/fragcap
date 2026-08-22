# Quickstart: validating the live capture status display

## Prerequisites

- A Windows machine with the `etw` and `live` features buildable
  (`cargo build -p fragcap-cli --features etw,live`), since the redraw is
  wired only into the live driver (research R-1). The pure-rendering unit
  tests need neither feature and run on any platform via `cargo test
  --workspace --locked`.
- A profile and a running or launchable title, exactly as any other manual
  `fragcap capture` run needs (see `docs/plans/README.md` / the S17 launch
  slice for how this project already exercises the live path manually).

## Automated validation (any platform, part of `cargo xtask ci`)

1. `cargo test -p fragcap-cli --lib --features etw live_status` runs the pure
   renderer's unit tests (`live_status::tests::*`): line counts and content
   for every combination named in `contracts/status-block.md` (no bound, a
   bound reached, no bound configured, zero holder-tally entries, fewer than
   5, more than 5, narrowed, not narrowed, every discard counter zero or
   non-zero, `NO_COLOR`'s effect via `use_color_flag`, a narrow width), plus
   the redraw erase-sequence tests (`live_status::redraw::tests::*`) and the
   heartbeat timer tests (`live_status::heartbeat::tests::*`): no line before
   30 seconds of no progress, one line at or after 30 seconds, the timer
   resetting on an intervening progress line.
2. `cargo test -p fragcap-core buffer live_stats pipeline` runs the new
   pipeline handle's tests: `Consumer::next_and_evicted` agrees with `next()`
   plus a separate `evicted()` read; a fresh `LiveStats` clone observes
   writes made through another clone; and `live_stats_taken_before_run_matches_the_final_report`
   confirms a `LiveStats` handle cloned before `Pipeline::run` reports the
   same `sink_dropped` and holder tally the final `PipelineReport` does.
3. `cargo test -p fragcap-cli --lib --features etw orchestrator::live_status_display_tests`
   runs `LiveStatusDisplay::tick`'s decision tests directly, independent of a
   real ETW/live capture (research R-5): JSON format emits `capture.progress`
   and never a redraw or heartbeat; a terminal draws a frame with no prior
   erase; a non-terminal stream never draws a redraw frame; `--quiet`/
   `--silent` suppress the human display entirely; and (SC-002) a simulated
   multi-tick non-terminal run never writes an escape byte anywhere in the
   captured stream.
4. `cargo test -p fragcap-cli --lib --features etw` (the full crate,
   `etw`-only avoids the `live` feature's `wpcap.lib` link requirement) runs
   every test above together as a regression check.

## Manual validation (Tier 2, Windows, watched to completion per `AGENTS.md`)

1. Run `fragcap capture --launch <a Steam app id> --max-bytes 50mb` from a
   real terminal (PowerShell or a ConPTY-backed terminal). Confirm a status
   block appears within one second of target acquisition, updates in place
   (no scrollback growth) at least once per second, and its counters move as
   packets arrive.
2. Redirect the same invocation's stderr to a file:
   `fragcap capture --launch <id> --max-bytes 50mb 2> run.log`. Confirm
   `run.log` contains no `\x1b` byte (`grep -c $'\x1b' run.log` is `0`) and,
   for a run left running past 30 seconds with the target still not
   producing new stage-transition lines, a heartbeat line appears.
3. Run the same invocation with `--json`, `--quiet`, `--silent`, and via the
   `extcap` integration once each; diff each against a baseline captured
   from the pre-feature binary (or the prior release) under the same
   inputs, confirming byte-for-byte equality (User Story 3).
4. Set `NO_COLOR=1` and repeat step 1; confirm the block still updates with
   the same layout and no ANSI color codes (`\x1b[3` sequences) in the
   captured raw bytes (a terminal emulator's screen buffer can be dumped, or
   the same redirect-to-file trick works since redirection also disables
   `use_color`'s terminal check, so this specific check needs a
   terminal-attached capture of raw bytes, e.g. via `script`/a PTY-capturing
   tool, not a plain redirect).
5. Compare `Cargo.lock` before and after the slice's commits
   (`git diff <before>..<after> -- Cargo.lock`); confirm zero added
   `[[package]]` entries (SC-004).

## Expected outcome

Every acceptance scenario in `spec.md`'s three user stories passes; the
sixteen-minute-silence reproduction in SC-005 shows the dominant contributor
in the status block within the first redraw after the volume gate admits its
first packets, not only in the end-of-run summary.
