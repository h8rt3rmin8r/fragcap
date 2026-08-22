# Contract: the non-terminal heartbeat line

Applies when stderr is not a real terminal, verbosity is `Normal`, and the
output format is `Format::Human` (FR-004, FR-005, FR-006).

## Trigger

Emitted through the existing `Emitter::progress` path (a single plain
`writeln!`, no escape sequence, matching every other human progress line
today) when at least 30 seconds (research-fixed constant, Clarifications
session 2026-08-22) have elapsed since the later of: the run's acquisition,
or the last line written through `Emitter::progress` (including a previous
heartbeat). A run that regularly produces progress lines (stage matches,
filter-narrowing transitions) may therefore emit zero heartbeats.

## Content

One line: elapsed time and packets written so far, in the same plain
`label: value` shape as today's existing progress lines (for example `still
capturing: elapsed 00:02:15, 4102 packets written`), carrying no ANSI escape
byte.

## Non-goals

Carries no byte breakdown, no discard counters, and no filter state; it
exists only to prove the run is alive (spec User Story 2), not to convey
live detail. A reader who needs detail redirects to `--json` or waits for the
completion summary.
