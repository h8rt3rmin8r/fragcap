# Deep Capture positioning

**Status:** planning record.\
**Date:** 2026-08-24.\
**Audience:** project owner, maintainers, future slice authors.

This document records the product and architecture decision to add Deep Capture to fragcap. It is not a feature slice and it is not an implementation specification. It is the source record that later constitution amendments, specification revisions, feature slices, documentation updates, and GitHub issues should cite.

## Decision

fragcap will present two user-facing capture experiences.

| Mode | User-facing meaning | Technical posture |
| --- | --- | --- |
| Capture | Passive, process-attributed packet capture. | Reads packets and system telemetry without placing fragcap on the application data path. |
| Deep Capture | Capture plus scoped application-layer inspection for traffic the selected target can route through a local inspection proxy. | Starts a local proxy, routes the selected launch through it where possible, manages a purpose-specific local CA, and correlates decrypted application records with the capture session. |

Capture remains the default and remains passive. Deep Capture is a deliberate, explicit, authorized inspection mode. It changes the connection path for the selected session, and the product will say that plainly.

The project will not describe Deep Capture as passive. The correct positioning is that fragcap began with passive attribution because attribution is the foundation, and Deep Capture adds the application-layer visibility that encrypted game traffic requires.

## Why Deep Capture exists

Modern PC game traffic is usually encrypted. Standard Capture still provides endpoint identity, timing, sizing, direction, process ownership, role attribution, flow lifetime, and loss accounting. That is useful, but it is not enough for many game developers, QA engineers, security researchers, and authorized operators who need to understand request and response behavior.

The prior Unity TLS research established the practical boundary. Passive packet capture cannot decrypt modern TLS 1.2 or TLS 1.3 sessions that use ephemeral key exchange. Hooking target methods, injecting instrumentation, or extracting TLS secrets from a process can work in a lab, but those approaches conflict with fragcap's public security posture and are brittle across game updates, engine versions, TLS implementations, and anti-cheat environments.

A local MITM inspection proxy is the viable direction because it is explicit, reversible, operator-visible, and comparatively light touch. When a target honors ordinary proxy configuration, fragcap can route only that launched session through a local proxy without changing system-wide proxy settings. That gives authorized users actionable application-layer data while preserving the project's refusal to inject code, install hooks, read target memory, or modify target binaries.

Deep Capture is therefore not a side tool. It is the next layer of the product: attributed capture for encrypted traffic, with application records tied back to the same target, process, role, flow, and session model users already see in Capture.

## Security posture

The existing passive posture becomes a mode-specific promise rather than the entire product definition.

The following techniques remain prohibited for fragcap:

- Code injection into a target process.
- Function hooking in a target process.
- Process handles carrying memory-read rights against protected clients.
- Executable image modification.
- Layered service providers and Winsock catalog modification.
- Packet interception or filtering drivers used to place fragcap inline below the application.
- Target TLS key extraction from process memory.
- Frida-based, debugger-based, or instrumentation-based decryption as a product capability.

The following techniques become allowed only for Deep Capture, under explicit user activation and visible cleanup rules:

- Starting a local inspection proxy owned by the fragcap session.
- Supplying proxy environment variables to a managed launch, such as `HTTP_PROXY`, `HTTPS_PROXY`, and related variables discovered during compatibility testing.
- Creating or locating a fragcap-owned local development CA for inspection.
- Installing trust for that CA only after explicit confirmation, with the thumbprint and store location recorded.
- Removing temporary trust, proxy state, key logs, and session metadata during cleanup.
- Emitting analyzer-compatible TLS key-log material for proxy-owned TLS tunnels, when the chosen proxy backend supports it.

Deep Capture must never silently weaken the machine. It must not install system-wide proxy settings by default. It must not silently add certificate trust. It must not leave trust or proxy state behind without reporting it. It must not pretend cleanup happened if it did not.

## User experience

Deep Capture should feel like an elevated Capture, not a second product.

The normal flow remains target-first:

```text
fragcap targets
fragcap capture <target>
fragcap deep-capture <target>
```

The exact CLI shape is left to the feature spec. The product requirement is that users pick a target once and receive one coherent session. Capture artifacts, proxy artifacts, status output, and analyzer integration all share the same session identity.

During a Deep Capture session, the user should see:

- The selected target.
- Whether fragcap is launching or attaching.
- Whether the proxy is active.
- Whether trust is already present, newly installed, refused, or unavailable.
- Whether proxy configuration reached the socket-owning process.
- Whether the observed protocol is inspectable.
- Where the session bundle is being written.
- What cleanup happened at shutdown.

Deep Capture should integrate with the existing live status and logging surface. Proxy events are not a separate logging universe. They should use the same severity, verbosity, structured event stream, timestamps, session identity, process identity, role labels, and machine-readable mode the rest of fragcap uses.

## Output model

Deep Capture needs a session bundle rather than one overloaded file.

