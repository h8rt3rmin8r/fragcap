# Research: Native Documentation and Reviewable Release Handoff

**Date**: 2026-09-15

## Documentation authority

Decision: Treat merged S149 and actual parser/readers as current implementation authority; retain historical changelog and spike documents as history. The read-only planning audit found provisional HAR status placeholders, omitted native transport/recovery/SDK coverage, a rejected README trust flag, and CLI descriptions that lag implemented ring/stream behavior. Correct each owning current surface rather than copying a stale side because its formatting looks canonical.

Rationale: P-9 and P-11 prohibit documenting unobserved facts or obsolete behavior. Existing `cli_reference` already compares public options to clap and tokenizes quoted worked commands without dispatch. Expand that mechanism rather than inventing a shell executor. Existing `ManifestDocument` tests validate published manifest specimens; target `schema validate` is not a generic artifact validator.

Alternatives: A text-only documentation sweep would leave examples unprotected. A CLI smoke that executes every example would violate user scope. A second parser would drift from the product.

## Independent review

Decision: Publish twelve-area source and test mapping plus immutable-identity and findings instructions with an explicit not-started template. Static CI checks readiness, not audit completion.

Rationale: #333 requires independent authorship and targeted installed Windows tests. Portable security gates and ordinary PR bot findings provide useful evidence but cannot stand in for that acceptance.

Alternatives: Claiming an empty register is clean invents approval. Requiring installed execution in S150 would conflict with user limits and the approved handoff scope. Branch-only provenance cannot identify audited bytes.

## Release preparation

Decision: Prepare v0.10.0 locally through the configured version-only operation and existing changelog/golden tools on `release/0.10.0`; never tag, publish, or mutate real host trust. Preserve the operator release gate.

Rationale: `release.toml` disables tag/push/publish and requires `release/*`. Current accumulated changes justify a minor release. Existing orchestration assumes main and unconditionally dispatches native tools, so invoke the underlying operations through the verified hidden launcher without changing those scripts.

Alternatives: Notes without a version-ready candidate defer the user's new bytes to another cycle. Running the main-only orchestrator would require unsafe preflight changes. Publishing or tagging exceeds authorization.
