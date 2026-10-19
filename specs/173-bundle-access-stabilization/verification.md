# S173 verification

## 2026-10-19 controlled Windows diagnosis

Windows Sandbox CLI was present. A new disposable environment used networking disabled, read-only synthetic test tools and a separate writable synthetic result folder. Its initial guest had UAC disabled and no desktop session, so that state was not counted as evidence. Guest-only UAC enabling and restart plus desktop connection established one ordinary session with a shell. Host UAC, accounts, trust and owner output were unchanged.

A SYSTEM fixture launcher queried the actual logged-in user's token and linked full primary token, created a hidden high process with that user's environment on its desktop, and required TokenElevation nonzero plus TokenElevationTypeFull. All commands used hidden/noninteractive process creation. The dedicated Rust test called production OutputRecipient::desktop_session without injected recipient authority. It validated session-shell proof, created nested output containers through production protection, exercised independent application/lifecycle/resource/sensitive writers, empty and final/atomic artifacts, then compared complete actual ordinary-context reads after reconciliation.

A separate disposable standard account logged on through the real Windows logon API. Its token had a distinct account and ordinary elevation. It read a sibling positive control but could neither enumerate the private bundle nor read its retained files. Restricted equivalents remained supplemental, never the substitute for elevated or unrelated-principal evidence.

## Fail-before and pass-after

The published dea276c recipient algorithm was temporarily selected solely to build a private synthetic regression executable, then fixed source was immediately restored. Its genuinely elevated run failed with exit 101 at production recipient acquisition and Windows error 1346. A diagnostic-only stage label isolated the exact failure to DuplicateTokenEx on the linked ordinary token. No baseline algorithm change remains in the source.

The fixed binary SHA-256 is `69f0df2a0d4d1cc9dde49e91c6f0806fe8f26ab6da112f04582a7ec5d72cdbcd`. Rebuilt after the TLS/QUIC patches and final diagnostics, its genuine elevated test completed with exit 0: one passed, zero failed, zero ignored. It exercised both matching ordinary recipient reads and the actual unrelated standard principal denial. The ordinary exact-shell identity/session refusal test was explicitly run separately and passed. The ordinary suite labels the dedicated desktop rows ignored; those ordinary runs are not claimed as elevated evidence.

## Focused checks

- New retired-verb integration regression failed against the old commands (inspection ran instead of parser rejection), then all three cli_bundle integration tests passed after retirement.
- Exact source/literal-rights lint regression failed with its closed stub, then passed with the narrow exception and broadened-rights/transplanted-source refusals.
- Enabled Windows bundle_access ordinary rows passed. The strict elevated row was executed separately as recorded above.
- Facade unit tests passed: 178 passed, two explicitly ignored helper/dedicated rows; the actual named-pipe authenticated handoff, identity/path/timeout/elevation refusals and legacy protected cleanup tests passed.
- CLI unit verification initially caught three obsolete guidance assertions after removal of the repair command. Those assertions were corrected to require manual access guidance and one bundle-root occurrence; final full-gate results are recorded below when complete.

## Required advisory remediation (#478)

Fresh all-feature cargo-deny initially identified seven advisories: RUSTSEC-2026-0338, 0340 and 0343 in quinn-proto, and RUSTSEC-2026-0346 through 0349 in Rustls. Minimal patch updates to quinn-proto 0.11.18 and Rustls 0.23.46 preserve all existing feature/provider declarations. Downloaded manifests and upstream source diffs were reviewed as recorded in research.md. No unsafe line was added or removed, no new package entered the graphs, and waiver/review expiry stayed fixed.

Fresh cargo deny --all-features check advisories licenses bans sources: exit 0, advisories OK, bans OK, licenses OK, sources OK. Six independent cargo check --manifest-path ... --locked runs passed for fuzz, native performance, both historical proxy spikes and their two audit graphs. Generated outputs stayed under the ignored root target directory. All seven lockfiles carry fixed affected versions.

Final enabled facade tests: 178 passed, two explicitly ignored dedicated/helper rows; bundle access: three passed, one explicitly ignored elevated row. The latter was separately executed in genuine elevated mode and passed as recorded above. The disposable guest was stopped after final evidence, destroying guest-only account credentials and settings.

## Full verification and delivery

