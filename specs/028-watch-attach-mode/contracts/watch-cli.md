# Contract: `watch` subcommand and attach-to-running

## `fragcap watch`

- `--exe <glob>` (required): the target image-name glob.
- `--path <substr>` / `--path-regex <re>` (optional): a path anchor; both may be
  combined with `--exe`.
- `--wait <dur>` (optional): acquisition timeout; absent means watch until
  interrupted.
- `--duration`, `--out`, `--sink`, `--no-payload`: as `run`/`tap`.
- Hidden `OfflineArgs` for tier-1 tests.

### Behavior
- Synthesizes a validated one-stage identity profile (authored fidelity) from the
  predicates; an empty match or a non-compiling regex is a configuration error
  (exit 2), reported as the profile's diagnostics.
- Arms launch-agnostically: no `steam://`, no managed launch. Captures the first
  process matching the identity, however started.
- If a matching process is already present at arm (startup snapshot), acquires it
  at arm; else waits for a start.
- With `--wait`, a never-appearing target ends `StopReason::AcquisitionTimeout`,
  surfaces the watch-time discard accounting, exit 1. A clean interrupt exits 0.
- Output byte-identical to an equivalent single-stage profile capture.

## CaptureSession::apply_snapshot
- Folds the records into the tree at `at` (via `apply_snapshot_at`), runs the same
  matching as `on_process_event`.
- A non-service stage binding an already-present process transitions the session
  to `Capturing`.
- Opens no process handle; reads only image name and path from the records.

## ObservationProvider (attach decision)
- `resolve` over a tree built from the startup snapshot returns an `observed`
  `Target` naming the already-running process, or `Unresolved` when none matches.
- The answer is reported (the honest observed stamp); it does not itself acquire.

## xtask gates
- `cargo xtask lint`: no `OpenProcess`/`ReadProcessMemory`/`WriteProcessMemory` in
  the new code.
- `cargo xtask deps`: no new edge; `cargo xtask license`: no new crate.
- `scripts/lint-docs.sh`: the `watch mode` glossary entry resolves; index
  reproduces.
