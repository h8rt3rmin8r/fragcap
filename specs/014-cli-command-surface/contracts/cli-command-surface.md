# Contract: CLI Command Surface

The public contract this slice establishes: the command grammar, the exit-code
mapping, the event schema, and the library and core surface additions. The
command grammar is fixed by specification section 17; this contract records the
S14 realization and what is deferred.

## Command grammar

```text
fragcap run       Capture using a profile          (implemented)
fragcap tap       Capture a process ad hoc         (implemented)
fragcap doctor    Report environment readiness     (implemented)
fragcap profile   Manage and validate profiles     (implemented)
fragcap replay    Run a capture file back          (stub: exit 2, slice S15/later)
fragcap steam     Enumerate titles, scaffold       (stub: exit 2, slice S17)
fragcap extcap    Analyzer integration             (stub: exit 2, slice S18)
```

`run` accepts every flag in section 17.2. `tap` accepts `--process <NAME>` and
`--duration <DUR>` plus the shared output and verbosity flags. `profile` has
subcommands `validate <REF>`, `list`, `show <REF>`, plus repeatable
`--profile-dir <DIR>`. All commands accept `-h`/`--help`; the tool accepts
`-V`/`--version`.

### Deferred within run (parsed, then rejected with exit 2 naming the slice)

- `--mode stream`, `--mode ring`, `--ring` (S15/S16)
- `--sink pipe:...`, `--sink tcp://...` (S15)
- `--launch` (managed launch, S17)

`--roles` and `--direction` are accepted and validated now: `--roles` scopes which
stages trigger capture; `--direction` is recorded and surfaced, full output
filtering deferred.

## Exit-code mapping

| Outcome | Exit |
| --- | --- |
| Capture completed; diagnostics passed | 0 |
| Operator interrupt during capture | 0 |
| `doctor` found a blocking problem | 1 |
| Target never acquired (acquisition timeout, nothing captured) | 1 |
| Capture driver or usable interface absent | 1 |
| Unrecoverable sink failure ended the run (output may be partial) | 1 |
| `show` reference well-formed but resolves to nothing | 1 |
| Bad arguments / clap parse error | 2 |
| Invalid profile (every diagnostic reported) | 2 |
| Unresolvable profile reference (bad form) | 2 |
| Unsupported mode, sink transport, or `--launch` requested | 2 |
| Not-yet-implemented command (`replay`/`steam`/`extcap`) | 2 |

`Exit` is centralized: each command returns `Result<(), CliError>` and the
library `run()` maps once via `From` impls for `ResolveError`, `LoadError`,
`PipelineError`, and `ConfigError`.

## Structured event schema (`--json`, newline-delimited on stderr)

```json
{"ts":"<RFC3339 Z>","event":"session.armed","interfaces":["<name>", ...]}
{"ts":"<RFC3339 Z>","event":"stage.matched","role":"<role>","pid":<n>,"proc":"<image>"}
{"ts":"<RFC3339 Z>","event":"stage.exited","role":"<role>","pid":<n>}
{"ts":"<RFC3339 Z>","event":"filter.narrowed","endpoints":<n>}
{"ts":"<RFC3339 Z>","event":"session.complete","packets":<n>,"attributed":<n>,"dropped":<n>}
```

`doctor --json` emits one record per check: `{"section","name","detail",
"status","remediation"?}`. Capture data never shares these streams: events and
progress go to stderr, capture data to sinks; when a sink writes to stdout, all
diagnostic output is on stderr.

## Output conventions

- Default: human progress and the completion summary on stderr.
- `--quiet`: suppress progress, keep warnings and errors.
- `--silent`: suppress all but errors.
- Errors are never suppressed.
- `--json`: machine event stream on stderr, independent of quiet/silent.

## doctor readiness contract

Sections and checks: Platform (OS, subsystem, privilege), Capture driver (npcap
present and version, loopback adapter, WinPcap API mode as two separate checks),
Tracing (process-event session), Interfaces (per adapter), Integration (analyzer
extcap), Profiles (bundled and user counts). Exit 0 when capture is possible,
1 when a blocking problem exists; optional-integration warnings never block.
Every `Fail` names a specific remediation. `doctor` only detects the driver and
never installs, downloads, or modifies it; when a non-default npcap option is
absent it is named individually with its exact remediation.

## Library and core surface additions

- `fragcap::session::CaptureSession::role_bindings() -> Vec<(u32,
  Option<Arc<str>>, Option<StageId>)>`
- `fragcap::session::RoleStampingAttributor` implementing `FlowAttributor`
- `fragcap::sink::write_json_string` (re-export of the now-`pub` sink escaper)
- `fragcap-core::size::parse(&str) -> Result<u64, SizeError>` (binary units)
- Facade re-exports: `ScriptedWatcher`, `ProcessScript`, `EtwWatcher` (behind
  `etw`)

`Pipeline`, `PacketSource`, `FlowAttributor`, `ProcessWatcher`, and `Sink` are
unchanged.
