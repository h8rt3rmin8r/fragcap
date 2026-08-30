# Feature Specification: Managed Direct-Executable Launch

**Feature Branch**: `codex/101-managed-direct-executable-launch`

**Created**: 2026-08-30

**Status**: Draft

**Input**: User description: "Kick off S101", implementing issue #254, add managed direct-executable launch for stored targets.

## User Scenarios & Testing

### User Story 1 - Launch a Stored Executable Under Capture (Priority: P1)

As an operator, I can select a stored target and ask Capture to start its exact executable after capture and process watching are armed, so standalone titles receive the same race-free managed-launch behavior as Steam titles.

**Why this priority**: Directly installed and non-storefront games currently require manual launch and retain the startup race that managed launch exists to remove.

**Independent Test**: A stored target whose single Windows client resolves beneath its install root launches through the public library API with its exact executable path, working directory, and argument vector, while a controlled child proves that its final socket-owning process is captured and attributed.

**Acceptance Scenarios**:

1. **Given** a stored target with one resolved Windows client and an install root, **When** Capture is prepared with managed launch, **Then** one immutable direct-executable launch contains the exact path, working directory, and arguments before capture resources open.
2. **Given** a prepared direct launch and an armed Capture watcher, **When** the launch executes, **Then** the target and its descendants are observed and attributed through the existing watcher and socket-table paths without opening a target process for inspection.
3. **Given** a missing, non-file, outside-root, or ambiguous executable, **When** managed launch is prepared, **Then** it is refused before Capture or launch effects begin with a specific reason.

---

### User Story 2 - Reuse the Same Launch in Deep Capture (Priority: P1)

As an authorized Deep Capture operator, I can run a compatible stored direct-executable target with session-scoped proxy variables applied to the same prepared launch, so the child inherits routing configuration without system-wide proxy mutation or post-effect re-resolution.

**Why this priority**: Deep Capture cannot safely support standalone titles until it owns their child environment and preserves its preflight decision across proxy and trust effects.

**Independent Test**: A controlled local executable launched through the public Deep Capture API receives the expected proxy environment, reaches the loopback proxy, owns the observed socket, produces Capture output, and leaves complete cleanup truth without a game account or external service.

**Acceptance Scenarios**:

1. **Given** a target with current same-case compatibility evidence and a prepared cold direct launch, **When** Deep Capture starts it, **Then** target-scoped proxy variables are added to that exact prepared launch and inherited by the child.
2. **Given** preflight has prepared a direct launch, **When** the proxy or trust stage changes filesystem or store state, **Then** launch still consumes the retained path, working directory, and arguments without resolving the target again.
3. **Given** a warm direct-executable case, **When** Deep Capture preflights, **Then** it is refused because fragcap cannot retroactively change a running process environment.

---

### User Story 3 - Preserve Launch and Cleanup Truth (Priority: P2)

As an operator or reviewer, I receive explicit launch failures, cleanup results, and unchanged Steam behavior, so adding direct launch cannot hide partial effects or weaken an existing launch path.

**Why this priority**: Proxy and trust effects make launch ordering and terminal truth security-relevant, while Steam is already a shipped contract.

**Independent Test**: Fault-injected tests cover preparation and spawn failures before and after session effects, compare terminal cleanup authorities, verify argument fidelity, and retain every Steam launch regression test.

**Acceptance Scenarios**:

1. **Given** a spawn failure after proxy or trust acquisition, **When** the session finalizes, **Then** every acquired resource receives a bounded cleanup attempt and the terminal report cannot claim complete success.
2. **Given** arguments containing spaces, empty values, Unicode, quotes, or shell metacharacters, **When** the direct launch starts, **Then** the child receives the exact argument vector without shell interpretation, normalization, or redaction.
3. **Given** a Steam-anchored stored target, **When** managed launch is prepared and executed, **Then** its existing protocol request and behavior remain unchanged.

### Edge Cases

- The selected target has no install root, no resolved Windows client, or more than one distinct Windows client.
- The stored executable is absolute, escapes the install root, names a directory, is missing, or changes after preparation.
- The executable or working directory contains spaces, Unicode, or characters meaningful to a command shell.
- An argument is empty or contains quotes, backslashes, whitespace, or shell metacharacters.
- The child exits immediately, spawns the final socket owner, or remains running after the bounded observation period.
- Proxy or trust succeeds but process creation fails.
- A caller attempts to apply proxy variables to a Steam protocol launch or a warm direct-executable case.
- The environment already contains proxy variables with different values.

## Requirements

### Functional Requirements

