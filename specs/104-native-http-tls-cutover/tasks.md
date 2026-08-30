# Tasks: Native HTTP/TLS Production Cutover

**Input**: Design documents from `/specs/104-native-http-tls-cutover/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Protocol, security, lifecycle, loss-accounting, facade, CLI, and source-policy tests are mandatory and precede their implementation.

## Phase 1: Setup

- [x] T001 Add exact direct base64 and httparse dependency declarations in Cargo.toml and crates/fragcap-proxy/Cargo.toml
- [x] T002 [P] Add HTTP/1.1 and TLS module declarations and public exports in crates/fragcap-proxy/src/lib.rs
- [x] T003 Verify ignore files and dependency-direction policy for S104 paths in .gitignore and xtask/src/deps.rs

## Phase 2: Shared Foundations

- [x] T004 Define finite HTTP, TLS, request-count, body, idle, upstream, leaf-cache, observation, and child-task limits in crates/fragcap-proxy/src/model.rs
- [x] T005 Define typed HTTP framing, protocol stage, TLS boundary, transformation, and accounting outcomes in crates/fragcap-proxy/src/model.rs and crates/fragcap-proxy/src/event.rs
- [x] T006 Define borrowed ProxySessionAccess, ProxyLaunchRoute, and ProxyTrustMaterial contracts in crates/fragcap/src/deep_capture/adapters.rs and crates/fragcap/src/deep_capture/model.rs
- [x] T007 Update mock adapter signatures and assert post-start material cannot appear before authorization in crates/fragcap/tests/deep_capture_session.rs

## Phase 3: User Story 1 - Native Sole Production Path (Priority: P1)

**Goal**: Run ordinary Deep Capture and both calibration phases through the native library backend with no external proxy tooling.

**Independent Test**: Controlled library and CLI sessions succeed with an environment containing no Python or mitmdump, use native backend identity, and leave no proxy/trust residue.

- [x] T008 [P] [US1] Add source-policy and release-input regression tests for external proxy commands and embedded Python in xtask/src/lint.rs
- [x] T009 [P] [US1] Add native backend identity, no-selector, and no-external-tool CLI assertions in crates/fragcap-cli/tests/cli_deep_capture.rs and crates/fragcap-cli/tests/cli_doctor.rs
- [x] T010 [US1] Add borrowed access, exact CA sharing, child-route secrecy, lifecycle ordering, and failure-injection tests in crates/fragcap/tests/native_proxy.rs and crates/fragcap/tests/deep_capture_session.rs
- [x] T011 [US1] Generate session capability and authority at native start and expose only borrowed proof/public identity from crates/fragcap-proxy/src/runtime.rs and crates/fragcap-proxy/src/certificate.rs
- [x] T012 [US1] Implement native facade proxy, trust, observation, and cleanup adapters in crates/fragcap/src/deep_capture/native.rs
- [x] T013 [US1] Pass borrowed post-start trust and route access through coordinator ordering in crates/fragcap/src/deep_capture/session.rs
- [x] T014 [US1] Apply authenticated proxy configuration only to the exact retained managed launch in crates/fragcap/src/managed_launch.rs and crates/fragcap-cli/src/commands/deep_capture.rs
- [x] T015 [US1] Remove LibraryProxyAdapter, controlled alternate proxy, process-global proxy environment, mitmdump/Python process control, discovery, parsing, logs, and demo code from crates/fragcap-cli/src/commands/deep_capture.rs
- [x] T016 [US1] Remove --proxy-backend and DeepCaptureProxyArg from crates/fragcap-cli/src/cli.rs and update command help tests
- [x] T017 [US1] Replace executable discovery with compiled native readiness in crates/fragcap-cli/src/doctor/probe.rs, crates/fragcap-cli/src/doctor/mod.rs, and crates/fragcap-cli/src/doctor/checks.rs

## Phase 4: User Story 2 - Complete HTTP/1.1 Forwarding and CONNECT (Priority: P1)

**Goal**: Forward bounded HTTP/1.1 and CONNECT while preserving informational responses, upgrades, half-close truth, and unambiguous framing.

**Independent Test**: The controlled matrix covers methods, target forms, persistence, fixed/chunked bodies, trailers, 1xx, CONNECT, upgrades, cancellation, half-close, malformed framing, and size/time limits with exact accounting.

- [x] T018 [P] [US2] Add standard Basic proxy authorization encoding, strict parsing, redaction, rotation, and zeroization tests in crates/fragcap-proxy/tests/authentication.rs
- [x] T019 [P] [US2] Add request-head, target-form, method, Host consistency, fixed-length, chunked, trailer, persistence, and informational-response tests in crates/fragcap-proxy/tests/http1_proxy.rs
- [x] T020 [US2] Add conflicting length/transfer coding, malformed chunk, control byte, obsolete folding, oversized, slow head, early EOF, and extra-byte refusal tests in crates/fragcap-proxy/tests/http1_proxy.rs
- [x] T021 [US2] Add CONNECT tunnel, upgrade, bidirectional payload, cancellation, timeout, and half-close tests in crates/fragcap-proxy/tests/http1_proxy.rs
- [x] T022 [US2] Add DNS empty/mixed/rebinding, listener recursion, private/mapped address, exact grant, cancellation, and budget assertions in crates/fragcap-proxy/tests/http1_proxy.rs and crates/fragcap-proxy/tests/upstream.rs
- [x] T023 [US2] Replace raw capability preface with URL-safe password generation and strict standard proxy authorization in crates/fragcap-proxy/src/auth.rs
- [x] T024 [US2] Implement bounded request and response head parsing with one framing decision in crates/fragcap-proxy/src/http1.rs
- [x] T025 [US2] Implement raw head retention, proxy-only field removal, target/Host validation, and transformation evidence in crates/fragcap-proxy/src/http1.rs
- [x] T026 [US2] Implement finite fixed, chunked, trailer, close-delimited, informational, and persistent message relay in crates/fragcap-proxy/src/http1.rs
- [x] T027 [US2] Implement CONNECT and accepted-upgrade bidirectional relay with bounded half-close and cancellation in crates/fragcap-proxy/src/http1.rs
- [x] T028 [US2] Integrate policy-checked upstream connection, owned protocol tasks, shutdown, and observation accounting in crates/fragcap-proxy/src/runtime.rs and crates/fragcap-proxy/src/upstream.rs

## Phase 5: User Story 3 - Client-Facing and Verified Upstream TLS (Priority: P1)

**Goal**: Inspect approved HTTPS using the exact session authority while upstream certificate and hostname validation always fail closed.

**Independent Test**: Controlled TLS 1.2 and 1.3 clients traverse CONNECT to valid origins; invalid name/chain, SNI mismatch, unsupported negotiation, alerts, timeouts, incomplete close, and rotation produce distinct bounded outcomes.

- [x] T029 [P] [US3] Add client-facing TLS 1.2/1.3, DNS/IP SAN, SNI, ALPN, authority rotation, and leaf-cache tests in crates/fragcap-proxy/tests/https_proxy.rs
- [x] T030 [US3] Add verified upstream valid-name, wrong-name, untrusted, expired, not-yet-valid, alert, timeout, cancellation, and close-notify tests in crates/fragcap-proxy/tests/https_proxy.rs
- [x] T031 [US3] Add client refusal, unsupported protocol, SNI/CONNECT mismatch, silence/inconclusive, and pinning-boundary classification tests in crates/fragcap-proxy/tests/https_proxy.rs
- [x] T032 [US3] Add full CONNECT plus inner HTTP request/response integration and coarse observation assertions in crates/fragcap-proxy/tests/https_proxy.rs
- [x] T033 [US3] Build one explicit-ring native-root upstream client configuration per session with HTTP/1.1 ALPN and injected test roots in crates/fragcap-proxy/src/tls.rs and crates/fragcap-proxy/src/upstream.rs
- [x] T034 [US3] Implement bounded client hello, exact CONNECT identity validation, session leaf resolution, TLS 1.2/1.3 server policy, and client handshake in crates/fragcap-proxy/src/tls.rs
- [x] T035 [US3] Implement separately typed verified upstream TLS handshake, alerts, negotiation evidence, timeout, and close outcome in crates/fragcap-proxy/src/tls.rs
- [x] T036 [US3] Run the HTTP/1.1 engine inside inspected CONNECT and emit complete HTTPS only after both TLS boundaries and final HTTP response succeed in crates/fragcap-proxy/src/runtime.rs

## Phase 6: User Story 4 - Thin Public Boundary and Truthful Integration (Priority: P2)

**Goal**: Keep protocol policy in libraries and preserve existing bundle/fact/event truth through the native cutover.

**Independent Test**: Equivalent library and CLI sessions produce the same ordered native lifecycle and coarse application outcomes, and no secret enters plans, debug output, events, artifacts, or parent process environment.

- [x] T037 [P] [US4] Add credential/private-material leak guards across plan debug, JSON events, bundle files, diagnostics, and parent environment in crates/fragcap/tests/native_proxy.rs and crates/fragcap-cli/tests/cli_deep_capture.rs
- [x] T038 [US4] Map native raw HTTP/TLS observations into existing CompatibilityObservation values without later-milestone completeness claims in crates/fragcap/src/deep_capture/native.rs
- [x] T039 [US4] Preserve backend identity, terminal outcomes, facts, and artifact compatibility in crates/fragcap-cli/src/commands/deep_capture.rs and crates/fragcap/src/deep_capture/session.rs
- [x] T040 [US4] Update doctor and Deep Capture human/JSON goldens for fragcap-native in crates/fragcap-cli/tests/goldens/
- [x] T041 [US4] Add ten-cycle success/failure cleanup, concurrent-session isolation, port reuse, trust idempotency, and forced-task reconciliation tests in crates/fragcap-proxy/tests/lifecycle.rs and crates/fragcap/tests/native_proxy.rs

## Phase 7: Polish and Cross-Cutting Gates

- [x] T042 [P] Update native cutover architecture and incomplete-status truth in AGENTS.md, docs/fragcap-specification.md, and docs/fragcap-spec-outline.md
- [x] T043 [P] Update public requirements, CLI references, and glossary entries in README.md, CONTRIBUTING.md, crates/fragcap-proxy/README.md, docs/glossary/, and site/content/docs/
- [x] T044 Add user-visible and dated architecture decision fragments in changelog.d/S104-native-http-tls-cutover.added.md and changelog.d/S104-native-http-tls-cutover.decisions.md
- [x] T045 Run focused tests and resolve every failure from specs/104-native-http-tls-cutover/quickstart.md
- [x] T046 Run the spec-kit convergence audit and append any discovered requirement-driven work to specs/104-native-http-tls-cutover/tasks.md
- [x] T047 Run cargo xtask ci and resolve every format, lint, test, dependency, specification, MSRV, package, documentation, encoding, mojibake, and platform failure
- [x] T048 Mark every completed task and set S104 status complete in specs/104-native-http-tls-cutover/tasks.md and specs/104-native-http-tls-cutover/spec.md
- [x] T049 Commit the verified slice locally with the repository conventional message and co-author trailer

## Dependencies and Execution Order

```text
Setup -> Shared Foundations -> US1 contracts -> US2 HTTP/1.1 -> US3 TLS
      -> US1/US4 production integration -> Polish -> Convergence -> Full CI -> Commit
