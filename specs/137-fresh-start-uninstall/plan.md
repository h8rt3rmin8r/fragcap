# Implementation Plan: Explicit Fresh-Start Uninstall

**Branch**: `codex/137-fresh-start-uninstall` | **Date**: 2026-09-09 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/137-fresh-start-uninstall/spec.md`

## Summary

Close issue #377 with a product-owned fresh-start command that inventories exact canonical user roots, previews a digest-bound destructive plan, reuses Doctor's Deep Capture recovery authority, deletes without following reparse points, and emits a truthful bounded report. The WiX uninstall flow adds an unchecked current-user choice and passes the initiating user's exact roots to that command. Silent removal requires two explicit properties. All-users cleanup remains a separate administrator preview-and-confirm flow because static WiX v3 UI cannot display a verified dynamic multi-profile inventory.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.88; WiX v3 XML; PowerShell 7 package harness

**Primary Dependencies**: Existing `fragcap-cli`, `fragcap::deep_capture` recovery API, `blake3`, `serde_json`, Windows registry and filesystem APIs already available through `windows-sys`; no new dependency or lockfile package

**Storage**: Canonical per-user `%APPDATA%\fragcap` and `%LOCALAPPDATA%\fragcap` trees; Deep Capture journals and session-owner registry; no schema migration

**Testing**: Rust unit and CLI contract tests over isolated roots, WiX/static xtask assertions, real Windows package certification lifecycle, full workspace and repository gates

**Target Platform**: Windows 10 and 11 x86_64 installer and CLI; deterministic unsupported behavior elsewhere

**Project Type**: Rust workspace CLI plus Windows MSI and certification harness

**Performance Goals**: Linear bounded traversal; one inventory pass for preview and one verified pass before execution; finite installer child execution under the existing 10-minute lifecycle bound

**Constraints**: Preserve by default, exact initiating-user roots, no recursive name search, no link following, preview-bound all-users authority, recovery before session deletion, truthful partial results, hidden child processes, no new runtime dependency

**Scale/Scope**: Two canonical roots per profile, bounded report, current-user MSI journey, separate administrative multi-profile journey, package certification and documentation synchronization

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **P-1 No covert target instrumentation**: Pass. S137 inventories product-owned files and replays only previously recorded exact cleanup obligations after explicit destructive consent.
- **P-2 Core stays platform-neutral**: Pass. Windows profile discovery and installer integration remain in `fragcap-cli`; no platform code enters `fragcap-core`.
- **P-3 Capture and attribution stay separate**: Pass. Neither boundary changes.
- **P-4 No silent loss**: Pass. Every deletion, retention, refusal, and failure is reported; partial cleanup cannot become success.
- **P-5 Compatibility outranks richness**: Pass. Ordinary uninstall remains byte-preserving and the new command is additive.
- **P-6 Glossary first**: Pass. Fresh start, canonical data root, reparse point, and recovery authority are defined in the S137 artifacts and synchronized documentation.
- **P-7 Wrappers stay thin**: Pass. WiX passes exact consent and roots to the Rust authority; it does not duplicate discovery or cleanup logic.
- **P-8 House standards apply**: Pass. UTF-8 without BOM, LF, format, lint, tests, docs, changelog, and script checks remain blocking.
- **P-9 The instrument does not lie**: Pass. Changed inventories refuse, excluded paths remain excluded, and incomplete deletion produces an exact partial report.
- **P-10 One path to a target**: Pass. Target resolution is unchanged; canonical data discovery has one shared implementation.
- **P-11 The specification describes what shipped**: Pass. Master specification, outline, roadmap, installer guidance, package certification, and changelog move with the behavior.

Post-design re-check: passed. The design keeps the MSI a thin consent and identity adapter, centralizes inventory and deletion, and invokes the existing Doctor recovery function rather than creating a second trust or journal interpreter. The static MSI deliberately omits an unsafe all-users checkbox and directs administrators to the preview-bound command. No constitution exception is introduced.

## Project Structure

### Documentation (this feature)

```text
specs/137-fresh-start-uninstall/
|-- spec.md
|-- plan.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   `-- fresh-start-cli.md
|-- checklists/
|   |-- security.md
|   `-- specification.md
`-- tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/
|-- cli.rs                             # additive fresh-start command arguments
|-- lib.rs                             # command dispatch
|-- paths.rs                           # exact canonical data-root resolution
|-- commands/
|   |-- fresh_start.rs                 # inventory, preview binding, cleanup, report
|   `-- mod.rs
`-- doctor/
    `-- fix.rs                         # shared exact recovery entry point

crates/fragcap-cli/tests/
`-- cli_fresh_start.rs                 # command contract and isolated-root coverage

crates/fragcap-cli/wix/
`-- main.wxs                           # unchecked current-user uninstall consent

scripts/
`-- Test-PackageCertification.ps1      # preserve, wipe, failure, and reinstall cases

docs/
|-- fragcap-specification.md
|-- fragcap-spec-outline.md
|-- maintainers/package-certification.md
`-- plans/README.md

site/content/docs/
`-- getting-started.mdx                # uninstall scope and recovery guidance

changelog.d/
|-- 377-fresh-start-uninstall.added.md
`-- s137-fresh-start-uninstall.decisions.md
```

**Structure Decision**: Put the destructive authority in one CLI command module so installer, operator, and tests consume identical root validation, inventory, recovery, deletion, and reporting. Expose only a narrow recovery entry point from Doctor's existing fix module. Keep WiX responsible for unchecked consent, lifecycle conditions, and initiating-user path transfer.

## Implementation Sequence

1. Add failing CLI and module tests for preserve-by-default parsing, exact root inventory, digest stability, changed-plan refusal, exclusions, reparse containment, recovery failure, partial deletion, and clean state.
2. Implement canonical root resolution, typed inventory, versioned digest, preview output, exact confirmation, safe recursive deletion, and bounded cleanup reporting.
3. Expose and reuse Doctor's exact recovery path before any canonical sessions entry can be removed.
4. Add the WiX maintenance dialog, exact current-user properties, guarded immediate and deferred actions, silent opt-in contract, and explicit administrative-flow guidance.
5. Extend package certification and static assertions for ordinary preserve behavior, confirmed current-user wipe, stale-property exclusion, reparse and recovery failures, all-users preview binding, and clean reinstall.
6. Reconcile master specification, outline, roadmap, package guidance, installation documentation, changelog, decisions, and quickstart.
7. Run spec analysis, focused tests, full CI parity, dependency stability, encoding and mojibake checks, package static checks, and final diff review.

## Complexity Tracking

No constitution violations require justification.
