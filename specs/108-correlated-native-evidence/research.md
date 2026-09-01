# Research: Correlated Native Evidence

## Deterministic correlation

**Decision**: Preserve one flow id per canonical session flow and add timestamped packet summaries with owner/fidelity segments. Connection-open events carry transport, endpoints, and accepted time. After inputs close, append one ordered correlation record per connection before the trailer.

**Rationale**: The current global closure cannot vary by connection, and live lookup makes late publication schedule-dependent. Interval overlap prevents endpoint reuse from leaking a later owner backward.

**Alternatives considered**: Latest-owner lookup, flow ids per socket incarnation, closest-time guessing, and target handles.

## Controlled evidence

**Decision**: Remove the controlled harness's fabricated flow id. Controlled child identity remains separate provenance and cannot prove packet correlation.

## HAR projection

**Decision**: Add native phase timing. Emit standard entries only when every mandatory HAR value exists. Preserve other transactions under `_fragcapPartialEntries`; attach `_fragcap` provenance and loss to standard entries. Encode retained binary content as base64 and publish atomically from bounded assembly.

**Rationale**: HAR mandates response and timing values. Placeholder values invent measurements; underscore-prefixed extensions are interoperable.

## Manifest version 2

**Decision**: Add one typed facade owner for strict version dispatch, v2 serialization, and semantic validation. Each expected artifact has one declaration with authority, finalization, completeness, loss, sensitivity, content type, and correlation capability. Write v2 only; read v1 conservatively without rewriting.

## Publication and paths

**Decision**: Atomically sync `manifest.prefix.json` before evidence writers. Publish one synced final manifest by rename, then remove the prefix. Centralize canonical relative-path validation. Share export generates a destination manifest instead of copying stale source claims.

## Dependency decision

**Decision**: Add no dependency. Existing serde_json and direct validation follow repository precedent.
