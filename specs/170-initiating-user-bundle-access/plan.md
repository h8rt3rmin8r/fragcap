# Implementation Plan: S170 initiating-user bundle access

**Branch**: `codex/s170-initiating-user-bundle-access` | **Date**: 2026-10-09 | **Spec**: [spec.md](spec.md)

## Summary

Implement #464's complete new-output and historical-repair contract in the facade and CLI. Exact Windows user identity, retained unelevated impersonation context and a pinned filesystem access contract replace Owner Rights as recipient authority. Every independent artifact writer preserves that contract. Historical repair validates one bounded complete population before any ACL change and reports outside retained evidence. Final recipient-context verification follows lifecycle reconciliation before accessible-success or analyzer guidance.

## Technical Context

**Language/Version**: Rust workspace, MSRV 1.88, existing exact windows-sys 0.36.1.

**Primary Dependencies**: Existing Win32 security, filesystem, threading, WTS and named-pipe bindings; serde_json and existing BLAKE3 vocabulary. Add required feature edges and entropy edge only if needed; no new lock package is planned.

**Storage**: Existing retained bundles, manifest v1/v2, resource journal and CLI v2 session owner registry. No database or retention migration. Permission results are separate command output, not modifications to capture or recovery contents.

**Testing**: Red-before/green-after security regressions, real synthetic Windows filesystem operations under limited/restricted or accepted helper tokens, existing controlled command tests and cargo xtask ci. All tools run through verified hidden foreground launch with redirected noninteractive I/O.

**Platform/Type**: Windows CLI/library; platform-neutral core remains unchanged. Existing Unix bundle protection remains supported without claiming Windows token proof there.

**Performance/Bounds**: One recipient handshake, 60-second connect/read deadline, one versioned message no larger than 4096 bytes; repair at most 256 artifact/container entries, depth 8 and bounded 4 MiB provenance input. Limit overflow is an explicit refusal before mutation. Verification reads permissions and bounded probes, never emits payload bytes.

**Scope**: Same-account default identity, explicit different-account handoff/destination, all retained writers and final access outcome, read-only exact inspection and confirmed historical repair. GC, purge, release and dependency PR #470 remain separate.

## Constitution Check

Pre-research gate PASS: P-1 no target handles, instrumentation or new traffic behavior; token impersonation concerns the tool's own context and explicit recipient helper. P-2/P-3 Windows adapters stay in facade/proxy, core and packet attribution stay unchanged. P-4/P-9 report failures and partial effects exactly and preserve evidence bytes. P-5 no analyzer format change. P-6 use existing user, bundle and access vocabulary and add glossary definitions where necessary. P-7 CLI is presentation around library authority. P-8 UTF-8/no-BOM/LF, single logical Markdown lines and shared four-space output layout. P-10 target identity remains unchanged. P-11 master specification and public docs describe the implementation without claiming a release or universal field proof. Full Spec-Kit analysis and repository gates are mandatory. Post-design gate PASS with the decisions below; no unresolved technical clarification remains.

## Phase 0: Research and decisions

Two required research agents inspected token/effective-access and historical-repair surfaces before implementation. Their primary Microsoft sources and decisions are consolidated in [research.md](research.md).

D1: Default production desktop resolution binds current token SessionId to WTS's exact session account and requires SID equality. Retain the same-account linked limited token, or an exact ordinary current impersonation token. Never guess an active-console user or infer recipient from Administrators ownership. Unprovable default identity refuses before effects; controlled fixtures/library calls explicitly bind their own current-account contract.

D2: `--output-recipient <SID>` requires an explicit `--bundle` and supports a different account through one finite local authenticated named-pipe handoff. Display a quoted `bundle access-authorize` command for the intended ordinary account. Server verifies exact SID, unelevated context, request and path. Missing, elevated, mismatched, remote or timed-out helpers refuse. The full session plan additionally binds recipient SID and destination; revalidate proof after approval. No password collection or process enumeration is used.

