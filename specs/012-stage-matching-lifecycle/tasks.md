# Tasks: Stage Matching and Session Lifecycle

Dependency-ordered. Test-driven: within each component the tests are written with
or before the implementation, and the component is not complete until they pass.

## T001 - Core binding method (`fragcap-core`)

- Add `ProcessTree::bind_stage(&mut self, id: NodeId, stage: StageId) -> bool` to
  `crates/fragcap-core/src/process/tree.rs`, writing the reserved
  `ProcessNode::stage` field. Bind-once: returns `false` for an unknown or
  already-bound node.
- Update the S11 placeholder test `the_stage_is_reserved_and_empty_until_s12` to
  reflect that S12 now writes the field: keep the default-empty assertion, add
  tests that `bind_stage` sets it, is idempotent per node, and returns `false`
  for an unknown node.
- Depends on: nothing. Blocks: T002, T004.

## T002 - Stage matcher (`fragcap-profile`)

- New `crates/fragcap-profile/src/matching.rs`: `predicates_hold`, `stage_for`,
  `bind_stages` per the contract. Reuse `ImagePattern::matches`,
  `PathRegex::regex`, and `ProcessTree::ancestry`/`node`/`bind_stage`.
- `pub mod matching;` in `crates/fragcap-profile/src/lib.rs`; re-export
  `stage_for`, `bind_stages` at `fragcap::profile::matching`.
- Depends on: T001. Blocks: T003, T004.

## T003 - Matcher tests (`fragcap-profile`)

- Tests (in `matching.rs` `#[cfg(test)]` or `crates/fragcap-profile/tests/matching.rs`):
  one per predicate; `cmdline_contains` against `CommandLine::Unavailable` does
  not match; `path_regex` uses the compiled expression; the conjunction rule (a
  stage with two predicates needs both); the ambiguous shared-image chain (three
  processes share an image name, `descends_from` binds the descendant of the
  launcher, `exe` alone would bind the first); multi-stage precedence (first in
  declaration order); `descends_from` fails when no ancestor is bound.
- Depends on: T002.

## T004 - Capture session (`fragcap`)

- New `crates/fragcap/src/session.rs`: `SessionState`, `StopReason`,
  `PacketDisposition`, `SessionConfig`, `SessionStats`, `CaptureSession` per the
  contract, with the five-state machine, the six stop conditions, and the
  watching-discard counter.
- `pub mod session;` in `crates/fragcap/src/lib.rs`.
- Depends on: T001, T002. Blocks: T005.

## T005 - Session tests (`fragcap`)

- Tests in `crates/fragcap/tests/session.rs` (tier-1, scripted watcher + scripted
  packets): arm-before-target; packets in Watching are Discarded and counted;
  first match transitions to Capturing and retains; no packet lost at the
  boundary; acquisition timeout from Watching to Complete; each of the six stop
  conditions reaches Complete via the same drain path; a service stage is never
  awaited; session conservation (`watching_discarded + retained` equals packets
  offered while armed).
- Depends on: T004.

## T006 - Glossary (P-6)

- Add entries to `docs/glossary.md`: stage matching, stage binding, capture
  session (with the five states), acquisition timeout, stop condition. Do not
  re-add Stage, Lifecycle class, Terminal stage, or Process tree.
- Depends on: T002, T004 (terms are stable by then).

## T007 - Changelog fragments

- `changelog.d/S12-stage-matching.added.md` (feature line) and
  `changelog.d/S12-stage-matching.decisions.md` (D-1 through D-6, dated).
- Depends on: T001..T006.

## T008 - Verify and commit

- Run `cargo xtask ci` in the foreground to completion; run `cargo xtask neutral`
  and `cargo xtask msrv` (expect exit 0 or a clean can-not-run 2).
- Stage the slice's files and commit with a conventional message. Halt before
  push.
- Depends on: T001..T007.
