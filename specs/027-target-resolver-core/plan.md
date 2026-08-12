# Implementation Plan: Target Resolution Cascade -- Resolver Core

**Branch**: `feat/target-resolver-core` | **Date**: 2026-08-12 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/027-target-resolver-core/spec.md`

## Summary

Introduce the target-resolution cascade as a first-class abstraction in
`fragcap-profile`, above the existing section 15.3 profile-reference lookup. A
`TargetResolver` holds a set of `TargetProvider`s, sorts them by an explicit
`Precedence` independent of registration order, queries them highest first, and
returns the first stamped `Target` or a named not-resolved outcome. Every answer
carries a targeting `FidelityTier` and a `Provenance`.

Two providers carry data: the `ProfileProvider` wraps `resolve()` and stamps its
answer with the profile's own declared fidelity and provenance; the
`ObservationProvider` scans the process tree with a supplied identity and stamps
an `observed` target. Three providers (`HintProvider`, `EngineRuleProvider`,
`PlatformWalkerProvider`) are registered and always answer "no answer" in this
slice; #78, S029, and S030 fill them.

The load-bearing gap this slice closes: the in-memory `Profile` currently
discards `kind`, `fidelity`, `provenance`, and `notes` after the S025 structural
validator confirms them. This adds `FidelityTier`, `Provenance`, and `Kind` to
`fragcap-profile::schema`, extends `Profile::parse` to retain them off the parsed
`serde_json::Value`, and gives `Profile` accessors for them.

The targeting `FidelityTier` (authored/verified/heuristic-unverified/observed) is
a new type in `fragcap-profile`, entirely separate from the attribution
`Fidelity` (Live/Retained/None) in `fragcap-core::attribution`, which this slice
does not touch. The names differ and the crates differ, so the two axes cannot be
confused.

The `run` command's profile path flows through the resolver and receives a
profile-backed `Target`, then extracts the `Profile` and proceeds exactly as
today, so capture output is byte-identical (SC-008). No launch-agnostic CLI
surface is added; that is S028.

## Technical Context

**Language/Version**: Rust, MSRV 1.82 (toolchain pinned 1.96)

**Primary Dependencies**: none new. The resolver is pure logic over already-parsed
`Profile`s and the `ProcessTree` the `matching` module already reads. `serde_json`
(runtime in fragcap-profile since S025) reads the retained metadata.

**Storage**: JSON profile files on disk (unchanged).

**Testing**: `cargo test`, `cargo xtask ci`, `cargo xtask deps`, `cargo xtask
lint`, `cargo xtask msrv` at 1.82.

**Target Platform**: platform-neutral crate (fragcap-profile).

**Project Type**: Rust workspace (library crates + CLI).

**Performance Goals**: resolution is interactive, once per capture; not a hot
path.

**Constraints**: nothing added to fragcap-core (allowlist ["bytes"], P-2); no
process handle (P-1); every not-resolved path named and surfaced (P-4); fidelity
carried, never inferred (P-9); capture output byte-identical (SC-008); UTF-8 no
BOM, LF, no em/en dashes.

**Scale/Scope**: one crate gains a resolver (~3 new source files), `schema.rs` and
`parse.rs` gain metadata surfacing, `matching.rs` gains one public matcher, the
CLI `run` path is rewired to call the resolver, plus master spec section and four
glossary entries.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive Observation**: The observation provider reads only the image name
  and path already in the `ProcessTree` snapshot (via the existing `matching`
  predicates); it opens no process handle and reads no memory. `cargo xtask lint`
  continues to forbid `OpenProcess`/`ReadProcessMemory`. PASS.
- **P-2 Core Stays Platform-Neutral**: The resolver, providers, `Target`,
  `FidelityTier`, `Provenance`, and `Kind` all live in fragcap-profile. Nothing is
  added to fragcap-core; `cargo xtask deps` stays green. No new external crate.
  PASS.
- **P-3 Capture And Attribution Stay Separate**: The resolver decides what to
  capture; it is not a `PacketSource` and not a `FlowAttributor`, and it does not
  touch the attribution `Fidelity`. PASS.
- **P-4 No Silent Loss**: When no provider answers, resolution returns a distinct,
  named not-resolved outcome; a capture is never armed against nothing without
  saying so. PASS.
- **P-5 Compatibility Outranks Richness**: No output format change; capture output
  is byte-identical (SC-008). PASS.
- **P-6 Glossary First**: The four new terms (provider, target resolver, resolution
  cascade, target) get glossary entries in this change; the master spec gains the
  cascade section; `scripts/lint-docs.sh` enforces it. PASS.
- **P-9 The Instrument Does Not Lie**: Every answer carries exactly one fidelity
  tier stamped by its source; an observation answer is `observed`, never verified
  or authored; the tier is carried, never inferred. PASS.

No violations. Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/027-target-resolver-core/
├── plan.md              # This file
├── research.md          # Phase 0: decisions D-1..D-8
├── data-model.md        # Phase 1: the types
├── quickstart.md        # Phase 1: how to exercise the resolver
├── contracts/
│   └── resolver-api.md  # Phase 1: the public API surface
├── checklists/
│   ├── requirements.md  # from /speckit-specify
│   └── resolver-invariants.md  # from /speckit-checklist
└── tasks.md             # from /speckit-tasks
```

### Source Code (repository root)

```text
crates/fragcap-profile/src/
├── schema.rs      # + FidelityTier (with rank), Provenance, Kind; Profile gains
│                  #   kind/fidelity/provenance/notes fields and accessors
├── parse.rs       # + retain kind/fidelity/provenance/notes off the parsed Value
├── matching.rs    # + pub first_live_match(preds, tree) -> Option<NodeId>
├── target.rs      # NEW: Target, TargetOrigin, ObservedTarget, TargetIdentity
├── resolver.rs    # NEW: TargetProvider trait, Precedence, TargetResolver,
│                  #   ResolutionRequest, Unresolved, ProviderError
├── providers.rs   # NEW: ProfileProvider, ObservationProvider, and the three
│                  #   no-answer stubs (Hint/EngineRule/PlatformWalker)
└── lib.rs         # + re-exports for the new public types

crates/fragcap-cli/src/
├── commands/run.rs  # call TargetResolver instead of resolve() directly;
│                    #   extract the Profile from the profile-backed Target
└── (assemble.rs unchanged: it still receives a Profile)

docs/
├── fragcap-specification.md          # + section 15.7 Target Resolution Cascade
└── glossary/process-and-attribution.md  # + provider, target resolver,
                                          #   resolution cascade, target
```

**Structure Decision**: Flat modules in fragcap-profile, matching the crate's
existing style (schema.rs, resolve.rs, matching.rs are each one file). Shared
metadata types (`FidelityTier`, `Provenance`, `Kind`) live in `schema.rs` because
`Profile` carries them and `target.rs` imports them, so there is one definition
each. The resolver is split into `target.rs` (the answer), `resolver.rs` (the
engine and its traits), and `providers.rs` (the concrete providers) so a later
slice adds a provider to `providers.rs` without touching the engine.

## Phased approach

- **Phase 0 (research.md)**: the eight design decisions, chief among them the
  precedence-versus-fidelity model, the `FidelityTier` ordering that makes
  higher trust the greater value, and the CLI error-mapping that preserves
  behavior.
- **Phase 1 (data-model.md, contracts/, quickstart.md)**: the concrete types and
  the public API, and a walkthrough exercising both live providers.
- **Phase 2 (tasks.md, via /speckit-tasks)**: the dependency-ordered task list.

## Complexity Tracking

No constitution violations; no entries.
