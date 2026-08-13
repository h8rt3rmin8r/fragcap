# Implementation Plan: Steam Platform-Walker Refactor

**Branch**: `feat/steam-platform-walker` | **Date**: 2026-08-12 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/030-steam-platform-walker/spec.md`

## Summary

Fill the `PlatformWalkerProvider` slot S027 registered, moving the real
implementation into `fragcap-steam` so Steam's installed-library enumeration
flows through the target-resolution cascade and composes with the S029
engine-rule provider. The walker contributes in two ways: it makes a title's
install directory available to the resolver as the request's `install_root` (so
the higher-precedence engine rule can name the socket holder from layout), and,
when the engine rule does not recognize the layout, it answers at its own
precedence by resolving a single client executable from the install directory.

The walker declines rather than guess. It resolves only when the install
directory holds exactly one plausible client executable (after dropping
installers, redistributables, helpers, and launcher stubs, reusing the existing
scaffold classifier's predicates). Zero plausible clients, or several, is a
decline, and the cascade falls through to runtime observation, which resolves the
game from the live socket-holding process. This is deliberate: selecting a client
by size among several candidates is the coincidental heuristic the library
research flagged as unreliable, so the walker, feeding automatic capture, does not
guess where a human-reviewed scaffold would. Every walker answer is stamped
`heuristic-unverified` with provenance `steam-library`, an honest name for the
library-manifest walk plus install-directory classification it performed; it never
claims `steam-appinfo`, a source it does not read.

Three surface changes make this fit:

- `TargetOrigin` gains a `PlatformWalker(WalkerTarget)` variant in
  `fragcap-profile`, distinct from the profile, engine-rule, and observed origins.
  `WalkerTarget` names the platform (`steam`), the resolved client (image name and
  full path), and the `MatchPredicates` the pipeline binds it by.
- `ResolutionNotes`/`Unresolved` gain walker ambiguity and unreadable notes,
  mirroring the engine-rule notes S029 added, so a declined-for-ambiguity or
  declined-for-unreadable-install outcome explains itself (P-4).
- The no-op `PlatformWalkerProvider` stub in `fragcap-profile` is retired; the
  real `SteamWalkerProvider` lives in `fragcap-steam` (an allowed dependency edge),
  and the CLI's resolver assembly (`run.rs`, `watch.rs`) constructs it. This
  respects the dependency direction: `fragcap-profile` must not depend on
  `fragcap-steam`.

Steam's `steam://` managed launch is unchanged and stays a convenience adapter.
No dependency is added: the walker reuses `fragcap-steam`'s existing filesystem
enumeration and classifier, and `std::fs` is the whole toolkit.

### Scope boundary made explicit (not buried)

The walker provider is wired into the production resolver vec, but, exactly like
the S029 engine-rule provider, it cannot yet *fire* a capture in production. The
`run` command today errors on a resolved target that carries no profile
(`run.rs`: "resolved a target with no profile, which run cannot capture yet"), and
its own module doc names driving a non-profile target as "a later slice." A profile,
when present, outranks the walker and engine rule, so the walker only matters for a
no-profile capture, which needs that non-profile capture path. Building it is a
cross-cutting integration that S027 through S029 all deferred and that this slice
also defers, surfaced at the pre-push halt rather than hidden. What S030 delivers
and proves end to end through the resolver in tests: the walker provider, its
composition with the engine rule over a Steam install directory, its own
single-client resolution, and graceful degradation to runtime observation. The
Steam enumeration-to-`install_root` helper is built and tested so the future
non-profile capture path has it ready.

## Technical Context

**Language/Version**: Rust, MSRV 1.82 (toolchain pinned 1.96)

**Primary Dependencies**: none new. The walker reuses `fragcap-steam`'s existing
library enumeration (`discover`, `discover_in`, `InstalledTitle`) and classifier
predicates (`scan`, `is_non_game`, `is_launcher` in `scaffold.rs`); `std::fs` and
`std::path` are the toolkit. `WalkerTarget`'s `MatchPredicates` is built in-crate
the same way S029's engine-rule target is.

**Storage**: reads Steam's local library manifests and install directories on
disk (and the registry to locate Steam, as `fragcap-steam` already does). Writes
nothing.

**Testing**: `cargo test`, `cargo xtask ci` (fmt, clippy, test, lint, deps,
license, wrappers, docs check), `cargo xtask deps` (the dependency-direction
gate, load-bearing here), `cargo xtask msrv` at 1.82. Fixtures are temporary
directory trees: a fake Steam library (library manifests plus install dirs)
composed with the engine-rule install-layout fixtures, in the spirit of the
existing `fragcap-steam` `TempTree` and the S029 `UnrealTree`.

**Target Platform**: `fragcap-steam` (Windows for the registry-backed `discover`;
`discover_in` and all the walker logic are portable and tested on the CI/dev
platform).

**Project Type**: Rust workspace (library crates + CLI).

**Performance Goals**: resolution is interactive, once per capture; the walker
scans one install directory. Not a hot path.