D3: Restricted private directory ACLs name recipient, distinct producer when needed and SYSTEM explicitly. Preserve that same selected recipient in every independently protected child and atomic/staging file without a mutable process-global selected account. Necessary proven-owned output-container traversal MUST be part of the bounded pinned preview (at most eight ancestors), and corrected with non-propagating recipient traversal/enumeration grants that leave sibling descriptors unchanged; arbitrary inaccessible custom/profile ancestors refuse. Producer-private ephemeral CA material remains producer-private and uses an explicit producer SID.

D4: `bundle access-inspect <bundle>` is read-only. `bundle access-repair <bundle> --authorize <inspection-id>` recomputes and binds complete current inventory, recipient, object identities, descriptors and provenance before mutation. CLI checks the exact owner registry/lease; active or unproven ownership refuses. A terminal manifest and matching closed, identity/count-validated journal establish inactive-writer source provenance; a crash prefix alone refuses. independently protected recognized sidecars are part of the inventory even when absent from manifest.

D5: Pin paths without reparse following and without delete sharing, refuse outside hard-link aliases, path replacement, unknown files and unsupported/unreadable authority. Apply permissions by object handle, not a canonicalize-then-path mutation. Avoid unchecked inheritance propagation; all affected children are pinned/validated. A repair result reports applied/unchanged/failed/verification-failed/not-attempted and supports fresh-preview retry without writing retained content or deleting anything.

D6: Verify actual parent traversal, directory enumeration and artifact open/read under the retained unelevated recipient context in dedicated synchronous threads. Failed verification remains separate from artifact-written and semantic completeness. Final verification occurs after reconciliation, so journal compaction and manifest rewrites cannot escape the contract. No accessible analyzer guidance precedes that proof.

## Phase 1: Design

[Data model](data-model.md), [access contract](contracts/access.md) and [quickstart](quickstart.md) fix shared interfaces and validation. Existing public function signatures remain compatible; new functionality is additive. Current/controlled library ownership is explicit rather than a hidden production default bypass. No async await is allowed while impersonating; every impersonation and restore result is checked on an isolated thread.

## Project Structure

```text
specs/170-initiating-user-bundle-access/
  spec.md, plan.md, research.md, data-model.md, quickstart.md
  contracts/access.md, checklists/requirements.md, checklists/access.md
  tasks.md, issue-acceptance.md, verification.md
crates/fragcap/src/deep_capture/
  access.rs, access/windows.rs, access_repair.rs
  artifacts.rs, mod.rs
crates/fragcap-proxy/src/windows/acl.rs
crates/fragcap-cli/src/
  cli.rs, commands/bundle.rs, commands/deep_capture.rs
  commands/calibrate.rs, session_ux.rs, workflow_help.rs, doctor/residue.rs
crates/fragcap/tests/bundle_access.rs
crates/fragcap-cli/tests/cli_bundle.rs
site/content/docs/, docs/fragcap-specification.md, docs/plans/README.md
changelog.d/S170.fixed.md, changelog.d/S170.decisions.md
```

## Implementation coordination and verification

After the blocking analyze gate, independent [P] tasks may be delegated. Recipient agent owns access.rs/windows adapter, artifacts.rs and facade Cargo feature additions. Repair agent owns access_repair.rs and producer-private proxy ACL. Root owns facade module exports, CLI integration, documentation, issue criterion audit and full verification. Agents agree shared interfaces before editing; no concurrent edits to an owning file. A dedicated evidence agent may own only bundle_access.rs after shared API settles.

Run focused owning security/CLI suites, both curated API variants, text hygiene and cargo xtask ci in the foreground. Automatically push/open/attach the official PR under explicit owner authorization. Automatic creation review is round one; at most one further combined code/security request is allowed. Resolve every returned finding, verify final-head CI and hand off for human merge. The PR conversation owns post-source-checkpoint external completion; no merge, release or owner-artifact repair is authorized here.

## Complexity Tracking

No constitution violation or new product crate. Token handoff and pinned repair objects are required by explicit different-account and historical permission semantics; a group-relative ACL or new-output-only correction cannot satisfy #464.
