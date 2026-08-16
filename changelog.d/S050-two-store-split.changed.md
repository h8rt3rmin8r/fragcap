<!-- spec-impact: 15.3, 15.6.2, 20.2, 24.5 -->

fragcap now keeps two stores in its per-user data directory: a shipped
`catalog.db` (disposable, replaced wholesale by a future catalog refresh) and a
user-owned `local.db` (where learned launch data accumulates and later user data
lives). A catalog refresh never touches `local.db`. Both are created on first run
with no elevation, and resolution consults the local store before the catalog.
