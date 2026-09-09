<!-- spec-impact: 17.7 -->
`fragcap targets` now places handle-sorted ready targets before a separate setup-needed group, omits empty groups, keeps one continuous snapshot-backed row sequence, and never recommends setup while a ready target exists. Each group sizes its own table without truncating target or evidence values, while target export and stable identity remain unchanged.
