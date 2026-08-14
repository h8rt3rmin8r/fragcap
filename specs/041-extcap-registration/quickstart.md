# Quickstart / validation: extcap registration

## CI-parity checks (foreground)

```sh
cargo xtask ci

# The register/uninstall integration tests and the protocol parser-regression.
cargo test -p fragcap-cli --test cli_extcap
```

## Integration expectations (in cli_extcap.rs)

- `extcap install --dir <tmp>` creates `<tmp>/fragcap.exe` (or `fragcap` off
  Windows) and exits 0; the destination path is printed.
- `extcap install --dir <tmp>` run twice is exit 0 both times (idempotent).
- `extcap uninstall --dir <tmp>` removes it and exits 0; run again with nothing
  present, still exit 0.
- With `FRAGCAP_EXTCAP_DIR=<tmp>`, `extcap install` then `doctor` shows the
  analyzer extcap check as installed; after `extcap uninstall` it shows not
  registered.
- Parser-regression: `--extcap-interfaces`, `--extcap-dlts`, `--extcap-config`,
  and `--capture --fifo ...` still parse and run in the bare top-level form and
  the explicit `extcap` form (the existing `cli_extcap.rs` tests, plus an
  explicit assertion that the bare forms are unaffected by the new subcommands).

## Manual (pre-push, Windows + Wireshark)

```sh
fragcap extcap install
# then confirm Wireshark's Capture list shows a fragcap extcap source
fragcap doctor            # analyzer extcap: installed in <dir>
fragcap extcap uninstall
fragcap doctor            # analyzer extcap: not registered
```

## Installer (deferred)

The Windows installer option is split to a dedicated follow-up slice (see
research.md D-4); it is not exercised here. The machine-wide registration path an
administrator needs is available now via `--dir`:

```sh
fragcap extcap install --dir "C:\Program Files\Wireshark\extcap"
```
