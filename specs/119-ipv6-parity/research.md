# Research: Complete IPv6 Parity

## Decision 1: Authorize One Exact Listener Family Per Session

**Decision**: Replace the port-only facade endpoint with one exact loopback `SocketAddr`. Keep IPv4 as the CLI default and add explicit IPv6 selection. Bind exactly that address.

**Rationale**: One exact endpoint keeps plan, authorization, runtime, routes, journals, and cleanup identical. Binding separate family-specific sockets avoids platform-dependent dual-stack wildcard semantics and makes the no-external-bind property directly testable.

**Alternatives considered**: One unspecified IPv6 socket with OS dual-stack behavior was rejected because `IPV6_V6ONLY` defaults vary and mapped peers become implicit. Two listeners per session were rejected because existing routing has one endpoint and a second listener expands lifecycle and correlation ownership without product need.

## Decision 2: Use A Finite RFC 8305-Style TCP Race

**Decision**: Deduplicate allowed DNS results canonically, interleave families while preserving resolver preference within each family, and start attempts 250 ms apart under the existing total connect deadline. Return the first success and drop every other future before forwarding.

**Rationale**: RFC 8305 recommends staggered attempts rather than simultaneous connections, a 250 ms default delay, and cancellation after the first success. A single owned future set makes one winner and finite cleanup explicit. The existing resolver API does not provide independent asynchronous A and AAAA answers, so S119 applies the connection-attempt portion after one bounded lookup and does not claim full asynchronous-resolution conformance.

**Alternatives considered**: Sequential attempts were rejected because one broken family consumes the full deadline. Simultaneous attempts were rejected because they create avoidable load and wider same-turn winner ambiguity. A new DNS dependency was rejected as disproportionate to issue #315.

**Primary reference**: [RFC 8305 sections 4, 5, and 8](https://www.rfc-editor.org/rfc/rfc8305)

## Decision 3: Preserve Numeric IPv6 Scope As Socket Metadata

**Decision**: Accept a bounded decimal zone on bracketed IPv6 literals, convert it to `SocketAddrV6::scope_id`, and permit it only for link-local or multicast addresses. Do not include the zone in certificate identity. Refuse empty, named, overflowed, or inapplicable zones.

**Rationale**: RFC 4007 defines zone indexes as local interface qualifiers, and current RFC 9844 notes that decimal interface numbers are commonly accepted by operating systems. Keeping the index in the socket value prevents it from being mistaken for globally meaningful authority data. Named interface resolution is host-specific and would require another platform seam.

**Alternatives considered**: Rejecting all zones would leave issue #315 incomplete. Preserving arbitrary zone strings was rejected because they cannot safely reach Rust socket APIs and are unbounded. Applying zones to global or loopback addresses was rejected because those addresses do not need a local interface qualifier.

**Primary references**: [RFC 4007](https://www.rfc-editor.org/rfc/rfc4007), [RFC 9844 sections 3, 5, and 6](https://www.rfc-editor.org/rfc/rfc9844)

## Decision 4: Separate Observed And Canonical Address Identity

**Decision**: Preserve the exact observed `SocketAddr` in events, while normalizing IPv4-mapped IPv6 to native IPv4 for policy comparisons, candidate deduplication, peer ownership, flow identity, and correlation.

**Rationale**: Mapped addresses can describe the same transport peer through two textual families. Canonical ownership prevents duplicate admission and attribution, while exact observed evidence avoids rewriting what the OS reported.

**Alternatives considered**: Normalizing every emitted address was rejected because it destroys the observed family. Treating mapped and native forms as different peers was rejected because it duplicates ownership and destabilizes correlation.

## Decision 5: Extend Existing Protocol And Artifact Authorities

**Decision**: Exercise IPv6 through the existing HTTP, TLS, SOCKS, TCP, UDP, QUIC, application JSON Lines, HAR, manifest, lifecycle, and correlation paths. Add family or selected-peer fields only where the current record lacks enough exact socket identity.

**Rationale**: IPv6 is an address-family dimension, not a new protocol or artifact. Reusing the existing authorities preserves conservation and avoids schema forks.

**Alternatives considered**: A separate IPv6 evidence stream or parallel protocol engines were rejected as competing authorities.

## Decision 6: Doctor Performs Two Exact Read-Only Bind Probes

**Decision**: Doctor binds `127.0.0.1:0` and `[::1]:0` independently, immediately drops each socket, and reports separate checks with exact failure text.

**Rationale**: A real ephemeral bind measures the capability Deep Capture needs without external traffic, persistent state, elevation, or wildcard exposure. Independent values prevent one family from masking the other.

**Alternatives considered**: Interface inventory was rejected because loopback presence does not prove bind readiness. A combined Boolean was rejected because it cannot name the unavailable family.
