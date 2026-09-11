<!-- spec-impact: 15, 17.2.1, 25, 26, 28.1 -->
`fragcap calibrate` now creates a durable target-bound workflow and can resume it
explicitly with `--resume <WORKFLOW_ID>`. Resume preserves candidate intent, exact
no-repeat history, and attempt numbering while revalidating current target, process, compatibility, recovery,
bundle, plan, and confirmation authority before later work. `--pause-for` records
login, EULA, gameplay, shutdown, or interruption work without session effects.
