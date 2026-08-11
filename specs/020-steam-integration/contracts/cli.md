# Contract: CLI surfaces

## `fragcap steam profile <app_id>`

Replaces the current `Steam(StubArgs)` stub.

**Input**: `app_id` (positional, string).

**Behavior**: discover installed titles; find `app_id`; scan its install directory;
classify executables; render a profile skeleton; validate it through `Profile::parse`;
print it to **stdout**.

**Success**: exit 0; a valid profile on stdout, led by a header comment stating the
classification is heuristic and must be verified against an observed session. The profile
declares `game.platform = "steam"` and `game.app_id = "<app_id>"` and at least one client
stage.

**Errors** (exit non-zero, message to stderr, nothing on stdout):

- app_id not installed -> names the app_id (FR-009).
- Steam not installed -> "no Steam installation found".
- non-Windows build -> "Steam integration is only supported on Windows".

**Guarantee**: the emitted profile passes section 15.4 validation unedited (FR-008/SC-001).

## `fragcap run --profile <ref> --launch`

Replaces the `assemble.rs` refusal `"managed launch (--launch) is not yet supported
(slice S17)"`.

**Preconditions** (validated in run assembly, before capture starts, FR-011):

- the loaded profile declares `game.platform = "steam"`, else usage error naming the
  missing key;
- the loaded profile declares `game.app_id`, else usage error naming the missing key;
- on a non-Windows build, `--launch` is refused as unsupported.

**Behavior**: assembles a `LaunchRequest { url = steam://run/<app_id> }`. During the run,
after the session reaches `Watching` (watcher attached) and the sinks/capture handle are
open, the title is started through the Steam protocol handler (FR-010).

**Verification**: the config validation, the URL, and the ordering (launch after arm) are
unit-tested offline. The physical launch is tier-2/manual and is not asserted as run in CI
(D5, P-9).

**Unchanged**: the seven-command surface; `--launch` without `--profile` still errors as
before; all other run flags behave as in S14/S16.
