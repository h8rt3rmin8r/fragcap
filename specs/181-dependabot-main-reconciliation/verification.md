# Verification: S181

## Baselines

- The #508 main docs run failed when offline compiled-test discovery could not download Tokio 1.53.3. Its PR skipped the failing root-contract step.
- New workflow regressions failed against the original workflows, identifying missing locked preparation, PR parity, site audit, and isolated graph coverage.
- Pinned pnpm reported five site vulnerabilities before updates: one critical, three high, and one low, including both direct and transitive Zod paths.
- Fresh native-spike audits reproduced stale Tokio/webpki and fourteen Zstd advisories. Both native graphs required a patched compression chain; their existing decoder features remain enabled.

## Local source qualification

- `cargo xtask ci`: passed the complete foreground sequence, including format, all-feature Clippy, workspace tests, repository conventions, graph/licensing/supply-chain policy, package contract, wrappers, skills, docs, specification, calibration, threat model, review records, fuzz, failure matrix, conformance, performance, and Windows registry authorities. Existing explicitly ignored desktop-shell and dedicated Windows matrix tests retain their separate execution boundaries.
- New dependency-workflow tests: 2 passed, including mutations of locked preparation, ordering, conditional validation, narrowed audits, suppressed failures, omitted graphs, and cold-cache wiring.
- Fresh cargo-deny 0.20.2 advisory checks: all seven committed Rust graphs passed with zero advisory errors/warnings. Full root advisory/license/ban/source checks also passed; existing permitted duplicate-version warnings remain warnings under unchanged policy.
- `cargo check --locked --all-targets` passed for all six isolated projects against their patched lockfiles.
- Both corrected native-spike manifests also passed `cargo +1.88.0 check --locked --all-targets`, verifying their already-required dependency floor.
- Node 24.11.0 and pnpm 10.26.0: frozen site installation passed, complete advisory audit found no known vulnerabilities, and all 4 site unit tests passed.
- Production site export passed with the pinned toolchain: 95 static pages and required `.nojekyll`/`CNAME` markers.
- Production accessibility: all 14 tests passed in 3.4 minutes, including every public route at three viewport widths.
- Final repository lint passed; all 30 changed files passed UTF-8, BOM, line-ending, final-newline, and mojibake inspection.

## Operational completion record

Hosted PR review/checks, integration, final-main checks, isolated cold-cache docs dispatch, Pages deployment, bot reevaluation, and final zero-alert/PR inventory are delivery operations recorded in [issue #511](https://github.com/h8rt3rmin8r/fragcap/issues/511). Source qualification does not claim these later operations have happened. The issue maintains their current checklist and exact run/SHA receipts after source publication.

## Scope and provenance

The root product graph, release versions, and published release identity are unchanged. Site Zod 4.6.6 incorporates #510 and removes both older direct/transitive copies; DOMPurify is 3.4.17. Six isolated locks use rustls-webpki 0.103.16; four historical locks use Tokio 1.53.3. Both native-spike graphs use compression-codecs 0.4.40, zstd 0.14.3, zstd-safe 8.2.0, and zstd-sys 2.1.2. The audit-only native project also advances async-compression from 0.4.12 to 0.4.43 and its coordinated Brotli dependencies. Pnpm 10.26.0 regenerates platform metadata while preserving unrelated package versions and integrity values.
