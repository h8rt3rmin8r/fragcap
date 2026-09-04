# Native Deep Capture Threat Model

This is the human review entry point for the native Deep Capture attack surface
shipped through S124. The canonical, executable inventory is
`deep-capture-threats.v1.json`; `cargo xtask threat-model` rejects incomplete
control ownership, missing or ignored tests, and unreviewed protocol or direct
proxy dependency changes.

## Security objective

Deep Capture is an explicit, target-scoped local inspection capability. Only the
selected target launch may use its exact loopback route and random session
capability. Every upstream destination is independently policy checked. Every
parser, observer, artifact writer, and lifecycle effect is finite, visible, and
recoverable from its owning evidence. Failure never expands into transparent or
system-wide forwarding.

Nothing in this path opens a target process handle, reads or writes target
memory, injects code or libraries, installs hooks, extracts target keys, modifies
an executable, or changes the system proxy. Child-scoped environment routing is
the concrete strategy. Any unavailable strategy is an explicit refusal.

## Trust boundaries and assets

The registry covers nine transitions: operator to immutable plan, target to
route, local client to listener, proxy to DNS, proxy to origin, bytes to parser,
TLS to session and upstream authorities, observation to artifact, and resource
obligation to recovery. The protected assets are the session capability,
routing and upstream authority, target ownership, per-session CA private key,
decrypted content, sensitive artifacts, and exact cleanup obligations.

No one observation proves a different boundary. A loopback connection does not
prove target ownership, a destination name does not prove an allowed resolved
address, successful client-facing TLS does not prove upstream trust, a filename
does not prove artifact ownership, and a PID or occupied port does not prove a
recoverable resource.

## Threat and control map

The version 1 registry carries ten high-risk rows:

1. unrelated local proxy use and confused-deputy forwarding;
2. prohibited upstream access and SSRF;
3. DNS answer rebinding and unsolicited UDP replies;
4. HTTP request framing desynchronization;
5. connection, stream, parser, retention, and peer exhaustion;
6. session certificate authority and upstream trust abuse;
7. artifact path, cleanup, and export theft;
8. interrupted cleanup and unsafe recovery;
9. target process or generation substitution;
10. protocol ambiguity and transparent downgrade.

Each row names prevention, detection, containment, evidence authorities, and
path-specific executable negative tests. S125 records no residual-risk
acceptance. A high-risk row without an enabled test fails the gate.

## Normalization rules

Normalization may expose one canonical identity but may not erase a security
distinction. IPv4-mapped IPv6 addresses are canonicalized before listener and
scope checks. Names are resolved by the proxy and every selected address is
policy checked. Dot segments, escaped paths, conflicting lengths, duplicate or
ambiguous framing, malformed certificate identities, changed QUIC endpoints,
and oversized representations are refused or bounded with named evidence. No
such input becomes eligible merely because a parser can produce a value.

## Review triggers

Update this model and its registry in the same change whenever any of these
changes:

- the exhaustive native protocol-family vocabulary;
- a direct normal or Windows-target dependency of `fragcap-proxy`;
- authentication, target ownership, routing, DNS, destination policy, TLS,
  certificate, parser, retention, artifact, cleanup, or Doctor authority;
- a referenced negative test or its failure semantics.

The gate compares the first two inventories mechanically and verifies every test
reference. Each protocol family also owns an explicit threat and executable
abuse-case mapping, so adding its name to an inventory cannot satisfy review by
itself. Reviewers remain responsible for detecting semantic changes inside an
existing dependency or authority.

## Scope boundary

This review closes issue #323. Fuzzing, performance and soak tests, Windows
integration, packaging, supply-chain automation, produced-artifact validation,
and final native completion remain owned by #324 through #334. This model does
not declare Deep Capture complete.
