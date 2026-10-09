# Verification: S171

## Scope and isolation

Issue #458 is the acceptance authority. All destructive verification uses synthetic temporary roots and controlled leases; owner AppData, real trust stores and games are outside performed verification. No release or package publication is part of this slice. Exact destructive collection is Windows-only; portable builds must report unsupported rather than claim unchecked unlink races are covered.

## Red evidence

The new CLI collection/retention parser regression failed before implementation because `bundle collect` was unknown (one failed, 351 filtered out). Doctor regressions failed before correction because 1,000 empty containers exhausted the global scan allowance and 200 Healthy findings displaced a subsequent Stale finding (two failures, nine existing passes).

## Focused results

`cargo test -p fragcap-cli --lib --locked` passed 371 tests, zero failures or ignored tests, on Windows. This includes automatic age/count/byte planning, current-bundle exclusion, saved/custom/legacy policy, active generations, exact owner retirement and separate purge, interrupted empty owner retirement, absent-container registry retirement, both residue backlogs and the actual Doctor probe backlog. Subsequent final authority hardening and expanded controlled tests are verified by the final full gate below.

The expanded CLI run passed 376 tests, zero failures, after Global namespace, conservative legacy corroboration, physical-root maintenance identity, registry hard-link/ancestor guards and bounded read corrections. The ancestor-replacement regression first failed against zero-access directory pins, then passed against read-capable pins. Facade verification passed six module and 14 integration tests before the final shared ancestor-pin correction; the final gate below includes that correction and its added regressions.

## Final local gate

The final Windows `cargo xtask ci` run completed with `ci: all checks passed`. Format, all-target/all-feature Clippy, workspace tests, lint, dependency/license/supply-chain checks, package certification, wrappers/skills, documentation/help, specification, guided acceptance, threat model, review-handoff readiness, immutable review registry, fuzz/failure inventories, conformance, performance authority and Windows integration authority passed. Conformance reported 21 required rows, six protocols, independent peers and ten artifact authorities; live TShark execution belongs to the hosted analyzer tier. Review-handoff readiness validates the intake contract and does not assert independent review was performed.

The final run includes 376 CLI unit tests, seven collector module tests, 14 collector integration tests and six existing bundle-access integration tests, all passing. The actual filesystem fixtures verify ancestor rename exclusion, open-writer refusal, linked/replaced-object refusal, every interrupted content checkpoint, external progress recovery, preserved empty roots and separately authorized purge. The issue acceptance audit covers all 14 criteria. Text hygiene checks include strict UTF-8, no BOM, LF, final newline, no trailing whitespace, dash restrictions and mojibake sanity.

After both external review findings and the hosted descriptor correction, the complete Windows `cargo xtask ci` gate was rerun and again reported `ci: all checks passed`. This final source run contains 377 CLI unit tests and all 11 collector module, 14 collection integration and six bundle-access integration regressions. All 43 changed files pass text hygiene. Hosted exact-head results are recorded in the PR completion checkpoint rather than inferred from this local result.

## Hosted handoff boundary

The initial hosted Linux all-feature Clippy gate rejected a redundant return in the non-Windows Unsupported branch. The follow-up expresses that branch as its return value without changing platform behavior; format passes locally and the exact follow-up head must pass the hosted Linux gate before handoff.

The automatic first Codex round found that a journal-less Undetermined owner still advertised a recovery action which the recovery authority refuses. The correction offers exact cleanup only for proven Inactive or explicitly confirmable LegacyUnproven owners. A real version-2 registry fixture with an absent Local lease and a live producer PID failed at Doctor's action-selection assertion before the fix. Afterward, 106 focused Doctor tests pass, including inspection-only guidance, no cleanup offer and unchanged registry/container contents. The final hosted head includes this correction.

The single second Codex round found the equivalent eligibility error after an outstanding journal exists. Owner recovery eligibility is now shared by journal actions and journal-less findings. The extended regression first failed at actual offered-action selection, then passed with 106 focused Doctor tests. It additionally demonstrates that the journal alone has a recovery action, owner uncertainty suppresses its offer, exact recovery refuses before replay, and both journal and registry bytes remain unchanged. No third round is requested.

Hosted Windows tests exposed retirement-store validation based on rendered SDDL text. Synthetic numeric-SID alias and reversed-ACE-order cases reproduced refusals before the correction. Validation now compares the exact producer owner SID, protected DACL and exactly two full-access object/container-inheritable allow ACEs for producer and SYSTEM. Extra or duplicate grants, wrong owner, unprotected ACLs, inherited ACEs, weaker rights and deny ACEs remain rejected. These descriptor tests parse synthetic descriptors in memory and grant no account permissions; the final hosted Windows account is verified by CI.

Official PR #473 is published. Both authorized code-review rounds have completed; their findings are addressed in follow-up commits, and no third round is requested. Exact final-head hosted CI and security scan findings remain the handoff gate at this source checkpoint. The final PR conversation records completion of T020 with commit identity, hosted checks, review responses and round count. Owner review and merge remain outside agent authority.
