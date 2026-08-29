# Contract: Deep Capture Adapters

## General rules

Adapters perform narrow external effects and return typed results. They do not choose lifecycle order, classify evidence, select compatibility facts, determine the overall outcome, render CLI text, or choose exit codes. Inputs contain no CLI types.

Every blocking operation receives a finite budget. Every error contains a stable code, lifecycle stage, and optional diagnostic detail. Resource-producing operations return owned leases rather than raw child processes or untracked handles.

## Required boundaries

- **ProxyBackend**: describes and starts one loopback-only proxy. Startup returns a `ProxyLease` that supplies session material and observations, supports bounded stop, and supports bounded cleanup.
- **TrustManager**: acquires optional explicit current-user trust and returns a `TrustLease`. Reachability mode never invokes it. Cleanup removes only session-owned trust material.
- **LaunchAdapter**: prepares and starts the resolved managed launch without process-global proxy environment mutation. Proxy variables are scoped directly to the owned launch.
- **CaptureRunner**: consumes the facade-owned prepared ordinary Capture request and returns packet, flow, process, stop, and ownership observations. It does not implement a Deep Capture-specific acquisition path.
- **CompatibilityRepository**: resolves existing prerequisites during preflight and appends one candidate fact at a time. It never aggregates or overwrites evidence.
- **ArtifactSink**: persists one named artifact role and returns an independent result. Required JSON artifacts use atomic replacement where practical; manifest is attempted last.
- **Clock**: supplies wall time and monotonic time for evidence and budget enforcement.
- **SessionIdSource**: supplies deterministic injectable session and plan identifiers.
- **EventSink**: accepts ordered typed events and reports delivery success or failure without owning policy.

## Resource ownership

Each lease permits one authoritative stop or cleanup attempt. The coordinator records and caches the result. Explicit cleanup followed by `Drop` cannot duplicate the effect. `Drop` is a best-effort safety backstop only and cannot supply successful audit evidence.

## Production and controlled implementations

The facade provides production adapters for the shipped external backend, current-user trust, managed launch, ordinary Capture, target-store facts, filesystem artifacts, system time, identifiers, and event forwarding. Controlled tests can replace all privileged, platform, process, network, filesystem, and time effects.

No adapter contract permits system proxy mutation, pinning bypass, target process memory access, injection, hooks, target key extraction, executable modification, Winsock modification, or an interception driver.
