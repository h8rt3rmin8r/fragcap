# Implementation Plan: Deep Capture doctor readiness and cleanup

**Branch**: `codex/218-deep-capture-doctor`

**Issue**: #218

## Summary

Extend `fragcap doctor` so Deep Capture has a visible readiness and cleanup surface before the MVP proxy orchestration lands. The implementation keeps the existing architecture: probes gather raw facts read-only, classifiers are pure, human and JSON output render from the same report, and `doctor --fix` acts only on structured actions carried by findings.

## Technical Context

**Language**: Rust.

**Primary crates**: `fragcap-cli`.

**Dependencies**: No new dependency.

**Testing**: Unit tests over injected doctor facts, CLI golden updates, and workspace gates.

## Constitution Check

- **P-1 No covert target instrumentation**: Pass. The slice opens no target process, injects nothing, and does not alter proxy or trust state during ordinary `doctor`.
- **P-2 Core stays platform-neutral**: Pass. Work is confined to the CLI doctor surface.
- **P-4 No silent loss**: Pass. Sensitive residue and unfinished cleanup become visible rows.
- **P-9 The instrument does not lie**: Pass. Unknown CA/proxy states are reported as unknown or warnings, not as success.
- **P-10 One path to a target**: Pass. The existing doctor probe/check/action path is extended rather than duplicated.

## Scope

In scope:

- Deep Capture doctor fact model.
- Read-only session storage scan.
- Proxy backend availability and version detection for `mitmdump`.
- Analyzer key-log environment readiness.
- CA trust state model and classifier.
- Deep Capture cleanup action for unfinished manifests and known sensitive session sidecars under fragcap-owned storage.
- Human and JSON output.

Out of scope:

- Native proxy backend implementation.
- CA creation or trust installation.
- System-wide proxy configuration.
- Arbitrary deletion outside fragcap-owned session storage.
