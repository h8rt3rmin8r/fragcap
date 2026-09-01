<!-- spec-impact: 25, 28.1 -->
**2026-09-01:** S110 closes issue #305 as the Native Deep Capture 2 conformance gate. Portable loopback tests remain offline and bounded, analyzer proof runs in a dedicated TShark CI tier, committed evidence rejects secrets and drift, and generic TCP, SOCKS, UDP, QUIC, HTTP/3, and IPv6 remain under milestone 3 issues #310 through #315.

**2026-09-01:** Review strengthened the analyzer authority from ordinary packet parsing to key-log-dependent application proof. The committed pcapng now carries a synthetic TLS 1.3 exchange, and TShark must decrypt the exact HTTP method and host; transport fields or an ignored key log cannot pass.
