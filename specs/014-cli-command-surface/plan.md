# Implementation Plan: CLI Command Surface (run, tap, doctor, profile)

**Branch**: `feat/cli-command-surface` | **Date**: 2026-08-10 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/014-cli-command-surface/spec.md`

## Summary

S14 turns the library into a tool. It converts the `fragcap-cli` crate from a
bin-only skeleton into a lib-plus-thin-bin so the whole command surface is
testable without spawning a process, and implements four working commands
(`run`, `tap`, `doctor`, `profile`) plus three registered stubs (`replay`,
`steam`, `extcap`). `run` and `tap` compose the existing pieces into a real
capture: the `Pipeline` owns the packet threads and the shared attributor, and a
new `SessionDriver` thread owns the `CaptureSession`, joined by two seams that
add no new trait and touch no existing one, a `RoleStampingAttributor` decorator
that populates the role and stage fields `Attribution` already carries, and a
`TeeCountingSink` feeding the session so it stays the single authority for the
volume bound and its P-4 counters stay conservation-checked. `doctor` is a pure
`Inputs` to `Report` classifier over a thin platform probe, detect-never-install,
naming the two non-default npcap options individually. The whole slice is
tier-1 testable offline through `ReplaySource` + `ScriptedAttributor` +
`ScriptedWatcher`; the live, socket-table, and ETW paths compile behind their
features and run only on a developer machine. Two dependencies are added, both on
`fragcap-cli` only: `clap` (derive) for the argument grammar and `ctrlc` so an
operator interrupt is a clean stop rather than a killed process.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82

**Primary Dependencies**: two new, both on `fragcap-cli` only (top of the
dependency graph): `clap` 4 with the `derive` feature (the section 17.2 argument
grammar and `-h`/`-V`); `ctrlc` (a portable console-interrupt hook so an operator
interrupt becomes `StopReason::Interrupt`, exit 0). `serde_json` stays
dev-only; the `--json` events are hand-rolled over the sink's JSON string
escaper. The user profile directory is read from `%APPDATA%` via `std::env`, no
crate. Terminal detection uses `std::io::IsTerminal` (stable in 1.82), no crate.
An additive, pure `fragcap-core::size` grammar module (std only) mirrors the
existing `duration` module for `--max-bytes` and the ring size form.

**Storage**: N/A (the tool reads profiles and writes capture files through the
existing sinks; it introduces no store of its own).

**Testing**: `cargo test` at tier 1, in `crates/fragcap-cli/tests/`. `run`/`tap`
end to end over `ReplaySource` + `ScriptedAttributor` + `ScriptedWatcher` (all
facade-re-exported, so the CLI reaches them with no sibling dev-dependency and no
P-3 violation); `doctor` over hand-built `Inputs` with text and JSON goldens;
`profile` over fixture profiles and temp directories; argument grammar and the
exit-code table by driving the library `run()` entry directly. No capture driver,
no elevation, no game. Tier-2 (`live`, `socket-table`, `etw`) compiles under the
`--all-features` clippy gate and is `#[ignore]`d so the runner skips it.

**Target Platform**: the tool is Windows-facing (capture and `doctor` probes are
`cfg(windows)`), but the orchestration decisions, the `doctor` classification and
rendering, the argument grammar, the event emission, and the exit mapping are all
platform-neutral and exercised on any target. `fragcap-core` stays neutral: the
only core change is the optional pure `size` module.

**Project Type**: Rust library workspace with one binary crate (the library is
the product; the CLI is one consumer of it).

**Performance Goals**: not a hot path. The `SessionDriver` selects over an event
channel, a retained-packet channel, and a ~50ms tick; the pipeline's per-packet
path is unchanged. No per-packet allocation is added on the acquisition side.

**Constraints**: `fragcap-core` stays platform-neutral and its allowlist stays
`["bytes"]` (P-2); the two new deps sit on the binary crate only and nothing
depends on it (P-3); the session and pipeline compose without either seam trait
naming the other (P-3); every discard the tool reports is an existing named
counter, and no count is fabricated (P-4, P-9); output files stay readable by
unmodified analyzers (P-5); new CLI vocabulary gets glossary entries (P-6); text
hygiene, no em/en dashes, UTF-8 no BOM LF (P-8).

