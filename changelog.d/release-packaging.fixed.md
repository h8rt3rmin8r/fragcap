The Windows distribution archive now contains only the binary, `LICENSE`, and
`NOTICE`. It no longer ships maintainer tooling from `scripts/`, the
repository-oriented README, the shell wrappers (unnecessary in the archive now
that the binary handles elevation itself), or a false `INCOMPLETE.txt` that fired
on the empty profiles directory fragcap never populates (issues #54, #57).