- **FR-001**: The public `fragcap` library MUST expose one managed-launch preparation and execution surface consumed by Capture and Deep Capture.
- **FR-002**: Managed launch for a stored direct target MUST resolve exactly one Windows client executable from the existing target entry and MUST NOT create a second target or launch storage shape.
- **FR-003**: A prepared direct launch MUST carry an exact executable path, explicit working directory, and ordered argument vector as distinct typed values, never as a raw shell command.
- **FR-004**: The direct executable MUST resolve beneath the stored install root, and preparation MUST reject a missing install root, missing or ambiguous client, absolute stored client, path escape, missing file, or non-file before capture, proxy, trust, or launch effects begin.
- **FR-005**: Capture MUST consume the prepared launch only after its existing process watcher and packet capture resources are armed.
- **FR-006**: Direct launch MUST use operating-system process creation with explicit arguments and MUST NOT invoke a command shell, association handler, script host, or command evaluator.
- **FR-007**: Launch arguments MUST reach the child in their original order and value, including empty, Unicode, quoted, whitespace-containing, and shell-metacharacter values; logs and reports MUST NOT silently normalize or redact them.
- **FR-008**: Deep Capture MUST apply its target-scoped proxy environment to the same prepared direct launch retained during preflight and MUST NOT resolve the target, executable, working directory, or arguments again after effects begin.
- **FR-009**: Direct-executable Deep Capture MUST support only a cold launch owned by the session. A warm direct process MUST be refused before effects because its environment cannot be changed retroactively.
- **FR-010**: Direct launch MUST NOT mutate system-wide proxy settings, inject into the target, read target memory, or open a target process for inspection. Existing watcher and attribution mechanisms MUST bind the launched process and descendants.
- **FR-011**: The selected stored target MUST remain the sole identity and compatibility-fact owner across Capture, Deep Capture, process observation, and fact persistence.
- **FR-012**: Failures discoverable during launch preparation MUST be reported before mutable effects. A process-creation failure after proxy or trust acquisition MUST preserve a partial session and bounded cleanup truth for every acquired resource.
- **FR-013**: Existing Steam protocol managed launch behavior, validation, and public compatibility MUST remain unchanged.
- **FR-014**: The CLI MUST accept `capture --target <selector> --launch` and `deep-capture <selector> --launch` for eligible stored direct targets without adding a raw-command option or direct launch for `--process`.
- **FR-015**: CLI help, master specification, specification outline, glossary, security documentation, and Deep Capture guidance MUST describe the supported direct-launch boundary and its refusal cases.
- **FR-016**: Automated verification MUST use a controlled local executable to prove environment inheritance, exact argument delivery, process-descendant observation, final socket ownership, ordinary Capture output, proxy reachability, failure cleanup, and unchanged Steam behavior without a game account or external service.
- **FR-017**: The implementation MUST add no direct process-handle inspection, target memory access, system proxy mutation, or command-shell invocation, as enforced by tests and repository gates.

### Key Entities

- **Prepared Managed Launch**: An immutable side-effect-free launch decision, either an existing Steam protocol request or a direct executable configuration.
- **Direct Executable Configuration**: The exact executable path, working directory, ordered arguments, and target-scoped environment additions used for one child creation.
- **Stored Target**: The existing local database row that owns identity, install root, launch entries, and compatibility facts.
- **Launch Outcome**: The typed success or failure of issuing a prepared launch, including cleanup responsibility where a resource was acquired.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every supported stored direct target produces one prepared launch through the public library, and Capture and Deep Capture consume that same value without a second resolution.
- **SC-002**: Controlled verification proves exact delivery of 100 percent of an argument corpus covering empty, whitespace, Unicode, quotes, backslashes, and shell metacharacters.
- **SC-003**: Controlled verification proves the direct child inherits session proxy variables, reaches the loopback proxy, and is correlated to Capture and final socket ownership with no external service or account.
- **SC-004**: Every injected process-creation failure after proxy or trust acquisition produces a non-complete terminal result and records bounded cleanup attempts for all acquired resources.
- **SC-005**: All existing Steam launch tests and all full repository gates pass with zero unrecorded behavior changes.
- **SC-006**: Static and runtime tests find zero command-shell launches, system-wide proxy mutations, target-memory reads, or target-process inspection handles in the new path.

## Assumptions

- The direct launch is cold and session-owned; attaching Deep Capture to an already-running direct target remains unsupported because environment changes would not propagate.
- The install root is the executable's working directory unless the stored launch record later gains an explicitly governed working-directory field.
- Existing launch entries are the source of the client executable. This slice does not reinterpret `executable_hint` as an executable identity or migrate target storage.
- Existing launch-entry `arguments`, when present in governed target data, are parsed into an explicit argument vector during side-effect-free preparation. No shell syntax is supported.
- The process returned by operating-system creation is not queried or instrumented. Process lifecycle and descendants remain observed through the existing process watcher.
- The external `mitmdump` backend and current-user trust behavior remain unchanged.
