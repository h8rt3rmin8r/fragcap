<!-- spec-impact: 17.5, 17.6 -->
`fragcap capture` no longer goes silent from acquisition until the run stops.
On a real terminal, a status block redraws in place at least once a second,
showing elapsed time, the bound process, packets and bytes written against
any configured `--max-bytes`/`--max-packets` bound, the capture filter's
narrowing state, every discard counter, and the top per-process contributors
to the file so far. This is the fix for the run that prompted issue #186: a
sixteen-minute capture whose last visible line was `filter narrowed to 0
endpoint(s)`, during which 91 percent of the eventual file went to an
unrelated background process with no way for the operator to notice until the
run ended.

When standard error is not a terminal (a redirected or logged run), the
redraw never appears, and output is otherwise unchanged, except that a
run left silent for thirty seconds now gets a single plain heartbeat line
reporting elapsed time and packets written, so a long redirected run is not
silent either. `--json` gains an optional periodic `capture.progress` event
carrying the same counters (never appearing outside `--json`); `--quiet`,
`--silent`, `--mode stream --out -`, and the `extcap` integration are
unaffected.

`NO_COLOR` disables the block's color and leaves its layout unchanged,
matching the existing `doctor` command's contract. No new runtime dependency
was added; the redraw is two hand-rolled ANSI escape sequences, extending the
pattern `doctor` already established.
