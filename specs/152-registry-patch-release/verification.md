# S152 Verification Record

## Scope and Authorization

S152 explicitly authorizes full spec-kit, controlled implementation, automatic branch push and official PR. It does not authorize human merge, tag/publication, deployment approval, installed sensitive application execution, production Doctor, real games or operator-host trust changes. Branch `release/0.10.1` follows pinned version-preparation policy. Actual publication remains immutable v0.10.0. This record is chronological and distinguishes completed evidence from pending checks.

## Specification Gate (2026-09-15)

Specify, clarify, checklist, plan, tasks and blocking analyze completed before implementation. Both requirements-quality checklists pass 16/16. Ten functional requirements and four success criteria map to 19 tasks, with 100 percent coverage and no artifact/constitution findings. Installed helpers successfully resolved feature paths, created the plan template and resolved tasks prerequisites. The pre-plan checklist helper's plan requirement was explicitly overridden by constitution ordering, then the post-plan prerequisite check succeeded. No extensions or hooks exist. Two plan-skill-requested research agents worked read-only.

## Protection Configuration (2026-09-15 19:53:39 UTC)

Before: environment id 21294112472, name `crates-io`, no protection rules, deployment policy null, administrator bypass true. No unrelated timer or custom protection existed. Secrets and variables were neither read nor mutated.

The first nested CLI request incorrectly serialized reviewers as an object and returned HTTP 422 without changes; the subsequent policy request returned expected 404 because custom policies were still disabled. A corrected scoped environment JSON request succeeded. Although current official mutation schemas omit administrator bypass, the bounded `can_admins_bypass=false` request took effect. Independent fresh public GET confirmed false; this observation supersedes the earlier research assumption that an operator UI action might be needed. No UI setting change was performed and no deployment was approved.

After: same environment id/name, exactly one required-reviewer rule (id 65684159), sole User reviewer id 46768484/login `h8rt3rmin8r`, self-review prevention false, administrator bypass false, one branch-policy protection rule and custom-only policy (`protected_branches=false`, `custom_branch_policies=true`). A complete public allowance GET returned `total_count=1`, one tag rule id 60078197/name `v*`/type `tag`, with no branch allowance. Existing environment identity, secrets and variables were preserved.

Two freshly downloaded public API records passed `cargo xtask release-guard target/s152-environment.json target/s152-policies.json`, exit 0. The output explicitly states configuration/wiring verified and no deployment approved or release published. Configuration readback is not actual approval-held release evidence; the next separately authorized release must establish that independently.

## Test-First Guard Implementation

`cargo test -p xtask --locked release_guard` before implementation: expected red, 3 passed and 5 failed. Negative classes exposed missing/unapprovable reviewer, bypass, wrong identity/unknown rules, wider/partial allowance, malformed/oversized records and removed approval/dependency wiring. After minimal implementation all initial 8 tests passed. Two additional tests cover stale/skippable/nonblocking guard mutation, bounded/missing input and bad invocation. Full xtask and workspace gates are pending below.

## Candidate Preparation

Installed `cargo release version 0.10.1 --workspace --config release.toml` preview and `--execute --no-confirm` version-only step both exited 0. The execute step changed versions only, without commit, tag, push or publication. Root and isolated lock graphs reconcile first-party versions only. Candidate writer assertions, staged-binary identity, portable conformance matrix/report and not-started independent-review record align to 0.10.1; no independent review result is invented.

Owned synthetic golden generators passed: facade goldens 7/7, CLI capture 24/24 and CLI extcap 19/19. Binary sizes are unchanged; embedded product version moves from 0.10.0 to 0.10.1. No fixture traffic or schemas change.

