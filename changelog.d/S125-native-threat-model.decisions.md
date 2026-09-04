<!-- spec-impact: 19, 25, 28.1 -->

- 2026-09-04: Kept the canonical threat inventory as versioned JSON under
  `docs/security` and its validator in xtask. This makes the same reviewed data
  readable by people and enforceable in CI without adding product code or a
  dependency.
- 2026-09-04: Accepted no implicit high-risk residual risk. Every high-risk row
  must retain exact executable negative evidence, while fuzzing, performance,
  Windows integration, packaging, supply-chain, and final completion remain in
  issues #324 through #334.
- 2026-09-04: The full gate exposed that controlled lifecycle tests could reuse
  a retained bundle path after operating-system process identifier reuse. Added
  a per-invocation time component to the test-only path so cleanup-interruption
  evidence remains deterministic without changing product behavior.
- 2026-09-04: Added the threat-model command explicitly to both hosted CI
  operating-system legs. The workflow runs repository gates individually, so
  wiring only the composite local `cargo xtask ci` command would not protect
  pull requests.
- 2026-09-04: Closed the review inventory over parsed enum variants and match
  arms instead of line layout, and made executable evidence reject malformed
  objects plus line-commented, block-commented, ignored, and conditional test
  declarations. Formatting a Rust arm or disabling a test cannot silently
  preserve security-review currency.
