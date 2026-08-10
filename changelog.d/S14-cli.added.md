### Added

- **The command surface.** `fragcap` now exposes seven commands. `run` captures
  a game with a profile, `tap` captures a named running process ad hoc, `doctor`
  reports environment readiness, and `profile` validates, lists, and shows
  profiles. `replay`, `steam`, and `extcap` are registered as stubs that name the
  slice delivering them and exit 2, so the help foreshadows the whole tool.
  Specification section 17, the last of the capture-to-file CLI slices (S01
  through S14).
- **The CLI is a library plus a thin binary.** `fragcap_cli::run` is the testable
  entry, so the whole surface, the exit contract, the structured event stream,
  and the completion summary included, is driven from tests without spawning a
  process. The binary is a shim that exits with its code.
- **`run` captures end to end offline.** It resolves a profile, overlays the
  command-line options onto the profile's `[capture]` defaults, arms before the
  target exists, waits for it, captures while it runs, and stops on the first of a
  duration, packet, or byte bound, a terminal stage exit, all targets exiting, or
  an operator interrupt. Each attributed packet carries the role and stage of the
  process that owned its flow. Interrupt handling through `ctrlc` makes an
  operator interrupt a clean exit-0 stop rather than a killed process.
- **`doctor` classifies readiness without ever installing.** A pure `Inputs` to
  `Report` classifier over a thin, read-only probe reports platform, capture
  driver, tracing, interfaces, integration, and profiles, naming the two
  non-default npcap options (loopback capture support and WinPcap API
  compatibility mode) individually with their exact remediations, and exits 1
  only when a blocking problem exists.
- **`profile validate` reports every diagnostic in one pass** and exits 2 on an
  invalid profile; `list` reports the bundled and per-directory counts; `show`
  reports the resolved profile and its source, exiting 1 on a well-formed
  reference that resolves to nothing.
- **The size-literal grammar.** `fragcap-core::size` parses an integer plus a
  required binary unit (`b`, `kb`, `mb`, `gb`), rejecting zero and a missing or
  unknown unit, mirroring the existing duration grammar so `--max-bytes` and the
  ring window (slice S16) share one grammar with a profile.
- **The role-stamping bridge.** `fragcap::session::RoleStampingAttributor`, a
  `FlowAttributor` decorator holding a published `pid` to role and stage snapshot,
  populates the role and stage fields `Attribution` already carries, joining the
  session's profile knowledge to the packet path without either the pipeline or
  the attribution crate learning about profiles.
- **Facade re-exports** of `ScriptedWatcher`, `ProcessScript`, `EtwWatcher`
  (behind `etw`), and the sink crate's JSON string escaper, and the promotion of
  `write_json_string` to `pub`, so the command line reaches the offline substrate
  and hand-rolls its event JSON over the one escaper the sinks use.
- **Two new dependencies**, both on `fragcap-cli` only: `clap` (derive) for the
  argument grammar and `ctrlc` for the interrupt hook.
- **CLI glossary entries.** `docs/glossary.md` gains `Readiness check`,
  `Lifecycle event`, `Completion summary`, and `Effective configuration`
  (constitution P-6).
