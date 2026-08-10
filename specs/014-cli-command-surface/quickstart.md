# Quickstart: CLI Command Surface (tier-1 validation)

This is the offline validation path. It proves every command with no capture
driver, no elevation, and no game, so it runs anywhere `cargo test` runs. Live,
socket-table, and ETW behavior compile behind their features and are exercised
only on a developer machine.

## Prerequisites

- The workspace builds: `cargo build --workspace`.
- The offline substrate re-exported by the facade: `ReplaySource` (a recorded
  pcap replayed as a `PacketSource`), `ScriptedAttributor` (attribution from a
  text script), and `ScriptedWatcher` (a scripted process-event timeline).
- Fixture profiles and a recorded capture fixture (reuse the committed corpus).

## Validate the command surface and exit contract

```sh
cargo test -p fragcap-cli --test cli_args
```

Confirms: the value grammars (duration, size, sink spec, profile ref, direction,
roles) parse and reject as specified; the exit-code table holds (bad args and
invalid profile map to 2, and the stub commands to 2); `--help` lists all seven
commands and each stub reports its delivering slice.

## Validate doctor over injected environments

```sh
cargo test -p fragcap-cli --test cli_doctor
```

Confirms: for each constructed `Inputs` (ready; npcap absent; loopback option
absent; WinPcap API mode absent; not elevated; no interfaces), the report
classifies every check, the human rendering matches the golden (including the
"Ready to capture." case), `--json` emits one record per check, every `Fail`
carries a remediation, the two npcap options fail independently, and the exit code
is 0 when capture is possible and 1 when blocked.

## Validate profile management

```sh
cargo test -p fragcap-cli --test cli_profile
```

Confirms: `validate` reports a valid profile with its source and exits 0; an
invalid profile reports every diagnostic in one pass and exits 2; `list` reports
bundled and user counts over a constructed profile directory; `show` reports the
resolved profile and its source, and a well-formed reference that resolves to
nothing exits 1.

## Validate run and tap end to end

```sh
cargo test -p fragcap-cli --test cli_run
```

Confirms, driving the library `run()` entry over `ReplaySource` +
`ScriptedAttributor` + `ScriptedWatcher`:

- `run --profile <ref>` produces a `.fcapng` (and a `.jsonl`) matching a committed
  golden, with each attributed packet carrying the expected role and stage from
  the scripted bindings.
- The `--json` event sequence matches a golden (armed, matched, exited, narrowed,
  complete).
- The completion summary surfaces the discard counters and satisfies the
  conservation identity (received + buffer_dropped + refusals == packets_captured
  for every sink).
- A fired interrupt flag stops the run cleanly and yields exit 0.
- Bounds (duration, packet, byte) each stop the run for the named reason; an
  acquisition timeout with no target yields exit 1.
- `tap --process <name> --duration <dur>` synthesizes a validated one-stage
  profile and captures that process through the same engine and exit contract.

## Full gate

```sh
cargo xtask ci
```

Runs fmt, clippy (`--all-targets --all-features -D warnings`, which compiles the
tier-2 feature paths), the workspace tests, and the repository lint, deps, and
license checks.
