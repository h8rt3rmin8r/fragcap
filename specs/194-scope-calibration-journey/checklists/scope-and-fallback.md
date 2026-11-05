# Scope and Fallback Requirements Checklist

**Purpose**: Review the quality and completeness of S194's scope, migration, fallback, and operator requirements

**Created**: 2026-11-05

## Exact Scope and Migration

- [x] CHK001 Are requested scope, effective scope, middleware handling, and policy identity defined as exact applicability dimensions? [Completeness, Spec §FR-001]
- [x] CHK002 Is legacy migration behavior explicit about preservation, absent dimensions, and inapplicability to game-only cases? [Clarity, Spec §FR-002]
- [x] CHK003 Are inferred recommendations distinguished from measured compatibility facts? [Consistency, Spec §FR-003]
- [x] CHK004 Is scope included in every durable identity and no-progress boundary that can affect resume or retry? [Coverage, Spec §FR-004]
- [x] CHK005 Are the observations eligible to create positive game facts stated exhaustively, including excluded evidence sources? [Completeness, Spec §FR-005]

## Retry and Passive Fallback

- [x] CHK006 Are predecessor preservation, fresh authority, cleanup reconciliation, and the one-successor bound defined for retry? [Completeness, Spec §FR-007]
- [x] CHK007 Are all retry refusal classes specified before effects? [Coverage, Spec §FR-008]
- [x] CHK008 Does packet-only fallback define effective mode, evidence window, omissions, and prohibited positive claims? [Clarity, Spec §FR-009]
- [x] CHK009 Is midstream scope switching expressly prohibited and separated from a future authorized attempt? [Consistency, Spec §FR-010]

## Operator and Evidence Contracts

- [x] CHK010 Is one public scope vocabulary consistent across calibration, Deep Capture, help, and generated commands? [Consistency, Spec §FR-011]
- [x] CHK011 Does pre-effect authorization enumerate every new scope, relay, fallback, lifetime, output, and cleanup effect? [Completeness, Spec §FR-012]
- [x] CHK012 Are human, JSON, quiet, silent, redirected, color, Unicode, and narrow output requirements all covered? [Coverage, Spec §FR-013]
- [x] CHK013 Are Doctor and target-detail readiness dimensions separated from runtime readiness and current obligations? [Clarity, Spec §FR-014]
- [x] CHK014 Are artifact lineage, counters, loss, omissions, paths, and excluded middleware contents specified consistently? [Completeness, Spec §FR-015]
- [x] CHK015 Is package artifact certification separated from post-release installed-product behavior? [Consistency, Spec §FR-017]
- [x] CHK016 Are prohibited instrumentation, routing, storage, and retry expansions explicit? [Security, Spec §FR-018]
