# S168 controlled validation

## Prerequisites

Use the pinned Rust workspace and repository CI prerequisites. No real game, trust-store change, existing private bundle, or remote origin is required for controlled regressions. Execute console tooling with the Windows hidden-process guarantee and watch output to completion.

## Focused checks

Run owning display/Doctor tests, target/Steam discovery fixtures, calibration CLI fixtures, session_ux tests, and native typed evidence tests. Assert actual visible columns and exact values, constant healthy-summary row count with an actionable mixed inventory, no unrelated warning for an exact selected target, and ordered warm-Steam refusal with zero slow downstream effects.

Layout/Doctor results (2026-10-09): `cargo test -p fragcap-cli --lib --locked` passed 331 tests with no failures or ignored tests. Focused `cargo test -p fragcap-cli --lib doctor:: --locked` passed 95 tests and `cargo test -p fragcap-cli --lib commands::steam:: --locked` passed 11 tests. The clean integration batch `cargo test -p fragcap-cli --test cli_doctor --test cli_capture --test cli_steam --test cli_bundle --test cli_help --locked` passed 68 tests (Doctor 20, capture 24, Steam 7, bundle 2, help 15). The Doctor human golden was deliberately regenerated and its diff reviewed; the structured golden did not change. After strengthening the history fixture to 150 healthy records across 50 sessions plus mixed/actionable/active/unknown/unsupported/cleanup-failed records and a scan-limit condition, `cargo test -p fragcap-cli --lib doctor::checks::tests::healthy_history_is_constant_size --locked` passed its focused test. All runs used the verified hidden launcher. These results establish controlled repository behavior, not real-game or private-history field verification.

## Calibration fixture

Use synthetic equivalents of six accepted connections, five protocol terminal errors, one idle timeout following seven HTTP 200 completions, unavailable final-client ownership, partial trace/manifest and successful cleanup. Expect an inconclusive attempted verdict, separate current stored assessment, distinct counts and one supported blocker/next action. Exercise first, second, resumed, stale/current, interrupted/failed and successful cases, plus late owner-release evidence.

## Full verification and handoff

Run `cargo xtask ci` and applicable owning documentation/controlled checks. Reconcile each original criterion in issue-acceptance.md with exact source/test evidence. Push and open the official PR under owner authorization; verify hosted checks and every review disposition on the final head. Stop for owner final review and merge.