Fresh supply-chain snapshot: Linux-all digest `9e0cf3ee6bd450b1043865c876dceb7559b96912155682dcad1c3e46ce1a89a4` (187 packages/448 edges), Windows-all `0236443021441e81a966f7e0a95cb7143e3a9d6b8888c6ffd778a3c5d34613bd` (186/424), shipped Windows `fe46ebdbc5016bf5064739e1e497ad4827fe771272e1ffb50ff247e1af3d59d0` (148/317). Rebinding is first-party metadata reconciliation, not new unsafe or independent product approval; all third-party packages, providers, exceptions and expiry remain unchanged.

The established changelog generator assembled four S151/S152 fragments into prepared 0.10.1 and reset Unreleased, exit 0. Its attempted git removal of the two untracked S152 fragments printed pathspec diagnostics; the generator had already removed the consumed local files, while tracked S151 deletions were staged. No forced removal was used. The complete dated S152 decision is retained in assembled CHANGELOG.md. `cargo xtask notes 0.10.1` passed. Actual published-release.json, baseline markers, v0.10.0 notes/tags/assets and historical evidence remain unchanged.

## Operator Policy Revision (2026-09-15 20:00:53 UTC)

The operator explicitly rejected disabling administrator bypass. A bounded request restored true on the same environment, and separate public GET confirmed `can_admins_bypass=true`. Sole owner review, self-review prevention false, custom-only deployment policy and the exact single tag allowance id 60078197 remain unchanged. No unrelated setting, secret, variable or repository administrative permission changed. No deployment was approved or bypassed. Current spec/plan/contracts/checklists and instructions now preserve the requested administrator override; normal reviewer approval and deliberate bypass are explicitly distinct. The initial false-policy configuration evidence above remains chronological history, not current policy. The previously requested manual settings step is withdrawn.

Read-only reanalysis of the revised artifacts passes with unchanged 100 percent coverage and both 16/16 checklists. Test-first revision changed the positive fixture to true and negative mutation to false: the old validator produced expected red (7 passed, 3 failed). Requiring the operator-selected true state then passed all 10 guard tests. The first full CI/site runs began under the initial policy, so they do not substitute for final current-policy verification.

Fresh public environment/allowance downloads under the restored policy passed the new live `release-guard`, exit 0. Short notes validation and all 199 task-runner tests then passed, with zero failures or ignored tests. No deployment approval, bypass or publication was performed.

## Current-Policy Local Gates (2026-09-15 20:10 UTC)

The repeated full `cargo xtask ci` finished with `ci: all checks passed`, exit 0. This includes formatting, all-target/all-feature Clippy, workspace tests, conventions/dependencies/licensing, offline release guard and supply-chain checks, static package authority, shell wrappers, vendored skills, documentation and both command-parser feature sets, 14 actual-publication surfaces, guided acceptance, threat model, independent-review readiness, fuzz/failure inventories, fresh controlled native protocol evidence and performance/Windows authority checks. The 21 required conformance rows and ten artifact authorities pass. Installed/real-trust matrix-only tests retain their explicit exclusions locally and must run in isolated hosted tiers; local checks do not claim those outcomes or independent review.

The subsequent watched `cargo xtask msrv` built the entire workspace at the declared Rust 1.88 floor, exit 0. The rebuilt production documentation exported 74 pages, all 4 changelog unit tests passed and all 13 production browser contracts passed (2.6 minutes), exit 0. Changed text files pass strict UTF-8 decoding, no BOM/CR/replacement or mojibake indicators and single LF termination; `git diff --check` is clean. Actual published-release.json has no diff and all three lockfiles change first-party versions only.

Issue #416 now explicitly documents retained administrator bypass and normal owner review, with completed configuration criteria and CI/review still pending. A fresh open-issue inventory contains only the same nine known items, without new incoming issues.

## Official PR (2026-09-15 20:11 UTC)

