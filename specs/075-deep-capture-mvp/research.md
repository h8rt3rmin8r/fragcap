# Phase 0 Research: Deep Capture MVP

## Decisions

### R-1: `mitmdump` is the MVP backend

**Decision**: The first functional MVP uses an external `mitmdump` child process as the local inspection proxy backend.

**Rationale**: The repository already has read-only doctor detection for `mitmdump`, the operator has it installed locally, and it is the fastest auditable path to prove launch-scoped proxy routing, application observation, bundle correlation, and cleanup. A native Rust backend is still a desirable follow-on, but it should be selected after the product semantics are proven.

**Alternatives considered**:

- Native Rust proxy first: deferred because dependency selection, TLS stack behavior, root handling, WebSocket handling, HAR shaping, and MSRV/licensing review would delay the MVP before the product path is proven.
- Python mitmproxy embedded or frozen into the distribution: rejected for the MVP because it adds packaging complexity and obscures the backend boundary.
- No proxy backend until native Rust is ready: rejected because it leaves Deep Capture untestable as a product.

### R-2: Stored target only

**Decision**: The MVP requires a stored target and does not support raw `--process` Deep Capture.

**Rationale**: Deep Capture needs compatibility facts, managed launch metadata, stable target identity, and a local fact update destination. Raw process names cannot provide those without creating a second target path.

**Alternatives considered**:

- Support `--process` for convenience: rejected for the MVP because it cannot safely answer scoped launch or compatibility update requirements.
- Auto-create a target from `--process`: deferred because it would mix target authoring with a sensitive capture command.

### R-3: Refuse unknown launch-scoped proxy compatibility

**Decision**: For real targets, Deep Capture starts only when stored facts show a known compatible scoped proxy path. Unknown compatibility is a refusal, not an attempted run.

**Rationale**: Prior fact-finding showed that storefront and publisher launch handoffs can escape the original process environment. Attempting those paths under Deep Capture creates network hangs and ambiguous results while teaching the user little.

**Alternatives considered**:

- Try unknown paths and learn opportunistically: rejected for the default MVP because launch and trust changes are too sensitive to run as a guess.
- Fall back to system-wide proxy settings: rejected by the constitution and product posture.

### R-4: Synthetic controlled target is the verification anchor

**Decision**: The MVP verification path includes a controlled local target that can be launched by tests, routed through the proxy, and made to perform predictable HTTP and HTTPS requests.

**Rationale**: The acceptance criteria require a proxy path demonstration that does not depend on a third-party game account, remote service availability, or real local title data. The controlled target also gives CI a deterministic path through bundle and status generation.

**Alternatives considered**:

- Use locally installed games as the primary test: rejected for committed verification because it risks PII/title leakage and cannot run in CI.
- Use only fake adapters: insufficient because the feature must prove real process, proxy, and output orchestration semantics.

### R-5: Trust mutation is adapter-driven and explicit

**Decision**: CA material and current-user trust changes go through a dedicated trust adapter. Every trust mutation requires explicit user confirmation or a documented pre-confirmed command flag.

**Rationale**: Trust is the most sensitive Deep Capture side effect. Keeping it behind an adapter lets tests prove refusal, install, retained-trust, and cleanup behavior without touching the real machine.

**Alternatives considered**:

- Silent trust installation for convenience: rejected by P-1 and Deep Capture positioning.
- Session-only untrusted CA: insufficient for HTTPS targets that rely on normal OS trust.

### R-6: Bundle writer is shared session infrastructure

**Decision**: Deep Capture writes the bundle contract from #216. HAR remains an observability-driven output type, not a mode-exclusive artifact.

**Rationale**: The user experience is Capture elevated by application visibility. Packet truth stays in `.fcapng`; application semantics and sensitive analyzer aids stay in sidecars; the manifest explains both.

**Alternatives considered**:

- Proxy-only logs: rejected because they do not correlate with packets, roles, or cleanup.
- A custom archive format: rejected because it would hide ordinary pcapng compatibility and delay analyzer workflows.

### R-7: Privacy scan is a feature gate

**Decision**: The implementation task list includes an explicit scan of committed artifacts for local paths, real title names, account identifiers, endpoints, tokens, and captured payload material.

**Rationale**: The fact-finding process used real local software, but the repository must not commit those identities or any user/account material.

**Alternatives considered**:

- Rely on reviewer memory: rejected because this is exactly the kind of accidental leakage a gate should catch.
