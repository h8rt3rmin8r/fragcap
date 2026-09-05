# Feature Specification: Native Windows Integration Matrix

**Feature Branch**: `codex/129-windows-integration-matrix`

**Created**: 2026-09-04

**Status**: Complete

**Input**: User description: "S129 completes the release-critical native Windows integration matrix from issue #327 under spec-kit autopilot."

## User Scenarios & Testing

### User Story 1 - Block Release Regressions on Windows (Priority: P1)

A release maintainer can run one closed Windows matrix that covers every native completion domain and refuses a successful result when a required row is missing, skipped, duplicated, stale, or failed.

**Why this priority**: Windows is the supported product platform. A green portable test suite cannot authorize a Windows release when platform behavior was not exercised.

**Independent Test**: Validate the matrix against its registry and execute it on a supported Windows host. The run succeeds only when every required row reaches its exact expected terminal outcome and all evidence reconciles.

**Acceptance Scenarios**:

1. **Given** the reviewed registry and a supported Windows host, **When** the completion matrix runs, **Then** every required row produces one versioned result with environment, expectation, observation, evidence, and terminal status.
2. **Given** a required row is missing, duplicated, conditionally skipped, stale, or lacks executable evidence, **When** the gate validates the run, **Then** the gate fails and names the row.
3. **Given** the production protocol, lifecycle, artifact, or recovery inventories change, **When** the static gate runs, **Then** unreviewed matrix drift fails ordinary continuous integration.

---

### User Story 2 - Prove Windows Effects Are Scoped and Reversible (Priority: P2)

An authorized tester can exercise native Deep Capture on Windows across non-admin operation, explicit consent refusal, current-user trust, capture-driver presence and absence, IPv4 and IPv6 loopback, crash recovery, sensitive artifacts, and analyzer consumption without leaving undeclared machine state.

**Why this priority**: These are the Windows-only boundaries where a portable test can pass while the installed product fails or leaves security-sensitive residue.

**Independent Test**: Run the effect-bearing rows in an isolated scratch identity and session root, compare exact before and after inventories, and require either the declared success or declared refusal for each row.

**Acceptance Scenarios**:

1. **Given** a non-elevated user, **When** native controlled sessions and diagnostics run, **Then** user-scoped paths succeed or refuse exactly without requesting machine-wide authority.
2. **Given** trust consent or elevation is denied, **When** the relevant operation is requested, **Then** no trust, routing, listener, key, or bundle effect occurs and the refusal is explicit.
3. **Given** an exact current-user test authority is approved, **When** trust is added and the session terminates or recovery runs, **Then** only that exact authority is removed and the before and after trust inventories match.
4. **Given** Npcap is present or represented by the controlled absent state, **When** Capture and Deep Capture readiness are evaluated, **Then** their independent verdicts remain truthful and Deep Capture does not become dependent on Npcap.
5. **Given** a controlled crash after a journaled effect, **When** Doctor recovery runs, **Then** exact owned residue is reconciled and unrelated state is unchanged.

---

### User Story 3 - Retain Public-Safe Release Evidence (Priority: P3)

A reviewer can inspect a bounded, sanitized Windows report that proves the executed binary identity, host capabilities, row outcomes, artifact checks, and residue reconciliation without exposing secrets or machine-specific operator data.

**Why this priority**: The completion claim must be reproducible and reviewable, while raw bundles and local machine identity cannot be published.

**Independent Test**: Validate a report produced from a complete run and attempt to validate reports containing secrets, absolute user paths, host names, raw certificate material, or missing row evidence.

**Acceptance Scenarios**:

1. **Given** a complete Windows run, **When** its public-safe summary is generated, **Then** it records the registry digest, product revision, binary digest, environment capabilities, every row result, cleanup reconciliation, and explicit omissions.
2. **Given** evidence contains capability credentials, private material, payload bytes, account identifiers, host names, or absolute user paths, **When** sanitization validation runs, **Then** publication fails and identifies the prohibited field class.
3. **Given** a failure occurs, **When** evidence is archived, **Then** the failure remains diagnosable through bounded typed facts and no secret-bearing raw artifact is uploaded.

### Edge Cases

- A hosted runner lacks Npcap, Wireshark, elevation, or interactive desktop access.
- A host capability changes between preflight and execution.
- A required row observes the correct refusal for an unavailable capability.
- A child process times out, exits without a report, or attempts to open a visible console window.
- Current-user trust already contains the exact certificate or a thumbprint collision with different bytes.
- Cleanup is interrupted after the effect but before the release record.
- IPv6 exists but loopback binding or routing is unavailable.
- Analyzer output is available but does not consume the generated capture and key log together.
- An installed-layout binary differs from the source revision or binary digest named by the report.
- Public-safe evidence is complete but the raw run prefix was interrupted.

## Requirements

### Functional Requirements