| Artifact | Role |
| --- | --- |
| `.fcapng` | Packet truth: attributed frames, timing, sizes, directions, interfaces, loss accounting, and compatibility with unmodified analyzers. |
| `.jsonl` | Packet or application event stream for scripting and downstream analysis. |
| `.har` | HTTP-oriented transaction export when HTTP semantics are available. |
| TLS key log | Analyzer aid for proxy-owned TLS tunnels, treated as sensitive session material. |
| Session manifest | The bundle index: target, mode, start and stop times, artifact paths, proxy backend, CA thumbprint, compatibility facts, cleanup result, and correlation anchors. |

The `.fcapng` file remains the packet-capture artifact. It should not be forced to carry every decrypted application object. Instead, it carries enough session and flow correlation for another artifact to join against it.

HAR support should belong to the utility-wide output model, not only Deep Capture. A standard Capture can emit HAR only when HTTP semantics are actually observable, such as plaintext HTTP or a preexisting decrypted stream. Deep Capture is the mode expected to make HAR useful for modern HTTPS traffic.

The TLS key log is not the decrypted output. It is an analyzer aid. It must be marked sensitive, session-scoped, and never produced without the operator asking for analyzer integration or selecting an output profile that includes it.

## Scope model

Deep Capture starts with one simple scope: the selected target session.

The first version should avoid a host policy engine, broad filter language, or a large set of controls that users must understand before they can run the tool. The user selected a target. fragcap should capture and inspect everything it can observe for that target and record what happened.

The scope boundary is still important:

- No system-wide proxy settings by default.
- No attempt to inspect unrelated applications.
- No silent fallback from target-scoped launch configuration to machine-wide configuration.
- No claim that omitted traffic was inspected.

Host allow lists, deny lists, payload toggles, and other controls can be added later if real workflows require them. The initial design should keep the operator path short and the output complete.

## Compatibility facts

Deep Capture will not work uniformly across all games. The product should treat compatibility as observed data rather than repeated troubleshooting.

The local SQLite store should record behavior facts per target, with provenance and freshness. Candidate facts include:

