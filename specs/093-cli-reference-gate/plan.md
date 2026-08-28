# Implementation Plan: CLI Reference Gate

**Branch**: `codex/093-cli-reference-gate` | **Date**: 2026-08-28 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/093-cli-reference-gate/spec.md`

## Summary

Turn the public CLI reference into a checked contract against `fragcap_cli::command()`. A new CLI integration test will recursively compare visible command paths, locally owned options, aliases, value sets, and clap defaults with human-visible MDX headings and tables; derive sink tokens from the parser source; and parse every executable example without dispatch. `cargo xtask docs check` will run the test for the default and network-capable command variants, while the same slice corrects the current v0.7.0 reference.

## Technical Context

**Language/Version**: Rust 2021 on the workspace's Rust 1.82 minimum; MDX for the public reference

**Primary Dependencies**: Existing clap command model, Rust standard library, existing CLI test dependencies, Fumadocs and Next.js site

**Storage**: One public MDX reference, one Rust integration test, one task-runner module edit, S093 specification artifacts, and one changelog fragment; no runtime storage change

**Testing**: Focused default and `net` CLI-reference integration tests; `cargo xtask docs check`; `cargo xtask docs build`; formatting, lint, diff, encoding, and complete `cargo xtask ci` gates

**Target Platform**: Hermetic repository checks on supported development platforms; statically exported Windows CLI documentation

**Project Type**: Rust CLI workspace plus public documentation site

**Performance Goals**: No runtime impact; deterministic reference validation completes as part of the existing documentation gate

**Constraints**: No runtime dependency, command-grammar change, command dispatch, capture, network access, elevation, game, user store, trust change, proxy backend, workflow edit, release edit, or master-specification edit; UTF-8 without BOM; LF; soft-wrapped Markdown prose

**Scale/Scope**: The complete v0.7.0 public command tree, every visible named option, the default and `net` variants, all accepted sink schemes and modifiers, all executable CLI-reference examples, and one global stream-routing contract

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 No Covert Target Instrumentation**: Pass. Validation constructs and parses clap values only and cannot dispatch a command or touch a target.
- **P-2 Core Stays Platform-Neutral**: Pass. The gate lives in a `fragcap-cli` integration test. `xtask` invokes Cargo as a subprocess and does not depend on the CLI crate.
- **P-3 Capture And Attribution Stay Separate**: Pass. No capture or attribution implementation changes.
- **P-4 No Silent Loss**: Pass. The documentation explicitly preserves lifecycle, diagnostic, and sink routing distinctions.
- **P-5 Compatibility Outranks Richness**: Pass. No capture format changes.
- **P-6 Glossary First**: Pass. The slice uses existing command, sink, profile, target, and Deep Capture terminology and introduces no new domain term.
- **P-7 Wrappers Stay Thin**: Pass. `xtask` only composes the existing glossary linter and focused Cargo tests.
- **P-8 House Standards Apply**: Pass. Rust, MDX, and specification artifacts follow repository and ShruggieTech standards.
- **P-9 The Instrument Does Not Lie**: Pass. Public command claims are derived from clap, sink tokens are derived from the parser, and examples are parsed through the same grammar without execution.
- **P-10 One Path To A Target**: Pass. The reference documents only the shipped target and profile entry points.
- **P-11 The Specification Describes What Shipped**: Pass. The v0.7.0 command implementation is the authority; the page is corrected to match it.

## Project Structure

### Documentation (this feature)

```text
specs/093-cli-reference-gate/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── cli-reference-contract.md
├── checklists/
│   ├── cli-contract.md
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-cli/
├── src/
│   ├── args.rs
│   ├── cli.rs
│   └── lib.rs
└── tests/
    └── cli_reference.rs

site/content/docs/reference/cli.mdx
xtask/src/docs.rs
changelog.d/246-cli-reference-gate.fixed.md
```

**Structure Decision**: Keep command-tree comparison beside the command owner as a `fragcap-cli` integration test, where the public `command()` seam is directly available. Preserve P-2 by having `xtask` run that focused test through Cargo instead of adding a dependency on the CLI crate. Make visible command headings and option tables the documentation contract, so reviewers and the gate inspect the same source. Run the contract under default features and `net`; derive sink tokens from `args.rs` because clap intentionally exposes only the sink value shape, not its internal scheme grammar.

## Complexity Tracking

No constitution violations or complexity exceptions are needed.

## Phase 0: Research

See [research.md](research.md).

## Phase 1: Design

See [data-model.md](data-model.md), [contracts/cli-reference-contract.md](contracts/cli-reference-contract.md), and [quickstart.md](quickstart.md).

## Post-Design Constitution Check

The completed design still passes all eleven principles. In particular, it adds no reverse dependency from `xtask`, no second command schema, no executable documentation harness, and no network or machine-state requirement. Visible MDX remains the single documentation authority, while runtime command and sink parsers remain the behavioral authorities.
