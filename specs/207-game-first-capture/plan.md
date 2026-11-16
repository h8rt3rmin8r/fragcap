# Implementation Plan: Game-first Deep Capture

**Branch**: `codex/s207-game-first-capture` | **Date**: 2026-11-15 | **Spec**: [spec.md](spec.md)

## Summary

Make an ordinary authorized Deep Capture run target the exact final game, leave publisher and platform updater traffic on its native route, and learn compatibility from the run. Carry one typed route choice from pre-effect authorization through library preparation and applied effects. Keep explicit full-chain inspection, including SOCKS-only launcher exposure, as an advanced choice. Report absent or delayed SOCKS protocol evidence as unknown.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.89; PowerShell 7 for controlled Windows scripts.

**Primary Dependencies**: Existing `fragcap` facade, CLI, proxy, targets store, capture and direct WinDivert adapter. No new dependency planned.

**Storage**: Append-only compatibility facts and native session bundle. Schema version 16 widens only the fact route vocabulary through a transactional table rebuild; historical rows and artifact readers remain available.

**Testing**: Focused Rust unit and integration tests, `cargo xtask ci`, controlled Windows route and unrelated-sentinel matrix, exact-head CI.

**Target Platform**: Windows native Deep Capture; driver-free contract tests on other supported build hosts.

**Project Type**: Rust workspace library and CLI.

**Performance Goals**: Preserve the existing bounded packet, proxy, and cleanup paths; no synchronous per-packet storage work.

**Constraints**: P-1 explicit effects and exact target ownership; P-4 conservation and unrelated continuity; P-9 observed fact provenance; no target process memory access or game-file modification.

**Scale/Scope**: Five approved issues (#611, #608, #612, #614, #609) under epic #607. Warm-platform fallback, complete result presentation, history UI, and installed-title qualification remain separate.

## Constitution Check

**Before design**: P-1 permits the existing exact-game direct route only after a complete effect plan and authorization. Launcher updater bypass must preserve unrelated traffic. P-4 requires honest loss; P-9 forbids treating route reachability as TLS or plaintext proof. The historical compatibility store is evidence, not permission. No new constitution exception is needed.

**After design**: The route selection is immutable through preparation and application; exact process generation governs direct route and retained evidence; default publisher environment receives no Fragcap proxy variables; observations may be unknown; old facts remain readable. Controlled tests must cover wrong-route refusal, unrelated sentinel continuity, first-run admission, and provenance before this gate passes.

## Project Structure

```text
crates/fragcap/src/deep_capture/
  model.rs          session intent and route choice
  routing.rs        effect construction and validation
  session.rs        one preflight and applied-effect contract
  policy.rs         ordinary capture admission and observed facts
  application.rs    application provenance
crates/fragcap-proxy/src/socks5.rs         protocol-prefix classification
crates/fragcap-cli/src/cli.rs             ordinary and advanced options
crates/fragcap-cli/src/commands/deep_capture.rs  plan, authorization, run and output
crates/fragcap-cli/src/commands/workflow_help.rs current guidance
integration/           controlled route and packet-scope matrix
specs/207-game-first-capture/  specification and acceptance artifacts
docs/fragcap-specification.md current architecture of record
docs/plans/README.md chronological slice record
changelog.d/          unreleased feature and decisions fragments
```

**Structure decision**: Evolve the existing library session plan and CLI instead of adding a second launch coordinator. Keep existing store migrations and artifact readers intact.

## Sequence

1. Add route-selection and scope contract tests, including an authorized-versus-applied mismatch that refuses before effects.
2. Make game-only exact direct routing the ordinary default. Carry the same route object through library preflight and execution. Add an explicit full-chain/SOCKS choice and test launcher environment, exact owner, packets, and application evidence.
3. Correct SOCKS timeout, EOF, and partial-prefix provenance; distinguish the pre-TLS CONNECT handoff. Add positive and negative protocol fixtures before code changes.
4. Remove calibration entry paths and prior-positive admission. Preserve fact recording and historical readback. Update directly affected help and architecture prose.
5. Run focused tests, CI parity, controlled Windows route matrix, and exact-head checks. Reconcile documentation and changelog, commit, push, publish PR, resolve at most two Codex review rounds, and stop before merge.

## Decisions

- **One route authority**: Build the route from the selected intent once, before authorization. The concrete loopback port may be bound later; all effect names, route kinds, scope and cleanup obligations are fixed. Mismatch is a pre-effect refusal.
- **Game-only means no launcher proxy effects**: Do not give managed intermediates Fragcap proxy variables. The exact final client uses the approved direct component. A managed root must remove ambient proxy values that could otherwise leak into a supposedly native route; those removals are explicit effects.
- **Advanced scope is explicit**: Full-chain packet and application retention and SOCKS-only launcher routing are one deliberate advanced choice, separately authorized. A failed game-only run never upgrades itself to that choice.
- **Ordinary capture is the measurement**: The current capture writes compatibility facts but historical facts cannot gate another capture or authorize a different effect. Retire live calibration controls and preserve old rows and bundle readers.
- **Protocol claims require positive evidence**: Missing or partial SOCKS prefix remains unknown. TLS, cleartext and decryption claims are tied to the stage that actually observed them; later bytes cannot turn an earlier unobserved interval into plaintext.
- **Controlled closure**: Repository-controlled and disposable hosted Windows tests close this implementation slice. A fresh installed private-game result is an independent field observation, not an issue completion prerequisite.

## Complexity Tracking

No new crate, dependency, or routing mechanism is added. A schema version 16 migration is necessary because SQLite version 15 CHECK constraints reject the truthful `direct-socket` strategy and new route identities. Historical fact and bundle readers remain; the public calibration command and admission path are removed.
