<!-- spec-impact: 15, 17.2.1, 29 -->
Deep Capture no longer reports proxy environment propagation as confirmed merely because traffic reached the final client. Routing and propagation remain separate observations, ordinary eligibility uses current final-client routing evidence, TLS acceptance requires a correlated final-client flow, and silence remains inconclusive.
