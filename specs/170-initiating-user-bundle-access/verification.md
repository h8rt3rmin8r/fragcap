# S170 verification

2026-10-09 source checkpoint. All commands use a verified hidden foreground runner with redirected noninteractive I/O.

## Security regressions and focused gates

- Exact retained-file SID regression failed against the old protected OW/SYSTEM descriptor, then passed with explicit user SID/SYSTEM and no OW.
- Private CA producer-SID regression failed against its old OW descriptor, then passed with producer/SYSTEM only.
- Mismatched journal terminal identity was accepted by the old closure assumption; the regression failed before correction and passes with exact identity/count validation.
- Actual Windows bundle integration: five passed, zero failed or ignored. Complete fixture reads, protected/inherited/atomic outputs, unrelated restricted-token denial with positive control, historical denial/repair, byte/mtime conservation, sibling conservation and stale/unknown/external-hard-link refusal are covered.
- Repair unit tests: seven passed, including actual SetSecurityInfo error 5 after an earlier applied correction, unchanged bytes, explicit partial report and successful fresh-preview retry.
- Real local handoff/token tests: five passed, including finite missing-helper timeout, wrong path, mismatched identity and elevated-token rejection.
- CLI library: 349 passed; controlled Deep Capture integration: 30 passed; bundle CLI integration: three passed.
- Curated public API with `deep-capture`: 10 passed with default features and 10 passed with `--no-default-features`.
- Clippy across all targets and features passes with warnings denied.

## Completion gate

Full serial `cargo xtask ci` completed with exit 0 and `ci: all checks passed` on 2026-10-09. This includes formatting, warning-denied Clippy, workspace and documentation tests, lint/dependency/license/supply-chain checks, package certification, wrappers, skills, documentation, CLI reference, specification alignment, guided-calibration acceptance, threat model, independent-review handoff readiness, immutable review records, fuzz/failure inventories, HTTP/TLS conformance, performance authority and Windows integration authority. Handoff readiness does not claim independent review, and retained conformance does not claim a new live TShark run.

The final source audit verified exact recovery commands retain both bundle path and selected SID, access verification follows manifest reconciliation, repair validates closed journal identity/count, issuer-private material remains producer-only, and policy reconciliation adds only the two direct edges to already-resolved BLAKE3/getrandom packages. The exact-SID recovery action passes all 16 session UX tests. Text hygiene and final staged diff checks pass.

Hosted checks and returned external review disposition are recorded on the official pull request after this committed source checkpoint. The owner explicitly authorized push and official PR creation; automatic creation review is round one, and at most one additional combined code/security request is permitted. The owner retains merge and release authority. T021 remains open in this source checkpoint until the live PR completion record establishes the external gates.

## Verification limits

No real game, live Npcap session, owner-sensitive historical bundle, OS account creation, owner profile ACL mutation or trust mutation is used. Historical group-owner and unrelated-human scenarios use truthful labeled Windows restricted-token equivalents with actual filesystem operations. A different authenticated human account has not been field-tested. Production one-byte read probes establish actual access without reading or exposing full sensitive payloads; controlled fixtures use complete reads. Access, evidence completeness and external resource cleanup remain independent.
