# Data Model: extcap registration

This slice adds a CLI subcommand and register/uninstall behavior; there is no
persistent data model beyond a file on disk. The relevant shapes:

## CLI grammar additions (cli.rs)

`ExtcapArgs` gains one field:

| Field | Type | Meaning |
| --- | --- | --- |
| `action` | `Option<ExtcapAction>` | The register/uninstall subcommand, or `None` for the analyzer protocol form |

New types:

```text
enum ExtcapAction {
    Install(ExtcapInstallArgs),
    Uninstall(ExtcapInstallArgs),
}

struct ExtcapInstallArgs {
    dir: Option<PathBuf>,   // --dir override; default is paths::extcap_dir()
}
```

All existing `ExtcapArgs` flags are unchanged. When `action` is `Some`, the
register/uninstall path runs and the protocol flags are ignored; when `None`, the
existing protocol dispatch runs exactly as before.

## Shared constant (paths.rs)

```text
pub const EXTCAP_BINARY: &str = "fragcap.exe";  // Windows
pub const EXTCAP_BINARY: &str = "fragcap";      // other platforms
```

Referenced by both `commands::extcap` (the register target name) and
`doctor::probe` (the presence check), so they agree by construction (R-6).

## Registration operation (commands/extcap.rs)

| Step | Source | Notes |
| --- | --- | --- |
| target dir | `--dir` else `paths::extcap_dir()` | `None` -> error (undetermined location, FR-008) |
| source binary | `std::env::current_exe()` | `Err` -> error (undetermined binary, FR-008) |
| install | `create_dir_all(dir)`, `copy(exe, dir/EXTCAP_BINARY)` | overwrites an existing registration (refresh, FR-002); create/copy failure -> error |
| uninstall | `remove_file(dir/EXTCAP_BINARY)` if present | absent -> success no-op (FR-003) |
| result | destination path (install) or removed/no-op notice (uninstall) to stdout | errors to stderr via the emitter |

## Doctor agreement

No change to the doctor probe logic. After install, `extcap_status()` finds
`EXTCAP_BINARY` in the target directory and the integration check reports
installed; after uninstall it is absent and the check reports not registered
(FR-005). The end-to-end test drives this by pointing both at one directory.
