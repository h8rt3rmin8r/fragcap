# Data Model: Slice 043

No code entities. The "data" is the installer's properties, custom actions, and the
registry search that gate the optional registration.

## Installer properties

| Property | Meaning | Default | Public (settable on the command line) |
| --- | --- | --- | --- |
| `REGISTEREXTCAP_USER` | Register for the installing user | opt-in via checkbox | yes |
| `REGISTEREXTCAP_MACHINE` | Register for all users (needs Wireshark) | off | yes |
| `WIRESHARK_DIR` | Wireshark install directory from a registry search | from registry | resolved, not set by hand |

## Registration actions (per scope, mirroring the Defender pattern)

| Scope | Immediate action | Deferred action | Impersonate | Command line | Return |
| --- | --- | --- | --- | --- | --- |
| Per-user | sets CustomActionData | WixQuietExec | yes | `"[INSTALLDIR]fragcap.exe" extcap install` | ignore |
| Machine-wide | sets CustomActionData | WixQuietExec | no | `"[INSTALLDIR]fragcap.exe" extcap install --dir "[WIRESHARK_DIR]extcap"` | ignore |

Registration is forward-only (revised after PR review): there is no rollback and no
unregister-on-uninstall, because extcap registration is user-managed, idempotent
state and an installer-owned undo would delete a registration the install does not
own. Conditions: per-user runs when `REGISTEREXTCAP_USER=1`; machine-wide runs when
`REGISTEREXTCAP_MACHINE=1 AND WIRESHARK_DIR`. Users unregister with `fragcap extcap
uninstall`.

## Sequencing

All registration actions run after `InstallFiles` (so `fragcap.exe` exists on disk
before it is invoked), matching where the Defender action is sequenced.

## Invariants

- The install succeeds regardless of registration outcome (`Return="ignore"`).
- The extcap command surface is unchanged; only its existing `install` / `uninstall`
  and `--dir` are invoked.
- `doctor` continues to report the true registration state (optional warning when not
  registered), so the docs and the tool agree (P-9).
