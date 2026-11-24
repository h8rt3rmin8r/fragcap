# Contract: Disposable Source Disposition Ledger

## Public tracker inventory

These 18 issues are the closed S212 audit set. Changed issues require an exact readback of body, status, labels, milestone, and dependencies; reviewed-unchanged issues require a recorded reason for retaining their present scope. Earlier completed children remain indexed by the epic and are not reopened.

The separately discovered CI blocker [#640](https://github.com/h8rt3rmin8r/fragcap/issues/640) is outside this 18-issue reconciliation set. Its finite S212 exception does not close the dependency-replacement issue.

| Issue | S212 disposition |
| --- | --- |
| [#607](https://github.com/h8rt3rmin8r/fragcap/issues/607) | Reconcile the epic's evidence map and child status. |
| [#594](https://github.com/h8rt3rmin8r/fragcap/issues/594) | Close the obsolete composite gate while retaining its historical observations and independent successor work. |
| [#635](https://github.com/h8rt3rmin8r/fragcap/issues/635) | Close the completed comparison without promoting it to a general compatibility claim. |
| [#636](https://github.com/h8rt3rmin8r/fragcap/issues/636) | Narrow to the bounded HTTP-head failure count; keep open until its controlled contract is merged. |
| [#622](https://github.com/h8rt3rmin8r/fragcap/issues/622) | Keep the wider documentation alignment separately actionable. |
| [#596](https://github.com/h8rt3rmin8r/fragcap/issues/596) | Correct priority and dependency metadata for remaining product work. |
| [#632](https://github.com/h8rt3rmin8r/fragcap/issues/632) | Correct tracker metadata without changing the observed defect. |
| [#633](https://github.com/h8rt3rmin8r/fragcap/issues/633) | Correct tracker metadata without changing the observed defect. |
| [#634](https://github.com/h8rt3rmin8r/fragcap/issues/634) | Correct tracker metadata without changing the observed defect. |
| [#613](https://github.com/h8rt3rmin8r/fragcap/issues/613), [#615](https://github.com/h8rt3rmin8r/fragcap/issues/615), [#617](https://github.com/h8rt3rmin8r/fragcap/issues/617), [#620](https://github.com/h8rt3rmin8r/fragcap/issues/620), [#638](https://github.com/h8rt3rmin8r/fragcap/issues/638) | Review and retain independent work; #617's generic relay lifetime/terminal classification is not fixed by the HTTP-head count. |
| [#610](https://github.com/h8rt3rmin8r/fragcap/issues/610), [#621](https://github.com/h8rt3rmin8r/fragcap/issues/621), [#637](https://github.com/h8rt3rmin8r/fragcap/issues/637), [#580](https://github.com/h8rt3rmin8r/fragcap/issues/580) | Review and retain current disposition without expanding this slice. |

## Original source inventory

This is the planned public extraction boundary for the 20 modified tracked files in the preserved diagnostic worktree. Each row names a repository-relative path, not a private research location. The implementation phase must verify every substantive hunk against this ledger and the final diff before retiring any original work. `Extract count only` means a reviewed hunk, never a whole-file copy. `Retain privately` does not claim that the experiment is correct or shipped.

| Original tracked path | Planned disposition | Public boundary |
| --- | --- | --- |
| `Cargo.lock` | Retain privately | No experiment-only package enters S212. |
| `Cargo.toml` | Retain privately | No experiment-only crate feature enters S212. |
| `crates/fragcap-cli/src/cli.rs` | Retain privately | No diagnostic experiment switch enters S212. |
| `crates/fragcap-cli/src/commands/assessment.rs` | Extract count only; retain other hunks privately | Debug-report count projection only. |
| `crates/fragcap-cli/src/commands/deep_capture.rs` | Retain privately | No launch or experiment control enters S212. |
| `crates/fragcap-cli/src/events.rs` | Retain privately | No experiment event contract enters S212. |
| `crates/fragcap-proxy/src/http1.rs` | Extract count only; retain other hunks privately | Bounded head-read error count only. |
| `crates/fragcap-proxy/src/inspection.rs` | Retain privately | No inspection-policy experiment enters S212. |
| `crates/fragcap-proxy/src/model.rs` | Extract count only; retain other hunks privately | Optional typed connection field only. |
| `crates/fragcap-proxy/src/runtime.rs` | Extract count only; retain other hunks privately | Existing trace propagation only. |
| `crates/fragcap-proxy/src/socks5.rs` | Retain privately | No relay behavior change enters S212. |
| `crates/fragcap-proxy/src/tls.rs` | Retain privately | No TLS ordering or provider change enters S212. |
| `crates/fragcap-proxy/tests/https_proxy.rs` | Retain privately | Experiment tests do not qualify S212. |
| `crates/fragcap-proxy/tests/inspection.rs` | Retain privately | Experiment tests do not qualify S212. |
| `crates/fragcap/src/deep_capture/application.rs` | Retain privately | No application evidence schema experiment enters S212. |
| `crates/fragcap/src/deep_capture/diagnostics.rs` | Extract count only; retain other hunks privately | Facade typed copy only. |
| `crates/fragcap/src/deep_capture/direct_bridge.rs` | Retain privately | No helper or relay timing change enters S212. |
| `crates/fragcap/src/deep_capture/direct_runtime.rs` | Retain privately | No direct routing or packet work change enters S212. |
| `crates/fragcap/src/deep_capture/native.rs` | Extract count only; retain other hunks privately | Conditional native diagnostic projection only. |
| `crates/fragcap/src/deep_capture/route_events.rs` | Retain privately | No route-event timeline change enters S212. |

Final verification must record the exact public diff, confirm all 20 paths remain represented, and preserve the original dirty worktree until extracted changes pass controlled tests and review. The six extract paths may gain new targeted tests in the clean branch; those are S212 tests rather than copied experimental test files. `crates/fragcap/src/deep_capture/route_report.rs` is a seventh, newly authored public diagnostic source path, not one of the original 20. It projects only valid counted head-read failures from bounded native `proxy.jsonl` into the sanitized bundle debug report; `crates/fragcap-cli/tests/cli_bundle.rs` proves same-record readback.

## Independent CI repair files

These seven public files repair a pre-existing audit gate and are not part of the original 20-file diagnostic extraction. No product workspace `Cargo.lock` change is authorized by this repair.

| Public path | Bounded purpose |
| --- | --- |
| `.github/workflows/audit.yml` | Pin locked `cargo-deny 0.21.0` for the existing audit matrix. |
| `deny.toml` | Keep `unsound = "all"` and add nine exact advisory-ID exceptions for `smoltcp@0.12.0` only, expiring 2026-12-15 UTC. |
| `spikes/http-mitm-proxy/Cargo.lock` | Resolve patched `url 2.5.9` and `form_urlencoded 1.2.3` in this isolated graph. |
| `spikes/native-proxy/Cargo.lock` | Resolve the same patched line in this isolated graph. |
| `supply-chain/policy-v1.json` | Pin the same audit tool, mirror nine governed exceptions with exact parent, expiry, and removal condition, and renew the due rustls review against unchanged source and graph evidence. |
| `xtask/src/supply_chain.rs` | Validate the tool pin and exact denial-policy mirror; test missing, extra, broad, stale, and spoofed exception records. |
| `docs/maintainers/supply-chain.md` | Document the temporary exception, its limits, and removal procedure for maintainers. |

The bounded source review and positive/negative audit outcomes are recorded in [#640](https://github.com/h8rt3rmin8r/fragcap/issues/640). The exception is not a general safety conclusion. A dated changelog decision records why the pinned audit workflow and governed policy changed.
