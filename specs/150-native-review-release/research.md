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

## Authorized #413 research (2026-09-15)

Decision: Explicitly expand S150 following the operator's approval, preserving workload and performance authority. The fresh unchanged-code Windows campaign 34967816667 failed `quic-off` windows 4 and 5 with 389/371 events and 222625/213671 payload bytes queue-dropped at peak 4096. Its raw SHA-256 is `9c861b124091228b8359e941bbd2c61f720b4d3df128ce6fbe787e1fbce2649b`, source synthetic merge `6148765542737eed2f99cfb8fd88c65b5bdb9cd8`. This independently repeats the original hard-loss symptom, not a timing breach.

Rationale: Local inspection and the plan skill's read-only research agent agree that `writer_loop` uses the default 8 KiB byte buffer, so byte capacity dominates its existing 64-event batch limit. QUIC ingress produces distinct generic UDP and SOCKS forwarding records, plus already coalesced HTTP/3 stream/body records. Removing or coalescing datagrams would alter observations. `event_json` also deep-clones two owned object maps, making each record perform redundant allocation. The reports do not establish the exact historical Windows scheduler or storage stall; the supported diagnosis is avoidable writer write/allocation amplification, requiring post-fix Windows verification.

Decision: Test an explicit 64 KiB byte buffer while preserving the finite 64-event pending limit, nonblocking 4096-event queue, timeout/final flush, exact failure accounting, and existing serialization. Move owned JSON object maps instead of deep-cloning them. Prove physical write-count improvement with a deterministic counting writer and compare exact serialized output; extend mixed-payload storage-failure checks. Fresh fixed-code campaigns, not guesses about host scheduling, determine whether this correction clears #413.

Alternatives: Increasing the queue or weakening budgets hides the issue and is rejected. Backpressure into forwarding violates the existing sidecar contract. Eliminating per-datagram records changes truth and is rejected. Broad serialization redesign, new dependency, platform priority changes, or flush suppression are unnecessary and increase risk. Classification reuse is a smaller optional optimization and is deferred to keep the correction narrow. Existing controlled harness child launchers use null stdin and `CREATE_NO_WINDOW`; no installed product or real trust-store mutation is required for baseline/fixed measurement.
