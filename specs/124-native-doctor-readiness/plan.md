# Implementation Plan: Native Deep Capture Doctor Readiness

**Branch**: `codex/124-native-doctor-readiness` | **Date**: 2026-09-03 | **Spec**: `specs/124-native-doctor-readiness/spec.md`

## Summary

Replace Doctor's legacy Deep Capture placeholders with one bounded native
runtime inventory, prove live session ownership without PID-reuse ambiguity,
reuse the existing journal recovery authority for exact repairs, and derive
separate Capture and Deep Capture verdicts from one stable ordered check set.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88

**Primary Dependencies**: Existing standard library, serde_json, workspace crates, and the already-resolved Windows API bindings; no new package

**Storage**: Existing bounded session-owner records, resource journals, manifests, and artifact paths beneath the configured session root

**Testing**: Unit readiness matrices, bounded-inventory fixtures, PID-reuse and unrelated-listener cases, recovery integration, CLI output contracts, and xtask gates

**Target Platform**: Windows product runtime; portable controlled tests require no elevation, driver, game, account, trust mutation, or Internet

**Project Type**: Rust workspace with CLI-owned diagnostics over facade-owned native lifecycle authorities

**Performance Goals**: One bounded pass over the session root; no packet, proxy, or launch hot path waits on Doctor inventory work

**Constraints**: Read-only probing creates no state; no process handle, broad deletion, inferred ownership, new recovery policy, new package, or packaging claim

**Scale/Scope**: Issue #321 only; runtime and installed-state diagnostics consumed later by issue #329 packaging validation

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1**: PASS. A generation-specific named synchronization lease proves an
  active fragcap session. Doctor opens no target process and requests no process
  or memory rights.
- **P-2/P-3**: PASS. Existing facade journal and manifest authorities remain
  authoritative. The CLI owns inventory presentation and repair orchestration.
- **P-4/P-9**: PASS. Every scan limit, parse error, unknown version, ambiguous
  endpoint, refusal, and cleanup failure remains a stable visible finding.
- **P-5**: PASS. Packet truth and pcapng are unchanged.
- **P-6/P-8**: PASS. Stable report vocabulary, documentation, specifications,
  and text gates are included.
- **P-10/P-11**: PASS. The read-only path has no target-specific storage side
  effect, and completion language remains bounded to issue #321.

Post-design check: PASS. Ownership proof is session-scoped, the inventory is
finite, recovery is delegated to the existing journal plan, and Deep Capture
readiness includes Capture prerequisites without merging their verdicts.

## Architecture and Phases

1. Replace PID-only registrations with a session-generation lease whose live
   named object is held for the exact session lifetime and whose bounded record
   contains the bundle, PID, and opaque lease identity.
2. Build one pure bounded inventory over owner records, journals, manifests,
   trust facts, endpoints, and declared artifacts. Preserve every limitation.
3. Map each latest resource obligation to one health and recovery finding,
   distinguishing active work, healthy history, stale work, cleanup failure,
   unknown evidence, and unsupported platform state.
4. Feed native inventory findings into Doctor checks and remove the obsolete
   external-backend and orphan-process placeholders.
5. Add explicit Capture and Deep Capture verdict records to human and JSON
   output while retaining the existing overall command exit contract.
6. Route confirmed fixes through the existing exact recovery implementation,
   re-inventory afterward, then complete controlled tests and repository gates.

## Project Structure

```text
specs/124-native-doctor-readiness/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
└── tasks.md
crates/fragcap-cli/src/doctor/
├── mod.rs
├── probe.rs
├── checks.rs
├── residue.rs
└── fix.rs
crates/fragcap-cli/tests/
docs/fragcap-specification.md
docs/fragcap-spec-outline.md
docs/glossary/capture-and-networking.md
docs/plans/README.md
changelog.d/
```

**Structure Decision**: Keep diagnostic policy and presentation in
`fragcap-cli`, consuming the existing public native journal and artifact
contracts. A dedicated `doctor::residue` module centralizes bounded read-only
inventory and session-owner proof so probing and repair cannot drift.

## Complexity Tracking

No constitution violation requires an exception.