Conventional commit `40bd566c5f2e7d82fcc25827aa4a77b0cb47fc76` includes the required Codex co-author trailer. Explicit user permission authorized its automatic push to `release/0.10.1` and official [PR #417](https://github.com/h8rt3rmin8r/fragcap/pull/417) against main. The tracked working tree was clean after push. Issue #416 remains open, Status In progress, Stage PR review and Slice S152. Automatic Codex review started on the initial PR head; no manual review request has been used.

## Workflow Regression Follow-Up (2026-09-15)

Local review identified a bounded static-check weakness: matching normalized dependency fields anywhere in a job could accept nested step text, and job-level `continue-on-error` was not rejected. A new negative test failed against the initial implementation with `accepted nonblocking identity`. Minimal job-indentation checks and fail-closed failure overrides then passed all 11 guard tests and the full 200 task-runner tests, exit 0. This strengthens existing FR-004/FR-005 without changing operator policy, product behavior, workflow effects or dependencies. Repeated full local verification and the authorized second review will cover the corrected final head.

## Pending Hosted Gates and Handoff

Automatic first-round Codex review completed on `40bd566` at 20:16 UTC with one P2 finding ([discussion](https://github.com/h8rt3rmin8r/fragcap/pull/417#discussion_r4019813300)): certification can make the identity snapshot stale before release creation. A new regression test failed with `accepted release creation without fresh readback`. The release job now independently fetches/verifies protection immediately before creation, and the static checker rejects missing or delayed release/publication guards. All 12 guard tests and all 201 task-runner tests pass, exit 0. The early check remains for certification. Current instructions/contracts and the dated changelog decision agree with the corrected three-boundary workflow. No environment setting was changed. Full corrected local gates, push, comment disposition and the one permitted manual second review remain pending.

At 20:21 UTC the complete corrected local CI and subsequent MSRV build both finished, combined exit 0. The current three-boundary guard passed newly fetched public environment/allowance records, short notes validation and conventions lint, exit 0. The rebuilt site again exported 74 pages, passed 4/4 unit tests and all 13 production browser contracts (2.6 minutes), exit 0. The reviewer received an acknowledgment linking the exact correction and red/green evidence; resolution follows the verified push. Final hosted checks/review evidence will be recorded on PR #417 and reconciled here when available.

Current-head hosted CI and all review dispositions remain pending. Field #372, independent #333/#413, documentation/final #331/#334/#278 and deferred #155/#94 remain open. Completion will be recorded only after actual commands/checks/reviews succeed.

## Second and Final Review (2026-09-15 20:35 UTC)

Correction commit `809bc221d61d15219d2a811085b018a1895388c0` was pushed and the first stale-readback thread resolved. The one permitted manual second review was requested at 20:22:55 UTC and completed at 20:25:45 UTC. It reported one P1 [missing Actions read permission](https://github.com/h8rt3rmin8r/fragcap/pull/417#discussion_r4019888803). Both primary API read references and GitHub's job-permission replacement semantics support explicit least-privilege authenticated read authority.

A new effective-permission test reproduced expected red with `accepted permission drift actions: read`. Workflow defaults and the release job override now include `actions: read`, with existing Contents read/write scopes unchanged. Static checks distinguish inherited and replacement maps and refuse missing, overridden or unnecessarily broad Actions permissions. All 13 guard tests pass, exit 0, with no ignored tests. Owner administrative bypass remains enabled; no environment settings, credentials, deployment approvals or publication changed. Full local verification, the verified correction push and this thread's disposition remain pending. The two-round review budget is exhausted; no further trigger is authorized.

At 20:38 UTC the repeated full local CI and subsequent Rust 1.88 MSRV build completed, combined exit 0. All 202 task-runner tests passed, with zero failures or ignored tests; the live guard passed freshly downloaded public environment/allowance records and short notes validation passed, exit 0. Fresh readback again confirms administrator bypass true on unchanged environment id 21294112472. The production site exported 74 pages, passed all four unit tests and all 13 browser contracts (2.6 minutes), exit 0. All nine changed text files pass strict encoding/mojibake checks and the diff is whitespace-clean. The prior `809bc22` head's hosted checks are all complete without failures, but cannot substitute for the forthcoming correction head's hosted checks. Commit/push and final thread resolution follow this verified local result.
