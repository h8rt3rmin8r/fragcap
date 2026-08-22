<!-- spec-impact: none -->
The vendored agent skills now come from a single upstream,
<https://github.com/shruggietech/skills> at release v1.11.0, verified against
that release's published checksums and copied in unmodified. The house Markdown
and PowerShell standards were both stale and are refreshed, and the house Bash
standard is vendored for the first time, closing a gap `skills/README.md` had
carried unmet since before slice S18. A new `cargo xtask skills` check, part of
the ordinary gate set, holds `.agents/skills/`, `skills-lock.json`, and git's
index to agreement with one another.
