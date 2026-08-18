# Implementation Plan: CLI surface rework

**Branch**: `054-cli-surface-rework` | **Date**: 2026-08-17 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/054-cli-surface-rework/spec.md`

## Summary

Collapse the three capture verbs (`run`, `tap`, `watch`) into one `capture` verb
whose flags are orthogonal, so all five section-9.1 captures become expressible;
retire the profile-file surface completely (the S051 US5 deferral); realign the
command namespaces to the two stores (`catalog` writes `catalog.db`, `targets`
writes `local.db`, `steam profile` becomes `targets add --steam`); group the
`--help` surface under four headings; and make bare `fragcap` print the targets
listing with a `--help` footer. The change is confined to `fragcap-cli`
(argument grammar, command dispatch, the assembly seam) plus documentation and the
master-specification section that describes the command surface. No capture,
attribution, pipeline, sink, or core code changes.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82 (verified locally under the
GNU-host `1.96.0-x86_64-pc-windows-gnu` toolchain because this machine has no MSVC
linker; CI runs the real MSVC build).

**Primary Dependencies**: `clap` (derive) for the argument grammar; no new
dependency. The store access (`fragcap::targets`) and launch (`fragcap::steam`)
paths already exist.

**Storage**: Unchanged. `catalog.db` and `local.db` (S050) are read/written through
the existing `fragcap-targets` store API; this slice only moves which command
namespace reaches them.

**Testing**: `cargo test` in `fragcap-cli` (unit + `tests/` integration), driving
the capture path through the hidden offline substrate (recorded source, scripted
attributor, scripted process timeline) with no capture driver, no elevation, no
game. clap parse/usage assertions for the grammar; `--help` and bare-invocation
snapshot-style assertions for presentation.

**Target Platform**: Windows (the capture target); the CLI grammar and offline
tests build and run on any host.

**Project Type**: CLI (a Cargo workspace facade with a `fragcap-cli` crate).

**Performance Goals**: N/A (argument parsing and dispatch; no hot path).

**Constraints**: No capture capability may be removed (FR-003/FR-004). Text hygiene
(UTF-8/LF, no dashes). The change touches master-specification section 17, so P-11
lock-step applies.

**Scale/Scope**: One crate's argument grammar and dispatch; ~10 command files
touched; documentation examples across the docs tree; one master-spec section; one
changelog fragment; glossary entries for any new term.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Note |
| --- | --- | --- |
| P-1 Passive observation | PASS | Surface reorganization only; no technique added. |
| P-2 Core stays neutral | PASS | No `fragcap-core` change; all work in `fragcap-cli`. Dependency direction unchanged. |
| P-3 Capture/attribution separate | PASS | The assembly seam feeds the same pipeline; the source/attributor split is untouched. |
| P-4 No silent loss | PASS | No discard path added or removed; counters untouched. |
| P-5 Compatibility outranks richness | PASS | Output format untouched; readable by unmodified analyzers. |
| P-6 Glossary first | GATED | New user-facing terms (the `capture` verb as the unified capture surface; the `catalog`/`targets` namespace-to-store binding as named terms) get glossary entries in this change if not already present. |
| P-7 Wrappers stay thin | PASS | No wrapper parsing/logic added. |
| P-8 House standards | PASS | Text hygiene enforced by the linter; applies to all touched files. |
| P-9 Instrument does not lie | PASS | No observation altered; a `--launch` misconfig is a loud usage error, not a silent no-op. |
| P-10 One path to a target | REINFORCED | `capture --target` resolves the single S051 stored form; `targets add --steam` uses the same `TargetSource` machinery. Retiring the parallel `--profile`/`--install-dir`/`--steam` capture inputs removes a second path to a target, which is exactly what P-10 asks for. |
| P-11 Spec describes what shipped | GATED | The command surface is master-spec section 17. This slice edits section 17 in the same change, and the changelog fragment carries a `spec-impact: 17` line (plus 15.7 for the relocated signature seed if its section text names a command). No release is cut mid-slice, so the `Applies-To` version lock-step is unaffected. |

No violations. Two gated items (P-6 glossary, P-11 spec section 17) are ordinary
obligations discharged inside the slice, not deviations. No Complexity Tracking
entries required.

## Project Structure

### Documentation (this feature)

```text
specs/054-cli-surface-rework/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (command-surface contracts)
├── checklists/          # requirements.md (specify), cli-surface.md (checklist)
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/
├── cli.rs               # clap grammar: add Capture command + CaptureArgs; add
│                        #   Catalog command + CatalogCommand; move seed/import/
│                        #   export/seed-engine/seed-signatures off TargetsCommand;
│                        #   add --steam to TargetsAddArgs; remove Run/Tap/Watch and
│                        #   RunArgs/TapArgs/WatchArgs; remove Profile/ProfileArgs/
│                        #   ProfileCommand; add help_heading groupings; add the
│                        #   no-subcommand bare-invocation path.
├── assemble.rs          # collapse effective_config_for_tap/for_watch into the
│                        #   capture assembly; keep effective_config (now the
│                        #   capture path) and effective_config_for_extcap.
├── lib.rs               # dispatch: Command::Capture -> commands::capture; remove
│                        #   Run/Tap/Watch/Profile arms; add Catalog arm; bare
│                        #   invocation -> targets listing + footer.
├── commands/
│   ├── capture.rs       # NEW: the unified capture command (from run.rs, folding
│   │                    #   tap.rs/watch.rs target synthesis).
│   ├── catalog.rs       # NEW: catalog namespace (import/export/seed/seed-engine/
│   │                    #   seed-signatures/update), moved from targets.rs.
│   ├── targets.rs       # local.db ops only (add [+ --steam], list, show, discover,
│   │                    #   scan); catalog ops removed.
│   ├── steam.rs         # residual Steam-specific ops; profile scaffolding removed.
│   ├── run.rs tap.rs watch.rs profile.rs   # REMOVED.
│   └── mod.rs           # module list updated.
└── (paths.rs, args.rs, orchestrator.rs, output.rs, emit.rs unchanged in behaviour)

docs/…                    # every command example updated to the new surface.
docs/fragcap-specification.md   # section 17 (command surface) rewritten.
docs/glossary/…           # entries for any new term; index regenerated.
changelog.d/S054-*.md     # changelog fragment with spec-impact header.
```

**Structure Decision**: Single-crate change in `fragcap-cli`. The capture engine,
attribution, pipeline, and stores are reused unchanged through their existing APIs;
this slice reshapes the argument grammar, the dispatch, and the assembly seam that
sits above them, then updates the docs and the one master-spec section that
describe that surface.

## Phase 0: Research

See [research.md](research.md). Resolves: how `--target` resolves an S051 stored
target into a capture stage; how `--process` plus path anchors synthesizes the
one-stage capture the old `tap`/`watch` built; where `--launch`'s anchor comes from
once `--profile` is gone; the clap mechanics for help grouping and a
no-subcommand default; and the `catalog update` net-gated shape.

## Phase 1: Design & Contracts

- [data-model.md](data-model.md): the capture-invocation and command-namespace
  models, and the flag-to-behaviour mapping table.
- [contracts/](contracts/): the `capture` command contract, the namespace/dispatch
  contract, and the help-and-bare-invocation contract.
- [quickstart.md](quickstart.md): runnable validation for the five captures, the
  removal negatives, the namespace moves, and the presentation behaviour.
