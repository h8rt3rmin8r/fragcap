# Implementation Plan: Proxy Bypass and Local-Destination Policy

**Branch**: `codex/122-proxy-bypass-policy` | **Date**: 2026-09-03 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/122-proxy-bypass-policy/spec.md`

## Summary

Close issue #318 by replacing the untyped empty `NO_PROXY` effect with one immutable facade-owned bypass policy. Parse a closed conventional rule grammar, canonicalize and order it, match requested authorities without DNS inference, add exact session infrastructure exclusions only after endpoint selection, and project the reviewed policy into every uppercase and lowercase managed-child proxy variable. Keep the native proxy's resolved-address policy authoritative for local/private denial and controlled-origin grants. Extend existing plan and bundle authorities with canonical policy and conserved routing-decision evidence, where intentional bypass is scope rather than proxy loss.

## Technical Context

**Language/Version**: Rust 1.88 workspace MSRV

**Primary Dependencies**: Existing standard library and workspace crates only; no new dependency or lockfile package

**Storage**: Existing session plan and bundle JSON artifacts; no durable database migration

**Testing**: Rust unit, facade integration, CLI parsing and controlled Deep Capture tests, `cargo xtask ci`

**Target Platform**: Windows production path; platform-neutral parser and policy tests

**Project Type**: Rust workspace library and CLI

**Performance Goals**: Policy construction is bounded by a small CLI rule list; matching is linear in canonical rules and occurs outside the packet hot path

**Constraints**: No system proxy mutation, no transparent fallback, no automatic local bypass, no DNS trust carry-over, no dependency addition, no raw proxy schema rewrite

**Scale/Scope**: One implemented child-environment route, four typed rule families, two address families, uppercase and lowercase environment variants, one session listener, and exact controlled origins

## Constitution Check

- **P-1, no covert target instrumentation**: PASS. Policy is explicit, plan-visible, child-scoped, confirmation-gated, reversible, and auditable. It adds no process access, system proxy mutation, pinning bypass, or key extraction.
- **P-2 and P-3, architecture boundaries**: PASS. The facade owns route policy and decisions, `fragcap-proxy` retains resolved-address enforcement, and the CLI only parses and presents inputs.
- **P-4 and P-9, loss and truth**: PASS. Bypass is a named scope outcome, local refusal remains distinct, and undetermined evidence remains visible. No intended bypass is reported as proxy loss.
- **P-5, compatibility**: PASS. Existing packet and raw proxy formats remain unchanged. Plan and bundle JSON additions are additive.
- **P-6, glossary**: PASS. Durable bypass and routing-decision terms receive entries in this change.
- **P-7 and P-8, thin wrappers and standards**: PASS. Conventional proxy environment variables remain the boundary, with explicit refusal where their safe projection cannot preserve policy.
- **P-10, one target path**: PASS. The policy belongs to the existing prepared target session and creates no alternate resolver.
- **P-11, specification truth**: PASS. Master specification, outline, and roadmap record S122 as the milestone-3 exit without claiming Deep Capture completion.

Post-design recheck: PASS. Typed facade policy plus existing proxy address enforcement closes the issue without widening architecture or permissions.

## Project Structure

### Documentation (this feature)

```text
specs/122-proxy-bypass-policy/
|-- spec.md
|-- plan.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- checklists/
|   |-- requirements.md
|   `-- security.md
|-- contracts/
|   |-- bypass-rule.md
|   `-- routing-decision.md
`-- tasks.md
```

### Source Code (repository root)

```text
crates/fragcap/src/deep_capture/
|-- model.rs
|-- routing.rs
`-- session.rs

crates/fragcap-proxy/src/
`-- upstream.rs

crates/fragcap-cli/src/
|-- cli.rs
|-- events.rs
`-- commands/deep_capture.rs

crates/fragcap/tests/
|-- deep_capture_routing.rs
`-- deep_capture_session.rs

crates/fragcap-cli/tests/
`-- cli_deep_capture.rs
```

**Structure Decision**: Put parsing, canonical matching, plan identity, and scope-decision accounting in the facade's existing routing module. The proxy crate continues to own post-resolution address permission and exact controlled-origin grants. The CLI maps repeated arguments into facade rules and renders existing plan/artifact structures. This avoids a second destination policy and keeps target-environment semantics out of the transport crate.

## Implementation Phases

1. Add failing parser and matcher tests for every rule family, alias, port, malformed input, canonical order, and listener collision.
2. Implement immutable bypass rules, policy construction, endpoint-bound infrastructure exclusions, destination decisions, and `NO_PROXY` projection in the facade.
3. Add failing environment isolation and session-plan tests, then carry policy through `SessionConfig`, preflight, applied routing, and all upper/lowercase variables.
4. Add CLI inputs and failing plan/bundle evidence tests, then render canonical policy and reconciled decision summaries through existing authorities.
5. Strengthen proxy destination tests for mixed answers, repeated resolution, listener aliases, and exact controlled origins without changing its ownership boundary.
6. Update architecture documents and glossary, run analysis again, and execute the full repository gate.

## Decision Log

### 2026-09-03: Conventional input with closed interpretation

Accept conventional comma-delimited or repeated `NO_PROXY`-style tokens: exact DNS, leading-dot DNS suffix, IP literal, CIDR, and optional port on exact DNS or IP authority. Reject `*`, schemes, paths, credentials, empty tokens, ambiguous unbracketed IPv6 ports, and CIDR host bits. The parser, not a target library, defines canonical identity and evidence meaning.

### 2026-09-03: Suffix includes apex

A leading-dot rule matches both its apex and descendants at a DNS label boundary. Similar textual suffixes do not match. This is deterministic and avoids the incompatible apex behavior found across client libraries.

### 2026-09-03: Infrastructure is built in but not operator policy

The exact selected listener address and canonical aliases become session infrastructure exclusions in the plan after endpoint selection. They are displayed separately and cannot be supplied as operator bypass rules. Broad localhost or loopback rules remain explicit operator choices and never become upstream grants.

### 2026-09-03: Controlled origins stay proxy-routed

The controlled protocol lab's exact loopback origins remain grants inside `DestinationPolicy`; they are not added to `NO_PROXY`. This preserves conformance evidence while preventing arbitrary local-service inspection.

### 2026-09-03: DNS has two independent decisions

Bypass matches the requested canonical hostname before resolution. If no bypass applies, every resolved socket address is checked on that attempt by `DestinationPolicy`. A prior answer, answer order, or mixed public answer cannot authorize a private or listener answer.

### 2026-09-03: Bypass is scope, not loss

An intended bypass is recorded by rule and authority where destination evidence exists. It advances neither proxy loss nor refusal. Evidence that cannot be localized remains an explicit undetermined count rather than an inferred bypass.

## Complexity Tracking

No constitutional violation or complexity exception is required.
