<!-- spec-impact: 13.7, 27.3 -->
Deep Capture now reports transport-ended TLS handshakes and structured invalid TLS messages separately, preserving the accepted connection, handshake stage, transport category, and any explicit peer alert. A completed CONNECT remains distinct from successful TLS or inspected HTTP work.
