# Implementation Plan: Evidence and Diagnostic Debt Reconciliation

**Branch**: `codex/s212-evidence-diagnostic-reconciliation` | **Date**: 2026-11-24 | **Spec**: [spec.md](spec.md)

## Summary

Reconcile the active Deep Capture issue queue with evidence and the constitution's implementation-closure rule, then extract one general diagnostic from the preserved disposable worktree. A bounded HTTP/1.1 head read will carry the exact observed byte count through the existing connection diagnostic and native `proxy.jsonl` only when a head-read failure has occurred. Terminal diagnostics JSON projects the same in-memory fact, while a separately requested bundle debug report summarizes valid counted failures from native records. The count never contains head content. Separately, repair a pre-existing CI audit blocker with a finite parent-scoped exception and patched isolated spike locks, while issue #640 retains the permanent replacement. The public branch excludes experiment-only TLS, routing, timing, dependency, and inspection changes.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.89; Markdown UTF-8 without BOM and with LF endings.

**Primary Dependencies**: Existing Tokio HTTP/1.1 reader, proxy connection diagnostic, facade mapping, native lifecycle stream, CLI terminal diagnostics projection, and bounded bundle debug-report builder. The diagnostic adds no crate or product workspace lockfile package. The separate audit repair pins `cargo-deny 0.21.0` and updates `url` and `form_urlencoded` only in two isolated spike locks.

**Storage**: Existing native `proxy.jsonl` connection diagnostic; terminal diagnostics JSON and an operator-requested bundle debug report are separate projections. GitHub issues remain the public work queue. No new native bundle artifact or durable store.

**Testing**: Controlled asynchronous read fixtures, proxy and facade unit tests, terminal diagnostics JSON tests, native artifact-to-bundle-debug-report readback for the same connection ID, locked all-features and six isolated cargo-deny audits with direct-parent and expired-waiver negative canaries, offline governed-policy validation with drift canaries, repository lint and Rust gates, and applicable PR CI. No live target, elevated driver, or installed-product claim.

**Target Platform**: Windows product, with platform-independent source tests where existing crates permit them.

**Project Type**: Rust workspace with proxy, facade, and CLI crates plus a public issue tracker.

**Performance Goals**: Acquire the count on the existing bounded head-read path with no second read, wait, retained payload, or connection task.

**Constraints**: Preserve current head-size and timeout bounds, terminal codes, inspection scope, routing, cancellation, and cleanup. A clean zero-byte EOF is ordinary no-head completion, while a zero-byte I/O error is a counted failure. Omit the new key when it does not apply rather than writing `null`. Keep `unsound = "all"`, scope nine temporary exceptions to `smoltcp@0.12.0`, mirror them in governed policy, expire them on 2026-12-15 UTC, and keep #640 open for the actual dependency replacement. A routine unchanged-source review renewal must cite exact source and graph evidence.

**Scale/Scope**: Reconcile the closed set of 18 affected public issues and 20 modified tracked source files in the original diagnostic worktree; extract only the six Rust diagnostic files named below. The independent #640 CI repair changes the exact files listed in the disposition ledger, covering the audit workflow, denial policy, governed policy and validator, and two isolated spike locks. Add controlled tests, specification, planning, and changelog text.

## Constitution Check

*Gate: checked before research and rechecked after the contract design.*

