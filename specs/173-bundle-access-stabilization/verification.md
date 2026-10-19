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
