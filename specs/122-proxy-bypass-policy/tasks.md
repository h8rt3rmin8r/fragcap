# Tasks: Proxy Bypass and Local-Destination Policy

**Input**: Design documents from `specs/122-proxy-bypass-policy/`

## Phase 1: Setup and Specification Gates

- [X] T001 Read the constitution, authorized-use context, master specification, roadmap, conventions, contributing guide, issue #318, and active feature state
- [X] T002 Create and select `codex/122-proxy-bypass-policy` in `.specify/feature.json`
- [X] T003 Complete specification, autopilot clarifications, requirements checklist, security checklist, plan, research, data model, contracts, and quickstart
- [X] T004 Run spec-kit analysis and resolve every cross-artifact coverage or consistency finding

---

## Phase 2: Foundational Typed Policy

- [X] T005 Add failing parser tests for exact DNS, suffix, IP, CIDR, port, IPv6, mapped aliases, duplicates, canonical order, and malformed inputs in `crates/fragcap/src/deep_capture/routing.rs`
- [X] T006 Add failing matcher tests for apex/suffix boundaries, ports, CIDRs, requested-name matching, listener aliases, and controlled-origin separation
- [X] T007 Implement immutable `BypassRule`, `BypassPolicy`, requested destination, canonicalization, deterministic parsing, matching, and projection in `crates/fragcap/src/deep_capture/routing.rs`
- [X] T008 Run focused facade routing tests

**Checkpoint**: Bypass policy has one deterministic, dependency-free authority.

---

## Phase 3: Session and Environment Ownership

- [X] T009 Add failing session tests for explicit empty policy, selected rules, infrastructure binding, unsafe collision refusal, and plan immutability
- [X] T010 Add failing applied-route tests proving uppercase and lowercase proxy variables replace ambient values exactly
- [X] T011 Carry bypass inputs through `SessionConfig`, preflight, endpoint-bound `RoutingPlan`, and `AppliedRoute`
- [X] T012 Project canonical operator plus infrastructure rules into `NO_PROXY` and `no_proxy` while retaining distinct HTTP and proxy-resolved SOCKS URLs
- [X] T013 Run focused session and routing integration tests

**Checkpoint**: The reviewed policy exclusively owns every managed-child proxy variable.

---

## Phase 4: CLI, Plan, and Evidence

- [X] T014 Add failing CLI tests for repeated and comma-delimited `--proxy-bypass`, malformed pre-effect refusal, canonical plan output, and controlled environment isolation
- [X] T015 Add `--proxy-bypass` parsing and mapping in `crates/fragcap-cli/src/cli.rs` and `crates/fragcap-cli/src/commands/deep_capture.rs`
- [X] T016 Extend human and JSON preauthorization plan output with canonical rules, infrastructure, environment ownership, DNS semantics, and no fallback
- [X] T017 Extend existing compatibility and manifest bundle authorities with policy identity and conserved routing-decision summary without changing raw proxy detail
- [X] T018 Add controlled evidence tests proving bypass is scope with zero proxy loss and incomplete evidence remains undetermined
- [X] T019 Run focused CLI, event, artifact, and controlled launch tests

**Checkpoint**: Policy and decisions are reviewable before effects and auditable afterward.

---

## Phase 5: Local-Destination and DNS Security

- [X] T020 Add proxy tests for canonical listener aliases, localhost resolution, unrelated local/private ranges, mixed public/private answers, answer order, and repeated rebinding resolution
- [X] T021 Preserve exact controlled-origin grants as proxy-routed session resources and verify no operator bypass grants proxy permission
- [X] T022 Verify every resolved candidate is checked on every connection attempt and no refused destination receives transparent direct fallback
- [X] T023 Run focused upstream, HTTP, SOCKS, UDP, QUIC, IPv6, and conformance tests

**Checkpoint**: Infrastructure cannot recurse, local services cannot be inspected implicitly, and DNS cannot carry stale permission.

---

## Phase 6: Documentation and Cross-Cutting Verification

- [X] T024 Update `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, and `docs/plans/README.md` for S122 and milestone-3 closure without a #334 completion claim
- [X] T025 Add glossary entries, regenerate `docs/glossary/index.md`, and add S122 feature and decision fragments under `changelog.d/`
- [X] T026 Update `AGENTS.md` with the landed S122 boundary and no dependency change
- [X] T027 Re-run spec-kit analysis and resolve every cross-artifact finding
- [X] T028 Run focused quickstart commands and `cargo xtask ci` in the foreground
- [X] T029 Audit UTF-8 without BOM, LF, whitespace, Unicode dashes, mojibake, issue #318 scope, no staged `.specify/feature.json`, and dependency lock stability
- [X] T030 Mark every task `[X]`, review the full diff, commit with a conventional S122 message, push the authorized branch, and open the official PR closing #318

---

## Dependencies and Execution Order

- Phase 1 establishes the approved contract.
- Phase 2 blocks every story because it defines rule identity and matching.
- Phase 3 binds that policy to the existing immutable session and route.
- Phase 4 exposes the bound policy and decisions through existing authorities.
- Phase 5 proves transport-side local and DNS safety without moving policy ownership.
- Phase 6 reconciles documentation and the full gate after implementation.

## Implementation Strategy

1. Make parsing and matching independently green.
2. Bind the policy after endpoint selection and replace the complete inherited environment surface.
3. Add operator input and additive evidence projection.
4. Close adversarial local and DNS cases using the existing resolved-address authority.
5. Reconcile documentation and run the complete gate before push.
