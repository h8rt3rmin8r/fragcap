# Tasks: Authenticated SOCKS5 TCP Routing

**Input**: Design documents from `specs/114-authenticated-socks5-tcp/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by the autopilot protocol and the security-sensitive acceptance contract. Write tests before implementation.

**Organization**: Tasks are grouped by independently testable user story.

## Phase 1: Setup And Contracts

**Purpose**: Establish reviewed slice and wire contracts without changing dependencies.

- [x] T001 Record S114 specification, research, data model, contracts, quickstart, and completed requirements checklists in `specs/114-authenticated-socks5-tcp/`
- [x] T002 Confirm no dependency or lockfile change is required in `crates/fragcap-proxy/Cargo.toml` and `Cargo.lock`

---

## Phase 2: Foundational Protocol And Evidence Types

**Purpose**: Add the bounded grammar and typed evidence shared by all stories.

- [x] T003 [P] Add failing unit tests for capability-bound SOCKS credentials and secret-safe formatting in `crates/fragcap-proxy/src/auth.rs`
- [x] T004 [P] Add failing wire parser, reply mapping, classification, and truncation tests in `crates/fragcap-proxy/src/socks5.rs`
- [x] T005 Add session credential verification and authenticated `socks5h` URL construction in `crates/fragcap-proxy/src/auth.rs`
- [x] T006 Implement bounded SOCKS5 greeting, authentication, CONNECT, reply, and prefix classification types in `crates/fragcap-proxy/src/socks5.rs` and export them from `crates/fragcap-proxy/src/lib.rs`
- [x] T007 Add SOCKS accounting and typed tunnel application events in `crates/fragcap-proxy/src/model.rs` and `crates/fragcap-proxy/src/application.rs`

**Checkpoint**: The protocol contract is independently unit tested with no listener or upstream effect.

---

## Phase 3: User Story 1 - Route A TCP Connection Through SOCKS5 (Priority: P1)

**Goal**: Authenticate, CONNECT, and forward IPv4, IPv6, and domain TCP traffic with bounded half-close behavior.

**Independent Test**: A real loopback SOCKS client exchanges exact bytes with controlled origins across every address form and clean half-close.

- [x] T008 [US1] Add failing valid negotiation, IPv4, IPv6-available, domain, pipelining, bidirectional, and half-close lab cases in `crates/fragcap-proxy/tests/socks5_proxy.rs`
- [x] T009 [US1] Integrate first-octet SOCKS recognition and authenticated CONNECT into the shared listener in `crates/fragcap-proxy/src/runtime.rs`
- [x] T010 [US1] Reuse cancellable upstream DNS, policy, and connect ownership and add configured bounded bidirectional forwarding in `crates/fragcap-proxy/src/runtime.rs` and `crates/fragcap-proxy/src/upstream.rs`
- [x] T011 [US1] Verify the focused P1 loopback matrix passes in `crates/fragcap-proxy/tests/socks5_proxy.rs`

**Checkpoint**: Valid SOCKS5 TCP traffic works independently without changing HTTP behavior.

---

## Phase 4: User Story 2 - Refuse Unauthorized Or Invalid Clients (Priority: P2)

**Goal**: Prove unrelated local and malformed clients cannot cause DNS, connect, or payload effects.

**Independent Test**: Every invalid credential, method, request, address, policy, deadline, and cancellation case returns a finite exact refusal with zero unauthorized origin accepts.

- [x] T012 [US2] Add failing no-auth, wrong-user, wrong-password, malformed, truncated, unsupported-command, unsupported-address, policy, DNS, timeout, and cancellation cases in `crates/fragcap-proxy/tests/socks5_proxy.rs`
- [x] T013 [US2] Map parser, authentication, policy, DNS, connect, timeout, cancellation, and I/O failures to exact replies and terminal accounting in `crates/fragcap-proxy/src/socks5.rs` and `crates/fragcap-proxy/src/runtime.rs`
- [x] T014 [US2] Add security assertions for zero pre-authentication upstream work, constant-time credential use, bounded fields, and secret absence in `crates/fragcap-proxy/tests/authentication.rs` and `crates/fragcap-proxy/tests/socks5_proxy.rs`

**Checkpoint**: Unauthorized local clients have no proxy tenancy.

---

## Phase 5: User Story 3 - Correlate And Classify Accepted Tunnels (Priority: P3)

**Goal**: Publish truthful SOCKS route, classification, connection, byte, loss, lifecycle, and correlation evidence through existing authorities.

**Independent Test**: HTTP, TLS, and opaque prefixes plus route and artifact tests reconcile one connection id and exact terminal conservation without exposing secrets.

- [x] T015 [US3] Add failing classification, byte-accounting, queue-loss, terminal-conservation, and existing HTTP regression cases in `crates/fragcap-proxy/tests/socks5_proxy.rs` and `crates/fragcap-proxy/tests/lifecycle.rs`
- [x] T016 [US3] Emit typed SOCKS negotiation, CONNECT, DNS-owner, classification, byte, and terminal events in `crates/fragcap-proxy/src/runtime.rs`
- [x] T017 [US3] Serialize SOCKS events and update declared exports without claiming generic TCP payload semantics in `crates/fragcap/src/deep_capture/application.rs` and `crates/fragcap/src/deep_capture/lifecycle.rs`
- [x] T018 [US3] Add a secret-bearing SOCKS URL to `ProxyRoute` and resolve `ALL_PROXY` separately in `crates/fragcap/src/deep_capture/adapters.rs`, `crates/fragcap/src/deep_capture/native.rs`, and `crates/fragcap/src/deep_capture/routing.rs`
- [x] T019 [US3] Add facade route and artifact correlation coverage in `crates/fragcap/tests/deep_capture_routing.rs` and existing deep-capture tests
- [x] T020 [US3] Update the controlled target environment and CLI integration assertions for distinct HTTP and SOCKS URLs in `crates/fragcap-cli/src/commands/deep_capture.rs` and `crates/fragcap-cli/tests/cli_deep_capture.rs`

**Checkpoint**: SOCKS tunnels are auditable and correlated through existing authorities.

---

## Phase 6: Documentation And Full Verification

**Purpose**: Reconcile architecture and prove the complete workspace remains green.

- [x] T021 [P] Add SOCKS5, SOCKS5 CONNECT, and proxy-owned DNS glossary entries in `docs/glossary/capture-and-networking.md` and regenerate `docs/glossary/index.md`
- [x] T022 [P] Update issue status, protocol matrix, architecture revision, outline, proxy README, and slice sequencing in `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, `crates/fragcap-proxy/README.md`, `docs/plans/README.md`, and `AGENTS.md`
- [x] T023 [P] Add feature and architecture-decision fragments in `changelog.d/S114-authenticated-socks5-tcp.added.md` and `changelog.d/S114-authenticated-socks5-tcp.decisions.md`
- [x] T024 Run focused unit, proxy lab, facade, and CLI tests from `specs/114-authenticated-socks5-tcp/quickstart.md`
- [x] T025 Run `cargo xtask ci`, dependency and lockfile drift checks, UTF-8 and mojibake checks, and `git diff --check`
- [x] T026 Mark every task complete and perform final spec, plan, contract, security, and issue #310 acceptance reconciliation in `specs/114-authenticated-socks5-tcp/tasks.md`

---

## Dependencies & Execution Order

- Phase 1 precedes all implementation.
- Phase 2 blocks every user story.
- User Story 1 supplies the valid tunnel used by User Stories 2 and 3.
- User Story 2 hardens admission and refusal before evidence completion.
- User Story 3 depends on the stable valid and refusal outcomes.
- Documentation and full verification follow all stories.

## Parallel Opportunities

- T003 and T004 touch independent foundational files.
- T021, T022, and T023 touch independent documentation and changelog files after code stabilizes.
- All runtime integration tasks remain sequential because they share listener state and accounting.

## Implementation Strategy

1. Complete the bounded parser and credential checks under failing unit tests.
2. Land a valid loopback CONNECT and relay before adding the refusal matrix.
3. Add typed evidence and facade routing only after transport outcomes stabilize.
4. Keep each controlled case offline and finite.
5. Finish with the full repository gate and exact issue acceptance audit.