```

US1 establishes borrowed access contracts before US2 and US3 fill the runtime. The actual sole-path deletion waits until HTTP and TLS are proven so no intermediate commit claims a nonfunctional native default. Tests precede each corresponding implementation.

## Parallel Opportunities

- T002, T008, T009, T018, T019, T029, T037, T042, and T043 affect independent files at their phase boundary.
- HTTP framing and TLS certificate tests can be authored independently after shared model types exist.
- Documentation updates may be prepared after final contracts stabilize, but completion language waits for the integration tests.

## Implementation Strategy

Deliver the complete three-issue slice before the pre-push halt. The minimum reviewable increment is authenticated plain HTTP forwarding, but it is not the slice's release boundary. Production cutover occurs only after client-facing and upstream TLS, facade ordering, source-policy gates, and cleanup reconciliation all pass.

## Phase 8: Convergence

- [x] T050 Add negotiated TLS version, ALPN, and requested identity to distinct client/upstream boundary observations per FR-011 and FR-013 (partial)
- [x] T051 Expand malformed, oversized, persistent, chunked, trailer, upgrade, cancellation, and half-close runtime coverage per FR-006, FR-007, and SC-002 (partial)
- [x] T052 Add full CONNECT-path invalid-name and untrusted-upstream failure assertions with no decrypted success per FR-010, FR-012, and SC-003 (partial)
- [x] T053 Preserve truthful pre-effect Steam routing refusal and explicit bundle-directory ownership in tests and current documentation per FR-003, FR-005, and FR-017 (partial)

## Phase 9: Third-party review hardening

- [x] T054 Restore packet-side flow and attribution correlation for real native observations without claiming the broader #302 cross-artifact contract
- [x] T055 Carry the exact controlled child process identifier into native observations instead of substituting the parent process
- [x] T056 Relay `Expect: 100-continue` informational responses before awaiting the request body, including early final-response handling
- [x] T057 Retain one metadata observation and failure reason for every parsed request that fails after admission
