# Tasks: FileVersion stamp and extcap scope flags

**Feature**: 048-extcap-version-and-scope | **Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

All in `fragcap-cli` plus one docs page and the AGENTS.md inventory. No pinned
artifact. Verify with `cargo xtask ci` and `cargo xtask msrv`.

## Phase 1: US1 - FileVersion stamp

- [ ] T001 Add the build-dependency to `crates/fragcap-cli/Cargo.toml`: a new `[target.'cfg(windows)'.build-dependencies]` table with `winresource = { version = "0.1.31", default-features = false }`.
- [ ] T002 Restructure `crates/fragcap-cli/build.rs`: outer guard `target_os == "windows" && target_env == "msvc"`; inside it call `#[cfg(windows)] stamp_version_resource()` unconditionally, then emit the npcap `/DELAYLOAD` linkargs only when `CARGO_FEATURE_LIVE` is set; add `cargo:rerun-if-env-changed=CARGO_PKG_VERSION`.
- [ ] T003 Implement `#[cfg(windows)] fn stamp_version_resource()` in build.rs: parse `CARGO_PKG_VERSION` into the (major,minor,patch) 4-tuple and packed u64; `winresource::WindowsResource::new()`, `set_version_info(FILEVERSION/PRODUCTVERSION, packed)`, `set("FileVersion"/"ProductVersion"/"ProductName"/"FileDescription"/"OriginalFilename"/"LegalCopyright", ...)`, `compile()` with `if let Err(e) => println!("cargo:warning=version resource not stamped: {e}")`.
- [ ] T004 Regenerate and stage `Cargo.lock` (run a plain `cargo build -p fragcap-cli` once) so it carries `winresource` + `version_check`; confirm no `toml` was added and no unrelated churn.
- [ ] T005 [US1] Verify under MSRV: `rustup toolchain install 1.82` then `cargo xtask msrv`. If winresource fails on 1.82, apply the research.md fallback (pin a working winresource patch; else hand-rolled `.rc`) and re-verify. This gates the crate choice.

## Phase 2: US2 - scope flags

- [ ] T006 [US2] In `crates/fragcap-cli/src/cli.rs`, extend `ExtcapInstallArgs` with `--user` and `--system` bool flags in a clap `ArgGroup` (id e.g. `extcap_scope`, `multiple = false`) that also contains `--dir`, so at most one is accepted. Document each flag; default (none) = per-user.
- [ ] T007 [US2] In `crates/fragcap-cli/src/commands/extcap.rs`, make dir resolution scope-aware: change `resolve_dir` (and the `install`/`uninstall` dispatch that calls it) to take the scope, resolving `--dir` -> path, `--system` -> `paths::system_extcap_dir()`, else -> `paths::extcap_dir()`, each `None` mapped to the existing "could not determine ... pass --dir" error. Keep the idempotent self-copy guard and messages unchanged.

## Phase 3: Tests

- [ ] T008 [US2] Extend `crates/fragcap-cli/tests/cli_extcap.rs`: `extcap install --system` registers into the dir named by `FRAGCAP_SYSTEM_EXTCAP_DIR`; `--user` (and no flag) into `FRAGCAP_EXTCAP_DIR`; `extcap install --user --system` (and `--dir` + a scope) exits non-zero (clap conflict); `extcap uninstall --system` removes from the system dir. Keep the existing idempotency/doctor tests green.

## Phase 4: Docs, inventory, changelog

- [ ] T009 In `site/content/docs/reference/cli.mdx` (extcap section), document `--user`/`--system`; present `fragcap extcap install --system` as the machine-wide form and keep `--dir` as the advanced override. Do not change the doctor guidance text.
- [ ] T010 Add the `winresource` row to the `AGENTS.md` dependency-inventory table (kind `build, windows-only`, added by S048), and a short prose note (two packages `winresource` + `version_check`, `default-features = false` to drop `toml`/MSRV-1.85, MIT/Apache-2.0) mirroring the S034/S035 write-ups.
- [ ] T011 Add `changelog.d/048-extcap-version-and-scope.fixed.md` (the exe FileVersion no longer reports 0.0.0.0) and `.added.md` (the `--user`/`--system` scope flags), dash-free, UTF-8/LF.

## Phase 5: Verification

- [ ] T012 Run `cargo xtask ci` in the foreground to green (fmt, clippy, tests, lint, deps, license). Confirm the `doctor-ready` golden is unchanged and the extcap tests pass.
- [ ] T013 Confirm the FileVersion on a Windows release build: `cargo build --release --locked -p fragcap-cli --features live,socket-table,etw` then `(Get-Item target\release\fragcap.exe).VersionInfo.FileVersion` equals the crate version and `--version` is unchanged.
- [ ] T014 Confirm `git diff --stat` touches only `fragcap-cli` (build.rs, Cargo.toml, cli.rs, commands/extcap.rs, tests), `Cargo.lock`, `site/content/docs/reference/cli.mdx`, `AGENTS.md`, the changelog, and `specs/048-...` - no pinned artifact (no workflows/toolchain/release.toml/scripts/wix), no `fragcap-core`.

## Dependencies

- T001 -> T002 -> T003 (dep, then build.rs uses it); T004 after T001-T003; T005 gates the crate.
- T006 -> T007 -> T008.
- T009-T011 after the code; T012-T014 gate the commit.

## MVP

T001-T004 (the stamped FileVersion) is one shippable increment; T006-T008 (scope
flags) is the second. Both are wanted in this slice per the approved plan.
