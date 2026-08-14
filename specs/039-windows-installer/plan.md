# Implementation Plan: Windows installer (MSI) and hint-database default with first-run bootstrap

**Branch**: `039-windows-installer` | **Date**: 2026-08-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/039-windows-installer/spec.md`

## Summary

fragcap becomes installable and hint-ready out of the box. Three things ship
together and reinforce each other: the hint database gains a per-user default
location (`%APPDATA%\fragcap\hint.db`, a sibling of the existing profile
directory) that a first-run bootstrap creates when absent; a barebones, empty
current-schema hint database is produced offline from a committed empty seed and
shipped; and an unsigned Windows installer (MSI) places the program plus that
database template, puts fragcap on the system path, best-effort excludes its own
install directory from Windows Defender, and points the user at the separately
required capture driver. The release then offers three downloads, each with a
checksum: a portable archive, the installer, and the loose database. The binary
change is small and standard-library only; the installer is new authoring; the
release workflow (a pinned artifact) gains the steps that build and checksum the
new outputs. Everything the automated gate cannot exercise (the installed-MSI
runtime experience) is documented as manual verification, the same posture the
project already holds for live capture.

## Technical Context

**Language/Version**: Rust, workspace edition 2021, MSRV 1.82 (must stay green).
Installer authoring is WiX v3 XML driven by `cargo-wix`; both are build-time
tooling on the release runner, not workspace dependencies.

**Primary Dependencies**: none new in the workspace. The bootstrap uses only the
standard library (`std::env::current_exe`, `std::fs`). `fragcap-targets`'
`Store::open` (existing) creates the empty schema. `cargo-wix` and WiX v3 are
release-runner tools; they add nothing to `Cargo.lock`.

**Storage**: the existing embedded SQLite hint store (`fragcap-targets`). No
schema change this slice (the store is already at v2 from S038). The barebones
database is that schema with no rows.

**Testing**: `cargo test --workspace --locked`. The binary bootstrap is tested
offline with tempdirs (default-path resolution with the environment set and
unset; bootstrap absent-to-empty, absent-plus-template-to-copied, and
present-untouched). The barebones seed is validated by the existing offline
`targets import` and `export` round-trip. The installer's runtime behavior is
manual-verify (see quickstart.md); `cargo wix` authoring can be built where WiX
is available and is exercised for real at release-tag time.

**Target Platform**: Windows for the installer and the `%APPDATA%` default; the
bootstrap helper is a pure function over paths and builds and tests on the
neutral non-Windows target (P-2). The default-path resolver mirrors the existing
profile-directory resolver, which is already Windows-shaped via `APPDATA`.

**Project Type**: Rust workspace (library crates plus a CLI facade) plus release
packaging and documentation.

**Performance Goals**: not a performance slice. The bootstrap runs once (first
run only); later runs find the file present and no-op. Copying an empty database
template is a few kilobytes.

**Constraints**: no new workspace dependency (FR-026); MSRV 1.82 green; no
`Cargo.toml` version bump (FR-027); P-1 absolute, no process handle (FR-024);
capture driver never bundled or downloaded, only linked (FR-025); UTF-8 no BOM,
LF, no em or en dashes including the `.wxs` and Rust comments (FR-028); the
pinned release workflow change carries a dated decision (FR-029).

**Scale/Scope**: one small binary helper plus its tests, one committed seed file,
one installer definition, a bounded set of release-workflow steps, and the
docs/glossary/spec edits. No new crate.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive Observation Only**: Satisfied. The bootstrap reads and writes only
  fragcap's own per-user database file and copies a template fragcap ships; it
  opens no process handle and touches no target. The installer's Windows Defender
  exclusion is an OS-configuration action scoped to fragcap's own install
  directory, performed with the rights the installer already holds; it does not
  read, modify, or attach to any process, its memory, its traffic, or the network
  stack, so it is outside the technique denylist. The `OpenProcess` /
  `ReadProcessMemory` / `WriteProcessMemory` lint stays clean. FR-024 makes this
  explicit.
- **P-2 Core Stays Platform-Neutral**: Satisfied. `fragcap-core` is untouched. The
  bootstrap helper and default-path resolver live in `fragcap-cli` (already the
  platform-facing crate) and are pure over paths; the pure helper builds and tests
  on the neutral target. `cargo xtask deps` is unaffected (no new edge).
- **P-3 Capture And Attribution Stay Separate**: Not engaged. No capture or
  attribution logic changes; the bootstrap only ensures the hint store exists
  before the existing resolution and accumulation run.
- **P-4 No Silent Loss**: Satisfied. A bootstrap failure warns rather than
  proceeding silently; the release checksum step covers every artifact so no
  download ships unverifiable; the installer surfaces the capture-driver
  prerequisite rather than leaving live capture to fail opaquely.
- **P-5 Compatibility Outranks Richness**: Not engaged for output formats. The
  distribution change is additive: the portable archive still exists (now carrying
  the database), and unmodified analyzers are unaffected.
- **P-6 Glossary First**: New terms (MSI installer, unsigned installer, Windows
  Defender exclusion) get glossary entries in this change, and the generated index
  is regenerated (FR-021).
- **P-7 Wrappers Stay Thin**: Not engaged. No shell wrapper changes; the installer
  is declarative WiX authoring, not a wrapper that parses output. The Defender
  custom action invokes a single platform cmdlet with no output parsing.
- **P-8 House Standards Apply**: Satisfied. SPDX where the project requires it,
  UTF-8 no BOM, LF, no em or en dashes across all new files including `main.wxs`
  and Rust comments; `cargo fmt`, `clippy -D warnings`, the repository linter, and
  the documentation gate all apply (FR-028).
- **P-9 The Instrument Does Not Lie**: Satisfied and central. The bundled database
  is empty rather than seeded with unverified titles that would go stale; the
  unsigned installer is documented as unsigned with the checksum as the integrity
  check rather than implying a signature; the capture driver is linked, never
  bundled; and the installed-MSI runtime behavior is documented as manual-verify
  rather than asserted as tested.

**Licensing**: no new workspace dependency, so `Cargo.lock` is unchanged
(SC-007) and no per-crate license surface changes. `cargo-wix` and WiX v3 are
build-runner tools (both permissively licensed) and enter no artifact. The
capture driver is not bundled; its download page is only linked.

**Pinned artifacts**: `.github/workflows/release.yml` is pinned and is modified
by this slice; a dated `changelog.d/039-windows-installer.decisions.md` records
that change, the archive-contract amendment, the frozen UpgradeCode, the
unsigned-by-design posture, the best-effort Defender exclusion, and the
default-on accumulation. Surfaced at the pre-push halt (FR-029).

**Result**: PASS. No violations; Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/039-windows-installer/
├── plan.md              # This file
├── research.md          # Phase 0: installer toolchain choice, DB-writability reconciliation, Defender CA, rejected alternatives
├── data-model.md        # Phase 1: entities (default path, template, barebones DB, installer, artifact set)
├── quickstart.md        # Phase 1: offline validation + the manual MSI checklist
├── contracts/
│   └── seams.md         # Phase 1: the binary, seed, installer, and release contracts
├── checklists/
│   ├── requirements.md  # from /speckit-specify
│   └── distribution.md  # from /speckit-checklist
└── tasks.md             # from /speckit-tasks (not created here)
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/
├── paths.rs             # NEW default_hint_db_path() mirroring user_profile_dir; hint_db_path unchanged
├── commands/run.rs      # NEW ensure_default_hint_db(default, template) pure helper; default layered on with provenance; called run-path only
└── cli.rs               # --hint-db doc updated (new default; override semantics)

assets/
└── hint-seed.json       # NEW kind:"export" document with empty records (the barebones DB source)

wix/
├── main.wxs             # NEW WiX v3 installer definition
└── License.rtf          # NEW license shown by WixUI_InstallDir (from LICENSE)

.github/workflows/
└── release.yml          # MODIFIED (pinned): install WiX+cargo-wix; build hint.db; build MSI; ship zip+loose DB; broaden checksums

README.md                # MODIFIED: Windows installer section + quick-links row
site/content/docs/getting-started.mdx  # MODIFIED: install step + SmartScreen note
docs/glossary/platform-and-distribution.md  # MODIFIED: three new terms
docs/glossary/index.md   # REGENERATED via scripts/lint-docs.sh fix
docs/fragcap-specification.md            # MODIFIED: 24.5, 20.2, 15.3

changelog.d/
├── 039-windows-installer.added.md      # NEW
├── 039-windows-installer.changed.md    # NEW
└── 039-windows-installer.decisions.md  # NEW (dated)
```

