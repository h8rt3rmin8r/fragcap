# Tasks: correct the schema $id host to fragcap.com

**Feature**: 047-schema-id-host | **Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

Four one-line edits plus a decision fragment, in one commit so the drift and CLI
tests never go red between edits. Verify with `cargo xtask ci`.

## Phase 1: The identity edits (one commit)

- [ ] T001 In `docs/schema/target-schema.v1.json` line 3, change the `$id` host from `fragcap.dev` to `fragcap.com` (`"$id": "https://fragcap.com/schema/target/v1.json"`).
- [ ] T002 In `crates/fragcap-profile/assets/target-schema.v1.json` line 3, make the identical change, so the two copies stay byte-identical (the drift test asserts equality).
- [ ] T003 In `crates/fragcap-cli/tests/cli_schema.rs` line 79, update the asserted substring to `"$id": "https://fragcap.com/schema/target/v1.json"`.
- [ ] T004 In `specs/025-master-json-schema/contracts/master-schema.contract.md` (around lines 11-13), change the example host to `fragcap.com` and add a dated note that the host was corrected once (pre-1.0) from `fragcap.dev`, pointing at the decision fragment, so the "never changed for a published version" statement does not contradict the change.

## Phase 2: Record the decision

- [ ] T005 Add `changelog.d/047-schema-id-host.decisions.md` recording the deliberate pre-1.0 identity correction (fragcap.dev -> fragcap.com), why (fragcap.dev is not owned; fragcap.com is the real domain; the `$id` is an opaque identifier nothing dereferences; v1 is embedded-only, not registry-published), dash-free, UTF-8/LF.

## Phase 3: Verification

- [ ] T006 Run `cargo xtask ci` in the foreground to green (fmt, clippy, tests incl. the schema drift tests and cli_schema, lint, deps, license, docs).
- [ ] T007 Run `grep -rn "fragcap.dev" .` and confirm zero matches; run `diff docs/schema/target-schema.v1.json crates/fragcap-profile/assets/target-schema.v1.json` and confirm no differences.
- [ ] T008 Confirm `git diff --stat` touches only the four files above, the changelog decision, and `specs/047-schema-id-host/` (no unrelated file).

## Dependencies

- T001-T004 land together; T003 must accompany T001/T002 or the CLI test goes red.
- T006-T008 after the edits and the decision fragment.

## MVP

T001-T003 is the functional change (schema identity plus the test that guards it);
T004/T005 keep the contract and governance record honest; T006-T008 verify.
