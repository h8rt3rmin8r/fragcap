# Research: Direct Socket Route

## Decision 1: Use the selected signed WinDivert component for packets and socket events

WinDivert NETWORK can block and reinject packets but has no process ID. SOCKET and FLOW expose process ID and endpoints but no payload; FLOW events cannot be blocked. The adapter must observe before managed launch, join an exact socket event to process generation and packet tuple, and retain a bounded unknown state when ordering is insufficient. A SOCKET observer uses sniff plus receive-only flags so observing a connect does not block it. WinDivert timestamps use QueryPerformanceCounter, not Unix time: event-to-packet ordering stays in that clock domain, while a bounded sampled conversion is needed only to compare with existing process-generation evidence. The official component packaged by S199 is the only driver dependency. [Upstream layer and timestamp documentation](https://reqrypt.org/windivert-doc.html).

The broad interception alternative, adopted by mitmproxy's redirector, handles all nonloopback traffic and reinjects unselected packets. It is unsuitable as a default for this workstation because unselected packet handling would share the route's user-mode queue and could disrupt unrelated UDP or service traffic. The S200 design starts with a PID-bearing, nonblocking event observer and opens narrowly filtered packet handles for exact tuples only after owner proof. A missed first packet is an explicit bypass; positive routing requires observing the initial TCP SYN or first UDP datagram through that handle. A bounded provisional first-packet handle may be evaluated only if the sentinel matrix proves its effect. [Mitmproxy Windows redirector](https://github.com/mitmproxy/mitmproxy_rs/blob/main/mitmproxy-windows/redirector/src/main2.rs).

## Decision 2: Use a bounded local TCP/IP stack at the packet-to-stream boundary

Rewriting a destination to a loopback listener and relying on the Windows stack to accept the modified flow has uncertain local-address and loopback-direction semantics, especially for IPv6 and reply injection. A local stack gives explicit TCP sequence, retransmission, half-close, and UDP datagram ownership, and keeps the native proxy at its existing stream boundary. `smoltcp` 0.12.0 supports IPv4/IPv6 and Rust 1.80, within this workspace's 1.89 floor; it and its `managed` 0.8.0 dependency carry 0BSD. Both require exact reviewed, explicitly approved dependency-license exceptions before inclusion. Newer 0.13 and 0.14 require Rust 1.91. No Python component or mitmproxy API is introduced. [smoltcp release notes](https://github.com/smoltcp-rs/smoltcp/releases), [smoltcp license](https://github.com/smoltcp-rs/smoltcp/blob/main/LICENSE-0BSD.txt).

## Decision 3: Bind original flow separately from helper socket

The native proxy currently looks up the owner of its client-facing socket. Once the local route stack creates a helper leg, that lookup names Fragcap. A session-private one-use receipt ledger must provide the original flow as an alternate typed authority for a bridge-created connection; the helper socket remains separately visible. Missing or consumed receipts cannot become positive game ownership. The bridge negotiates existing HTTP CONNECT for a client-facing TLS case and SOCKS5 CONNECT or UDP ASSOCIATE for opaque raw transport, while the final classification comes from the proxy's observed protocol and TLS facts. A destination port alone never decides inspectability.

## Decision 4: Compare WFP connect redirection

Microsoft's ALE connect-redirection model can select a specific connection and preserve its original destination before packets move, which directly avoids the event-to-packet race. It requires a WFP callout driver that is outside the S199 selected component and package policy. WinDivert remains selected; its race is exposed as a named bypass rather than concealed as successful routing. [Microsoft connect redirection](https://learn.microsoft.com/en-us/windows-hardware/drivers/network/using-bind-or-connect-redirection).

## Open implementation proof

The narrow-handle event timing may cause a high bypass rate for short-lived sockets. The disposable Windows matrix, not a paper design, decides whether the selected adapter meets #581 and #582. If no positive raw-socket flow can be established without broad unrelated disruption, the S200 gate is red and the route cannot be offered as complete. That outcome would require an explicit architecture decision rather than an invented success claim.

## Decision 5: Report the WinDivert driver queue limit

The owner approved keeping WinDivert with explicit uncertainty. WinDivert can lose packets in its kernel queue before `WinDivertRecv` returns them and provides no exact pre-receive drop counter. Adapter accounting is therefore exact only for delivered packets. The product must surface the pre-receive count as unmeasurable, and the controlled original-flow oracle plus active sentinels must show zero observed missing replies or unexplained loss. This is a documented change to FR-006 and SC-003, not a proof that no kernel-queue drop occurred.

## Dependency approval and source review, 2026-11-10

The owner explicitly approved only `smoltcp` 0.12.0 and its transitive `managed` 0.8.0 under 0BSD for S200. `cargo info` confirms the exact smoltcp Rust floor of 1.80, 0BSD metadata, and the selected feature set without raw-socket, TUN/TAP, libc, or defmt support. Its 18 source `unsafe` blocks are in optional Unix device adapters that are disabled for the selected Windows path. `managed` has 0BSD metadata and no source `unsafe` block in the inspected downloaded crate. The exact Cargo graph and source digests will be recorded after lock resolution; the ordinary deny policy admits only the two named exceptions and retains its original global allowlist.