- **FR-001**: The project MUST define one versioned, closed Windows integration registry whose required rows cover non-admin execution, consent denial, elevation denial, Npcap present and absent behavior, IPv4 and IPv6, current-user trust lifecycle, sensitive ACL and key-log behavior, process watching, firewall and loopback scope, crash recovery, analyzer consumption, staged installed-layout execution, and final residue reconciliation.
- **FR-002**: Every registry row MUST declare a stable identity, authority class, required host capabilities, exact setup, executable evidence, expected terminal outcome, owned effects, prohibited effects, cleanup assertion, and publication policy.
- **FR-003**: The ordinary repository gate MUST reject schema drift, missing or duplicated row identities, unknown capabilities or authorities, conditionally disabled required tests, stale evidence references, and incomplete coverage of the native completion inventories.
- **FR-004**: A Windows executor MUST run required rows without treating unavailable capabilities as implicit skips. A row MAY expect an exact unavailable or refused outcome when that outcome is the reviewed purpose of the row.
- **FR-005**: The executor MUST establish one immutable preflight capability snapshot and bind every result to that snapshot. Capability drift during a run MUST make the run incomplete.
- **FR-006**: Effect-bearing rows MUST use project-owned synthetic targets, local origins, scratch storage, exact test certificate identities, and loopback-only destinations.
- **FR-007**: Machine-wide proxy mutation, firewall-rule mutation, silent trust changes, packet interception beyond the separately installed Npcap capture path, target hooks, target memory access, target key extraction, and external network traffic MUST remain prohibited.
- **FR-008**: Non-admin and denial rows MUST prove that administrative authority is neither requested nor inferred. Any process handle MUST state its access rights and carry no memory rights.
- **FR-009**: Trust rows MUST mutate only the current-user store after explicit test authorization, record the exact certificate identity before mutation, and restore the exact before state on normal completion and recovery.
- **FR-010**: Capture-driver rows MUST distinguish build-time SDK presence, installed runtime presence, and absence. Npcap MUST remain separately acquired and MUST NOT enter committed or uploaded artifacts.
- **FR-011**: The matrix MUST exercise the production native binary from a staged installed layout. Final MSI/archive install, upgrade, repair, uninstall, and distribution-content certification remain owned by issue #329.
- **FR-012**: IPv4 and IPv6 rows MUST preserve the selected loopback family through plan, listener, route, evidence, and cleanup without wildcard binding.
- **FR-013**: Crash and restart rows MUST enter through the production resource journal and Doctor recovery authority, distinguish exact actions, no-actions, and refusals, and preserve unrelated state.
- **FR-014**: Key-log and analyzer rows MUST use proxy-owned TLS key material, prove unmodified analyzer consumption, and remove or retain artifacts exactly according to the authorized policy.
- **FR-015**: Each effect-bearing row MUST compare before and after inventories for listeners, child processes, trust, routing environment, journals, session leases, temporary keys, sensitive files, and declared retained outputs.
- **FR-016**: The run report MUST be append-safe and versioned, preserve exact failures, and terminate with one reconciliation record covering every expected row and all owned effects.
- **FR-017**: The public-safe summary MUST contain no capability credential, private key, payload content, raw certificate bytes, account identifier, host name, or absolute operator path. Sanitization MUST be validated before upload.
- **FR-018**: The required CI tier MUST run on Windows for pull requests and `main`, archive only validated public-safe evidence, and fail when a required portable or hosted-runner row does not execute.
- **FR-019**: Environment-dependent physical rows that a hosted runner cannot safely provide MUST require separately retained, current, validated evidence before the release completion gate can pass; they MUST NOT be marked skipped or inferred from simulations.
- **FR-020**: S129 MUST add no recurring schedule, no background monitor, no final packaging claim, no supply-chain completion claim, and no Deep Capture feature-completion claim.

### Key Entities

- **Windows Integration Registry**: The reviewed versioned set of required Windows rows and closed vocabularies.
- **Host Capability Snapshot**: Immutable facts about privilege, Npcap runtime, analyzer, address families, interactive access, and staged binary identity.
- **Integration Row**: One exact setup, action, expectation, effect boundary, evidence reference, and cleanup obligation.
- **Row Result**: The observed terminal outcome and bounded evidence for one registry row.
- **Residue Inventory**: Before and after observations of all fragcap-owned Windows effects and retained outputs.
- **Run Report**: Append-safe raw execution evidence with one terminal reconciliation.
- **Public-Safe Summary**: Validated derived evidence suitable for pull-request and release review.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One successful Windows run contains exactly one terminal result for 100 percent of required rows and zero skipped, missing, duplicated, stale, or unexpected results.
- **SC-002**: One source or registry change that creates an unreviewed completion domain causes the ordinary static gate to fail.
- **SC-003**: Every effect-bearing row ends with zero undeclared listeners, child processes, trust entries, routing effects, leases, journals requiring action, temporary keys, or sensitive files.
- **SC-004**: Non-admin and denied-authority rows complete with zero machine-wide mutations and zero unexpected elevation prompts.
- **SC-005**: Npcap-present and Npcap-absent evidence each produces the exact independent Capture and Deep Capture readiness outcomes defined by the registry.
- **SC-006**: Both IPv4 and IPv6 loopback rows complete with no wildcard listener and no address-family drift.
- **SC-007**: Crash recovery produces one exact action, no-action, or refusal for every retained obligation and leaves unrelated sentinel state byte-identical.
- **SC-008**: Unmodified analyzer consumption identifies the expected packet and TLS evidence from the staged binary output with zero custom plugin requirement.
- **SC-009**: Public-safe evidence validation detects every seeded prohibited secret or machine-identity class and publishes zero prohibited values.
- **SC-010**: The required hosted Windows job completes within 30 minutes; separately retained physical-host evidence carries a reviewed expiry and blocks release when absent or stale.

## Assumptions

- S129 closes issue #327 at the Windows integration authority boundary and does not absorb final distribution packaging from #329.
- The existing controlled target, local protocol origins, resource journal, Doctor recovery, conformance, failure, fuzz, threat, and performance authorities remain the implementation under test rather than being duplicated.
- GitHub-hosted Windows runners are suitable for portable Windows rows but do not reliably provide Npcap runtime, an interactive desktop, elevation transitions, or installed Wireshark.
- The operator's authorized Windows development host can supply current physical evidence for environment-dependent rows without exposing its identity or raw artifacts.
- Final release packaging will consume this matrix and replace staged-layout evidence with MSI/archive evidence under #329.
