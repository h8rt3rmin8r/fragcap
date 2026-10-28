# Implementation Plan: S182 Installed-product recovery

**Branch**: `codex/s182-installed-product-recovery` | **Date**: 2026-10-27 | **Spec**: [spec.md](spec.md)

## Summary

Repair the eight source and evidence defects found during the v0.10.8 installed-product audit. Keep the existing target store, Capture pipeline, native proxy, managed-launch authority, and package formats. Introduce exact relocation recovery, time- and cause-scoped packet omissions, consistent observation deadlines, exact owner shutdown with passive helper observation, final evidence reconciliation, portable Windows startup, current physical reference authority, and version-matched operator instructions.

## Technical Context

**Language/Version**: Rust workspace with the root MSRV; PowerShell release automation; MDX documentation.
**Primary Dependencies**: Existing workspace crates, Npcap runtime, Windows IP Helper and ETW, native Rust TLS/proxy, SQLite targets store, WiX packaging. No new runtime crate planned.
**Storage**: Existing SQLite target/calibration stores; pcapng, JSON Lines, manifest, cleanup artifacts. Add evidence fields only where needed.
**Testing**: Cargo tests, `cargo xtask` gate, native protocol matrix, physical Windows integration, MSI/ZIP certification, advisory checks, continuous two-hour native proxy soak.
**Target Platform**: Windows x64 MSVC. Controlled source tests can run without a driver.
**Project Type**: CLI and library workspace with MSI and portable distribution.
**Performance Goals**: Bounded packet/process evidence memory, finite stage/cleanup deadlines, existing loss accounting, passing soak validator.
**Constraints**: No target memory rights or unproven process control; no fabricated readiness or zero metrics; no real-title identity in public artifacts.
**Scale/Scope**: Eight existing issues, four user stories, runtime/CLI, package/release harness, same-version documentation.

## Constitution Check

*Gate before research and after design: PASS with the contracts below.*

- P-1: Explicit scoped Deep Capture and exact created-process object validation remain. Relocation revalidates target authority; helpers receive no unproven control.
- P-4 and P-9: Omission cause, interval, counters, incomplete artifacts, deadlines, and cleanup remain independent facts. Unknown evidence never becomes zero or a positive calibration fact.
- P-3 and P-10: Each change stays with its owning crate and the sole production target/proxy paths.
- P-5 and P-11: Same-version docs, package imports, physical evidence, and release provenance match delivered behavior. Candidate checks do not claim published behavior.
- Governance: Repository-controlled issues close on controlled gates; later field failures become versioned defects. No merge, tag, or release in S182.

## Design Phases

1. Record data, CLI, evidence, package, and qualification contracts; add focused failing regressions.
2. Repair target relocation, observation/correlation, managed shutdown, and terminal reconciliation.
3. Repair Windows package and physical reference; update operator pages after behavior lands.
4. Run source, protocol, physical, clean package, advisory, and soak gates; reconcile issues and PR review.

## Project Structure

```text
specs/182-installed-product-recovery/    # spec, plan, research, model, contracts, tasks
crates/fragcap-targets/src/               # target identity, relocation, store
crates/fragcap-core/src/                  # packet omission authority
crates/fragcap/src/deep_capture/          # correlation and proxy lifecycle
crates/fragcap-cli/src/commands/          # calibration, targets, shutdown
crates/fragcap-cli/tests/                 # CLI and physical regressions
integration/ and xtask/src/               # package and Windows qualification
.github/workflows/ and site/content/docs/ # release build and operator pages
docs/plans/README.md and changelog.d/     # chronological index and change record
```

**Structure Decision**: Extend existing crate ownership and versioned contracts; add no alternate capture route.

## Complexity Tracking

No constitution exception. The eight issue scope is owner approved because the findings block one installed-product calibration and Deep Capture path.
