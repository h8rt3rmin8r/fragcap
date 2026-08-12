# Contract: help text and elevation gate

Covers #66, #67 (help text) and #56 (elevation gate).

## Help text (#66, #67)

The command grammar is clap-derive; the user-facing help is the doc comment on
each field/variant. The contract is about what the rendered `--help` MUST and
MUST NOT contain.

### MUST NOT appear in any `--help` output

- Argument-parser implementation notes. Specifically the `--roles` long-help
  paragraph about `value_delimiter` / a custom `value_parser` returning
  `Vec<String>` panicking (`cli.rs` roles field, and the shorter `extcap` copy).
  Relocate that rationale to a `//` source comment on the field.
- Internal roadmap slice identifiers of the form `S` followed by digits
  (S15/S16/S17). Present today on: `Replay` variant ("slice S15"), `ModeArg`
  `Stream`/`Ring`, `RunArgs.ring`, `RunArgs.launch`, `args.rs` `RingWindow`, and
  `commands/stub.rs`.

### MUST appear / be corrected

- A not-yet-available capability reads as "not yet implemented" (or
  "not yet available") with no internal identifier. Example: `Replay` becomes
  "Run a capture file back (not yet implemented)."
- `--roles` help states only behavior: "Comma-separated list of roles that
  scopes which stages trigger" (wording may vary), on both `run` and `extcap`.
- `--launch` help describes the shipped managed-launch behavior rather than
  "deferred to slice S17". Managed launch shipped in S17 and `--launch` is wired
  (Windows-only, via `assemble.rs build_launch`). The corrected copy describes
  launching the title under capture; if the flag has preconditions (Windows,
  a Steam app id in the profile) the help states them without a slice id.

### Acceptance

- `fragcap run --help`, `fragcap extcap --help`, `fragcap --help`, and every
  subcommand help contain no substring matching `S1[0-9]` as a slice id and no
  `value_parser`/`Vec<String>` note. Asserted by a test that captures help output
  (clap can render to a string) and scans it.

## Elevation gate (#56)

### Behavior

- On a command that opens the live capture driver - `run`, `tap`, and
  `extcap` in capture mode - assembly checks elevation via the existing
  current-process-token `is_elevated()` (reused from `doctor/probe.rs`; P-1 safe,
  opens no target handle).
- If not elevated, assembly returns a failure **before** the capture source is
  built / the driver is opened, carrying a message of the form:
  "live capture requires Administrator; re-launch from an elevated terminal
  (right-click > Run as administrator)." Exact wording may vary; it MUST state
  that elevation is required and how to obtain it.
- Exit class: **1** (expected environment-precondition failure), matching the
  existing "no live capture backend" refusal.

### Non-behavior (explicitly excluded)

- No auto-relaunch: the gate MUST NOT invoke `ShellExecuteW "runas"` or spawn a
  separate elevated console. Detect, instruct, refuse only.
- Offline and read-only commands do not check elevation: `replay`, `profile`
  (all subcommands), `steam profile`, `doctor`, and argument validation run
  unelevated. `doctor` continues to *report* elevation as an existing
  Platform-section check; it does not refuse.

### Ordering with the missing-backend refusal

- A binary built without the `live` backend already refuses with "no live
  capture backend" (exit 1). When both conditions hold (no backend AND not
  elevated), the backend-absence refusal is what fires, because there is nothing
  to elevate for. The elevation gate applies only on a build that *has* the live
  backend. Tests cover the featured-build path for the elevation message.

### Placement (P-1 / P-2)

- The check and refusal live in `crates/fragcap-cli/src/assemble.rs` behind
  `#[cfg(windows)]`; the shared elevation predicate is exposed from the
  Windows-only probe path. Nothing enters `fragcap-core`. On non-Windows builds
  the capture path is already absent, so no gate is compiled.
