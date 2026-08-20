<!-- spec-impact: none -->
The agent instruction files no longer assert things that stopped being true. The
standing verification block in `AGENTS.md` had six items, four of which were
stale, and it was phrased as a prohibition, so every agent session was directed
to repeat them; most consequentially it still said live capture had never
executed, which it did on 2026-08-20. The block now states its rule once and
sorts its items into what has been discharged, with the evidence and the date,
and what is still outstanding, with what would discharge it. `AGENTS.md` and
`CLAUDE.md` also no longer claim a slice-numbered completion state that went
stale roughly forty slices ago; both now name `specs/` and `changelog.d/` as the
authority instead.
