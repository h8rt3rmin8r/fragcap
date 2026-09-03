# Feature Specification: Proxy Bypass and Local-Destination Policy

**Feature Branch**: `codex/122-proxy-bypass-policy`

**Created**: 2026-09-03

**Status**: Draft

**Input**: User description: "Implement S122 as the explicit proxy bypass and local-destination correctness slice for issue #318 under spec-kit autopilot."

## User Scenarios & Testing

### User Story 1 - Authorize an Exact Bypass Policy (Priority: P1)

An authorized operator can declare which target destinations may bypass the session proxy, review the normalized rules before authorization, and know that inherited proxy variables cannot silently change that scope.

**Why this priority**: An ambiguous or inherited bypass list can leak target traffic outside inspection or route unrelated traffic into the proxy.

**Independent Test**: DNS domains, IP addresses, CIDRs, ports, and IPv6 rules normalize deterministically; malformed or ambiguous rules refuse before effects; and the managed child receives only the reviewed proxy and bypass environment.

**Acceptance Scenarios**:

1. **Given** an explicit set of valid bypass rules, **When** preflight builds the session plan, **Then** the plan shows one canonical ordered policy and the exact managed-child environment projection.
2. **Given** ambient uppercase or lowercase proxy variables, **When** the route is applied, **Then** every supported proxy and bypass variable is overwritten from the authorized plan and no ambient value survives.
3. **Given** a malformed, wildcard-all, user-info, path-bearing, ambiguous IPv6, invalid CIDR, or invalid port rule, **When** preflight parses it, **Then** the session refuses before bundle creation, listener bind, trust mutation, or launch.

---

### User Story 2 - Keep Infrastructure and Local Services Outside Target Scope (Priority: P1)

The proxy listener and session-owned infrastructure always remain outside target-destination scope, while controlled origins explicitly authorized for conformance remain proxy destinations rather than accidental bypasses.

**Why this priority**: A self-proxy loop can deadlock or recurse, and broad localhost bypass can either hide controlled evidence or inspect unrelated local services.

**Independent Test**: Listener aliases, mapped addresses, localhost names, loopback ranges, private addresses, and controlled origins each receive a stable, distinct decision with no wildcard bind or implicit local grant.

**Acceptance Scenarios**:

1. **Given** any textual alias of the exact listener endpoint, **When** policy evaluates it, **Then** it is classified as session infrastructure and cannot be treated as a target origin or sent recursively through the proxy.
2. **Given** an unrelated loopback, link-local, private, or unique-local destination, **When** no exact operator bypass or controlled-origin grant applies, **Then** the proxy refuses it rather than inspecting or forwarding it.
3. **Given** a controlled origin owned by the current protocol lab, **When** the target addresses it, **Then** the origin remains explicitly proxy-routed and the existing exact local destination grant authorizes only that socket.

---

### User Story 3 - Audit Bypass Decisions Without Inventing Loss (Priority: P2)

An operator can distinguish traffic intentionally excluded by the reviewed bypass policy from proxy refusal, proxy loss, and traffic that never produced enough evidence for a destination decision.

**Why this priority**: Treating intended bypass as loss makes accounting misleading, while omitting it makes scope changes invisible.

**Independent Test**: Plan, route evidence, compatibility artifact, and manifest agree on policy identity and observable decision counts; direct bypasses remain scoped decisions, never proxy loss, and proxy-only evidence marks their count unavailable instead of inventing zero.

**Acceptance Scenarios**:

1. **Given** a destination matching one explicit rule, **When** a caller has localized destination evidence, **Then** policy evaluation returns the canonical rule, bypass outcome, and `operator-policy` authority without incrementing proxy loss; proxy-only bundle evidence marks the direct count unavailable.
2. **Given** a proxied DNS name whose answers include a local or private address, **When** the proxy resolves it, **Then** each answer is rechecked by the destination policy and the connection refuses without transparent fallback.
3. **Given** incomplete packet, process, or destination evidence, **When** the bundle finalizes, **Then** the policy remains visible and the undecidable observation is reported separately from bypassed, proxied, refused, and lost traffic.

### Edge Cases

- Domain matching is ASCII case-insensitive and ignores one trailing DNS root dot, while malformed empty labels and non-ASCII names refuse.
- A bare or leading-dot DNS rule matches its apex and descendants at a label boundary but not a merely similar name.
- A port-qualified rule matches only that port; an unqualified rule matches every valid port.
- IPv4-mapped IPv6 addresses have one canonical IPv4 identity.
- CIDR host bits are rejected rather than silently normalized.
- Duplicate rules collapse after canonicalization, and canonical ordering is stable across input order.
- The wildcard token `*` is refused because it would bypass the complete target scope.
- A DNS name may change answers between requests; no prior public answer authorizes a later private answer.
- A listener port selected after preflight is projected as built-in infrastructure without broadening the operator policy.

## Requirements

### Functional Requirements

