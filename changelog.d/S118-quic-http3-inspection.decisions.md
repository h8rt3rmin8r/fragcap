<!-- spec-impact: 13.7, 19.6, 25, 28.1 -->
- **2026-09-03, S118:** Promoted exact-pinned Quinn and added Hyperium `h3` plus `h3-quinn` under the existing rustls and ring stack. Zero round-trip application data and active migration are refused because replay and path changes cannot preserve one target-scoped authorization across two terminated QUIC connections (#314).
