# Phase 1 Quickstart: validating S039

Two tiers, mirroring the project's honesty posture. Tier 1 is fully automated and
offline. Tier 2 is the manual MSI checklist, recorded in the PR, because the
installed-installer runtime cannot be exercised by the check set (the same reason
live capture is manually verified).

## Tier 1 - automated, offline

Run from the repo root.

1. **Full gate**:
   ```sh
   cargo xtask ci
   cargo xtask neutral
   cargo xtask msrv        # 1.82
   ```
   Expected: all green. `cargo xtask deps` (inside ci) shows no new edge;
   `Cargo.lock` is unchanged (SC-007). The docs check fails if
   `docs/glossary/index.md` was not regenerated after adding the new terms.

2. **Default-path resolution** (unit tests in `paths.rs`):
   - With `APPDATA` set and no flag/env, `default_hint_db_path()` returns
     `<APPDATA>\fragcap\hint.db`.
   - With `APPDATA` unset, it returns `None`.

3. **First-run bootstrap** (unit tests in `run.rs`, tempdirs):
   - default absent, no template -> an empty current-schema store is created at
     the default path.
   - default absent, template present -> the template is copied to the default
     path.
   - default present -> the existing file is left byte-for-byte unchanged.

4. **Barebones database round-trip**:
   ```sh
   cargo run -p fragcap-cli -- targets import assets/hint-seed.json --db /tmp/hint.db
   cargo run -p fragcap-cli -- targets export --db /tmp/hint.db
   ```
   Expected: import exits 0 and produces a valid v2 store; export prints a valid,
   empty `kind:"export"` document (SC-003).

5. **Installer authoring build** (where WiX is available):
   ```sh
   cargo wix --nocapture     # or the project's cargo-wix invocation
   ```
   Expected: `main.wxs` compiles and links to an `.msi`. This is the same command
   the release job runs at tag time.

## Tier 2 - manual MSI verification (record results in the PR)

On a clean Windows machine, from the built `.msi`:

- [ ] Running the unsigned installer shows the platform's unrecognized-publisher /
  SmartScreen warning; proceeding as documented installs it.
- [ ] After a per-machine install, `fragcap` resolves in a newly opened terminal
  (system PATH updated).
- [ ] `Program Files\fragcap\` contains `fragcap.exe`, `hint.db`, `LICENSE`,
  `NOTICE`.
- [ ] First `fragcap run` bootstraps `%APPDATA%\fragcap\hint.db` from the template
  and local accumulation writes there.
- [ ] `Get-MpPreference` shows the install directory excluded after install; the
  install still completed even where the exclusion was refused (tamper
  protection).
- [ ] The installer exit dialog offers and opens the npcap download page.
- [ ] Add/Remove Programs lists fragcap with the About URL.
- [ ] Uninstall removes the files, the PATH entry, and the Defender exclusion.
- [ ] (Deferred to v0.3.1+) A major upgrade over a prior version replaces rather
  than duplicates the install.

## Release-time verification (at the v0.3.0 tag)

- [ ] The `artifacts` job installs WiX + cargo-wix, builds `hint.db` via
  `targets import`, and builds the `.msi`.
- [ ] The release exposes the portable zip (with `hint.db`), the `.msi`, and a
  loose `hint.db`, each with a `.sha256`.
- [ ] The optional `msiexec /qn` install+uninstall smoke step passes (it does not
  assert Defender state).
