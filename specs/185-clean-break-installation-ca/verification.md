# S185 verification record

## Local source gates (2026-10-31)

| Check | Result | Evidence boundary |
| --- | --- | --- |
| `cargo test -p fragcap-proxy --lib installation_tests` | Pass, 3 tests | Exact root reload, two leaf issuances under the same root, mismatched signer, and altered root signature. These are controlled certificate tests, not live HTTP CONNECT or HTTP/3 sessions. |
| `cargo test -p fragcap-cli --lib ca::tests` | Pass, 3 tests | Public identity validation and seven controlled health states; no Windows trust-store mutation. |
| `cargo test -p fragcap-cli --test cli_ca_startup` | Pass, 1 test | The actual hidden binary entry reports the observed CA state before help, version, structured version, and a non-Deep status command while preserving stdout. |
| `cargo test -p fragcap --features deep-capture --test deep_capture_session unresolved_route_owner_retains_installation_trust_without_a_session_trust_obligation -- --exact` | Pass, 1 test | A controlled failed route-owner release leaves the installation trust lease unretired; it does not prove a real managed game or proxy remains reachable. |
| `cargo test -p fragcap-cli --test cli_deep_capture` | Pass, 31 tests in the full workspace gate | Existing controlled Deep Capture behavior and CLI contract. |
| `cargo xtask supply-chain` | Pass | Reviewed policy tracks x509-parser promotion and the measured graph; no new lock package. |
| `cargo xtask spec`, `cargo xtask threat-model`, `cargo xtask package-certification`, `cargo xtask windows-integration` | Pass individually | Source and static registry checks, not MSI execution or physical trust-store readback. |
| `cargo xtask ci` | Pass on the final source tree | Full workspace gate after the startup test, clean cutover, supply-chain policy, and Spec-Kit edits; 21 required conformance rows pass. |
| `git diff --check` and mojibake search | Pass | No whitespace errors or replacement-character corruption in S185 source/spec fragments. |

## Hosted and physical evidence

The official PR's exact-head CI must qualify the signed package and execute the MSI on its disposable Windows runner. The runner's package script checks clean install, exact fingerprint persistence through repair/reinstall/upgrade/uninstall, first-install pre-S185 cutover, explicit removal, and exact transaction rollback. These claims remain pending until the job finishes and its output is read.

The local workstation has not installed or trusted the candidate CA, run a real-title Deep Capture session, measured a standard-user startup against the protected key ACL, or exercised UAC denial. The leaf test uses the same signer path that both HTTP CONNECT and QUIC/H3 call, but it is not a wire-level two-session conformance result. S185 preserves the exact historical physical integration registry; no new physical acceptance claim is inferred from it.

The MSI treats a CA provisioning failure as a failed installation transaction and invokes its exact-resource rollback. An already installed passive binary remains usable after a failed repair/upgrade; a failed clean MSI transaction does not leave a new binary. The first executable launch from a portable copy continues with a specific CA warning and refuses Deep Capture. This distinction is carried into the PR for owner review against issue #529's passive-usability wording.
