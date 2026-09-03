# Implementation Plan: Exhaustive Protocol Classification

**Branch**: `codex/120-protocol-classification` | **Date**: 2026-09-03 | **Spec**: [spec.md](spec.md)

## Summary

Add one versioned facade classification contract over native proxy evidence, enumerate every published traffic family and inspectability boundary, preserve raw failure authorities, gate compatibility promotion on explicit proof, and derive application, manifest, and CLI summaries from the same classifications. Close issue #316 without changing forwarding, routing, calibration breadth, or artifact authority.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88

**Dependencies**: Existing standard library, serde_json, and workspace stack; no new dependency planned

**Storage**: Additive classification schema version 1 fields in application JSON Lines version 2, compatibility JSON, and derived bundle summaries; manifest version 2 omission vocabulary remains schema-compatible

**Testing**: Exhaustive classification table tests, invalid-state tests, proxy-to-facade mapping tests, compatibility eligibility tests, application prefix and summary reconciliation, manifest omission tests, CLI human and JSON integration, existing native conformance matrix, full xtask gate

**Platform**: Windows production with portable deterministic tests; no live target, capture driver, elevation, Internet, or trust mutation required

**Constraints**: Raw evidence preserved, unknown distinct from unsupported and failed, parser failure never a compatibility verdict, forwarding independent from retention and writers, one stable serialized vocabulary

**Scope**: Issue #316 only; #317, #318, #319, and #334 remain open

## Constitution Check

- **P-1**: PASS. Classification observes the existing scoped proxy path and adds no routing, process access, trust effect, or interception capability.
- **P-2/P-3**: PASS. Raw proxy types remain in the leaf `fragcap-proxy` crate. The facade owns public classification policy and the CLI only renders derived results.
- **P-4/P-9**: PASS. Detailed evidence and loss remain intact. Unknown, unsupported, failure, truncation, and writer loss cannot be normalized into cleaner claims.
- **P-5/P-8**: PASS. Packet truth is unchanged and application and manifest additions remain readable by existing version-aware consumers. Full formatting, lint, schema, dependency, and analyzer gates apply.
- **P-6**: PASS. Protocol classification, detection state, inspectability state, and outcome reason receive glossary entries in this slice.
- **P-10/P-11**: PASS. Target storage is unchanged and the master specification records only the classification boundary completed by S120.

Post-design check: PASS. The design adds no dependency, privileged effect, artifact owner, alternate routing path, or compatibility-fact store.

## Architecture And Phases

1. Inventory every currently emitted protocol, inspectability, refusal, parser, retention, and writer label and map it to one public classification schema.
2. Add bounded typed classification entities and validation in the facade, with an exhaustive published traffic matrix and stable serialization labels.
3. Convert raw native proxy observations into facade classifications while retaining raw reason and protocol evidence.
4. Emit additive classification objects and schema identity in application JSON Lines and compatibility output, including exact trailer counts.
5. Centralize manifest omission reason and severity construction without changing artifact ownership.
6. Restrict compatibility fact candidates and calibration outcomes to classifications carrying their required direct evidence.
7. Derive human and JSON CLI summary counts from classified observations and prove reconciliation against detailed records and bounded loss.
8. Update the architecture, normative traffic matrix, outline, glossary, plan status, proxy README, AGENTS, issue record, and changelog.
9. Run focused tests, the blocking analysis remediation pass, and the full repository gate.

## Project Structure

```text
specs/120-protocol-classification/
|-- spec.md
|-- plan.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/protocol-classification.md
|-- checklists/{requirements.md,security.md}
`-- tasks.md

crates/fragcap/src/deep_capture/{classification.rs,application.rs,manifest.rs,model.rs,native.rs,policy.rs,mod.rs}
crates/fragcap/tests/{application_stream.rs,deep_capture_session.rs,native_conformance.rs,native_proxy.rs}
crates/fragcap-cli/src/{events.rs,commands/deep_capture.rs}
crates/fragcap-cli/tests/cli_deep_capture.rs
docs/{fragcap-specification.md,fragcap-spec-outline.md,plans/README.md}
docs/glossary/{capture-and-networking.md,index.md}
crates/fragcap-proxy/README.md
AGENTS.md
changelog.d/
```

## Complexity Tracking

No constitution exception is required. The facade classification layer is necessary because raw proxy errors and public product outcomes have different authorities. Reusing free-form strings directly would preserve the ambiguity issue #316 exists to remove, while moving product policy into `fragcap-proxy` would invert the documented dependency boundary.
