# Native product independent-review handoff

Published baseline: [v0.10.1](https://github.com/h8rt3rmin8r/fragcap/releases/tag/v0.10.1).

S150 prepared this handoff under delivery #411; v0.10.0 was published on 2026-09-15 from `787739edfa8d748e25cb4b5c4965f5c936850d16`. **Independent review has not been performed.** The twelve-area scope and controlled engineering evidence are readiness inputs, not a security verdict. Documentation #331 and independent review #333 remain open for reviewer findings and reconciliation; #413 retains installed-build retest and #334/#278 remain final completion authorities. Green CI, publication or a bot thumbs-up is not the independent installed-build audit.

## Freeze the published review candidate

Use immutable tag `v0.10.1` at `a7d24962999d38d7ff130722859d473543864862` and the certified final downloads from successful [release run 35033304343](https://github.com/h8rt3rmin8r/fragcap/actions/runs/35033304343), summarized in the [publication handoff](../maintainers/v0.10.1-release-handoff.md). Do not retarget the tag or substitute locally rebuilt bytes. Before review, independently freeze the tag/source revision, Cargo.lock SHA-256, ZIP/MSI/catalog SHA-256 and sizes, certification report, exact official features, target triple, product/native-backend versions and observed packaged build identity. Those published bytes include S151 Doctor diagnostics and S152 owner-controlled registry protection; independent installed testing remains reviewer-owned. The [record template](native-product-review-record.v1.json) remains intentionally not-started and null/empty; fill a separate reviewer-owned record from actual frozen evidence, not from implementation-author approval.

An independent reviewer copies the template to a separate review record, supplies reviewer identity and relationship to implementation authors, independence declaration, dates, methods, tools, environment and privileges, and the frozen candidate identities. Keep raw sensitive evidence private. Preserve the original not-started template; `cargo xtask review-handoff` validates readiness, not a filled review report or finding approval.

## Twelve required areas

The closed [scope registry](native-product-review-scope.v1.json) maps architecture (including threat model), dependencies, unsafe code, parsers, TLS, certificate issuance, trust, listener isolation, routing, artifact protection, recovery, and packaging to existing source paths, attributed non-ignored tests, explicit package features, and specific review methods. Readiness requires Git-tracked Cargo test ownership and actual test-harness discovery under exactly the declared features with defaults disabled. The validator builds test targets and runs only their `--list` interface, not the referenced tests or the product. File-level and enclosing-module configuration cannot silently turn a referenced test into zero-test evidence. No area may disappear because it is inconvenient. Test references establish reproducibility and do not claim to cover every risk or replace independent source inspection. In particular, a denylist lint test does not prove FFI or unsafe memory correctness.

Review the governing [constitution](../../.specify/memory/constitution.md), [authorized-use context](../../AI_CONTEXT.md), [master specification](../fragcap-specification.md), [threat model](deep-capture-threat-model.md), closed [dependency policy](../../supply-chain/policy-v1.json), and [package certification policy](../maintainers/package-certification.md). Confirm P-1 exclusions, no open-proxy behavior, no pinning bypass, immutable exact authorization, scoped current-user trust, independently checked upstream destinations, finite ownership and retention, explicit loss, and exact recovery refusal.

## Reproduce controlled engineering evidence

From the frozen source checkout, use the pinned toolchain and locked dependencies. The ordinary local checks are:

```text
cargo xtask review-handoff
cargo xtask ci
cargo xtask msrv
cargo xtask conformance
cargo xtask threat-model
cargo xtask fuzz
cargo xtask failure-matrix
cargo xtask supply-chain
cargo xtask package-certification
cargo xtask windows-integration
```

Default inventory/contract commands are static or controlled, not permission to install or inspect a real game. Record each command, version, exit status, timestamp, exact test filters and passed/failed/skipped counts. Independently repeat registry-referenced targeted tests with `--no-default-features` and the exact declared `--features` values (omit `--features` for an empty array); include parser abuse, failed upstream trust, unauthenticated local clients, destination refusal, CA identity/ownership mismatch, retained sensitive evidence, export transformations, and interrupted cleanup. The two facade integration-test references require `--features deep-capture`. Verify at least one matching test actually executes. Preserve skipped/indeterminate outcomes as incomplete, never passed. Analyzer conformance, parser fuzz campaigns, physical Windows integration and final-package dynamic evidence have distinct CI/operator prerequisites; use their existing documented harnesses and report identities rather than renaming an offline gate as a dynamic result.

## Installed Windows review is reviewer/operator-owned

Use a disposable Windows environment and the exact published, digest-verified MSI/ZIP, not the agent's sensitive host installation. Review final package certification first, then inspect the installed build under the frozen identity. Obtain explicit local authorization for every dynamic effect. Isolate local controlled origins and non-loopback access, synthetic data roots, owned test credentials and CA, and disposable accounts. Exercise authenticated/refused listener admission, scoped routing and destination policy, trust add/remove and failure recovery, private artifact protection/export, bounded parser behavior, and installer ownership/lifecycle with the existing constrained harness methods. Never point destructive certification or fresh-start tests at real research data.

No real game, account, traffic interception, host trust mutation, installed sensitive-product execution, or fresh-start deletion is delegated to the implementation agent. Optional real-title compatibility and Doctor #372 reproduction are operator work on newly published bytes, not prerequisites that require agent execution or an invented approval. Record any unexecuted installed-build review area explicitly and keep #333 open.

## Findings, remediation and independent retest

Give every finding a stable ID, area, severity, exact candidate/reproduction identity, evidence location, impact, owner and linked issue. Record remediation commit/PR, changed assumptions, and a separate independent retest with candidate identities, method and actual outcome. Critical/high findings require fixes and independent retests before #333 acceptance. Medium residual risks require an explicit owner and disposition; operator risk acceptance must name the finding and scope, not rely on a generic thumbs-up. Do not self-accept risks or infer permission from a bot response. Rejected or not-reproducible findings still need evidence-backed disposition; unresolved and indeterminate results remain open.

After remediation, publish a public-safe summary of actual scope, frozen bytes, independent reviewers, methods, findings by severity, dispositions, retest identities, remaining limits and omitted tests. Remove credentials, local title/account names, private paths/endpoints, host identifiers, raw payloads and key material. Reconcile documentation against reviewed executable/API behavior before requesting #331 closure. Only actual independent acceptance can satisfy #333; #334 remains a separate final gate.
