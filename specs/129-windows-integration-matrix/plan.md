# Implementation Plan: Native Windows Integration Matrix

**Branch**: `codex/129-windows-integration-matrix` | **Date**: 2026-09-04 | **Spec**: `specs/129-windows-integration-matrix/spec.md`

**Input**: Feature specification from `specs/129-windows-integration-matrix/spec.md`

## Summary

Create one versioned closed Windows completion registry, validate its coverage and source evidence in the ordinary gate, execute finite hosted and physical Windows tiers against a staged production binary, reconcile all owned effects, retain only denylist-validated public-safe evidence, and wire a required pull-request workflow without absorbing final package certification.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88, pinned product toolchain 1.96.0

**Primary Dependencies**: Existing workspace graph only, including `serde_json`, `windows-sys`, product Deep Capture crates, and standard-library process/network/filesystem APIs

**Storage**: Versioned JSON registry and sanitized reference summary in the repository; append-safe raw report and scratch bundles under ignored `target/`

**Testing**: Registry mutation tests, report and sanitization tests, hosted Windows integration, explicit physical-host integration, existing protocol/conformance/failure/performance authorities, full `cargo xtask ci`

**Target Platform**: Supported Windows 11 x86-64 product host; portable static validation remains cross-platform

**Project Type**: Rust workspace, CLI task runner, Windows integration tests, and GitHub Actions gate

**Performance Goals**: Required hosted tier below 30 minutes, finite per-row deadlines, bounded child output and report sizes

**Constraints**: Loopback and synthetic targets only; no automatic elevation, install, scheduled task, service, firewall mutation, system proxy mutation, or external traffic; Npcap is never redistributed; raw evidence is never uploaded

**Scale/Scope**: One closed registry spanning all #327 Windows domains, two complementary execution tiers, one staged production executable identity, and one release-consumable physical summary

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1**: PASS. All traffic is local and authorized. The matrix adds no target instrumentation, system proxy, firewall rule, silent trust, process-memory handle, or key extraction. Current-user test trust is explicit and exactly removed.
- **P-2/P-3**: PASS. Registry/orchestration lives in `xtask`; narrow integration probes depend on public facade and leaf APIs. Core, capture-source, and attribution boundaries remain unchanged.
- **P-4/P-9**: PASS. Required rows cannot skip, raw failed prefixes remain incomplete, every effect is inventoried, and summaries preserve exact typed failure facts without inventing success.
- **P-5**: PASS. Analyzer rows require unmodified tooling and no output schema change.
- **P-6/P-8**: PASS. New completion-matrix vocabulary receives glossary entries, and all Rust/Markdown/workflow files remain linted.
- **P-10/P-11**: PASS. Target storage is unchanged. S129 records only the Windows integration boundary and leaves packaging plus final completion claims to #329 and #334.
- **Licensing**: PASS. Npcap SDK is a temporary build input; no SDK or runtime component enters repository or uploaded evidence.
- **Pinned artifacts**: PASS with a required dated decision. The new Windows workflow is the explicit deliverable of #327 and contains no schedule.

Post-design check: PASS. Hosted and physical evidence stay distinct, all effects use existing production owners, the staged-layout boundary removes the #327/#329 cycle, and final package certification remains explicit future work.

## Architecture and Phases

1. Freeze closed registry and report contracts, including the hosted/physical split and package handoff.
2. Add red static validator tests for schema, row identity, source coverage, workflow enforcement, report reconciliation, evidence currency, and publication hygiene.
3. Add narrowly missing Windows probes for staged binary, authority denial, current-user trust, Npcap states, loopback family, analyzer/key-log, and residue.
4. Implement finite hidden child orchestration, immutable preflight, exact row outcomes, append-safe reports, and closed public summaries.
5. Add the required Windows workflow and integrate static validation into `cargo xtask ci`.
6. Run hosted-equivalent and physical tiers on the authorized development host, preserve the validated summary, and reconcile all effects.
7. Update architecture, glossary, release notes, and issue evidence; converge and run the complete repository gate.

## Project Structure

```text
specs/129-windows-integration-matrix/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
├── checklists/
└── tasks.md
integration/
├── windows-native-matrix-v1.json
└── windows-native-reference-v1.json
crates/fragcap-cli/tests/
└── windows_native_integration.rs
xtask/src/
├── main.rs
└── windows_integration.rs
.github/workflows/
└── windows-integration.yml
docs/glossary/
docs/fragcap-specification.md
docs/fragcap-spec-outline.md
docs/plans/README.md
AGENTS.md
changelog.d/
```

**Structure Decision**: Keep the execution and evidence authority in the existing task runner, place only production-facing Windows probes beside the CLI integration suite, and store the reviewed registry plus sanitized reference summary in a dedicated integration directory. This reuses shipped APIs and prevents test orchestration from entering the product graph.

## Complexity Tracking

No constitution violation requires an exception.