**Constraints**: no new dependency; nothing added to `fragcap-core`; the walker
provider lives in `fragcap-steam` and `fragcap-profile` gains no dependency on it
(P-2, P-3, deps gate); filesystem and registry reads only, no process handle, no
memory read, no network (P-1); every decline or degrade is a named, surfaced
outcome (P-4); fidelity carried and honest, provenance names what was done (P-9);
UTF-8 no BOM, LF, no em/en dashes.

**Scale/Scope**: `fragcap-steam` gains a `walker` module (the provider and the
`client_for` resolver) and an enumeration helper; `fragcap-profile` gains the
`PlatformWalker` origin, the `WalkerTarget` type, and two resolution notes, and
loses its walker stub; `run.rs`/`watch.rs` swap the stub for the real provider;
one master-spec subsection plus a section-16 reframe and one glossary entry.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive Observation**: The walker reads library manifests, install
  directories, and the registry (all already done by `fragcap-steam`). It opens
  no process handle, reads no process memory, launches nothing, and makes no
  network call. `cargo xtask lint` continues to forbid the handle APIs; none
  appear. PASS.
- **P-2 Core Stays Platform-Neutral**: Nothing is added to `fragcap-core`. The
  walker provider lives in `fragcap-steam`; the `PlatformWalker` origin and notes
  live in `fragcap-profile`. No new external crate. PASS.
- **P-3 Capture And Attribution Stay Separate / dependency direction**: The
  walker touches neither `PacketSource` nor `FlowAttributor`. The provider lives
  in `fragcap-steam` (edge `fragcap-steam -> fragcap-profile`, allowed); the
  reverse edge is not introduced, so `cargo xtask deps` stays green. The facade
  and CLI, which depend on both crates, assemble the resolver. PASS.
- **P-4 No Silent Loss**: Every non-answer is named. A decline (not installed, no
  plausible client) lets the cascade continue; an ambiguous install records a
  walker-ambiguity note; an unreadable install records the path; malformed
  manifests are surfaced as enumeration warnings. Nothing is dropped silently.
  PASS.
- **P-5 Compatibility Outranks Richness**: No output format change. A walker
  target flows through the existing cascade; where materialized as a target
  artifact it validates against the master schema. PASS.
- **P-6 Glossary First**: "platform walker" gains a full glossary entry in this
  slice, promoted from the referenced-only mention it has today, cross-linked to
  provider, resolution cascade, engine rule, and target. The master-spec cascade
  section documents the platform walker and section 16 is reframed. PASS.
- **P-9 The Instrument Does Not Lie**: The walker stamps `heuristic-unverified`
  and never higher. It declines rather than guess a client by size among several
  candidates. Its provenance, `steam-library`, names the method actually used and
  does not claim `steam-appinfo`, a source it does not read. PASS.
- **Licensing / dependencies**: no new crate. PASS.

No violations. Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/030-steam-platform-walker/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── platform-walker.md
├── checklists/
│   ├── requirements.md   # spec quality (from /speckit-specify)
│   └── platform-walker.md # requirements-quality checklist (from /speckit-checklist)
└── tasks.md             # /speckit-tasks output (not created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/fragcap-profile/src/
├── target.rs             # + TargetOrigin::PlatformWalker(WalkerTarget) + WalkerTarget type
├── providers.rs          # - retire the no-op PlatformWalkerProvider stub
├── resolver.rs           # + walker ambiguity + unreadable notes on ResolutionNotes/Unresolved
└── lib.rs                # re-export WalkerTarget

crates/fragcap-steam/src/
├── walker.rs             # NEW: SteamWalkerProvider (impl fragcap_profile::TargetProvider),
│                         #   client_for(install_dir) -> ClientResolution, reusing scaffold predicates
├── scaffold.rs           # extract/share scan + is_non_game + is_launcher for walker.rs
├── library.rs            # + install_root_for(app_id) enumeration helper (discover + find)
└── lib.rs                # re-export SteamWalkerProvider, install_root_for

crates/fragcap-cli/src/commands/
├── run.rs                # swap PlatformWalkerProvider stub for the fragcap-steam SteamWalkerProvider
└── watch.rs              # same swap in the watch resolver assembly

docs/
├── glossary/process-and-attribution.md   # full "Platform walker" entry
└── fragcap-specification.md              # section 15.7 walker subsection + section 16 reframe
```

**Structure Decision**: The provider lives in `fragcap-steam` because that is the
only crate that both holds Steam knowledge and is allowed to depend on
`fragcap-profile` (the deps gate forbids the reverse). The `WalkerTarget` origin
and the resolution notes live in `fragcap-profile` beside the rest of the cascade
vocabulary, constructed by `fragcap-steam` through their public constructors, the
same split S029 used for its engine-rule origin. The classifier predicates
(`scan`, `is_non_game`, `is_launcher`) are shared from `scaffold.rs` rather than
duplicated, so the walker and the human-reviewed scaffold agree on what an
installer or a launcher is.

## Complexity Tracking

No constitution violations. This section is intentionally empty.
