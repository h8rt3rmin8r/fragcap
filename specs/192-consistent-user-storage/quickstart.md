# Quickstart: Consistent User Storage

1. Review the exact current-user root selected by `fragcap doctor --json`; a fresh user has an absent selected path and no effective destination yet.
2. Run a target operation that requires a default database. It inventories relevant legacy roots, then either copies one unambiguous source or reports a conflict without replacing existing data.
3. Run Doctor again and compare selected and effective catalog, local, session, and registry paths. Inspect the transition record when older bundles remain at their original locations.
4. In controlled Windows checks, use two independent processes under the same disposable account, with different AppData variables. Read the synthetic target from both and independently inspect the opened-file destination. Check an actual packaged host after release.
5. Preview fresh-start before execution and confirm the exact current-user Fragcap roots and the persistent marker that prevents retained AppData state from being imported again.
