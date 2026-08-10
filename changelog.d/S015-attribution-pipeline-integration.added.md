### Added

- **The attribution refresh is driven by the pipeline.** `Pipeline::run`'s
  section 8.6 control thread now drives `FlowAttributor::refresh` on the section
  11.2 cadence, so a connection opened after capture starts is re-read into the
  published snapshot and becomes attributable, and enters the narrowed filter,
  rather than the snapshot staying frozen at construction. Resolves issue #19.
- **Phase-two narrowing is restricted to profiled processes.** The kernel filter
  now admits only endpoints owned by a profiled process (specification section
  12.2), not every socket on the machine. `FlowAttributor::active_endpoints_owned`
  carries the owning identifier the plain endpoint list drops, and the session's
  `RoleStampingAttributor` joins it against the stage bindings it already holds to
  filter the narrowing input. Resolves issue #18.
- **The CLI refresh stopgap is gone.** `FlowAttributor::refresh` now takes
  `&self`, so the pipeline shares and refreshes one attributor through the
  `Arc<dyn FlowAttributor>` the capture threads resolve against; the CLI
  `RefreshDriver` control thread and the read/write `PublishedResolver` split it
  depended on are no longer needed on the live path. The resolve path stays
  lock-free (section 11.6).
- **`OwnedEndpoint` and glossary entries.** `fragcap-core::flow` gains
  `OwnedEndpoint` (an endpoint paired with its owning process identifier), and
  `docs/glossary.md` gains `OwnedEndpoint` and `Profiled endpoint set`
  (constitution P-6).
