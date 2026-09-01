# Implementation Plan: Correlated Native Evidence

**Branch**: `codex/108-correlated-native-evidence` | **Date**: 2026-09-01 | **Spec**: `specs/108-correlated-native-evidence/spec.md`

## Summary

Replace schedule-dependent proxy correlation with final interval reconciliation against timestamped packet-flow history, project evidence-complete HTTP transactions into HAR 1.2 while retaining partial transactions in a namespaced extension, and move manifest ownership into the facade as a typed version 2 contract with version 1 reading and atomic crash-prefix publication.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88

**Primary Dependencies**: Existing standard library, serde_json, base64, and workspace crates; no new package

**Storage**: Append-only application JSON Lines, bounded transaction assembly, atomic HAR and manifest files, durable manifest prefix

**Testing**: Unit, permutation, fault-injection, schema drift, round-trip, CLI integration, controlled lab, and xtask gates

**Target Platform**: Windows product runtime; portable tests need no elevation, driver, game, account, or Internet

**Project Type**: Rust workspace with libraries, facade, and CLI

**Performance Goals**: Capture never waits on correlation; forwarding is independent of retention; memory is bounded; final joins are deterministically ordered

**Constraints**: No target handles, guessed ownership, placeholder HAR values, silent loss, v1 rewrite, unsafe paths, completion claim, or new package

**Scale/Scope**: Issues #303, #302, and #335; current native HTTP/TLS protocols only; later transports are representable but not implemented

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1**: PASS. Correlation uses packet and proxy evidence only.
- **P-2/P-3**: PASS. Flow truth stays in core, connection facts in the proxy leaf, and joins and artifacts in the facade.
- **P-4/P-9**: PASS. Loss and uncertainty reconcile without invented values.
- **P-5**: PASS. Packet truth remains pcapng and standard readers remain supported.
- **P-6/P-8**: PASS. Vocabulary and repository gates are updated.
- **P-10/P-11**: PASS. Target storage is untouched and shipped-state prose remains honest.

Post-design check: PASS. Final reconciliation corrects the global closure and fabricated controlled flow id while preserving live streaming.

## Architecture and Phases

1. Add timestamped flow history and accepted-connection identity.
2. Append deterministic connection reconciliation and accounting before the application trailer.
3. Add proxy-owned phase timing and bounded HTTP transaction assembly.
4. Emit standard HAR entries only when mandatory facts exist and retain partials in `_fragcapPartialEntries`.
5. Add typed manifest v1 reading and v2 writing, validation, schema, and atomic prefix-to-final publication.
6. Integrate CLI bundle, cleanup, export, doctor, documentation, and gates.

## Project Structure

```text
specs/108-correlated-native-evidence/
crates/fragcap-core/src/flow.rs
crates/fragcap-proxy/src/application.rs
crates/fragcap-proxy/src/http1.rs
crates/fragcap-proxy/src/http2.rs
crates/fragcap/src/deep_capture/application.rs
crates/fragcap/src/deep_capture/har.rs
crates/fragcap/src/deep_capture/manifest.rs
crates/fragcap/src/deep_capture/native.rs
crates/fragcap-cli/src/commands/deep_capture.rs
crates/fragcap/assets/deep-capture-manifest.v2.schema.json
docs/schema/deep-capture-manifest.v2.json
```

**Structure Decision**: Extend existing boundaries and move durable manifest and HAR semantics from CLI presentation into the facade. Add no crate or reverse dependency.

## Complexity Tracking

No constitution violation requires an exception.
