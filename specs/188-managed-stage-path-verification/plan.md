# Implementation Plan: S188 Managed stage path verification

**Branch**: `codex/s188-managed-stage-path-verification` | **Date**: 2026-11-02 | **Spec**: [spec.md](spec.md)

## Summary

Close #547 by supplying a separately verified canonical executable path to the existing managed stage matcher when ETW reports only a basename, preserving S187's exact path and ordered ancestry. Carry specific non-match limitations and the true acquisition stop reason through the composed CLI result. Prove the corrected simple Steam and publisher-chain paths with controlled production capture and analyzer readback.

## Technical Context

**Language/Version**: Rust workspace, MSRV 1.88.
**Primary Dependencies**: Existing Windows ETW watcher, query-only process identity, profile matcher, session gate, Npcap test tier, and TShark; no new runtime package expected.
**Storage**: Existing process trace, pcapng, proxy JSON Lines, manifest, and cleanup artifacts; additive limitation evidence must remain readable by existing readers.
**Testing**: Focused Rust unit and integration tests, controlled Windows live-source regression, `cargo xtask ci`, applicable isolated package gate, exact-head hosted CI.
**Target Platform**: Windows product; pure stage and result decisions remain platform-neutral where practical.
**Project Type**: CLI, facade, process tree, profile matcher, Deep Capture composition.
**Performance Goals**: Query only declared managed stage candidates; no query per packet and no process-object work for unrelated ETW events.
**Constraints**: No memory rights, global basename fallback, command-line path inference, false stage binding, active-workstation UI focus, private field artifact disclosure, or release publication.
**Scale/Scope**: Active issue #547 only; S187 and v0.10.12 are the merged baseline.

## Constitution Check

P-1 permits a named query-only process-object path and creation-time read while prohibiting target memory access; managed shutdown rights do not follow from stage matching. P-2 keeps Windows calls outside `fragcap-core`. P-3 keeps packet attribution separate from process stage proof. P-4 and P-9 require original ETW text, explicit unverified outcomes, and independent packet/proxy facts. P-5 preserves ordinary pcapng compatibility. P-7 keeps CLI presentation thin and session/matcher decisions in their owners. Constitution 1.8.0 bars a default installed-product claim from a redirected controlled fixture. The pre-design gate passes.

## Project Structure

### Documentation

```text
specs/188-managed-stage-path-verification/
  spec.md
  plan.md
  research.md
  data-model.md
  contracts/stage-acquisition.md
  quickstart.md
  tasks.md
  checklists/
```

### Source Code

```text
crates/fragcap-core/src/process/tree.rs
crates/fragcap-profile/src/matching.rs
crates/fragcap/src/session.rs
crates/fragcap/src/deep_capture/process.rs
crates/fragcap-cli/src/orchestrator.rs
crates/fragcap-cli/src/commands/target_resolve.rs
crates/fragcap-cli/src/commands/deep_capture.rs
crates/fragcap-cli/tests/
docs/fragcap-specification.md
docs/plans/README.md
changelog.d/
```

**Structure Decision**: Keep original ETW event text on the existing tree node and attach an optional verified path to that node before matching. The facade supplies query-only event-bound identity only for declared managed candidates. The existing profile matcher and session gate own binding and packet admission, and the CLI only maps the typed capture outcome to presentation.

## Design and Execution Order

1. Write failing basename-only synthesized-profile tests through the real `CaptureSession`, plus wrong-path, later-PID-generation, older-instance, missing-query, and escaped-parent cases. Cover the ordered publisher chain and confirm the existing full-path path remains valid.
2. Add optional verified path to process-tree nodes without altering `ProcessEvent::Started.image`. Use a narrow path-aware event fold and the shared matcher so profile and session decisions remain one implementation.
3. Build the managed event identity join using `ProcessIdentity::observe` with only reviewed query rights, candidate-name filtering, same-account/session and canonical basename checks, and directional bounded creation time. Return a bounded reason when proof is unavailable and keep control receipts separate.
4. Add a controlled isolated Windows production-path case that runs the ordinary live orchestrator and scoped packet gate with a synthetic Steam chain and real loopback traffic, then reads the pcapng with unmodified TShark. Include declared publisher stages and adverse identity cases without private title data.
5. Preserve typed `CaptureOutcome.stop_reason` on the no-acquisition live branch and carry it into Deep Capture's human and structured top-level error. Add compositional timeout tests that reconcile partial packet, proxy, manifest, and cleanup facts.
6. Update the master specification, slice ordering, glossary if new terms are introduced, and changelog fragments. Run Spec-Kit analysis as a blocking gate, focused tests, foreground `cargo xtask ci`, isolated Windows gate, exact-head CI, and UTF-8/mojibake checks. Commit, automatically push, open the official PR with `Closes #547`, address every review thread with at most one second `@Codex` request, and hand the green PR to the owner for merge.

## Post-Design Constitution Recheck

Pass, subject to implementation and controlled evidence. The query-only join supplies path evidence without new target control, raw ETW observations remain unmodified, and ambiguity cannot create a stage match. Controlled Windows traffic tests the actual packet gate while preserving private installed-product evidence boundaries.
