<!-- spec-impact: 17.2.2, 25, 28 -->
S148 closes issue #379 without launching a target or changing network, trust, artifact, schema, dependency, or structured-output behavior. Embedded help states exact-case compatibility, certificate-pinning, and sensitive-artifact limits; runtime session failures retain their existing terminal evidence, and general Deep Capture completion remains issue #334.

The planned calibration error decorator is intentionally placed at the CLI dispatch boundary in `lib.rs` instead of inside the calibration state machine. That boundary already distinguishes human and JSON execution, so it can add prose without coupling presentation to durable workflow, fact, plan, or effect authority.
