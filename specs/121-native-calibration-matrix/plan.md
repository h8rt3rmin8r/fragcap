# Implementation Plan: Complete Native Calibration Matrix

**Branch**: `codex/121-native-calibration-matrix` | **Date**: 2026-09-03 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/121-native-calibration-matrix/spec.md`

## Summary

Close issue #317 by making calibration evidence applicable to one exact native case. Extend the existing append-only target fact table with closed routing-strategy, loopback-family, and protocol-family columns; preserve all older rows as legacy-incomplete evidence; centralize applicability and latest-row selection in `fragcap-targets`; and pass the prepared native case through facade policy, CLI planning, artifacts, events, target detail, and controlled conformance. Existing reachability and TLS phases remain the two bounded operations. Reachability may observe any selected protocol without trust, while TLS adds explicit trust evidence for the selected protocol. Positive protocol facts come only from S120 classifications matching the selected family.

## Technical Context

**Language/Version**: Rust 1.88 workspace MSRV

**Primary Dependencies**: Existing workspace crates only (`rusqlite`, `serde_json`, native Deep Capture stack); no new dependency or lockfile package

**Storage**: Existing local SQLite target store, schema version 10 through one additive v9-to-v10 migration

**Testing**: Rust unit, integration, CLI, migration, controlled native matrix, `cargo xtask ci`

**Target Platform**: Windows production path; platform-neutral target-store model and offline controlled tests where already supported

**Project Type**: Rust workspace library and CLI

**Performance Goals**: Applicability selection remains linear in one target's bounded local fact history and adds no packet-path work

**Constraints**: Append-only evidence, exact scoped authorization, no age-based staleness, no system proxy changes, no real trust mutation in controlled tests, no dependency addition, no #318 bypass implementation

**Scale/Scope**: Three launch cases, one implemented routing strategy, two loopback families, the complete S120 native protocol-family vocabulary, two calibration phases, and every existing compatibility row

## Constitution Check

- **P-1, no covert target instrumentation**: PASS. Calibration remains explicit, target-scoped, plan-visible, confirmed, finite, reversible, and auditable. No prohibited process or network technique is added.
- **P-2 and P-3, architecture boundaries**: PASS. SQLite compatibility vocabulary remains in `fragcap-targets`; session orchestration remains in the facade; the CLI remains an adapter. Capture and attribution remain separate.
- **P-4 and P-9, loss and truth**: PASS. Existing rows survive migration, conflicts remain append-only, mismatches remain visible, losses remain counted, and silence creates no positive fact.
- **P-5, compatibility**: PASS. Existing pcapng and raw proxy formats do not change. Additive JSON fields and schema migration preserve older evidence.
- **P-6, glossary**: PASS. New durable terms receive entries in the same change.
- **P-7 and P-8, thin wrappers and standards**: PASS. No wrapper logic changes; all changed text follows repository conventions.
- **P-10, one target path**: PASS. Facts remain attached to the existing `targets(id)` row and use no parallel resolver.
- **P-11, specification truth**: PASS. The master specification and outline will record the shipped S121 contract while retaining #318 and #334 as open.

Post-design recheck: PASS. The selected additive schema and shared applicability authority satisfy every gate without a deviation.

## Project Structure

### Documentation (this feature)

```text
specs/121-native-calibration-matrix/
|-- spec.md
|-- plan.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- checklists/
|   |-- requirements.md
|   `-- security.md
|-- contracts/
|   |-- calibration-case.md
|   `-- compatibility-store-v10.md
`-- tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-targets/src/
|-- compatibility.rs
|-- schema.rs
`-- store.rs

crates/fragcap/src/deep_capture/
|-- model.rs
|-- policy.rs
|-- session.rs
`-- manifest.rs

crates/fragcap-cli/src/
|-- cli.rs
|-- events.rs
`-- commands/
    |-- deep_capture.rs
    `-- targets.rs

crates/fragcap-cli/tests/
`-- cli_deep_capture.rs

crates/fragcap/tests/
|-- deep_capture_session.rs
`-- native_conformance.rs
```

**Structure Decision**: Extend the three existing ownership layers. `fragcap-targets` owns stored case vocabulary, migration, applicability, and ordering. The facade owns observation-to-fact policy and session authority. The CLI owns argument mapping and presentation only. This avoids a second compatibility model and keeps the exact-match rule reusable by ordinary Deep Capture and target detail.

## Implementation Phases

1. Add failing target-store model, migration, append-only, and applicability tests.
2. Implement schema version 10 and the shared exact-case vocabulary and selector.
3. Add failing facade policy tests for protocol filtering and exact current prerequisites, then carry case identity through the prepared session and fact candidates.
4. Add failing CLI and artifact tests, then expose selected protocol and complete case identity through plans, events, persistence, bundles, manifests, and target detail.
5. Close the controlled IPv4/IPv6 protocol matrix, update specifications and glossary, run analysis again, and execute the full repository gate.

## Decision Log

### 2026-09-03: Additive nullable columns distinguish legacy absence from explicit inapplicability

New rows write closed tokens for routing strategy, address family, and protocol. Routing facts use the explicit `not-applicable` protocol token. Migrated rows retain NULL, which means legacy-incomplete rather than a fabricated default. Rebuilding the table or backfilling inferred values was rejected because both risk altering historical evidence.

### 2026-09-03: Applicability is fact-class aware

Launch case, backend identity and version, routing strategy, address family, fragcap version, and known target version apply to routing prerequisites. Protocol behavior, inspectability, and trust additionally require an exact protocol. A dimension may be ignored only when the fact class declares it inapplicable. Comparing every field indiscriminately was rejected because it would force a fake protocol onto protocol-independent routing evidence.

### 2026-09-03: Preserve the two-phase command and add an explicit selected protocol

The established reachability and TLS phases remain. Calibration adds one required protocol selection, with `routing` used for route-only reachability and S120 family tokens used for protocol cases. This keeps lifecycle and cleanup authority stable while making every protocol case addressable. Adding a third orchestration phase was rejected because it duplicates existing bounded observation behavior without a distinct effect class.

### 2026-09-03: No elapsed-time expiration

Rows become ineligible through explicit stale state or case mismatch. Retesting appends a new row and latest current applicable evidence governs. A global age threshold was rejected because no evidence supports one interval across game updates, backend changes, and protocol families.

## Complexity Tracking

No constitution violation or exceptional complexity is required.
