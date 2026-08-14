# Phase 0 Research: FileVersion stamp and extcap scope flags

## Decision 1: the version-resource crate (MSRV is the crux)

**Decision**: `winresource = { version = "0.1.31", default-features = false }` under
`[target.'cfg(windows)'.build-dependencies]` in `crates/fragcap-cli/Cargo.toml`.

**Rationale** (crates.io metadata, Aug 2026):

- `winresource 0.1.31`: license MIT, no declared `rust-version`. Its `toml ^1`
  dependency is gated behind the default `toml` feature only; `default-features =
  false` drops it. What remains is `version_check` (MIT/Apache-2.0, no deps, no
  MSRV). So the delta to `Cargo.lock` is exactly two packages (`winresource`,
  `version_check`), both allowlisted, and `toml` (which declares Rust 1.85 and
  would break the 1.82 gate, exactly as slice S05 documented) is NOT pulled. We set
  every VERSIONINFO field explicitly through the API, so the `toml` auto-read
  feature is unnecessary.
- `embed-resource 3.x`: rejected. Its `toml ^1` is a normal, non-optional
  dependency (no feature switch), so it re-introduces the MSRV-1.85 break, plus a
  larger graph (`cc`, `rustc_version`, `memchr`, `vswhom`, `winreg`).
- Hand-rolled `.rc` + `rc.exe`: feasible and dependency-free, but SDK/`rc.exe`
  discovery is the fragile part winresource already solves. Kept as the fallback.

**Residual risk and fallback (operator: "pick the MSRV-safe crate")**: winresource
has no declared MSRV, so it must be proven under 1.82 empirically
(`cargo xtask msrv`, after `rustup toolchain install 1.82`). If it fails: first pin
the highest winresource patch that builds on 1.82 (the `set`/`set_version_info`/
`compile` API is stable across 0.1.x); only if none builds, hand-roll the `.rc`.

## Decision 2: build.rs guard restructure

**Decision**: Split the current single `live && windows && msvc` guard. The version
resource stamps on every Windows-MSVC build (independent of `live`); the npcap
`/DELAYLOAD` linkargs stay behind `CARGO_FEATURE_LIVE`.

**Rationale**: FileVersion should be correct on any Windows build of the exe, not
only `live` ones. `stamp_version_resource()` is `#[cfg(windows)]` so it references
`winresource` only where the `cfg(windows)` build-dep is present; on non-Windows the
whole block is skipped and `build.rs` is a no-op (it already compiles/runs on all
hosts). `cargo xtask neutral` builds core/capture/attr for Linux and does not build
fragcap-cli, so it never exercises this. The version 4-tuple is parsed from
`CARGO_PKG_VERSION` ("0.3.0" -> 0.3.0.0), packed into the VS_FIXEDFILEINFO u64.
`res.compile()` failure becomes a `cargo:warning`, not a panic, so a dev box without
`rc.exe` still links (unstamped).

## Decision 3: scope-flag plumbing (reuse S044 resolvers)

**Decision**: `ExtcapInstallArgs` gains `--user` and `--system` bool flags in a clap
`ArgGroup` with the existing `--dir`, `multiple = false` so at most one is accepted.
`resolve_dir` becomes scope-aware: `--dir` -> that path; `--system` ->
`paths::system_extcap_dir()`; else -> `paths::extcap_dir()`; each `None` mapped to
the current "could not determine ... pass --dir" error.

**Rationale**: `paths::extcap_dir()` (per-user), `paths::system_extcap_dir()`
(machine-wide, `%ProgramFiles%\Wireshark\extcap`, env override
`FRAGCAP_SYSTEM_EXTCAP_DIR`), and `EXTCAP_BINARY` all already exist (S044), so no new
path logic is written; the flags are a thin selector over them. `install`/`uninstall`
share `ExtcapInstallArgs`, so scope applies symmetrically. Default (no flag) stays
per-user, byte-for-byte today's behavior. The MSI keeps `--dir "[WIRESHARK_DIR]\extcap"`
(WiX already resolves and gates on the dir), so no pinned main.wxs change.

## Single-sourcing and release interaction

`CARGO_PKG_VERSION` derives from `[workspace.package] version`; `scripts/New-Release.ps1`
bumps exactly that field, so the stamped FileVersion tracks releases with no
release-script change, and it is the same source `fragcap --version` (clap default)
uses, so the two cannot disagree. The MSI ProductVersion continues to come from
cargo-wix `--install-version` (git tag) and does not read the PE resource, so it is
untouched.
