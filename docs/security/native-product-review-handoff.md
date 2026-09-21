# Native product independent-review handoff

Published baseline: [v0.10.2](https://github.com/h8rt3rmin8r/fragcap/releases/tag/v0.10.2).

S150 prepared this handoff under delivery #411; S159 published the current v0.10.2 candidate on 2026-09-21. **Independent review has not been performed.** The twelve-area scope and controlled engineering evidence are readiness inputs, not a security verdict. Green CI, publication or a bot thumbs-up is not the independent installed-build audit. Any concrete finding against an active release enters the security advisory or defect workflow with its affected version and evidence boundary.

## Freeze the published review candidate

Use immutable tag `v0.10.2` at `13c9230d1e91b1fddd34de1f279963fa0e711ef4` and the certified final downloads from successful [release run 35557886659](https://github.com/h8rt3rmin8r/fragcap/actions/runs/35557886659), summarized in the [publication handoff](../maintainers/v0.10.2-release-handoff.md). The machine-readable [published review candidate](native-product-review-candidate.v1.json) freezes the tag, source revision, Cargo.lock SHA-256, release run, official feature set, target, all six public file sizes and SHA-256 digests, and the pinned predecessor. `cargo xtask review-record` validates that registry without running the product. Do not retarget the tag or substitute locally rebuilt bytes. These published bytes include S154 through S158 documentation, review-intake, deterministic certification and Windows observation corrections; independent installed testing remains reviewer-owned. The [record template](native-product-review-record.v1.json) remains intentionally not-started and null/empty; fill a separate reviewer-owned record from actual frozen evidence, not from implementation-author approval.

An independent reviewer copies the template to a separate review record, supplies reviewer identity and relationship to implementation authors, independence declaration, dates, methods, tools, environment and privileges, and the frozen candidate identities. Keep raw sensitive evidence private. Preserve the original not-started template; `cargo xtask review-handoff` validates readiness, not a filled review report or finding approval. Validate the separate completed record with `cargo xtask review-record <PATH>`. A zero exit means the closed schema, exact candidate, area coverage, installed cases and finding dispositions have no detected mechanical blocker. It does not establish reviewer authenticity or independence.

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

No real game, account, traffic interception, host trust mutation, installed sensitive-product execution, or fresh-start deletion is delegated to the implementation agent. Optional real-title compatibility and Doctor observation are operator work on newly published bytes, not prerequisites that require agent execution or an invented approval. Record any unexecuted installed-build review area explicitly and never claim it was performed.

The `Published review candidate` workflow supplies reproducible implementation-authored input on a disposable GitHub-hosted Windows runner. It downloads and independently verifies all six registry-bound v0.10.2 files, invokes the existing package-certification authority, executes separate portable and installed controlled native smoke under exact-program firewall containment, reconciles owned effects, and uploads only the bounded schema-version-4 report. Green hosted replay is candidate evidence, not the independent whole-product verdict.

The shipped controlled target exercises HTTP and HTTPS reachability. S158's Windows writer-start and terminal-drain correction is included in v0.10.2 and closed #413. That implementation result is not an independent whole-product security review.

## Findings, remediation and independent retest

Give every finding a stable ID, area, severity, exact candidate or reproduction identity, evidence location, impact, owner and linked defect or security advisory. Record remediation commit or pull request, changed assumptions, and a separate independent retest with candidate identities, method and actual outcome. Critical and high findings require fixes and independent retests before their defects close. Medium residual risks require an explicit owner and disposition; operator risk acceptance must name the finding and scope, not rely on a generic thumbs-up. Do not self-accept risks or infer permission from a bot response. Rejected or not-reproducible findings still need evidence-backed disposition; unresolved and indeterminate findings remain open as their own records.

After remediation, publish a public-safe summary of actual scope, frozen bytes, independent reviewers, methods, findings by severity, dispositions, retest identities, remaining limits and omitted tests. Remove credentials, local title or account names, private paths or endpoints, host identifiers, raw payloads and key material. Reconcile documentation against reviewed executable and API behavior in the same remediation flow.
