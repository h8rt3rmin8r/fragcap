# Implementation Plan: Consistent CLI Result Output

**Branch**: `codex/s179-cli-result-contract` | **Date**: 2026-10-22 | **Spec**: `specs/179-cli-result-contract/spec.md`

**Input**: Owner-approved S179 for issue #488, the global JSON, quiet, and silent command-result contract.

## Summary

Bring advertised global flags into agreement with actual CLI behavior. Preserve existing Capture, Doctor, JSON document, and extcap machine protocols. Add typed structured results to command handlers that currently write human prose in JSON mode, emit separate schema diagnostics, and apply one explicit quiet/silent precedence policy. A controlled matrix guards every top-level command family and the representative issue reproductions.

## Technical Context

**Language/Version**: Rust 2024, workspace MSRV 1.88.

**Primary Dependencies**: Existing `clap`, `serde_json`, `fragcap-profile`, `fragcap-targets`, and `fragcap-cli`; no new package.

**Storage**: Existing local and catalog SQLite files only; no schema or migration change.

**Testing**: Injected CLI writers, disposable stores, fixture-backed command matrix, `cargo xtask ci`, and hosted exact-head checks.

**Target Platform**: Windows product with Linux CI parity; no live Capture, game, or owner bundle needed.

**Project Type**: Native Rust command-line application.

**Performance Goals**: Result formatting does not add a second discovery, store scan, network fetch, or capture pass; one bounded serialization per result record.

**Constraints**: Preserve capture sink isolation, stderr lifecycle ownership, mandatory authorization and cleanup truth, exact exit classes, and existing machine document bytes.

**Scale/Scope**: One high-priority issue across the global CLI output boundary; deferred IGDB #155 and release publication remain separate.

## Constitution Check

*GATE: passed before research; recheck after design and at Spec-Kit analyze.*

- **P-1/P-2/P-3**: Output handling stays in the CLI and does not change target process access, proxy routing, capture data, or crate dependency direction.
- **P-4/P-9**: Structured counts and diagnostics reflect actual observed values; silence never hides a required action, failure, or selected machine record.
- **P-5/P-7**: Capture and extcap protocols, shell-wrapper stream ownership, and exported JSON artifacts keep their existing readable grammar.
- **P-8**: Rust formatting, source hygiene, UTF-8 without BOM, LF, Markdown source-line rules, and measured human columns remain required.
- **P-11**: Master specification describes the current released baseline separately from S179's unreleased source correction.
- **Workflow**: Specify, clarify, checklist, plan, tasks, analyze, implementation, full local gate, authorized push/official PR, hosted checks, and at most two Codex review rounds precede the owner merge handoff.

## Research Decisions

1. A selected machine stream takes precedence over human silence; otherwise `--json --silent` would produce unexplained empty successes. Human silent hides ordinary success prose and optional warnings while retaining required consent and errors.
2. Handlers serialize domain results directly. Human prose never becomes a string field pretending to be a machine result. Shared code only owns framing and output policy, while command handlers own typed facts.
3. Existing export documents, schema print, build identity, extcap protocol, Doctor checks, and Capture-family event streams remain authoritative. No new envelope or channel relocation is introduced.
4. Schema structural diagnostics expose their known code, pointer, and message; absent source positions are null. Parser syntax errors can carry observed line and column. Each defect is a separate record and a summary closes validation.
5. An unsupported flag combination may refuse with exit 2 before effects only when the existing command cannot preserve its required interaction or protocol; do not blanket-reject command families or discard a previously useful machine mode.
6. No generic stdout sink is installed globally because it could consume interactive prompts, explicit export documents, or analyzer protocol bytes. Gate ordinary result writes where their domain meaning is known.

## Project Structure

### Documentation

```text
specs/179-cli-result-contract/{spec,plan,research,data-model,quickstart,tasks}.md
specs/179-cli-result-contract/contracts/cli-results.md
specs/179-cli-result-contract/checklists/{requirements,output-contract}.md
docs/fragcap-specification.md
docs/plans/README.md
changelog.d/S179-*.fixed.md
```

### Source Code

```text
crates/fragcap-cli/src/{lib,cli,emit}.rs
crates/fragcap-cli/src/commands/{targets,catalog,schema,technologies,doctor}.rs
crates/fragcap-cli/tests/{cli_output_matrix,cli_targets,cli_catalog,cli_schema}.rs
```

**Structure Decision**: Keep command-specific result facts beside the command's domain operation and use one small CLI-local JSON framing helper. Avoid a second application result model in the facade or wrappers.

## Execution Order

1. Add failing controlled matrix tests for the reported target, catalog, schema, Doctor, and Capture cases, plus representative document and protocol preservation.
2. Add CLI-local output policy and framing, then implement typed result branches and human silent gates in affected command handlers. Preserve interactive and machine artifact paths.
3. Update help, master specification, chronological plan, and changelog fragment; verify requirements and output-contract checklists.
4. Run Spec-Kit analyze and resolve findings, execute focused tests and full CI parity in a verified hidden noninteractive console path, check encoding and diff, commit, push, publish the official PR, and monitor both review rounds and hosted checks.

## Complexity Tracking

No constitution deviation, new dependency, storage migration, release marker, or pinned process artifact is planned.
