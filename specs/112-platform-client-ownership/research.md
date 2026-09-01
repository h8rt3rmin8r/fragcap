# Research: Cold Platform-Client Ownership

## Decision 1: Split platform start from title dispatch

**Decision**: A platform plan carries two explicit actions. The first creates the exact canonical platform executable with the session environment. The second dispatches the selected application and is unavailable until the Capture session observes the created root as the bound platform role.

**Rationale**: Starting and dispatching through one protocol-handler call cannot prove which existing platform instance handled the request or which environment it inherited. An observed root transition supplies a testable authority boundary and makes the ordering exact.

**Alternatives considered**:

- Keep `ShellExecuteW(steam://run/...)`: rejected because it returns no owned process identity and may target a warm client.
- Start the platform and dispatch immediately: rejected because process creation success is not equivalent to observed session ownership.
- Poll for platform readiness: rejected because polling by image name weakens identity and duplicates the existing event-driven process watcher.

## Decision 2: Use a value-producing platform adapter

**Decision**: Define a platform adapter contract that performs side-effect-free preparation and returns one immutable platform launch value. Steam is the first implementation. Runtime effects remain methods on the prepared value and are driven by the shared Capture orchestrator.

**Rationale**: Future platforms need different discovery and dispatch details but the same authority transitions. Returning a closed value keeps preparation testable, prevents adapter callbacks from becoming an alternate coordinator, and preserves one target resolution path.

**Alternatives considered**:

- Put Steam branches directly in Deep Capture: rejected because it would couple policy and effects to one platform.
- Store a trait object as the launch plan: rejected because the existing configuration is cloneable, comparable, and inspectable; opaque runtime behavior would weaken those properties.
- Add a platform-specific target table: rejected by constitution P-10.

## Decision 3: Discover and canonicalize the installed platform before effects

**Decision**: The Steam adapter uses the existing registry-backed Steam installation discovery, derives `steam.exe` beneath that root, canonicalizes both root and executable, validates containment and file identity, and retains the selected application identifier.

**Rationale**: This reuses already-authorized local facts and gives the preflight plan the exact executable it will create. No post-proxy rediscovery or path substitution is allowed.

**Alternatives considered**:

- Trust a default Steam path: rejected because installations move and multiple registry views exist.
- Use the protocol handler registration command: rejected because it can contain indirection and does not establish the selected installation as the owned root.
- Query a running Steam process path: rejected because warm state must refuse and no target process handle is permitted.

## Decision 4: Keep ordinary Capture behavior stable

**Decision**: Owned platform preparation is an explicit internal mode used by Deep Capture. Ordinary `capture --launch` keeps the existing Steam protocol request and output behavior.

**Rationale**: S112 addresses Deep Capture environment propagation. Changing ordinary Capture would add startup and lifecycle semantics unrelated to its passive capture contract and would violate the slice's backward-compatibility requirement.

**Alternatives considered**:

- Replace every Steam launch with owned platform launch: rejected as an unnecessary product behavior change.
- Add a public CLI flag in S112: rejected because Deep Capture already selects the required mode and another operator choice would permit an unsafe combination.

## Decision 5: Synthesize an exact platform-rooted Capture profile

**Decision**: Deep Capture preparation derives a validated profile containing an exact nonterminal `platform` stage and the existing resolved client predicates augmented with `descends_from = "platform"`. Descendant-first stage ordering and exact single-owner reconciliation apply to this session.

**Rationale**: The shared process session already owns event ordering, ancestry, role publication, acquisition, terminal lifecycle, and ambiguity. Extending its profile preserves one ownership authority and lets the orchestrator observe the platform role before dispatch.

**Alternatives considered**:

- Track platform ownership in a second Deep Capture state machine: rejected because two authorities could disagree about the same process.
- Match only `steam.exe` basename: rejected because the prepared exact path would be discarded.
- Treat the platform as terminal acquisition: rejected because platform traffic is not the selected game client.

## Decision 6: Refuse warm state by image-name presence

**Decision**: Any same-named Steam image in the startup snapshot refuses the cold path before effects. The diagnostic states that path identity is unavailable under the permitted no-handle snapshot.

**Rationale**: A false warm refusal is visible and recoverable. A false cold claim silently routes through an environment fragcap did not establish. P-9 requires the conservative choice.

**Alternatives considered**:

- Open a process handle to query the executable path: rejected because the project intentionally keeps the no-target-handle invariant mechanical.
- Ignore unrelated same-named processes: rejected because image name alone cannot prove they are unrelated.

## Decision 7: Separate routing and propagation evidence

**Decision**: Client-correlated proxy reachability remains the routing fact. Propagation is confirmed only when that proxy connection belongs to the exact terminal client beneath the owned platform root in the same session. Platform-only traffic, launcher-only traffic, silence, loss, and ambiguity cannot confirm propagation.

**Rationale**: The route may reach a process for reasons other than inherited platform environment. Keeping facts separate lets later calibration explain what was observed without overclaiming cause.

**Alternatives considered**:

- Set both facts from any proxy connection: rejected because platform background traffic is not game traffic.
- Infer propagation from ancestry alone: rejected because ancestry proves process ownership, not network configuration use.
- Merge both facts into one support token: rejected because the existing compatibility schema deliberately separates them.

## Decision 8: Add no dependency or schema migration

**Decision**: Use the existing standard library process API, Steam discovery, profile model, process watcher, socket correlation, and compatibility fact keys.

**Rationale**: Every required capability is already in the workspace. A new process, IPC, or platform crate would add no missing semantic property.

**Alternatives considered**:

- Add a Windows process-management crate: rejected because no process control beyond ordinary child creation is required.
- Add platform launch tables: rejected because existing target and compatibility storage already carries the facts.
