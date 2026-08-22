<!-- spec-impact: none -->
Thirty-three vendored agent skills are removed from `.agents/skills/`, leaving
the four the constitution actually binds: `shruggie-bash`, `shruggie-markdown`,
`shruggie-powershell`, and `shruggie-speckit`. The set is now governed by a
stated admission test, recorded in `skills/README.md`: a skill is admitted only
if a named constitution principle binds this repository to it or a repository
gate executes it. Removed were `architect`, `baseline-restorer`,
`boy-scout-rule`, `code-review`, `debug`, `develop`, `document`, `explain`,
`explicit-configuration`, `find-skills`, `fix`, `gh-fix-ci`,
`legacy-code-safety`, `optimize`, `orthogonality-principle`, `plan`,
`professional-honesty`, `project-memory`, `proof-of-work`, `refactor`,
`review`, `rust-best-practices`, `rust-design-review`, `rust-skills`,
`shruggie-graph-memory`, `shruggie-html`, `silent-execution`,
`simplicity-principles`, `solid-principles`, `structural-design-principles`,
`test`, `token-efficiency`, and `traffic-analysis-pcap`. None was referenced by
any document, workflow, script, or gate in this repository, and none was
specific to fragcap; each remains available from its own upstream.
