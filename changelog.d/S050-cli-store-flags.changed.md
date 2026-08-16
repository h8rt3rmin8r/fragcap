<!-- spec-impact: none -->

`fragcap run` replaces the `--hint-db` flag (and `FRAGCAP_HINT_DB`) with
`--catalog-db` and `--local-db` (and `FRAGCAP_CATALOG_DB` / `FRAGCAP_LOCAL_DB`),
one per store; `doctor` now reports both store paths. There is no `--hint-db`
alias.
