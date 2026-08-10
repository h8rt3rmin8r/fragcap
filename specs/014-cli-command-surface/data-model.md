# Data Model: CLI Command Surface

The entities this slice introduces or assembles. Most are CLI-local value types;
two are additive surface on existing crates. Nothing here is persisted.

## EffectiveConfig (fragcap-cli)

The capture options actually used, formed by overlaying command-line options onto
a profile's `CaptureDefaults`. Command line wins; an option absent from both stays
absent (the `Option` overlay preserves the declared-versus-absent distinction the
profile schema depends on).

Fields (all resolved from `RunArgs`/`TapArgs` over `Profile.capture()`):

- `mode`: file (implemented) | stream | ring (rejected this slice)
- `out`: optional output path
- `sinks`: list of `SinkSpec`
- `duration`, `acquisition_timeout` (`--wait`): optional `Duration`
- `max_packets`: optional `u64`; `max_bytes`: optional `u64` (from `Size`)
- `roles`: optional set of role names scoping which stages trigger capture
- `direction`: in | out | both (recorded and surfaced; full filtering deferred)
- `interfaces`: list; `loopback`: bool
- `payload`: on | off (from `--no-payload` over the profile)
- `ring`: optional `RingWindow` (duration or size; rejected this slice)
- `launch`: bool (rejected this slice)

Derived: `SessionConfig` (duration, acquisition_timeout, packet_bound,
byte_bound) handed to `CaptureSession`.

## SinkSpec (fragcap-cli)

A parsed `--sink` value (or the `--out`/`--mode` shorthand).

- `File(path)` -> `PcapngWriter` (implemented)
- `JsonLines { path, payload }` -> `JsonLinesWriter` (implemented)
- `Pipe(name)`, `Tcp { host, port, format, payload }` -> parsed, then rejected as
  a configuration error naming slice S15

## Size and RingWindow (fragcap-cli over fragcap-core::size)

- `Size(u64)`: integer plus required unit `b`/`kb`/`mb`/`gb`, binary (1024-based),
  zero rejected. Grammar lives in the new `fragcap-core::size`.
- `RingWindow`: `Duration(Duration)` (via `fragcap-core::duration`) or `Size(u64)`.
  Accepted and validated; rejected as not-yet-supported this slice.

## doctor: Inputs, Status, Check, Report (fragcap-cli)

- `Inputs`: raw environment facts, entirely constructible in a test.
  - `os`, `subsystem` (native | wsl), `privilege` (elevated | not)
  - `npcap: Option<NpcapInfo { version, loopback_adapter: bool, winpcap_api_mode:
    bool }>`
  - `etw_available: Option<bool>` (None when the tracing capability is not built)
  - `interfaces: Vec<IfaceInfo { name, addr: Option<..>, up: bool, virtual: bool }>`
  - `extcap_installed: bool`
  - `bundled_count: usize`, `user_count: usize`
- `Status`: `Ok | Warn | Skip | Fail`
- `Check`: `{ section, name, detail, status, remediation: Option<String> }`;
  a `Fail` always carries a non-empty remediation.
- `Report`: ordered `Vec<Check>`; `Report::exit()` returns `Exit(1)` if any
  `Fail`, else `Exit(0)`. Renders as the aligned columns of section 26.3 (human)
  or one record per check (`--json`).

Check set and classification: platform (OS, subsystem, privilege), capture driver
(npcap present/version, loopback adapter, WinPcap API mode as two separate
checks), tracing (process-event session, per D-f severity rule), interfaces
(per adapter; never fails), integration (analyzer extcap; warn when absent),
profiles (bundled and user counts).

## Event (fragcap-cli)

The lifecycle records emitted on standard error under `--json`.

- `SessionArmed { interfaces: [name] }`
- `StageMatched { role, pid, proc }`
- `StageExited { role, pid }`
- `FilterNarrowed { endpoints }`
- `SessionComplete { packets, attributed, dropped }`

Each carries an RFC3339 `Z` timestamp. Hand-serialized over the sink escaper.

## CompletionSummary (fragcap-cli)

The end-of-run accounting, assembled from `PipelineReport.stats`
(`CaptureStats`) and `CaptureSession::stats()` (`SessionStats`). Surfaces
captured and attributed counts and every existing discard counter
(`watching_discarded`, `discarded_out_of_window`, `buffer_dropped`, per-sink
`sink_dropped`), plus the stop reason. Invents no counter.

## Exit (fragcap-cli)

`Exit(u8)` over the values 0, 1, 2 with `code()`. `From` impls map
`ResolveError`, `LoadError`, `PipelineError`, and `ConfigError` to their class.
Mapping is defined in the contract.

## Additive surface on existing crates

- `CaptureSession::role_bindings() -> Vec<(u32, Option<Arc<str>>,
  Option<StageId>)>` (fragcap facade `session`): the snapshot source for the
  role-stamping decorator.
- `RoleStampingAttributor` (fragcap facade `session`): a `FlowAttributor`
  decorator holding a published binding snapshot.
- `fragcap-sink::json::escape::write_json_string` promoted to `pub` and
  re-exported through the facade `sink` module.
- `fragcap-core::size`: pure binary size-literal grammar.
- Facade re-exports of `ScriptedWatcher`, `ProcessScript`, and (behind `etw`)
  `EtwWatcher` so the CLI can drive a `ProcessWatcher`.
