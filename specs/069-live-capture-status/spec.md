# Feature Specification: Live capture status display

**Feature Branch**: `069-live-capture-status`

**Created**: 2026-08-22

**Status**: Draft

**Input**: User description: "S069: live capture status display (issue #186).
A `fragcap capture` run says a handful of things at the start, then prints
nothing at all until it ends. On the run that prompted this that was sixteen
minutes of a dead terminal, and the last line on screen for the whole of it
was `filter narrowed to 0 endpoint(s)`. During the first 22 seconds of that
silence fragcap wrote 15,181 packets of unrelated background traffic into the
file, 91 percent of the eventual output, and the operator had no way to see it
happening; they found out by opening the result in Wireshark. fragcap already
computes almost everything a live view needs (admitted packets and bytes, a
per-process holder tally, the active endpoint count, filter-narrowing
transitions, every discard counter, the volume bound, and the bound
process/pid/role/stage); it just never shows any of it while the run is
alive. See the issue body for the full measured evidence and the proposed
shape: https://github.com/h8rt3rmin8r/fragcap/issues/186"

## Clarifications

### Session 2026-08-22

- Q: What heartbeat interval should a non-terminal run use so a redirected or
  logged capture is never silent for an unbounded stretch (FR-004)? → A: 30
  seconds. Coarse enough that a run with real progress lines rarely needs one
  at all (progress lines reset the interval), frequent enough that a reader
  tailing a log during a multi-minute silent stretch (the sixteen-minute case
  that prompted the issue) sees a heartbeat well before wondering if the
  process died.
- Q: When the per-process holder tally has more image names than the status
  block has room for, how many rows should the live display show before
  truncating (Edge Cases)? → A: The top 5 by bytes written, plus a trailing
  "N more" count if any remain. Matches the dominant-contributor framing the
  issue itself uses (one background process was 91 percent of the file); five
  rows is enough to show a dominant outlier plus a couple of runners-up
  without the block's height varying wildly run to run.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See capture working while it runs (Priority: P1)

An operator starts `fragcap capture` against a launched title from an
interactive terminal. Instead of a silent screen from the moment the target
is acquired until the run stops, they see a status block that updates in
place, at least once a second, showing elapsed time, the bound process,
packets and bytes written so far, progress against any configured volume
bound, the filter's narrowing state, and every discard counter the run is
already tracking.

**Why this priority**: This is the entire measured defect in issue #186. The
sixteen-minute silent run that prompted the issue would have shown, within
seconds, that 91 percent of the file was going to an unrelated background
process, the single fact the operator needed and had no way to see.

**Independent Test**: Run a capture against a synthetic pipeline with a
terminal-backed stderr, advance the underlying counters (admitted packets and
bytes, per-image tally, active endpoints, discard counters) programmatically,
and verify the rendered status block reflects each new value within one
redraw cycle, with earlier renders erased rather than accumulated as
scrollback.

**Acceptance Scenarios**:

1. **Given** a capture session bound to a process and running on a real
   terminal, **When** at least one second has elapsed since the run began,
   **Then** the terminal shows a status block reporting elapsed time, the
   bound process's name/pid/role/stage, packets and bytes written, and every
   discard counter.
2. **Given** a capture session with a `--max-bytes` or `--max-packets` bound
   configured, **When** the status block renders, **Then** it shows written
   volume against that bound.
3. **Given** a capture session with no volume bound configured, **When** the
   status block renders, **Then** it shows written volume with no bound
   comparison, rather than a misleading "against nothing."
