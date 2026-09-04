<!-- spec-impact: 25.5, 28.1 -->

- Added a versioned, executable native failure registry with thirty generated
  scenarios spanning both sides of seven journaled effects and eight lifecycle
  transitions.
- Added `cargo xtask failure-matrix` to validate ten controlled failure
  families, seven independently asserted outcome authorities, production
  effect and lifecycle-edge drift, and exact executable test evidence in
  ordinary CI.
- Early session failure after native proxy acquisition now performs the bounded
  listener stop exactly once before runtime cleanup, preserving the journaled
  cleanup and recovery contract.
