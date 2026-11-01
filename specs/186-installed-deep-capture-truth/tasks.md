# Tasks: S186 Installed Deep Capture truth

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [health-and-tunnel.md](contracts/health-and-tunnel.md)
**Issues**: #534 and #535

## Phase 1: Identity and controlled reproduction

- [x] T001 Verify branch, feature pointer, spec directory, and issue references all identify S186 and #534/#535.
- [x] T013 Run Spec-Kit analyze over spec, plan, contract, and tasks; resolve critical and high findings before implementation completion. Initial analysis: 10 requirements and 16 tasks, with complete issue mapping and no critical or high finding. Re-run if scope changes.
- [x] T002 [#534] Add a failing controlled CA health regression for denied signer metadata with valid public identity and exact root; preserve missing-key and public-resource failures.
- [x] T003 [#535] Add a failing HTTPS tunnel regression for bare Host under CONNECT, explicit default port, distinct Host, and separate header timeout.

## Phase 2: CA health repair

- [x] T004 [#534] Evaluate public identity, cutover, validity, and exact root independently from the protected signer in `ca.rs`; expose unverified private proof without weakening ACL.
- [x] T005 [#534] Keep elevated sign/verify preflight and test valid signer, missing signer, decrypt/sign failure, and no pre-effect continuation. Existing controlled signer and preflight regressions passed in the full gate; no physical elevated signing operation was performed.
- [x] T006 [#534] Exercise help, version, discovery, CA status, and Doctor with clean human and JSON output; qualify the actual restricted installed signer under a headless ordinary Windows account. The current ordinary account observed signer metadata access denied and the candidate reported public `ready`, unverified signing, and no repair advice.

## Phase 3: HTTPS tunnel repair

- [x] T007 [#535] Pass tunnel scheme and effective default port to all inner HTTP/1.1 request parses, including later requests on the same connection; retain raw bytes and HTTPS URL identity.
- [x] T008 [#535] Preserve exact target scope and upstream TLS verification for equivalent requests; refuse genuine unsupported cross-authority requests with a precise compatibility reason and verify HTTP/2 parity.
- [x] T009 [#535] Identify the header-timeout read stage and preserve its finite deadline and independent terminal accounting.

## Phase 4: Evidence reconciliation

- [x] T010 [#535] Prove a controlled five-authority plus one-timeout pattern retains six distinct terminal causes and no full HTTPS fact.
- [x] T011 [#535] Prove a complete verified HTTPS exchange reconciles proxy, application, HAR, compatibility, packet, and cleanup evidence without promoting HTTP-only entries. The controlled bundle test checks each artifact; its packet-to-application correlation is explicitly unavailable because the controlled packet flow is synthetic.
- [x] T012 Update master specification, slice ordering, and a changelog fragment with exact behavior and limits. Keep private field identities out of committed files.

## Phase 5: Verification and delivery

- [x] T014 Run focused tests, full foreground `cargo xtask ci`, controlled Windows/package gates, and encoding/mojibake checks; inspect all results. The final full gate passed after the bundle assertion was strengthened.
- [ ] T015 Audit S186 identity and task completion, commit locally, push the authorized branch, and open an official PR with `Closes #534` and `Closes #535` only if both gates pass.
- [ ] T016 Monitor final-head CI and all external review comments or thumbs-up reactions; address every finding and use at most one second `@Codex` review request, then hand the green reviewed PR to the owner before merge.