**Scale/Scope**: seven registered commands, four implemented; one capture engine
shared by `run` and `tap`; a `doctor` report of roughly a dozen checks; five
lifecycle event kinds.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive Observation (NON-NEGOTIABLE)**: the CLI composes only the permitted
  seams (section 19.2). It opens no process handle, injects nothing, hooks
  nothing, and installs no driver. `doctor` reads npcap presence and options from
  read-only registry, service, and adapter queries and never installs, downloads,
  or modifies the driver (the Licensing section's detection-not-installation rule).
  `tap` names a process to attribute, it does not open it. PASS.
- **P-2 Core Stays Platform-Neutral**: the only `fragcap-core` change is an
  additive pure `size` grammar module (std only); the allowlist stays `["bytes"]`.
  `clap` and `ctrlc` are declared on `fragcap-cli` alone. `cargo xtask neutral`
  still builds core for a backend-free target. PASS.
- **P-3 Capture And Attribution Stay Separate**: `RoleStampingAttributor` is a
  `FlowAttributor` decorator that does no packet acquisition; it wraps the inner
  attributor and populates the role and stage fields already on `Attribution`. The
  session and pipeline compose through a `StopHandle` and a published binding
  snapshot; neither `PacketSource` nor `FlowAttributor` names the other and neither
  gains a bound. Dependencies flow concrete toward abstract; nothing depends on
  `fragcap-cli`. PASS.
- **P-4 No Silent Loss**: the completion summary surfaces the existing named
  counters (`CaptureStats.buffer_dropped`, per-sink `sink_dropped`, and
  `SessionStats.watching_discarded` / `discarded_out_of_window`); the
  `TeeCountingSink` is an ordinary sink inside the pipeline's conservation
  identity, so nothing it counts escapes accounting. The CLI adds no new discard
  path and invents no counter. PASS.
- **P-5 Compatibility Outranks Richness**: `run` writes through the existing
  `PcapngWriter` and `JsonLinesWriter`; the output file remains a `.fcapng` an
  unmodified analyzer opens. The CLI adds no format. PASS.
- **P-6 Glossary First**: any new operator-facing term this slice introduces
  (for example the readiness report's status vocabulary, the lifecycle event
  names) gets a glossary entry in this change. PASS.
- **P-7 Wrappers Stay Thin**: this slice provides the machine-readable event
  stream (section 17.5) and the `doctor --json` records that let the S18 wrappers
  stay thin; it adds no wrapper and no capability a wrapper would otherwise parse
  human output for. PASS.
- **P-8 House Standards Apply**: all new files are UTF-8 without BOM, LF, no
  trailing whitespace, no em/en dashes; Rust passes `fmt` and `clippy -D
  warnings`. PASS.
- **P-9 The Instrument Does Not Lie (NON-NEGOTIABLE)**: the CLI alters no
  observation. Role and stage stamping annotates attribution metadata the writer
  already emits; it masks, drops, and reorders nothing. Every count reported is
  one the pipeline or session observed. PASS.

No violations; Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/014-cli-command-surface/
├── plan.md
├── spec.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── cli-command-surface.md
├── checklists/
│   ├── requirements.md
│   └── cli-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-cli/Cargo.toml            # + [lib]; + clap (derive), ctrlc; dev: serde_json, tempfile
crates/fragcap-cli/src/main.rs           # shrink to a shim: process::exit(fragcap_cli::run(env::args_os()).code())
crates/fragcap-cli/src/lib.rs            # pub fn run<I>(args) -> Exit; clap parse + dispatch (the testable entry)
crates/fragcap-cli/src/cli.rs            # clap derive: Cli, Command enum, per-command Args structs
crates/fragcap-cli/src/args.rs           # value parsers: Dur (delegates to core duration), Size (core size),
                                         #   SinkSpec, ProfileRef, Direction, Roles, RingWindow
crates/fragcap-cli/src/exit.rs           # Exit(0/1/2); From<ResolveError/LoadError/PipelineError/ConfigError>
crates/fragcap-cli/src/emit.rs           # Emitter: Human (stderr, quiet/silent) vs Json (NDJSON stderr)
crates/fragcap-cli/src/events.rs         # Event enum + hand-rolled NDJSON writer over the sink escaper
crates/fragcap-cli/src/output.rs         # human progress + completion summary rendering, stream routing
crates/fragcap-cli/src/orchestrator.rs   # SessionDriver + capture engine shared by run and tap
crates/fragcap-cli/src/assemble.rs       # build sources / attributor / sinks / watcher from EffectiveConfig
crates/fragcap-cli/src/paths.rs          # user profile dir from %APPDATA%; --profile-dir search paths
crates/fragcap-cli/src/commands/mod.rs
crates/fragcap-cli/src/commands/run.rs   # RunArgs -> EffectiveConfig -> orchestrator::capture
crates/fragcap-cli/src/commands/tap.rs   # TapArgs -> synthesized one-stage Profile (via load) -> capture
crates/fragcap-cli/src/commands/doctor.rs# probe::gather -> checks::run -> render -> exit
crates/fragcap-cli/src/commands/profile.rs# validate / list / show over resolve/load/Diagnostics
crates/fragcap-cli/src/commands/stub.rs  # replay/steam/extcap: "not yet implemented (slice SNN)"; exit 2
crates/fragcap-cli/src/doctor/mod.rs     # Inputs, Status, Check, Report (pure); Report::exit
crates/fragcap-cli/src/doctor/checks.rs  # pure fn(&Inputs) -> Check per readiness check
crates/fragcap-cli/src/doctor/probe.rs   # cfg(windows)/feature-gated real-input gathering (not unit-tested)
crates/fragcap-cli/tests/cli_args.rs     # value grammars + exit-code table
crates/fragcap-cli/tests/cli_doctor.rs   # injected Inputs -> text + json goldens + exit 0/1
crates/fragcap-cli/tests/cli_profile.rs  # validate/list/show over fixtures
crates/fragcap-cli/tests/cli_run.rs      # run/tap end-to-end offline; goldens; interrupt -> 0
crates/fragcap/src/session.rs            # + CaptureSession::role_bindings(); + RoleStampingAttributor
crates/fragcap/src/lib.rs                # re-export ScriptedWatcher, ProcessScript, EtwWatcher(etw); JSON escaper
crates/fragcap-sink/src/json/escape.rs   # make write_json_string pub
crates/fragcap-core/src/size.rs          # new: pure binary size-literal grammar (mirrors duration)
crates/fragcap-core/src/lib.rs           # + pub mod size
docs/glossary.md                         # + readiness status vocabulary, lifecycle event names (P-6)
changelog.d/S14-cli.added.md
changelog.d/S14-cli.decisions.md
```

**Structure Decision**: the CLI crate becomes a library with a thin binary so the
whole surface is driven from `run()` in tests, never by spawning a process (which
would need a process-runner dev-dependency and hide the offline seams). The
capture wiring lives entirely in `fragcap-cli` and the facade `session` module,
which is the one place already above both `fragcap-capture` and `fragcap-attr`
(decision D-1), so the `RoleStampingAttributor` bridge and the re-exports satisfy
P-3 without any sibling edge. The only `fragcap-core` change is the additive
`size` module, kept in core (not the CLI) so the S16 ring slice reuses it beside
`duration`. `Pipeline` and the four seam traits are untouched.

## Phase 0: Research

See [research.md](research.md). Decisions D-a through D-h record the argument
parser choice and placement, the interrupt-handling crate, the session-pipeline
composition (why they run side by side rather than nested), the role-stamping
bridge and its home, the volume-bound tee, the `doctor` pure-probe split and the
tracing-severity and npcap-option rules, the hand-rolled event JSON and stream
routing, and the size-literal grammar home and base.

## Phase 1: Design

See [data-model.md](data-model.md) for the entities (effective configuration,
`doctor` report and checks, lifecycle events, completion summary, exit contract)
and [contracts/cli-command-surface.md](contracts/cli-command-surface.md) for the
command grammar, exit-code mapping, event schema, and the facade and core surface
additions. [quickstart.md](quickstart.md) shows the tier-1 validation path.

## Complexity Tracking

No constitution violations; no entries.
