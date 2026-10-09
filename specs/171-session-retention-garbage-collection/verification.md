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

## Hosted handoff boundary

The initial hosted Linux all-feature Clippy gate rejected a redundant return in the non-Windows Unsupported branch. The follow-up expresses that branch as its return value without changing platform behavior; format passes locally and the exact follow-up head must pass the hosted Linux gate before handoff.

Official PR publication, exact final-head hosted CI and independent bot findings remain pending at this source checkpoint. External round one is automatic publication; at most one additional combined request is authorized. The final PR conversation records completion of T020 with commit identity, hosted checks, review responses and round count. Owner review and merge remain outside agent authority.
