# Implementation Plan: Non-Profile Production Capture Path

**Branch**: `feat/non-profile-capture` | **Date**: 2026-08-13 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from `specs/032-non-profile-capture/spec.md`

## Summary

Let `run` capture a target the cascade resolves without a profile. Add two
mutually-exclusive inputs beside `--profile`: `--install-dir <path>` (run the
cascade over a given install directory) and `--steam <app_id>` (resolve the app
id to its install directory through the existing Steam library lookup, then take
the same path). When the resolved target has no backing profile, read its
`MatchPredicates` identity, synthesize a one-stage profile stamped
`heuristic-unverified` (the same shape `watch` builds, with an honest fidelity),
and hand it to the shared capture engine. An install location the cascade
declines (unrecognized, ambiguous, unreadable, or a not-installed app id) is a
surfaced failure that captures nothing. The `run --profile` path is untouched and
byte-identical.

## Technical Context

**Language/Version**: Rust, edition 2021, MSRV 1.82.

**Primary Dependencies**: none new. Reuses `clap` (arg group), the S027 resolver
and its providers (`fragcap-profile`), the Steam library lookup
(`fragcap-steam::install_root_for`/`install_root_in`), `serde_json` (identity to
profile JSON, already used by `watch`), and the shared orchestrator/offline
harness.

**Storage**: none.

**Testing**: `cargo test --workspace --locked`. The offline capture harness
(`OfflineArgs` + a process script, as `run`/`watch`/`tap` tests already use)
drives the non-profile capture end to end with no game, driver, or Steam install.
Engine-rule/walker resolution is exercised with fixture install directories.

**Target Platform**: Windows primary; the added logic is CLI wiring and is
platform-neutral.

**Project Type**: CLI over a layered crate workspace.

**Performance Goals**: none specific; the added path is one resolution plus the
existing capture.

**Constraints**: P-1 (no process handle/memory; reuse the launch-agnostic
engine), P-4 (surfaced declines), P-9 (honest `heuristic-unverified` fidelity),
P-6 (glossary + spec), byte-identical `--profile` path, no new dependency, MSRV
1.82 green.

**Scale/Scope**: One command's argument surface and its resolve-then-capture
branch, plus tests. No new crate, no new public type beyond a small helper if
needed.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive Observation Only (NON-NEGOTIABLE)**: PASS. The non-profile path
  reuses `orchestrator::capture`, the same launch-agnostic engine `watch` uses:
  the session arms, folds a query-only startup snapshot (attach-to-running), and
  attributes from outside the process by socket table and ETW. No process handle
  is opened, no process memory is read, nothing on the denylist is touched.
  Resolution reads the filesystem (engine rule, walker) and the Steam library
  manifests, exactly as those providers already do.
- **P-2 Core Stays Platform-Neutral**: PASS. All changes are in `fragcap-cli`;
  `fragcap-core`'s allowlist is untouched. No crate below the facade gains a
  dependency.
- **P-3 Capture And Attribution Stay Separate**: PASS. No packet source or
  attributor is changed; the slice composes existing resolution and capture.
- **P-4 No Silent Loss**: PASS. A declined resolution (unrecognized, ambiguous,
  unreadable) and a not-installed Steam app id are surfaced command failures that
  capture nothing; the decline reason is carried from the resolver's `Unresolved`
  notes, not re-derived or swallowed.
- **P-5 Compatibility Outranks Richness**: PASS. Output format is unchanged; the
  synthesized profile drives the same writers. The `--profile` output is
  byte-identical.
