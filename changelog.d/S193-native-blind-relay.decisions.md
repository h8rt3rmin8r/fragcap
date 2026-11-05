<!-- spec-impact: 27.3 -->
# S193 Decisions (2026-11-04)

1. Reuse the S191 frozen owner decision on the existing authenticated listener. A hostname bypass or second listener would break same-origin game and launcher separation and duplicate destination and route-owner authority.
2. Relay HTTP/1 CONNECT and SOCKS TCP as original bytes. Plaintext HTTP/1 and cleartext HTTP/2 still use their framing engines, with excluded application event sinks and retention disabled; retained connection diagnostics carry the relay decision and transfer totals.
3. Keep UDP association ownership independent from its TCP control decision. An excluded association forwards QUIC Initial as an original datagram and cannot create the session-CA QUIC gateway.
4. Preserve the public CLI's launch-chain behavior until #561. This slice supplies the runtime branch consumed by that operator integration and by #560 calibration; it does not claim #542's launcher failure resolved or a pre-release installed-product test.
