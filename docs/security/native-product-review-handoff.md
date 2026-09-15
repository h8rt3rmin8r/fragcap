# Native product independent-review handoff

S150 prepares this handoff for candidate v0.10.0 under delivery #411. **Independent review has not been performed.** The twelve-area scope and existing controlled engineering evidence are readiness inputs, not a security verdict. Documentation #331 and independent review #333 remain open for reviewer findings and reconciliation; #334/#278 remain the final completion authorities. A green pull request or bot thumbs-up is not the independent installed-build audit.

## Freeze the candidate after merge and publication

The operator merges the release PR, tags the exact resulting main revision as `v0.10.0`, and lets the existing release pipeline publish only certified final bytes. Do not move a failed tag or substitute a locally rebuilt installer. Before review, freeze the actual tag/source revision, Cargo.lock SHA-256, package ZIP/MSI/catalog SHA-256 and sizes, certification run/report identity, exact official feature closure, architecture/target triple, product/native-backend versions, and observed packaged build identity. Those identities cannot truthfully be populated before the merge and package build. The committed [record template](native-product-review-record.v1.json) intentionally leaves them null/empty.

An independent reviewer copies the template to a separate review record, supplies reviewer identity and relationship to implementation authors, independence declaration, dates, methods, tools, environment and privileges, and the frozen candidate identities. Keep raw sensitive evidence private. Preserve the original not-started template; `cargo xtask review-handoff` validates readiness, not a filled review report or finding approval.

## Twelve required areas

The closed [scope registry](native-product-review-scope.v1.json) maps architecture (including threat model), dependencies, unsafe code, parsers, TLS, certificate issuance, trust, listener isolation, routing, artifact protection, recovery, and packaging to existing source paths, attributed non-ignored tests, and specific review methods. No area may disappear because it is inconvenient. Test references establish reproducibility and do not claim to cover every risk or replace independent source inspection. In particular, a denylist lint test does not prove FFI or unsafe memory correctness.

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

Default inventory/contract commands are static or controlled, not permission to install or inspect a real game. Record each command, version, exit status, timestamp, exact test filters and passed/failed/skipped counts. Independently repeat registry-referenced targeted tests; include parser abuse, failed upstream trust, unauthenticated local clients, destination refusal, CA identity/ownership mismatch, retained sensitive evidence, export transformations, and interrupted cleanup. Preserve skipped/indeterminate outcomes as incomplete, never passed. Analyzer conformance, parser fuzz campaigns, physical Windows integration and final-package dynamic evidence have distinct CI/operator prerequisites; use their existing documented harnesses and report identities rather than renaming an offline gate as a dynamic result.

## Installed Windows review is reviewer/operator-owned

Use a disposable Windows environment and the exact published, digest-verified MSI/ZIP, not the agent's sensitive host installation. Review final package certification first, then inspect the installed build under the frozen identity. Obtain explicit local authorization for every dynamic effect. Isolate local controlled origins and non-loopback access, synthetic data roots, owned test credentials and CA, and disposable accounts. Exercise authenticated/refused listener admission, scoped routing and destination policy, trust add/remove and failure recovery, private artifact protection/export, bounded parser behavior, and installer ownership/lifecycle with the existing constrained harness methods. Never point destructive certification or fresh-start tests at real research data.

No real game, account, traffic interception, host trust mutation, installed sensitive-product execution, or fresh-start deletion is delegated to the implementation agent. Optional real-title compatibility and Doctor #372 reproduction are operator work on newly published bytes, not prerequisites that require agent execution or an invented approval. Record any unexecuted installed-build review area explicitly and keep #333 open.

## Findings, remediation and independent retest

Give every finding a stable ID, area, severity, exact candidate/reproduction identity, evidence location, impact, owner and linked issue. Record remediation commit/PR, changed assumptions, and a separate independent retest with candidate identities, method and actual outcome. Critical/high findings require fixes and independent retests before #333 acceptance. Medium residual risks require an explicit owner and disposition; operator risk acceptance must name the finding and scope, not rely on a generic thumbs-up. Do not self-accept risks or infer permission from a bot response. Rejected or not-reproducible findings still need evidence-backed disposition; unresolved and indeterminate results remain open.

After remediation, publish a public-safe summary of actual scope, frozen bytes, independent reviewers, methods, findings by severity, dispositions, retest identities, remaining limits and omitted tests. Remove credentials, local title/account names, private paths/endpoints, host identifiers, raw payloads and key material. Reconcile documentation against reviewed executable/API behavior before requesting #331 closure. Only actual independent acceptance can satisfy #333; #334 remains a separate final gate.
