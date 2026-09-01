<!-- spec-impact: 24.2 -->
**2026-09-01:** Removed path filters from the Windows platform workflow because it runs the entire workspace. The previous crate list skipped platform regressions in crates the job actually tests. Main-branch push scope still prevents duplicate branch and pull-request runs, and repository lint now rejects a future filtered trigger.
