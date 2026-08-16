<!-- spec-impact: none -->

**2026-08-16** Added a `Specification version lock-step` step to `ci.yml`
(slice S049), running `cargo xtask spec`. It asserts the specification's
`Applies-To` field equals the workspace version and that every `changelog.d`
fragment carries a valid `spec-impact` line, making constitution principle P-11
enforceable in continuous integration.