4. **Given** the filter has not yet narrowed (endpoint count zero), **When**
   the status block renders, **Then** it reports that state plainly rather
   than omitting the filter line, since a stuck-at-zero narrowing is exactly
   the symptom the companion report (#185) describes.
5. **Given** successive redraws, **When** a new status block is written,
   **Then** the previous block's lines are erased in place rather than left
   as additional scrollback, so a sixteen-minute run does not fill the
   terminal with thousands of near-duplicate lines.

---

### User Story 2 - Non-terminal runs stay exactly as they are today, and are not silent either (Priority: P2)

An operator redirects `fragcap capture` output to a log file, or runs it
under a supervisor with no attached terminal. The redraw behavior does not
apply (escape sequences in a log file are noise, not information), but the
run is not silent for the whole of a long capture either: today's plain
progress lines still appear at the same points they do now, and the run
prints an occasional plain-text heartbeat line so a operator tailing the log
can see the run is still alive between milestones.

**Why this priority**: The issue is explicit that the redraw "must not appear
in a redirected log or in CI output," and a live-display feature that
regresses the non-terminal case (the majority of automated and CI usage)
would trade one blind spot for another.

**Independent Test**: Run the same capture session with stderr redirected to
a file (not a terminal) and verify the file contains no ANSI escape
sequences, that today's existing progress lines are unchanged, and that a
plain heartbeat line appears periodically during a long silent stretch.

**Acceptance Scenarios**:

1. **Given** stderr is not a terminal, **When** capture runs, **Then** the
   output file contains no cursor-control or color escape sequences.
2. **Given** stderr is not a terminal and more than one heartbeat interval
   elapses with no other progress line, **When** the interval elapses,
   **Then** a single plain-text heartbeat line is appended reporting elapsed
   time and packets written so far.
3. **Given** stderr is not a terminal, **When** compared against a capture
   run from before this feature existed under the same inputs, **Then** every
   line that existed before still appears, unchanged, aside from the added
   heartbeat lines.

---

### User Story 3 - The other output surfaces are untouched (Priority: P1)

An operator relies on `--json` for machine-readable events, `--mode stream`
with `--out -` to pipe capture bytes on stdout, `--quiet`/`--silent` to
control verbosity, or the `extcap` integration for Wireshark to drive capture
directly. None of these consumers see any change in behavior from this
feature: the human live display is strictly additive to the human,
verbose, terminal-attached case.

**Why this priority**: Tied with User Story 1 for priority because a
regression here breaks an existing, relied-upon contract (extcap capture from
Wireshark, or a `--json` consumer's event stream) in service of a feature
that exists to help the interactive human case. The issue enumerates this as
non-negotiable design constraints, not aspirations.

**Independent Test**: Run the same capture session once in `--json` mode,
once with `--mode stream --out -`, once with `--quiet`, once with `--silent`,
and once through the `extcap` capture path; diff stdout and the structured
stderr stream byte-for-byte against a pre-feature baseline captured from the
same inputs.

**Acceptance Scenarios**:

1. **Given** `--json` is set, **When** capture runs on a terminal, **Then**
   no human status block appears anywhere in the output; the JSON event
   stream is unchanged from before this feature, aside from the optional
   periodic `capture.progress` event permitted by FR-009.
2. **Given** `--mode stream --out -`, **When** capture runs, **Then** stdout
   contains only capture bytes, byte-identical to a pre-feature run with the
   same inputs; the live display, if any, appears on stderr only.
3. **Given** `--quiet`, **When** capture runs on a terminal, **Then** no live
   status block appears, matching the existing contract that quiet suppresses
   progress.
4. **Given** `--silent`, **When** capture runs on a terminal, **Then** no
   live status block and no heartbeat line appears; only warnings-suppressed,
   errors-only output remains, matching the existing contract.
5. **Given** the `extcap` capture path, **When** Wireshark drives capture
   through it, **Then** its output is unchanged from before this feature.

---

### Edge Cases

- The terminal is resized mid-run: the next redraw uses the new width; a
  redraw must not throw or corrupt the terminal if the block no longer fits
  the same number of columns it did at the previous draw.
- `NO_COLOR` is set: the status block still renders and still updates, but
  with no color codes, matching the existing `doctor` command's contract.
- The run stops (any `StopReason`) while a status block is on screen: the
  block is cleared or finalized cleanly before the completion summary prints,
  so the two never interleave or leave stray redraw artifacts on screen.
- A target is never acquired (the pre-acquisition wait): no status block
  appears yet, since there is nothing bound to report on; today's existing
  "waiting for it to start" progress line is unchanged.
- Stderr is a terminal but an extremely narrow one (for example, an 80-column
  ConPTY the operator has shrunk further): the block degrades by truncating
  or wrapping rather than panicking or writing out-of-bounds escape
  sequences.
- The process holder tally contains more image names than fit in the status
  block's available lines: the display shows the top 5 images by bytes
  written, plus a trailing count of how many more are not shown, rather than
  silently dropping data with no indication that something was omitted.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: When stderr is attached to a real terminal and verbosity is
  `Normal`, fragcap MUST render a status block during an active capture
  session, refreshed at least once per second, showing: elapsed time, the
  bound process's name, pid, role, and stage; packets and bytes written so
  far; progress against a configured volume bound; the filter's active
  endpoint count and narrowing state; and every discard counter already
  tracked by the session (watch-time, out-of-window, out-of-scope,
  scope-unresolved, buffer, sink).
- **FR-002**: Each redraw MUST erase the previous status block in place
  (cursor and line-clear control sequences) rather than appending new lines,
  so an arbitrarily long capture does not fill the terminal's scrollback with
  near-duplicate frames.
- **FR-003**: When stderr is not a real terminal, fragcap MUST NOT emit any
  cursor-control or color escape sequence; output for this case MUST remain
  byte-for-byte what it was before this feature, except for the addition of
  periodic heartbeat lines under FR-004.
- **FR-004**: When stderr is not a real terminal, verbosity is `Normal`, and
  more than a fixed interval has elapsed since the last progress line was
  written, fragcap MUST append a single plain-text heartbeat line reporting
  elapsed time and packets written, so a redirected or logged run is never
  silent for an unbounded stretch.
- **FR-005**: The live status block MUST NOT render, and no heartbeat line
  MUST appear, when the output format is `--json`; the JSON event stream's
  existing behavior is unchanged by this feature, except for the optional
  addition described in FR-009.
- **FR-006**: The live status block and the heartbeat line MUST both be
  suppressed under `--quiet`, matching the existing contract that quiet
  suppresses progress output; both MUST also be suppressed under `--silent`,
  matching the existing contract that silent suppresses everything but
  errors.
- **FR-007**: Nothing introduced by this feature MUST write to stdout under
  any combination of flags, including `--mode stream --out -`; the live
  display and heartbeat are stderr-only, matching the emitter's existing
  stream separation.
- **FR-008**: The `extcap` capture path MUST be unaffected by this feature:
  its output MUST be unchanged from a pre-feature baseline under the same
  inputs.
- **FR-009**: fragcap MAY emit a periodic `capture.progress` event in the
  `--json` event stream carrying the same underlying counters as the human
  status block, provided it never appears when `--json` is not set and never
  replaces or reorders any existing event in that stream.
- **FR-010**: Color in the status block MUST be disabled, with the block's
  layout otherwise unchanged, when the `NO_COLOR` environment variable is
  set, matching the existing `doctor` command's contract.
- **FR-011**: fragcap MUST NOT introduce a new runtime dependency (in
  `Cargo.lock`) to implement the live display; rendering MUST be built from
  hand-rolled ANSI control sequences, extending the existing pattern already
  used by the `doctor` command.
- **FR-012**: The status block's redraw and the completion summary MUST NOT
  interleave: when a capture session stops, any in-progress status block MUST
  be resolved (cleared or replaced by the summary) before the completion
  summary begins printing.
- **FR-013**: The status block MUST render before any target is acquired only
  if there is bound-process information to show; while a session is still
  waiting for its target, the existing pre-acquisition progress line MUST be
  the only output, unchanged from today.

### Key Entities

- **Live status snapshot**: a point-in-time read of the counters and state
  already maintained by an active capture session (admitted packets/bytes,
  volume bound if any, per-image holder tally, active endpoint count and
  narrowing state, every discard counter, elapsed time, and the bound
  process's name/pid/role/stage). This slice adds no new counters; it reads
  existing ones on a timer and renders them. Constructing a snapshot must not
  block or contend with the capture or output threads.
- **Redraw state**: the terminal-rendering seam that knows how many lines the
  previous frame occupied, so the next frame can erase exactly that many
  before writing the new one. Exists only for the terminal case; the
  non-terminal case has no redraw state, only the existing append-only
  writer plus a heartbeat timer.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On a terminal, an operator watching a capture run can tell,
  within one second of a change, how many packets and bytes have been
  written, whether the filter has narrowed, and whether the target's own
  traffic or someone else's is dominating the file, without waiting for the
  run to end or opening the output in a separate tool.
- **SC-002**: A capture run with stderr redirected to a file produces a file
  containing zero escape sequences, verified by a standing regression test
  that scans the captured bytes for the control characters the redraw would
  use.
- **SC-003**: A capture run's `--json` stream, `--mode stream --out -` stdout
  bytes, `--quiet` output, `--silent` output, and `extcap` output are each
  byte-identical to a pre-feature baseline under the same inputs, verified by
  a standing regression test per surface.
- **SC-004**: `Cargo.lock` gains zero new runtime packages as a result of
  this feature, verified by comparing the lockfile before and after the
  slice.
- **SC-005**: Reproducing the run that prompted issue #186 (a sixteen-minute
  capture with 91 percent of its volume attributed to a non-target process in
  the first 22 seconds) against this feature shows the dominant non-target
  contributor in the status block within the first redraw cycle after the
  volume gate admits its first packets, rather than only in the end-of-run
  summary.

## Assumptions

- This slice implements exactly the "first deliverable" the issue itself
  scopes out: the live status block, its terminal/non-terminal split, and the
  heartbeat line. The issue's second half, a general visual pass across the
  rest of the CLI (color for warnings and errors generally, thousands
  separators, byte-unit formatting elsewhere, restyling
  `CompletionSummary`), is explicitly described there as the longer pass the
  live display is "built inside," not this slice's deliverable, and is left
  for a follow-up.
- The refresh interval is a fixed constant chosen in the 4-10 Hz range the
  issue suggests (implementation detail for planning), polling the same
  atomics the pipeline already maintains; no configuration flag is added to
  change it, since the issue does not ask for one and the counters are cheap
  to read.
- The heartbeat interval (User Story 2) is a fixed 30-second constant,
  distinct from the sub-second terminal refresh rate, since its purpose is
  "prove the run is alive," not "show live detail," to a reader tailing a log
  file. It resets on every progress line, so a run with regular milestones
  rarely emits one at all.
- Color, where used, draws from the palette already settled for this project
  in `docs/brand/README.md` (Signal Cyan, Capture Orange, Fault), with a
  plain fallback when `NO_COLOR` is set or color support cannot be detected,
  reusing rather than duplicating the `doctor` command's existing ANSI
  module.
- "Every discard counter" in FR-001 means every counter already surfaced in
  `CompletionSummary` as of this slice (watching discarded, out of window,
  out of scope, scope unresolved, buffer dropped, sink dropped); a discard
  cause added by a later slice is out of scope here and would need its own
  follow-up to reach the live display, the same way it would need one to
  reach the completion summary.
- The live display reads counters produced by the pipeline and session layers
  (`fragcap-core`, `fragcap`) but is itself CLI-layer presentation, consistent
  with this project's existing placement of `CompletionSummary` and
  `Emitter` in `fragcap-cli`; no new counter or state is added to a lower
  layer to support it.
