# Contract: Stage Matching and Session Lifecycle API

The public surface S12 adds, and the invariants each element guarantees.

## `fragcap_core::process::tree`

- `ProcessTree::bind_stage(&mut self, id: NodeId, stage: StageId) -> bool`
  - Postcondition: if the node exists and was unbound, its `stage()` is now
    `Some(stage)` and the call returns `true`; otherwise the tree is unchanged
    and it returns `false`.
  - Invariant: a node binds to at most one stage (no rebinding).

## `fragcap_profile::matching` (re-exported at `fragcap::profile::matching`)

- `stage_for(profile, tree, node) -> Option<&Stage>`
  - Returns the first stage in `profile.stages()` order all of whose specified
    predicates hold for `node`, or `None`.
  - Pure: reads the tree (including current bindings for `descends_from`), mutates
    nothing.
- `bind_stages(profile, &mut tree)`
  - Iterates nodes in creation (`NodeId`) order; binds each unbound node to its
    `stage_for` result. After it returns, every node that matches a stage under
    causal ordering is bound.
- Predicate contract: `exe` case-insensitive on the file name; `path_contains`
  case-insensitive substring of the full path; `path_regex` reuses the compiled
  expression; `cmdline_contains` never matches an unavailable command line;
  `descends_from` requires a strict ancestor bound to the named role.

## `fragcap::session`

- `CaptureSession::new(profile, config) -> CaptureSession` in `Arming`.
- `attach(&mut self, at: Timestamp)` moves `Arming -> Watching`; panics or is a
  no-op if not in `Arming` (choose no-op with debug assertion).
- `on_process_event(&mut self, event)` (the event carries its own timestamp):
  - `Started`: applies to the tree, evaluates `stage_for`, binds on a match, and
    on the first match in `Watching` moves to `Capturing`.
  - `Exited`: if the exited node is bound to the terminal stage, stops with
    `TerminalStageExited`; if all non-service bound processes have exited and no
    non-service stage is still awaited, stops with `AllProcessesExited`.
- `on_packet(&mut self, len: u32) -> PacketDisposition`:
  - `Watching`: returns `Discarded`, increments `watching_discarded`.
  - `Capturing`: returns `Retained`, increments `retained` and `retained_bytes`,
    and stops with `VolumeReached` if a configured bound is met.
  - Any other state: `Discarded`, no counter movement (nothing is captured
    before arm or after drain).
- `on_tick(&mut self, now)`: acquisition timeout from `Watching` -> `Complete`
  (`AcquisitionTimeout`); duration bound -> `Draining` (`DurationReached`).
- `on_interrupt`, `on_sink_error`: `Draining` with `Interrupt` / `SinkError`.
- `finalize(&mut self)`: `Draining -> Complete`.
- Conservation invariant: `stats().watching_discarded + stats().retained` equals
  the number of packets passed to `on_packet` while armed and not yet complete.

## What this contract does not cover

- Wiring the session to a live `PacketSource`, filter installation, or stamping
  role/stage onto `Attribution` (S13, S14).
- Any change to `CaptureStats` or the pipeline conservation identity.