| Fact | Example values |
| --- | --- |
| Proxy environment honored | yes, no, unknown |
| Variables tested | `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, `NO_PROXY` |
| Launch path tested | Steam running, Steam cold start, Steam protocol, direct executable, publisher launcher |
| Final socket owner | observed executable and role |
| Environment inheritance | reached final client, stopped at launcher, escaped observed tree, unknown |
| TLS trust behavior | accepts local CA, certificate pinned, unknown |
| Protocol behavior | HTTP, HTTPS, WebSocket, non-HTTP TLS, QUIC, UDP, plaintext |
| Inspectability | full, metadata only, unsupported, unknown |
| Evidence source | observed run, user-confirmed, imported catalog, stale observation |
| Freshness | timestamp, fragcap version, target version when available |

The compatibility matrix should be generated from these facts. It belongs in the user-facing documentation and should also be visible through `fragcap targets`, `fragcap targets show`, and `fragcap doctor` where useful.

Facts should not be refreshed silently if refreshing requires a launch or a trust change. Read-only observations can update opportunistically. Anything that alters launch behavior or trust state requires the same explicit Deep Capture consent as the session itself.

## Managed launch compatibility

Deep Capture depends on whether proxy configuration reaches the process that owns the sockets. Cold direct-executable launch is now the deterministic managed case: fragcap creates the exact stored executable with child-only proxy variables and retains the prepared path, working directory, and arguments across session effects. Steam and publisher handoffs remain compatibility-dependent.

Observed and suspected behavior to measure:

- A Steam-distributed game launched directly may start Steam, crash, and then be relaunched by Steam.
- A cold Steam start may escape the original terminal's environment.
- A Steam protocol launch may differ from direct executable launch.
- Publisher launchers can add another handoff layer, such as Ubisoft Connect or the Elder Scrolls Online launcher.
- A launcher can be the invoked process while a descendant is the socket owner.
- A handoff can preserve process ancestry but lose environment inheritance, or preserve neither.

Compatibility calibration records process events, socket ownership, environment propagation where the existing posture permits it, and proxy reachability for the exact launch case.

If a launch path escapes environment scope, fragcap should report that fact and store it. It should not silently promote to system-wide proxy configuration as a workaround.

## Doctor and cleanup

Deep Capture makes `doctor` more important.

Read-only startup checks are acceptable and should be considered for every invocation, especially before Deep Capture starts. Silent fixes are not acceptable.

Doctor should eventually detect:

- Stale fragcap-owned CA certificates.
- A fragcap CA trusted in the wrong store.
- Missing or mismatched CA thumbprints.
- Orphaned proxy processes.
- Occupied proxy ports from prior sessions.
- Stale TLS key logs.
- Stale session manifests.
- Output directories containing sensitive Deep Capture artifacts.
- Proxy backend availability and version.
- Analyzer key-log configuration readiness.

Cleanup actions should be explicit. `doctor --fix` can remove stale state under confirmation. Deep Capture startup can offer inline cleanup when stale state blocks a run, but it should not mutate trust or delete artifacts silently.

## Proxy backend strategy

The backend decision remains open and needs a dedicated discovery spike.

The long-term preference is a native Rust implementation if one can satisfy the product and repository constraints. The fallback is orchestrating a mature external proxy in a controlled way. The project should not freeze a Python proxy into the installer without first documenting why native Rust is not viable.

The discovery must evaluate:

- License compatibility with the repository allowlist.
- Rust MSRV and edition compatibility.
- Windows support.
- HTTP CONNECT support.
- HTTP/1.1 and HTTP/2 behavior.
- WebSocket support.
- Certificate generation and cache behavior.
- Local CA lifecycle hooks.
- TLS key-log export for analyzer integration.
- Dependency graph size and risk.
- Whether the backend requires traffic interception drivers or system-wide redirect mechanisms.
- How failures are surfaced to the caller.
- Whether the backend can emit structured transaction events suitable for correlation.

Any backend that requires code injection, target hooks, process memory reads, Winsock catalog modification, or traffic interception drivers is out for fragcap's default Deep Capture design.

External proxy orchestration remains a valid baseline for research and comparison. It should not become the final product path by convenience alone.

## Documentation posture

The documentation should be rewritten coherently, not patched around contradictions.

The new product story:

- fragcap offers Capture and Deep Capture.
- Capture is passive process-attributed packet capture.
- Deep Capture is explicit scoped inspection for authorized sessions.
- The project refuses target instrumentation and unsafe system-wide networking changes.
- Encrypted traffic is opaque in Capture and inspectable in Deep Capture only when the target and protocol support proxy inspection.
- Compatibility is measured and reported, not promised universally.

Documents that must be updated when the feature moves from planning into specs:

- `.specify/memory/constitution.md`
- `docs/fragcap-specification.md`
- `docs/fragcap-spec-outline.md`
- `README.md`
- `docs/plans/README.md`
- `docs/glossary/`
- CLI help text and examples
- `doctor` documentation
- site getting-started and reference pages
- security and authorized-use guidance

The rewrite should not pretend the first release already had Deep Capture. It should say the original passive capture foundation is still true for Capture mode, and Deep Capture is the planned expansion that makes encrypted application-layer inspection first-class.

## Initial workstreams

The first GitHub issues should be discovery and governance issues, not detailed implementation tickets. The implementation breakdown will be better after the riskiest unknowns are measured.

1. Position Deep Capture in the architecture: amend the constitution and master specification to introduce Capture and Deep Capture as first-class modes.
2. Research native Rust proxy backends: compare candidates against licensing, MSRV, Windows support, TLS, HTTP, analyzer integration, and dependency graph requirements.
3. Measure Steam and publisher-launcher inheritance: determine when proxy environment reaches the socket-owning process and when it does not.
4. Design the session bundle: define `.fcapng`, JSONL, HAR, key-log, and manifest relationships and correlation anchors.
5. Design compatibility fact storage: extend the local SQLite model to record observed Deep Capture behavior per target.
6. Expand `doctor` for Deep Capture: read-only detection, explicit cleanup, proxy readiness, trust state, and sensitive artifact warnings.
7. Define the Deep Capture MVP: one selected target, managed launch, scoped proxy configuration, local CA lifecycle, inspectable HTTP and HTTPS, correlated outputs, and reliable cleanup.

Issue #219 implements that MVP with an external `mitmdump` child behind a
replaceable boundary. Real sessions reuse the ordinary Capture pipeline and a
session-local packet flow registry. Continuous verification launches a
placeholder child through a live deterministic loopback adapter and exercises
the same observation ingestion, bundle, compatibility, and cleanup paths. The
Windows trust path is current-user only, installed only after explicit
confirmation, and removed when the session installed it. Session-selected ports,
proxy-private CA material, empty key logs, and incomplete sessions are all
reported and cleaned without system-wide proxy mutation.

## Non-goals

Deep Capture does not introduce:

- Frida integration.
- IL2CPP method hooks.
- TLS key extraction from target processes.
- Memory scanning.
- Debugger attachment.
- Certificate pinning bypass.
- Executable patching.
- Packet rewriting.
- A claim that every game can be decrypted.
- A broad proxy for unrelated system traffic.

Research artifacts may remain in external notes. They should not be promoted into fragcap product capabilities unless the constitution and security posture explicitly allow them.

## Success criteria

Deep Capture is successful when an authorized operator can select a known-compatible PC game target, start one command, and receive a session bundle that contains:

- An attributed packet capture readable by unmodified analyzers.
- Decrypted HTTP or HTTPS transaction records where the target and protocol support proxy inspection.
- Correlation between packets, flows, processes, roles, and application records.
- Clear status explaining what was inspectable and what was not.
- A compatibility fact update for the target.
- A cleanup report naming trust, proxy, and sensitive artifact state.

The feature fails if it produces plaintext without provenance, mutates trust silently, falls back to system-wide settings without consent, leaves residual proxy or certificate state without reporting it, or makes Capture's passive promise ambiguous.
