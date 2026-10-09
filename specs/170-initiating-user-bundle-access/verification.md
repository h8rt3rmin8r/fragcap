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

## Hosted round-one corrections

The first hosted head exposed stale performance-harness dependency edges, Windows SDDL alias spelling in exact-SID test assertions, one CodeQL invalid-pointer annotation in ACL ACE decoding, and one external P1 review finding for newly created intermediate output directories. The standalone performance and fuzz lockfiles now add only the two existing-package edges without changing resolved versions. The locked local Windows short performance campaign passes all 14 cases. Exact-SID tests compare binary ACE identities rather than assuming Windows prints numeric SIDs instead of LA/LG aliases. ACL decoding checks valid/non-null ACL and ACE pointers, common ACE header/type/size, and bounded valid SID before interpreting variable-size records. A new nested-output regression demonstrated ordinary AccessDenied with the old intermediate creation, then passed with exact security attributes and verification at every new directory boundary, preserving pre-existing ancestor and sibling descriptors. Final local and hosted gates for this correction are recorded below and in the PR conversation.

The corrected source completed a second serial full `cargo xtask ci` with exit 0 and `ci: all checks passed` on 2026-10-09. All six bundle-access integrations, 170 facade library tests, both binary exact-SID ACL tests, 349 CLI tests, the remaining workspace/doc suites and every governance/conformance gate pass. Hosted final-head CI, CodeQL alert disposition and second-round review completion remain live PR checkpoints.

## Final-round corrections

The single authorized manual second round completed with two findings: legacy protected bundle cleanup must remain usable without adopting the new output contract, and issuer-private persistence must establish its ACL before creation rather than after writing. The legacy cleanup regression failed against the unsupported-root gate, then passed for final and validated crash-prefix indexes, open resource journals and a missing action journal. Cleanup preserves root descriptors and all non-selected bytes, uses the existing artifact sensitivity index, and creates a producer-private action journal when legacy protection prevents ordinary new-output preparation. It neither requires terminal access-repair provenance nor changes resource-recovery obligations.

Issuer persistence now creates producer/SYSTEM-only files with security attributes at creation, retains an exclusive handle and pinned parents, and removes only its newly created object through that handle on failed writes. Existing-file and NTFS alternate-stream regressions demonstrated actual failures before correction. Six issuer tests pass, including exact descriptor proof before the first byte, existing/hard-link content conservation, alternate-stream/device/normalization refusal and exact uncommitted cleanup. Different-account runtime identity remains outside this synthetic descriptor evidence.

Hosted elevated-runner fixture failures were traced to deriving the controlled group-denied token from the high producer token. The fixture now derives from the already validated ordinary recipient token and asserts ordinary integrity, exact user/session and deny-only groups; production authorization checks remain unchanged. The original CodeQL annotation was marked fixed on c336db8, while three related annotations require consuming GetAce output only after successful initialization. Production and test adapters now use MaybeUninit for that documented FFI output contract and retain explicit null/header/size/SID checks. No third review request is permitted.

The final-review source completed a third serial full `cargo xtask ci` with exit 0 and `ci: all checks passed` on 2026-10-09. This run passed 172 facade tests (one existing child probe ignored), 71 proxy tests, all six Windows bundle-access integrations, all six Windows access/token unit tests, 349 CLI tests and the remaining workspace, documentation and governance/conformance gates. Fresh exact-head hosted checks and CodeQL disposition remain live PR checkpoints. Local all-feature Clippy passes; local all-feature test linking requires an unavailable wpcap import library, so actual owning integration tests use the deep-capture feature and the full gate's selected feature set.

## Verification limits

No real game, live Npcap session, owner-sensitive historical bundle, OS account creation, owner profile ACL mutation or trust mutation is used. Historical group-owner and unrelated-human scenarios use truthful labeled Windows restricted-token equivalents with actual filesystem operations. A different authenticated human account has not been field-tested. Production one-byte read probes establish actual access without reading or exposing full sensitive payloads; controlled fixtures use complete reads. Access, evidence completeness and external resource cleanup remain independent.
