<!-- spec-impact: none -->
**2026-08-20** Declined a `cargo xtask lint` rule policing claims of the form "X
has never run", proposed as point 4 of issue #187. Such claims assert something
about external continuous-integration history rather than about repository
bytes, so the rule would have to query the forge over the network. Every other
rule in that gate (`OpenProcess`, the pcap transmit calls, BOM, CRLF, dashes,
SPDX) reads the working tree and nothing else, and it is the cheapest check in
the set precisely because it is hermetic, deterministic, and runnable offline.
Making `cargo xtask ci` depend on a network connection in order to police six
sentences is the worse trade. The mitigation taken instead is to attach a date
to every claim about something observed once, so a stale observation is visible
on reading rather than only on running something. Claims about how a check
behaves are invariant, carry no date, and name instead how to see the behavior;
the block says which kind is which, so a missing date is never ambiguous. Issue #187 anticipated
this outcome and accepted it.

**2026-08-20** Kept the standing verification block in `AGENTS.md` as a rule
with items under it, rather than renaming it to a status report or splitting the
rule away from the evidence. Four of its six items had been discharged, which
made its heading wrong as a label and not only as a count, and the obvious moves
were to rename it or to split it in two. Both lose the thing worth keeping: the
instruction to distinguish a check that did not run from a check that passed is
durable and must survive even when no item under it is outstanding, and it is
only concrete while the evidence sits beneath it. Reorganizing the items into
discharged and outstanding under one rule-stating heading also removed the count
that had been wrong since the list grew past two, rather than incrementing it.
