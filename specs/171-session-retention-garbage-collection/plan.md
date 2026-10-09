# Implementation Plan: S171 session retention and garbage collection

**Branch**: `codex/s171-session-retention-garbage-collection` | **Date**: 2026-10-09 | **Spec**: [spec.md](spec.md)

## Summary

Implement #458 through a facade-owned exact collection engine, CLI policy/lease orchestration and scalable read-only residue diagnosis. Default operational contents have declared finite retention; saved/custom and historical output retain the explicit cleanup promise. Routine deletion preserves root containers and leaves durable retirement authority outside them.

## Technical Context

**Language/Version**: Rust workspace, MSRV 1.88.

**Primary Dependencies**: Existing windows-sys, serde_json, BLAKE3, getrandom and standard filesystem primitives; no new lock packages.

**Storage**: Existing versioned manifest/resource/sensitive journals, owner registry and new root-level versioned retirement records. Exact journal/manifest closure and file identity remain distinct from retention selection.

**Testing**: Red-before/green-after synthetic real-filesystem tests, generation leases, deliberate failure injection, CLI help/preview/apply regressions and cargo xtask ci. Never run against owner AppData or real trust state.

**Target Platform**: Windows product, platform-neutral retained-filesystem contracts where supported.

**Performance Goals**: Constant bounded population per bundle (256 objects, depth 8, provenance at most 4 MiB), streaming container scan with visible finite bounds/deadline and independently reserved actionable findings. At least 1,000 healthy/empty containers cannot hide current residue.

**Constraints**: No reparse following, outside aliases, path-based deletion after dropping an identity guard, unknown files or undisclosed retention migration. Automatic hooks exclude the current returned bundle.

**Scale/Scope**: Full #458, including interruption, explicit historical reclamation and empty purge. Dependency PR #470 and releases are separate.

## Constitution Check

Pre-research PASS: P-1 adds no target instrumentation or traffic/trust effects. P-2/P-3 keep platform filesystem adapters in facade, capture/attribution unchanged. P-4/P-9 report planned versus actual reclamation and limits independently. P-5 evidence formats remain analyzer-compatible. P-6 glossary covers introduced lifecycle vocabulary. P-7 CLI orchestrates facade authority. P-8 UTF-8/no BOM/LF, house prose and shared measured terminal rendering. P-10 target/store identity unchanged. P-11 source correction remains unreleased and published baseline unchanged. Post-design PASS: whole-session eligibility and external retirement authority resolve historical retention and partial deletion without relaxing recovery or security gates.

## Phase 0: Research and decisions

[research.md](research.md) consolidates two required research agents. D1 finite policy applies only to newly declared managed history; explicit saved/custom and legacy evidence remain retained. D2 maintenance occurs after an authorized effectful shared Capture/calibration session has finalized and relinquished its lease, excluding current output. D3 all latest external obligations must be Released/NotApplied, while terminal Artifact Retained/Failed/TimedOut is compatible with selected evidence cleanup after writing has ended. D4 strict trailer version/session/count and terminal manifest precede deletion. D5 separate DELETE-capable pins and by-handle disposition are required on Windows. D6 synchronized root-level retirement records retain exact identity, population and progress before any child removal; retries never delete replacements. D7 routine GC preserves root directories and explicit purge handles only proven empty containers. D8 Doctor streams containers without charging healthy/empty populations to actionable finding capacity; scan limitations remain visible.

## Phase 1: Design

[data-model.md](data-model.md), [contracts/collection.md](contracts/collection.md) and [quickstart.md](quickstart.md) define interfaces and validation. CLI must check every matching owner lease and serialize maintenance; facade checks file/population/provenance identities before mutation. Initial/default policy limits: 30 days, 20 completed managed sessions, 2 GiB; oldest eligible first. Explicit include-retained is required for historical contents; custom roots require explicit selection rather than root registration alone. Collection report labels logical file bytes removed, not physical free-disk blocks.

## Project Structure

Facade: `crates/fragcap/src/deep_capture/collection.rs`, optional collection filesystem submodule, `mod.rs`, `artifacts.rs`, controlled `crates/fragcap/tests/session_collection.rs`. CLI: `crates/fragcap-cli/src/session_gc.rs`, `doctor/residue.rs`, `doctor/fix.rs`, `cli.rs`, `commands/bundle.rs`, `commands/deep_capture.rs`, `session_ux.rs`, relevant CLI tests. Documentation: master specification, plans README, glossary, public CLI/storage pages and changelog fragments.

## Coordination

Following the plan skill's research-agent dispatch, independent owning tasks run in parallel after the blocking analyze gate. Collection agent owns facade collection module, Windows deletion helper and collection tests. Registry agent owns session_gc orchestration, registry/Doctor changes and their tests. Root owns CLI declaration/dispatch, shared session integration, consent/retention metadata, documentation and full gates. Shared APIs are agreed before edits; no concurrent edits to one owning file.

## Complexity Tracking

New retirement state is necessary because collection removes its former manifest/journal authority and must survive interruption while preserving an actually empty container. Permission-only handles cannot substitute for exact deletion handles. No new crate or dependency.
