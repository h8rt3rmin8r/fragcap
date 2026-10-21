# Implementation Plan: Truthful Capture First-Use Guidance

**Branch**: `codex/s177-capture-first-use-truth` | **Date**: 2026-10-21 | **Spec**: `specs/177-capture-first-use-truth/spec.md`

**Input**: Owner-approved S177 grouping #487, #494, and #495 at the first-use Capture decision boundary.

## Summary

Make Doctor's existing current-token privilege check blocking for live modes, give a verified Steam client-only authoring result its own terminal calibration guidance, and make generated Deep Capture help and runtime grammar agree on mandatory managed launch. Preserve exact-plan input, compatibility evidence, and offline behavior.

## Technical Context

**Language/Version**: Rust 2024, workspace MSRV 1.88.

**Primary Dependencies**: Existing `clap`, `serde_json`, `fragcap`, and `fragcap-cli`; no new package.

**Storage**: Existing `local.db` target row; no schema migration. Help and verdicts are output projections.

**Testing**: Controlled CLI and module tests, `cargo xtask ci`, focused Windows build/tests when available.

**Target Platform**: Windows for actual live Capture; parser, verdict, and store tests remain platform-neutral where possible.

**Project Type**: Native Rust CLI and existing facade.

**Performance Goals**: No extra probe, process launch, network operation, or store read in Doctor. Setup and help add only bounded output.

**Constraints**: Exact LF plan confirmation stays strict; no false compatibility fact; no user-output changes that hide current resource history; no focus-stealing child console.

**Scale/Scope**: One current Doctor invocation or one target's setup-only calibration invocation; generated help at 40, 60, 80, and 100 columns.

## Constitution Check

*GATE: passed before research; recheck after design and at analyze.*

- **P-1**: No new capture, proxy, process-control, or trust technique. The current token is read by the existing probe; setup plans retain their exact authorization boundary.
- **P-2/P-3**: Changes remain in CLI presentation and command orchestration. Core and capture/attribution crate boundaries do not change.
- **P-4/P-9**: Readiness and terminal calibration claims reflect observed facts. Setup must not be relabeled as reached-client evidence.
- **P-6/P-8**: Existing vocabulary and Markdown style are retained; help is tested at narrow widths.
- **P-11**: Master specification, embedded help, controlled tests, and changelog fragment describe the same candidate behavior without a release claim.
- **Workflow**: Specify, clarify, checklist, plan, tasks, analyze, implement, exact-head verification, commit, authorized push/PR, bounded reviews, and CI are required. The user has explicitly authorized push and PR publication, superseding autopilot's ordinary pre-push halt.

## Research Decisions

1. **Doctor verdict**: Promote only the current-token privilege check from warning to failure for live modes. Machine prerequisites retain separate rows; structured `blocking_checks` names `Platform/privilege`. This reuses the existing failure/action/exit pipeline rather than adding another readiness axis that can drift.
2. **Setup-only state**: After the exact target update is read back, emit a distinct `setup-complete` guidance/verdict with durable target id, explicit no-reachability reason, and a PowerShell-quoted same-store rerun command. Other `TargetFrontDoor::Declined` paths keep their existing terminal meanings. No new store state is warranted.
3. **Launch grammar**: Mark `--launch` required in the generated parser and move it to Required inputs. Retain a plain runtime guard for direct internal calls, but the public parser and usage become authoritative.
4. **Examples**: Keep output-recipient as a one-line PowerShell command within an 80-column terminal, with a short placeholder path. Give a separate exact-LF example under calibration help. Do not suggest `.WriteLine()` because PowerShell on Windows emits CRLF.
5. **Sensitive options**: Expand the first-use option descriptions with concrete artifact/credential language while retaining precise advanced terms in long help.

## Project Structure

### Documentation

```text
specs/177-capture-first-use-truth/
  spec.md
  plan.md
  research.md
  data-model.md
  contracts/first-use-results.md
  quickstart.md
  tasks.md
  checklists/{requirements,first-use-contract}.md
docs/fragcap-specification.md
docs/plans/README.md
changelog.d/S177-*.md
```

### Source Code

```text
crates/fragcap-cli/src/doctor/{checks,mod,fix}.rs
crates/fragcap-cli/src/commands/calibrate.rs
crates/fragcap-cli/src/commands/deep_capture.rs
crates/fragcap-cli/src/{cli,workflow_help,session_ux}.rs
crates/fragcap-cli/tests/{cli_doctor,cli_calibrate,cli_help}.rs
```

**Structure Decision**: Reuse current CLI authorities and output types. Do not add a new crate, schema, or generic command-result system (#488).

## Execution Order

1. Write failing controlled assertions for current-token Doctor readiness and exit, setup-only terminal/next-run flow, mandatory launch help, 80-column PowerShell examples, and strict exact-plan input.
2. Implement Doctor classification through the existing check/action path and verify `doctor --fix` recheck semantics.
3. Emit setup-complete calibration guidance only after verified persistence; preserve all refusal branches.
4. Align `clap` launch grammar, refusal wording, and embedded help with accepted commands and sensitive consequences.
5. Update master specification, plan ordering, and changelog; run Spec-Kit analyze and exact-head verification before commit.

## Complexity Tracking

No constitution violation or new dependency is planned.
