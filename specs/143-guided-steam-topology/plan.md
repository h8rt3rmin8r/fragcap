# Implementation Plan: Guided Steam Client Setup

**Branch**: `codex/s143-guided-steam-topology` | **Date**: 2026-09-10 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/143-guided-steam-topology/spec.md`

## Summary

Extend `fragcap calibrate` with one Steam-only authoring boundary after target resolution and before the S139 proposal. An exact `steam:<appid>` target whose `launch_entries` field is absent may be joined to one fresh S142 discovery candidate with the same application identifier and exact install root. A complete domain-separated plan shows the first appinfo launch executable only as a proposal and asks the operator to attest that it is the socket-holding client. Human input defaults to decline and structured input returns the exact plan identifier. After confirmation, the command re-reads both authorities, rebuilds the plan, and calls a new facade-exported conditional target-store operation that acquires an immediate write transaction, compares the complete stored row, and changes only `launch_entries` plus fidelity. The target is then re-resolved by stable identifier and enters the unchanged S139-S142 path with a separately authorized session if one is selected.

## Technical Context

**Language/Version**: Rust 1.88 workspace MSRV

**Primary Dependencies**: Existing `rusqlite`, `serde_json`, `blake3`, `subtle`, `fragcap` facade, `fragcap-targets`, and S139-S142 guided calibration stack

**Storage**: Existing per-user SQLite target store version 10; one conditional update to an existing target row; no migration, new table, workflow record, or compatibility-fact change

**Testing**: Target-store unit tests, plan and input unit tests, CLI event tests, controlled Steam discovery integration tests, existing S139-S142 suites, and `cargo xtask ci`

**Target Platform**: Windows production Steam discovery and calibration path, with store policy and controlled fixtures testable on all CI hosts

**Project Type**: Rust workspace facade plus command-line binary

**Performance Goals**: Ineligible and already-declared targets add no discovery pass; eligible setup performs at most two bounded discovery passes, one immediate transaction, and one target re-read; one calibration session at most

**Constraints**: Feature-branch pull-request workflow; UTF-8 without BOM; LF; 100-column Rust; no new package or schema; no process handle; no process control; no topology observation launch; no hidden trust; no system proxy; no pinning bypass; no target TLS key extraction; no existing declaration overwrite; no non-Steam topology; no multi-attempt loop or workflow persistence

**Scale/Scope**: One conditional store operation and outcome type, one CLI plan and decision boundary, two additive event variants, controlled drift fixtures, focused tests, architecture/reference updates, and two changelog fragments

## Constitution Check

*GATE: Passed before research and passed again after design.*

- **P-1**: PASS. Setup reads local metadata and updates local target state only. No target process access, launch, proxy, trust, capture, or prohibited technique is added.
- **P-2**: PASS. SQLite ownership stays in `fragcap-targets`, orchestration stays in `fragcap-cli`, and `fragcap-core` remains unchanged.
- **P-3**: PASS. Capture and attribution seams are untouched.
- **P-4**: PASS. Discovery conservation and warnings are plan-bound; every input, drift, missing, changed, and failure outcome is explicit.
- **P-5**: PASS. Capture artifacts and analyzer compatibility are unchanged.
- **P-6**: PASS. Existing target, launch declaration, client, plan, and calibration vocabulary is reused; no new glossary term is introduced.
- **P-7**: PASS. No wrapper change is required.
- **P-8**: PASS. TDD, requirements and security checklists, analysis, convergence, full repository verification, encoding checks, and review convergence are required.
- **P-9**: PASS. Appinfo remains proposal evidence. Only the operator's positive assertion produces authored client authority, and existing or changed authority is never hidden or overwritten.
- **P-10**: PASS. S133 discovery supplies the candidate, the existing target row and storage shape remain authoritative, and the conditional update lives in the shared target store rather than a Steam-specific database path.
- **P-11**: PASS. Architecture documents will describe only the bounded authored-client setup and keep #380 open.
- **Third-party obligations**: PASS. No dependency, Npcap, license, redistribution, packaging, or external service change occurs.

## Project Structure

### Documentation (this feature)

```text
specs/143-guided-steam-topology/
├── checklists/
├── contracts/calibrate-steam-client-command.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-targets/
└── src/
    ├── lib.rs
    └── store.rs

crates/fragcap/
└── src/lib.rs

crates/fragcap-cli/
├── src/
│   ├── commands/calibrate.rs
│   └── events.rs
└── tests/
    ├── cli_calibrate.rs
    └── cli_reference.rs

docs/
├── fragcap-specification.md
├── fragcap-spec-outline.md
└── plans/README.md
```

**Structure Decision**: Put exact transactional mutation and its typed result in `fragcap-targets::Store`, export the result through the existing facade target module, and keep candidate acquisition, plan presentation, sequential input, revalidation, and durable handoff in `fragcap-cli`. The Steam crate and candidate model remain unchanged because the existing first appinfo executable is sufficient as a human-reviewed proposal but insufficient as authority by itself.

## Phase 0: Research Decisions

### 2026-09-10: Preserve every present launch declaration

Only `launch_entries == None` is eligible. An empty array, malformed value, or prior `no` or `unsure` answer is existing authority or evidence that needs an explicit repair workflow. Replacing it here would make the convenience path an undocumented editor.

### 2026-09-10: Treat the appinfo executable as one proposal, never a conclusion

Steam appinfo identifies an executable Steam invokes, and the current discovery model deliberately retains the first such value only as `executable_hint`. The plan labels that source and asks whether this exact executable holds the sockets. Only an affirmative response creates the resolved client shape. A negative, unsure, invalid, or absent answer changes nothing.

### 2026-09-10: Require exact target and install-authority joins

The target's canonical positive Steam anchor selects exactly one candidate by application identifier. Both stored and discovered install roots must exist and compare exactly. This avoids silently reconciling a moved install or joining metadata from a different library copy.

### 2026-09-10: Bind the complete source and proposed result

The plan uses deterministic canonical JSON and a `steam-client-setup-v1:` identifier. It binds the complete stored target, complete candidate, conserved discovery account and warnings, effective store path, exact proposed executable, resolved-client result, authoring operation version, and explicit no-effect list. Reusing the registration or Deep Capture identifier would merge distinct authorities.

### 2026-09-10: Rebuild the plan before an immediate conditional update

The CLI re-reads the target and repeats discovery after input. The store operation then starts an immediate transaction, reads the current row inside that transaction, compares it to the planned row, and updates only launch declaration plus fidelity when equal. This closes both the metadata-review gap and the final concurrent-writer gap.

### 2026-09-10: Stamp authored fidelity while preserving all other fields

The operator is asserting socket-holder identity, the same decision `targets add --exe` records as authored. The conditional operation writes the existing resolved-client shape and raises fidelity to `authored`, but retains classification source, provenance, evidence, and every other field. It does not claim appinfo observed network ownership.

### 2026-09-10: Continue by stable identifier with independent authorizations

After the update, a fresh stable-id lookup must return the authored row before the S139 proposal is rebuilt. Registration, Steam setup, and any later session each emit their own plan and consume their own complete input line.

## Phase 1: Design

The command contract is defined in [contracts/calibrate-steam-client-command.md](contracts/calibrate-steam-client-command.md), transient and persistent transitions in [data-model.md](data-model.md), and executable validation in [quickstart.md](quickstart.md). No entity requires a storage migration or dependency change.

## Complexity Tracking

No constitution violation or exceptional complexity is required.
