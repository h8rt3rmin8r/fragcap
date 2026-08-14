# Contract: extcap registration CLI

## `fragcap extcap install [--dir <path>]`

- Copies the running fragcap binary to `<dir>/<EXTCAP_BINARY>`, where `<dir>` is
  `--dir` if given else the per-user Wireshark extcap directory, creating the
  directory if needed.
- Overwrites any existing registration (refresh), so the registered copy matches
  the running binary. Idempotent: exit 0 whether or not a registration existed.
- Prints the destination path to standard output.
- Requires no elevation for the default per-user target.
- On an undetermined target directory, an undetermined binary path, or a write
  failure: prints an error to standard error and exits non-zero (never exit 0).

## `fragcap extcap uninstall [--dir <path>]`

- Removes `<dir>/<EXTCAP_BINARY>` if present; if absent, reports a no-op.
- Idempotent: exit 0 whether or not a registration existed.
- On a write/remove failure against an existing file: error to standard error,
  non-zero exit.

## Preserved: the analyzer protocol (unchanged)

All four invocations behave exactly as before, in BOTH forms:

- `fragcap extcap --extcap-interfaces` and `fragcap --extcap-interfaces`
- `fragcap extcap --extcap-dlts --extcap-interface fragcap` and the bare form
- `fragcap extcap --extcap-config --extcap-interface fragcap` and the bare form
- `fragcap extcap --capture --fifo <path> ...` and the bare `--capture ...` form

The bare top-level forms are routed to the `extcap` command by the existing
`route_extcap` shim; adding the `install`/`uninstall` subcommands does not change
that routing. A parser-regression test asserts all four bare forms still parse and
run.

## Machine-wide registration (administrators)

- The per-user default targets the current user's Wireshark extcap directory.
- To register for every user, an administrator points `--dir` at the system
  Wireshark extcap directory, for example
  `fragcap extcap install --dir "C:\Program Files\Wireshark\extcap"`.

## Installer (deferred to a dedicated slice)

The Windows installer option (an optional, at-install registration with a
per-user note and the run-`fragcap extcap install` guidance, offering both
scopes) is split to a dedicated follow-up slice so it gets a real WiX build and a
multi-user install test. It is not part of this slice.