Final cargo xtask ci completed with exit 0, including format, all-feature Clippy, complete workspace tests, lint/dependency/license/supply-chain/package/wrapper/skill/docs/spec gates, guided calibration, threat model, review readiness/candidate, fuzz inventory, failure matrix, HTTP/TLS conformance, performance authority, native proxy tests and enabled facade tests. Required ordinary tests and separate genuine elevated evidence are distinguished above.

The earlier full attempt caught an invalid threat-registry reference to the explicitly ignored elevated row. The ordinary registry now cites only enabled ordinary regressions and links separate controlled Windows evidence; no test was made vacuously successful or gate weakened.

Hosted final-head CI and bounded external review follow official PR publication. Their final authoritative status is the PR checks and review conversation. No release publication is part of this slice.

## Limits

Synthetic disposable Windows evidence proves the scoped acquisition, access and retirement behavior. No real game, live Npcap run, owner bundle, host account/UAC/trust change or different-human field run was used. No universal target compatibility or independent security audit is claimed.

## 2026-10-19 hosted certificate recovery and package correction

At 10:39 EDT (14:39 UTC), the original SDK endpoint passed ordinary Windows TLS validation and returned HTTP 200. A subsequent standard TLS 1.3 handshake observed leaf thumbprint 3BE57E70C25D7167DDC7BC5326F6008A4F29D44A, valid through 2026-12-17T09:03:16Z. The existing URL, certificate validation and exact SDK checksum were unchanged. Only the three failed jobs were rerun. Native integration and platform passed; package certification passed SDK acquisition and build, then exposed the separate S173 import-inventory omission described below.

Final-package PE inspection reported precisely the declared ordinary imports plus user32.dll. T017 records the owning shell-query API contracts and adds only that exact Windows system DLL to the closed inventory. The package-report mutation regression failed before this policy correction because the required User32 fixture was absent. The correction retains strict rejection of missing User32, extra unreviewed imports and all other identity/lifecycle/report mutations. Final corrected gate results follow the PR checks; no package certification success is claimed by this interim paragraph.

The corrected package-authority suite passed all eight tests, with zero failed or ignored. A subsequent full cargo xtask ci completed with exit 0 and all checks passed, including the extended mutation regression. Product runtime source, dependency graph, native recipient logic, SDK URL/checksum and pinned workflows/scripts did not change in this correction. Hosted certification of the corrected candidate remains required before final handoff.

## 2026-10-19 delivery record

All nine hosted workflows passed on corrected implementation/policy commit `6035429453c66c6ca1f93e181e264e9df987653c`: [ci](https://github.com/h8rt3rmin8r/fragcap/actions/runs/39274171257), [audit](https://github.com/h8rt3rmin8r/fragcap/actions/runs/39274171656), [fuzz](https://github.com/h8rt3rmin8r/fragcap/actions/runs/39274171390), [docs](https://github.com/h8rt3rmin8r/fragcap/actions/runs/39274171203), [platform](https://github.com/h8rt3rmin8r/fragcap/actions/runs/39274171393), [Native performance](https://github.com/h8rt3rmin8r/fragcap/actions/runs/39274171412), [Windows native integration](https://github.com/h8rt3rmin8r/fragcap/actions/runs/39274171268), [Windows package certification](https://github.com/h8rt3rmin8r/fragcap/actions/runs/39274171338) and [Published review candidate](https://github.com/h8rt3rmin8r/fragcap/actions/runs/39274171488). Corrected certification passed actual final bytes and lifecycle, including the closed User32 import inventory. Issue #480 was closed after its actual vendor recovery and affected-check criteria passed.

The automatic opening code review and second/final requested code review completed without findings on `5e8e7795707fac820aaca46ec101c813b547b4f9`. Subsequent commit 6035429 changes only the exact package inventory, its rejection regression and supporting records; runtime product source and dependencies are unchanged from that reviewed implementation. Direct inventories of all inline review threads and submitted reviews are empty. The bot summary and no-major-issues receipt are published in [PR #479](https://github.com/h8rt3rmin8r/fragcap/pull/479). Every actual finding has been reconciled (none were submitted).

The separate security portion was acknowledged but produced no distinct published security-review result. No independent security approval is claimed. The authorized two-round cap is exhausted; no further review request is permitted. This precise review limitation is carried to the owner for final review rather than silently treated as a security pass.

This completion record is a documentation-only revision after the green candidate above. Task completion does not waive its own final-head checks: the authoritative checks on the exact PR head must all be green before the handoff message is sent. The PR conversation records that final revision and its actual check results. Human final review and merge remain pending; no merge, tag or release is performed by this slice.