| Rule | Pre-design decision | Post-design check |
| --- | --- | --- |
| P-1, scoped Deep Capture | Count already observed bytes in the proxy only; no target memory, hooks, file edits, or broader routing. | The contract changes no collection method or routing decision. Pass. |
| P-4 and P-9, evidence truth | Report an exact observed count and preserve the existing terminal code; do not infer client intent or synthesize an HTTP exchange. | Clean EOF, read error, partial EOF, and size refusal have separate declared outcomes. No content is retained. Pass. |
| P-5, compatibility | Use an additive optional field in the existing version-one diagnostic; preserve readers and native artifact structure. The derived bundle report adds only counted, sanitized rows. | Omitted when unavailable or inapplicable; successful records retain their prior shape. Same-record bundle readback qualifies compatibility. Pass at design. |
| P-6, glossary first | Reuse existing HTTP-head and connection-diagnostic terms; define any genuinely new term in the master glossary before using it in the shipped specification. | The contract names an additive field and existing concepts, and the documentation task checks glossary coverage. Pass at design. |
| P-8, house standards | Use existing Rust crates and Markdown conventions, no new dependency or helper process. | Plan, contract, and task paths are within the slice; repository gates remain required. Pass. |
| Pinned artifact and dependency audit | A change to `.github/workflows/audit.yml` or `supply-chain/policy-v1.json` needs a dated changelog decision. A source-reviewed exception cannot silently turn unsound findings into a clean graph. | The waiver has nine exact advisory IDs, one allowed parent, a 2026-12-15 UTC expiry, offline policy mirroring, failing negative canaries, and separate #640 replacement. `changelog.d/S212-audit-exception.decisions.md` records the dated decision. Pass at design. |
| P-11, shipped-spec truth | Update the master specification and chronological plan only for behavior delivered by this branch. | No field outcome, experimental repair, or unverified S211 physical gate is described as shipped. Pass. |
| Development workflow | Complete specify, clarify, checklist, plan, tasks, and analyze before Rust implementation. | Analyze is a blocking gate; implementation follows only after it passes. Pass at design. |
| Issue closure | Close scoped implementation on controlled and repository evidence; create a separate active-release defect if later observed. | The tracker plan removes speculative future field runs as sole gates while preserving actual unresolved product defects. Pass. |

## Project Structure

### Documentation and tracker

```text
specs/212-evidence-diagnostic-reconciliation/
  spec.md
  checklists/
  research.md
  data-model.md
  contracts/http-head-failure-count.md
  contracts/disposition-ledger.md
  quickstart.md
  plan.md
  tasks.md
docs/fragcap-specification.md
docs/glossary/command-line-and-diagnostics.md
docs/glossary/index.md
docs/plans/README.md
changelog.d/
.github/workflows/audit.yml
deny.toml
spikes/http-mitm-proxy/Cargo.lock
spikes/native-proxy/Cargo.lock
supply-chain/policy-v1.json
xtask/src/supply_chain.rs
docs/maintainers/supply-chain.md
```

### Source and tests

```text
crates/fragcap-proxy/src/http1.rs
crates/fragcap-proxy/src/model.rs
crates/fragcap-proxy/src/runtime.rs
crates/fragcap/src/deep_capture/diagnostics.rs
crates/fragcap/src/deep_capture/native.rs
crates/fragcap/src/deep_capture/route_report.rs
crates/fragcap-cli/src/commands/assessment.rs
crates/fragcap-cli/tests/cli_bundle.rs
```

**Structure Decision**: Keep the existing proxy-to-facade-to-CLI boundary. `http1.rs` observes the count, proxy runtime retains it on the connection trace, the facade copies the typed fact, and the native and terminal JSON serializers emit it conditionally. The bundle builder reads the bounded native stream and projects only valid counted failures to `proxy.head_read_failures[]`; a CLI bundle fixture proves that path from the same source record. Seven Rust diagnostic source paths include `route_report.rs`, newly authored rather than copied from the original dirty worktree. The public tracker reconciliation and governed CI audit repair remain separately attributed.

## Delivery Boundary

The original dirty worktree remains intact until the clean branch and disposition ledger are independently reviewed. Extract only the bounded head-read count and its projections. Do not copy whole experimental files. The separate governed CI repair unblocks a pre-existing audit without claiming the affected dependency is repaired; #640 owns replacement before the finite waiver expires. Preserve unfinished direct-route and generic relay questions as independent issues, and do not equate successful source tests with an installed game result.
