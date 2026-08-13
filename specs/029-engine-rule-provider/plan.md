# Implementation Plan: Engine-Rule Provider (Unreal First)

**Branch**: `feat/engine-rule-provider` | **Date**: 2026-08-12 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/029-engine-rule-provider/spec.md`

## Summary

Fill in the `EngineRuleProvider` stub S027 registered at
`Precedence::EngineRule`, so the target-resolution cascade can name a game's
socket-holding client from the engine's documented on-disk install layout, with
no per-game data. The provider inspects a supplied install directory, matches it
against a small ordered set of engine rules, and, on a match, returns a `Target`
stamped `FidelityTier::HeuristicUnverified` with provenance `engine-rule`.

Unreal Engine is the mandatory rule: a `*-Win64-Shipping.exe` under a
`Binaries/Win64` directory is the socket holder that a root stub relaunches. The
rule is a filename-suffix-plus-path convention only. It reads no launcher token
and no post-run artifact (AppData does not exist before first launch, so it is
useless to a pre-launch resolver). Unity (`*_Data` directory plus
`UnityPlayer.dll`) and Ren'Py (`renpy` directory plus `.rpa` archives) are the
same provider with two more rules.

The provider is pure logic over the filesystem. It lives in a new
`engine_rule` module in `fragcap-profile`, the crate that already owns
resolution and matching, and adds no dependency: `std::fs` and `std::path` are
the whole toolkit. The `EngineRuleProvider` in `providers.rs` delegates to that
module, reads a new install-root input on `ResolutionRequest`, and declines
(returns no answer) whenever the input is absent, no rule matches, or a rule
matches ambiguously.

Three small extensions to the S027 surface make this fit without disturbing the
cascade engine:

- `ResolutionRequest` gains an optional `install_root` input and a
  `for_install` constructor. The existing constructors set it to `None`, so no
  existing caller changes and the profile and observation providers are
  untouched.
- `TargetOrigin` gains an `EngineRule(EngineRuleTarget)` variant, distinct from
  the profile and observed origins. `EngineRuleTarget` names the resolved client
  (engine, image file name, full path) and carries the `MatchPredicates` the
  pipeline binds it by, exactly as `ObservedTarget` carries its identity.
- `ResolutionNotes` gains an engine-rule ambiguity note, surfaced through
  `Unresolved`, so a declined-for-ambiguity outcome explains itself when nothing
  lower resolves either (P-4, P-9). This mirrors the existing
  `profile_not_found` note.

The targeting `FidelityTier::HeuristicUnverified` this provider stamps is
entirely separate from the attribution `Fidelity` (Live/Retained/None) in
`fragcap-core::attribution`, which this slice does not touch. An engine rule is
a filesystem heuristic and says so; it never claims a higher tier (P-9).

This slice makes no CLI change and adds no production wiring of the full
provider set. It demonstrates the provider's participation through the S027
`TargetResolver` in tests, exactly as S027 demonstrated its own providers. The
composition with the S030 platform walker is a design property: the walker will
populate the same `install_root` input the directory scan populates here, so the
walker feeds this provider unchanged.

## Technical Context

**Language/Version**: Rust, MSRV 1.82 (toolchain pinned 1.96)

**Primary Dependencies**: none new. The provider is `std::fs` and `std::path`
over an install directory. `MatchPredicates` is built in-crate via `Default` and
the existing `pub(crate)` setters (`set_exe`, `set_path_contains`) with an
`ImagePattern::new`.

**Storage**: reads a game install directory tree on disk. Writes nothing.

**Testing**: `cargo test`, `cargo xtask ci` (fmt, clippy, test, lint, deps,
license), `cargo xtask deps`, `cargo xtask lint`, `cargo xtask msrv` at 1.82.
Fixtures are temporary directory trees built at test time, in the spirit of
`fragcap-steam`'s `TempTree`; nothing is committed to `fixtures/`, so the corpus
drift check is unaffected.

**Target Platform**: platform-neutral crate (fragcap-profile). The rules target
Windows path conventions (`Binaries\Win64`, backslash separators) but match on
path components case-insensitively, and the tests build real temp directories,
so they run wherever `cargo test` runs.

**Project Type**: Rust workspace (library crates + CLI).

**Performance Goals**: resolution is interactive, once per capture, over one
install directory; not a hot path. The scan is bounded to the directories the
rules name (the install root and `Binaries/Win64`), not an unbounded recursive
walk.

**Constraints**: nothing added to fragcap-core (allowlist ["bytes"], P-2); no
process handle, no memory read, no launch, no post-run artifact, no launcher
token (P-1, FR-005); every decline reason observable (P-4, FR-009); fidelity
carried, never inferred, never upgraded (P-9, FR-003); deterministic and
iteration-order independent (FR-006); UTF-8 no BOM, LF, no em/en dashes.

**Scale/Scope**: one crate gains an `engine_rule` module (~1 new source file)
and one filled-in provider; `resolver.rs` gains one request input, one
constructor, and one accessor, plus an ambiguity note; `target.rs` gains one
origin variant and its carrier type; `lib.rs` re-exports the new public types;
one master-spec subsection and one full glossary entry.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive Observation**: The provider reads only the filesystem layout of
  an install directory. It opens no process handle, reads no process memory,
  launches nothing, and reads no post-run artifact or launcher token (FR-005).
  `cargo xtask lint` continues to forbid `OpenProcess`/`ReadProcessMemory`/
  `WriteProcessMemory`; none appear. PASS.
- **P-2 Core Stays Platform-Neutral**: The `engine_rule` module, the provider,
  the new `TargetOrigin` variant, and the request input all live in
  fragcap-profile. Nothing is added to fragcap-core; `cargo xtask deps` stays
  green. No new external crate, so the dependency graph is unchanged. PASS.
- **P-3 Capture And Attribution Stay Separate**: The provider touches neither
  `PacketSource` nor `FlowAttributor`. It produces a target identity; capture
  and attribution are downstream and unchanged. PASS.
- **P-4 No Silent Loss**: Every non-match path is a named outcome. A no-match
  declines and the cascade continues; an ambiguous match declines and records an
  engine-rule ambiguity note surfaced through `Unresolved`; an unreadable
  directory declines with the reason observable (FR-004, FR-009). Nothing is
  dropped without a name. PASS.
- **P-5 Compatibility Outranks Richness**: No output format changes. The
  resolved target flows through the existing cascade. PASS.
- **P-6 Glossary First**: The term "engine rule" gains a full glossary entry in
  `docs/glossary/process-and-attribution.md` in this slice, promoted from the
  named-example mention it has today, cross-linked to `provider`, `provenance`,
  and `fidelity tier`. The master-spec cascade section documents engine rules in
  the same change (FR-010). `scripts/lint-docs.sh` enforces entry completeness.
  PASS.
- **P-9 The Instrument Does Not Lie**: The provider stamps every answer
  `heuristic-unverified` and cannot stamp higher (FR-003). On ambiguity it
  declines rather than guessing a single candidate, so it never presents an
  arbitrary pick as the answer (FR-006). PASS.
- **Licensing / dependencies**: no new crate, so no license question. The one
  runtime crate the module uses is the standard library. PASS.

No violations. Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/029-engine-rule-provider/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (provider + module contract)
│   └── engine-rule.md
├── checklists/
│   ├── requirements.md   # spec quality (from /speckit-specify)
│   └── engine-rule.md    # requirements-quality checklist (from /speckit-checklist)
└── tasks.md             # /speckit-tasks output (not created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/fragcap-profile/src/
├── engine_rule.rs        # NEW: Engine enum, engine rules, resolve_engine() over a dir
├── providers.rs          # EngineRuleProvider::provide filled in (delegates to engine_rule)
├── resolver.rs           # ResolutionRequest gains install_root + for_install + accessor;
│                         #   ResolutionNotes/Unresolved gain the engine-rule ambiguity note
├── target.rs             # TargetOrigin::EngineRule(EngineRuleTarget) + carrier type
├── lib.rs                # re-export EngineRuleTarget / Engine as needed
└── (matching.rs, resolve.rs, schema.rs unchanged except schema stays as-is)

docs/
├── glossary/process-and-attribution.md   # full "Engine rule" entry
└── fragcap-specification.md              # section 15.7 engine-rule subsection
```

**Structure Decision**: Keep everything in `fragcap-profile`. It already owns
resolution (`resolve.rs`), matching (`matching.rs`), and the cascade
(`resolver.rs`, `providers.rs`, `target.rs`), and its only permitted workspace
edge is to `fragcap-core`. A new crate would add a workspace edge and a
`deps.rs` `EXPECTED` change for no benefit: the provider is pure logic the crate
is already the right home for. The engine-detection logic goes in its own
`engine_rule` module rather than swelling `providers.rs`, so the recognizer set
is testable in isolation and the provider stays a thin adapter (the same split
S027 used between `resolver.rs` and `providers.rs`).

## Complexity Tracking

No constitution violations. This section is intentionally empty.