- **P-6 Glossary First**: PASS (planned). The new term ("non-profile capture
  path") gets a glossary entry, and the specification documents it.
- **P-7 Wrappers Stay Thin**: PASS. All logic is in Rust; the shell wrappers are
  untouched.
- **P-8 House Standards Apply**: PASS (planned). SPDX headers, UTF-8/LF, no
  em/en dashes on new/edited source.
- **P-9 The Instrument Does Not Lie (NON-NEGOTIABLE)**: PASS. The synthesized
  identity is stamped `heuristic-unverified`, never `authored`, because it was
  resolved by a heuristic rather than typed by an operator. A declined resolution
  is surfaced, never a silent empty capture. The synthesized game identity is a
  generic placeholder (plus the app id as a fact for `--steam`), not a fabricated
  title.

**Result**: All gates pass. No Complexity Tracking entries.

## Project Structure

### Documentation (this feature)

```text
specs/032-non-profile-capture/
├── plan.md, research.md, data-model.md, quickstart.md
├── contracts/run-nonprofile-cli.md
├── checklists/{requirements.md, constitution-and-contract.md}
└── tasks.md   (/speckit-tasks output)
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/
├── cli.rs            # RunArgs: --profile becomes optional; + --install-dir,
│                     #   --steam; a required, mutually-exclusive ArgGroup
├── commands/run.rs   # branch: profile reference vs install-location; on a
│                     #   non-profile resolved target, synthesize + capture; on
│                     #   Unresolved, surface the decline reason
└── (tests)           # offline non-profile capture tests, arg-group tests

crates/fragcap-steam/  # reused unchanged: install_root_for / install_root_in
docs/glossary/*.md     # + "non-profile capture path" entry; index regenerated
docs/fragcap-specification.md  # document the non-profile capture path
changelog.d/032-non-profile-capture.{added,decisions}.md
```

**Structure Decision**: The whole slice lives in `fragcap-cli`. It is command
wiring over contracts every other crate already exposes: the resolver and its
providers, the Steam library lookup, and the shared orchestrator. No new type
crosses a crate boundary; at most a small private helper in `run.rs` synthesizes
the profile (mirroring `watch::synthesize_profile`) and extracts the identity
from a resolved non-profile target.

## Key Design Decisions

1. **Three inputs, one required, mutually exclusive, via a clap `ArgGroup`.**
   `RunArgs.profile` changes from a required `String` to an `Option<String>`, and
   `--install-dir: Option<PathBuf>` and `--steam: Option<String>` join it in a
   `required = true, multiple = false` group. clap reports "supply exactly one"
   as a usage error (exit 2) before any resolution runs, satisfying FR-005 with
   no hand-rolled validation. Existing `run --profile X` invocations are
   unchanged; the field is now read as `args.profile.as_deref()`.

2. **The non-profile branch reuses `effective_config(args, &synthesized_profile)`.**
   `effective_config` overlays CLI options onto the profile's `capture()`
   defaults; a synthesized profile declares no `capture` block, so every option
   comes from the command line, which is exactly right. No new
   `effective_config_for_*` is needed (unlike `watch`, whose smaller `WatchArgs`
   required one). This keeps the full `run` option surface (mode, roles,
   direction, interfaces, sinks, bounds) available on the non-profile path for
   free.

3. **Identity comes from the resolved target's existing `MatchPredicates`.**
   Every non-profile origin (`Observed`, `EngineRule`, `PlatformWalker`) already
   carries a `MatchPredicates` identity accessor. The synthesizer reads that and
   emits a one-stage JSON profile through `Profile::parse` (the same validating
   path `watch` uses), stamped `heuristic-unverified`. If the three origins do
   not already expose a uniform accessor, a small `Target` helper returning the
   resolved identity for a non-profile target is added in `fragcap-profile`
   (a pure accessor, no new dependency); research R2 settles which.

4. **`--steam` is sugar over `--install-dir`.** It calls
   `fragcap_steam::install_root_for(app_id)`; on success it feeds the resolved
   install directory into the identical `for_install` cascade path; a
   not-installed app id (or an install lookup error) is a surfaced failure (exit
   1) naming the missing title (FR-008). This keeps one non-profile branch rather
   than two.

5. **A declined resolution surfaces the resolver's reason.** The generic
   `From<ResolutionError>` reduces an `Unresolved` to "no target could be
   resolved", which loses the ambiguity/unreadable detail. The non-profile branch
   matches `ResolutionError::Unresolved` explicitly and renders its
   `ResolutionNotes` (engine-rule ambiguity, walker ambiguity, unreadable path)
   into the surfaced message, so FR-007's "names the reason" holds. The profile
   branch keeps the existing mapping unchanged.

6. **The profile path is a separate, unchanged branch.** When `--profile` is the
   input, `run` builds `for_reference` and captures exactly as today; its output
   is asserted byte-identical against the existing goldens (FR-006). The
   non-profile branch is reached only via `--install-dir`/`--steam`, which carry
   no profile reference, so `into_profile()` there is always `None` and the
   branch order is unambiguous.

## Complexity Tracking

No constitution violations. No entries.