**Structure Decision**: The binary change lives entirely in `fragcap-cli`, the
crate that already owns path resolution and the `run` command, so no crate
boundary moves and `cargo xtask deps` is unchanged. The default-path resolver is
a sibling of the existing `user_profile_dir`, and the bootstrap is a pure helper
(tempdir-tested) plus a thin run-path caller, keeping the platform-neutral build
green. The barebones database needs no new Rust: it is the existing offline
`targets import` applied to a committed empty seed. The installer is isolated in a
new top-level `wix/` directory. The only pinned-artifact edit is `release.yml`,
carried by a dated decision fragment.

## Notable design decisions

- **DB-writability reconciliation (copy-template-else-empty).** The installer must
  land the database where the user can write it at runtime (S038 accumulation
  writes on `run`), but a per-machine install puts program files under a
  directory a standard user cannot write. Chosen: ship the database as a
  read-only template beside the executable and have the binary bootstrap the
  writable per-user default on first run (copy the template when present, else
  create an empty store). One code path serves the installer and the portable
  archive; a future non-empty template needs no code change. Rejected: a WiX
  per-user component writing directly into `%APPDATA%` (installer-only, leaves
  raw-exe users unserved, and per-machine-to-per-user seeding via an HKCU keypath
  is fragile).
- **Bootstrap runs on the `run` path only.** Hint resolution and accumulation are
  already wired there; `watch`/`tap` are unchanged this slice (recorded in the
  spec clarifications). A watch-first user simply gets no default database until a
  `run`, which is acceptable and honest.
- **Barebones database is empty.** Operator decision, P-9: shipping specific
  unverified titles would bake in staleness; the substrate grows from the user's
  own machine (S038) and future community sync (#94), and the full curated corpus
  stays the out-of-band maintainer artifact.
- **Installer toolchain: cargo-wix + WiX v3.** Mature, single `main.wxs`, derives
  `ProductVersion` from the crate version at tag time. WiX v4/v5 dotnet-tool was
  the alternative; v3 via cargo-wix is the lower-friction, better-documented path
  for a Rust binary. See research.md.
- **Defender exclusion is best-effort (Return=ignore).** Tamper Protection or a
  disabled Defender can refuse `Add-MpPreference` even when elevated; a refusal
  must not fail the install. Documented as best-effort.
- **Unsigned by design.** Signing is issue #79 (non-blocking, out of scope); the
  checksum is the integrity mechanism and the docs explain the unrecognized-
  publisher warning.

These are recorded in `changelog.d/039-windows-installer.decisions.md` at
implementation, with the pinned-artifact and default-accumulation entries dated.

## Complexity Tracking

> No Constitution Check violations. This section is intentionally empty.
