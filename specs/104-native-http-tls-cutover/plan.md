# Implementation Plan: Native HTTP/TLS Production Cutover

**Branch**: `codex/104-native-http-tls-cutover` | **Date**: 2026-08-30 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/104-native-http-tls-cutover/spec.md`

## Summary

Resolve issues #290, #292, and #293 as one functional cutover. Replace the foundation listener's raw capability preface with standard authenticated HTTP proxy admission, implement a bounded wire-faithful HTTP/1.1 forward proxy and CONNECT handler, terminate approved client TLS with the exact session authority, establish verified upstream TLS, and expose post-start trust and launch material through borrowed facade contracts. Remove all Python and mitmdump production orchestration, route only the selected child, and make the native adapter the sole Deep Capture and calibration backend.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.88

**Primary Dependencies**: exact-pinned Tokio 1.53.1, Hyper 1.11.1 retained for the owned stack and later HTTP/2 work, rustls 0.23.43 with ring only, Tokio Rustls 0.26.4, rcgen 0.14.10, rustls-native-certs 0.8.4, direct base64 0.22.1 and httparse 1.10.1 edges already present in `Cargo.lock`

**Storage**: Existing Deep Capture bundle and local SQLite compatibility store; capability and private CA/leaf material remain session memory only

**Testing**: Rust unit/integration tests, controlled local client and origins, independent TLS certificate inspection, CLI integration tests, `cargo xtask ci`

**Target Platform**: Windows 10/11 production path; portable loopback protocol engine and deterministic tests

**Project Type**: Rust workspace library plus CLI

**Performance Goals**: 128 default concurrent client connections; finite per-connection headers, bodies, requests, idle time, upstream stages, and drain; ten clean repeated lifecycle cycles

**Constraints**: No external proxy process, Python, `mitmdump`, `certutil`, system proxy mutation, detached task, target-process access, pinning bypass, permissive TLS verifier, Internet-dependent test, unbounded message, or silent loss

**Scale/Scope**: Three tracker issues; HTTP/1.1 and HTTPS only; one session capability and authority per listener generation; later milestone protocol and artifact work remains deferred

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

- **P-1 No covert target instrumentation**: Pass. The operator selects Deep Capture explicitly; routing is child-only; trust is current-user, exact, confirmed, and reversible; no denied technique appears.
- **P-2 Core neutrality**: Pass. Protocol, TLS, certificate, and platform trust stay in `fragcap-proxy` or facade adapters. `fragcap-core` remains unchanged.
- **P-3 Capture and attribution separation**: Pass. Proxy observations join through the existing facade registry without entering packet acquisition or attribution backends.
- **P-4 No silent loss**: Pass. Protocol refusal, parsing, truncation, observation overflow, timeout, forced stop, and projection gaps have named counters or failures.
- **P-5 Compatibility outranks richness**: Pass. Packet truth and current bundle contracts remain readable; no custom packet format is introduced.
- **P-6 Glossary first**: Pass. Existing HTTP proxy, CONNECT, TLS, SNI, ALPN, and certificate-authority terms are reused; any new public term receives a same-change glossary entry.
- **P-7 Wrappers stay thin**: Pass. The CLI loses proxy orchestration and process control. Protocol and lifecycle behavior are library-owned.
- **P-8 House standards**: Pass. UTF-8 without BOM, SPDX headers, formatting, lints, dependency policy, and full CI remain gates.
- **P-9 The instrument does not lie**: Pass. A wire-level HTTP/1 codec is chosen because the pinned Hyper server API cannot relay arbitrary informational responses. Required transformations are recorded and raw boundary truth is retained within S104's explicit bounds.
- **P-10 One path to a target**: Pass. Native routing consumes the existing prepared target and managed launch. No proxy-specific target store or resolution path is introduced.
- **P-11 Specification describes what shipped**: Pass. The master specification and public status are updated only for the exact S104 cutover, while later completion language stays prohibited.
- **Licensing and dependency policy**: Pass. `base64` and `httparse` are MIT/Apache-2.0, declare compatible MSRVs, contain no native input, and add no lock package.

Post-design re-check: pass. Borrowed post-start access prevents secret persistence; the raw codec resolves the informational-response fidelity conflict; current-user trust and child-only routing remain exact; deferred features remain explicit omissions.

## Project Structure

### Documentation (this feature)

```text
specs/104-native-http-tls-cutover/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── native-proxy-api.md
│   └── http-tls-observation.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-proxy/src/
├── auth.rs
├── certificate.rs
├── event.rs
├── http1.rs
├── model.rs
├── runtime.rs
├── tls.rs
└── upstream.rs

crates/fragcap-proxy/tests/
├── authentication.rs
├── http1_proxy.rs
├── https_proxy.rs
└── lifecycle.rs

crates/fragcap/src/deep_capture/
├── adapters.rs
├── model.rs
├── native.rs
└── session.rs

crates/fragcap-cli/src/
├── cli.rs
├── commands/deep_capture.rs
└── doctor/

xtask/src/lint.rs
docs/fragcap-specification.md
docs/fragcap-spec-outline.md
README.md
site/content/docs/
changelog.d/
```

**Structure Decision**: Protocol parsing, forwarding, TLS, identity, and runtime ownership remain in the leaf `fragcap-proxy` crate. The facade gains borrowed, non-serializable post-start access so it can coordinate trust and selected-child routing without persisting secrets. The CLI deletes proxy implementations and only supplies presentation/effect adapters already owned by public contracts.

## Implementation Phases

1. Add direct parser and authorization edges and define stable protocol limits, results, and borrowed facade access contracts.
2. Write failing authentication, HTTP/1.1 wire, framing-refusal, destination-policy, TLS, lifecycle, facade, and CLI cutover tests.
3. Implement standard Basic proxy capability encoding and a bounded wire-level HTTP/1.1 codec that preserves arbitrary informational responses and records required transformations.
4. Integrate CONNECT, client-facing rustls termination, bounded leaf issuance, and separately verified upstream rustls transport into the runtime's owned task tree.
5. Adapt native session identity, trust, launch routing, observations, and cleanup through the facade; remove CLI-global proxy environment mutation.
6. Delete mitmdump/Python/external-proxy code and selector, update doctor readiness and controlled verification, and add the repository regression gate.
7. Update architecture/status documentation and changelog fragments, run focused and complete CI parity, converge against requirements, and commit locally.

## Complexity Tracking

No constitution violation requires an exception. The custom HTTP/1.1 boundary is proportional to #292: the pinned public Hyper server API cannot emit arbitrary upstream 1xx responses, so using it alone would silently lose a required observation and violate P-9. The codec uses the audited `httparse` parser and keeps HTTP/2 and higher-level projections out of scope.
