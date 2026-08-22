# Contract: the human status block

Rendered by a pure function (research R-5) over a `LiveStatusSnapshot`,
`use_color: bool`, and an optional terminal width. Applies only when stderr is
a real terminal, verbosity is `Normal`, and the output format is `Format::Human`
(FR-001, FR-005, FR-006).

## Content (every field required by FR-001)

1. **Header line**: the bound process's name, pid, role, and stage, or a
   waiting indicator if `process` is `None` (FR-013: this state is only
   reached once there is bound-process information to show, so in practice
   this branch is defensive rather than expected mid-run).
2. **Elapsed line**: elapsed time (`HH:MM:SS`), packets written, bytes
   written, and (when a bound is configured) written volume against the
   bound; when no bound is configured, volume is shown with no bound
   comparison (spec Acceptance Scenario 3).
3. **Filter line**: `narrowed, N endpoint(s)` when `narrowed` is true,
   otherwise an explicit "not yet narrowed" state (spec Acceptance
   Scenario 4) rather than omitting the line.
4. **Discards line**: every one of `watch_discarded`, `out_of_window_discarded`,
   `buffer_dropped`, `sink_dropped`, `scope_discarded`,
   `scope_unresolved_discarded`, labeled, matching the same counter set (and
   the same words) `CompletionSummary::render` already uses so the two never
   describe the same thing with different vocabulary.
5. **Process breakdown lines** (0 or more, only when `holder_tally` is
   non-empty): the top 5 entries by count, each as `<image> <count>`, plus a
   trailing `... and N more` line when more than 5 images are tallied
   (Clarifications session, 2026-08-22).

## Layout rules

- Total line count is deterministic from the snapshot's shape (whether
  `process` is set, whether a bound is configured, how many holder-tally
  entries exist up to the 5-row cap plus one overflow line), so
  `RedrawState::previous_lines` can always know exactly how many lines to
  erase before the next frame (FR-002).
- Color, when `use_color` is true, marks a non-zero discard counter distinctly
  from a zero one (reusing the existing `WARN`/`RESET` constants in
  `crates/fragcap-cli/src/color.rs`); layout is identical whether or not color
  is applied (FR-010).
- When a `width` is supplied and a line would exceed it, the line is
  truncated rather than wrapped, and never contains a raw escape byte past
  the truncation point (spec Edge Cases: narrow terminal).

## Redraw sequence

Before writing a new frame (after the first): write `\x1b[<n>A` where `n` is
`RedrawState::previous_lines`, then `\x1b[0J`, then the new frame's bytes,
then update `previous_lines` to the new frame's line count (FR-002, research
R-4).

## Non-terminal contract

When this contract does not apply (stderr is not a terminal), no byte defined
above is ever written; the existing plain progress lines are the only output,
plus the heartbeat line defined in `contracts/heartbeat-line.md` (FR-003).
