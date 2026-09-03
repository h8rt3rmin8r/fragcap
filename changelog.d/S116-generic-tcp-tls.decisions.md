<!-- spec-impact: 13.7, 19.6, 25, 28.1 -->
- Reuse the authenticated SOCKS5 and HTTP CONNECT routes, application JSON Lines, and existing body retention budget; no-ALPN trusted TLS becomes protocol-unknown only when its buffered prefix is not recognizable HTTP, and failed interception never downgrades.