- **FR-001**: The system MUST represent bypass policy as immutable typed rules for DNS domains, IP addresses, CIDR networks, optional ports where meaningful, and both IPv4 and IPv6. Bare and leading-dot DNS inputs MUST share apex-and-descendant domain-boundary semantics so target clients cannot broaden a narrower exact-name claim.
- **FR-002**: Rule parsing MUST be deterministic, reject ambiguous or unsafe syntax, canonicalize case and address aliases, reject CIDR host bits, remove duplicates, and produce stable ordering independent of input order.
- **FR-003**: The complete-bypass wildcard MUST be refused because it defeats target-scoped inspection.
- **FR-004**: Empty explicit operator policy MUST mean no target destination bypasses; ambient proxy and bypass environment MUST NOT widen or narrow the plan.
- **FR-005**: Managed child routing MUST overwrite uppercase and lowercase HTTP, HTTPS, all-proxy, and no-proxy variables from the authorized plan.
- **FR-006**: The exact session listener and its canonical aliases MUST be classified as session infrastructure and MUST never be accepted as an upstream target or create a self-proxy loop.
- **FR-007**: Localhost names, loopback ranges, link-local addresses, private IPv4 ranges, unique-local IPv6 ranges, unspecified addresses, multicast addresses, and mapped aliases MUST NOT gain proxy permission merely because they are local.
- **FR-008**: Controlled origins MUST be exact session-owned proxy destinations, distinct from both operator bypasses and listener infrastructure.
- **FR-009**: Bypass matching MUST evaluate the requested DNS authority before resolution; any destination that remains proxied MUST re-evaluate every resolved address through the existing destination policy on every connection attempt.
- **FR-010**: DNS rebinding, mixed public/private answers, and answer-order changes MUST NOT convert a refused local destination into an allowed proxy destination or transparent bypass.
- **FR-011**: The preauthorization plan MUST display canonical operator rules, built-in infrastructure exclusions, environment ownership, matching semantics, and the no-fallback rule.
- **FR-012**: Route evidence and bundle metadata MUST distinguish `proxied`, `bypassed`, `infrastructure`, `refused`, and `undetermined` outcomes with stable reasons and decision authority. An outcome not observable by that evidence authority MUST be explicitly unavailable rather than zero.
- **FR-013**: An intentional bypass localized by destination evidence MUST be a scoped routing decision and MUST NOT increment proxy queue, transport, protocol, or storage loss. Proxy-only evidence MUST NOT infer a direct bypass count from silence.
- **FR-014**: Decision accounting MUST reconcile all outcomes observable by its declared authority plus explicitly unlocalized observations without double-counting one destination.
- **FR-015**: Malformed policy, infrastructure collision, and unsupported projection MUST refuse before external effects.
- **FR-016**: Security tests MUST cover bare and leading-dot DNS boundaries, ports, IPv4 and IPv6 CIDRs, mapped addresses, listener aliases, localhost aliases, private ranges, mixed DNS answers, rebinding, inherited environment, duplicates, and malformed rules.
- **FR-017**: S122 MUST update the master specification, outline, roadmap, glossary, changelog fragments, and agent context without claiming Deep Capture feature completion before issue #334.
- **FR-018**: S122 MUST add no target process access, system proxy mutation, silent trust mutation, transparent fallback, dependency, or lockfile package.

### Key Entities

- **Bypass Policy**: The immutable, canonical collection of explicit operator rules plus separately identified session infrastructure exclusions.
- **Bypass Rule**: One typed DNS-domain, IP, or CIDR predicate with optional port applicability.
- **Routing Decision**: One stable classification of a requested destination as proxied, bypassed, infrastructure, refused, or undetermined, including authority and reason.
- **Controlled Origin Grant**: One exact session-owned local socket that remains proxy-routed and is authorized only inside the native proxy destination policy.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All valid rule classes and both address families round-trip to one canonical representation in 100 percent of parser and permutation tests.
- **SC-002**: All malformed, ambiguous, wildcard-all, infrastructure-colliding, and host-bit-bearing inputs refuse before effects in 100 percent of negative tests.
- **SC-003**: Ambient uppercase and lowercase proxy variables influence zero managed-child routing values across the controlled environment matrix.
- **SC-004**: Listener aliases and all tested local/private destinations produce no unintended upstream connection, while each controlled origin remains exactly proxy-routed.
- **SC-005**: Mixed-answer and rebinding tests refuse every non-public ungranted answer regardless of DNS order or earlier answers.
- **SC-006**: For every controlled evidence case, all decision counts observable by the declared authority reconcile to its retained observations, unavailable outcomes are named, and bypass adds zero proxy loss.
- **SC-007**: All repository verification gates pass with no new dependency or lockfile package.

## Clarifications

### Session 2026-09-03

- Q: Does an empty policy inherit the operator's `NO_PROXY`? -> A: No. Empty means no operator-selected target bypasses; every supported proxy variable is owned by the plan.
- Q: Do bare and leading-dot DNS rules differ? -> A: No. Both match the apex and descendants at a label boundary because conventional `NO_PROXY` consumers can broaden bare domains that way.
- Q: Are all local destinations automatically bypassed? -> A: No. Only exact listener infrastructure and explicit operator rules bypass; other local destinations are refused unless they are exact controlled-origin proxy grants.
- Q: When is a DNS bypass decision made? -> A: Against the canonical requested hostname before resolution; proxied answers are independently policy-checked on every attempt.
- Q: Does a bypass count as packet or proxy loss? -> A: No. It is a visible scoped routing decision with separate accounting.

## Assumptions

- Child-scoped environment routing remains the only implemented target routing strategy.
- The route projection uses conventional `NO_PROXY` tokens, but the typed policy and evidence contract remain authoritative when target implementations vary; unsupported safe projection refuses.
- Existing packet/process correlation, proxy application evidence, and manifest authorities are extended rather than replaced.
- Existing exact controlled-origin grants remain test-only/session-owned and do not authorize arbitrary local services.
