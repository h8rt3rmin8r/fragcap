# Implementation Plan: Direct Socket Route

**Branch**: `codex/s200-direct-socket-route` | **Date**: 2026-11-10 | **Spec**: [spec.md](spec.md)

**Input**: Owner-authorized S200 for #578, #581, and #582, including push and official PR publication, with final merge reserved for the owner.

## Summary

Build the controlled raw-socket and unrelated-traffic oracle first, then add a session-owned WinDivert socket/packet adapter and bounded local TCP/IP bridge to the existing native proxy. Preserve exact original process and destination evidence through a one-use route receipt and classify only what the proxy actually observes. A missed or uncertain first packet bypasses with explicit evidence. S200 ends only when the complete driver-free and disposable Windows matrix passes.

## Technical Context

**Language/Version**: Rust MSRV 1.89, pinned development Rust 1.96; Windows 11 x64.

**Primary Dependencies**: Packaged official signed WinDivert 2.2.2-A x64, existing native proxy and Windows APIs, and exact-reviewed `smoltcp` 0.12.0 plus `managed` 0.8.0 if the packet-to-stream design passes the explicit license and supply-chain approval gate.

**Storage**: Existing bounded proxy/application, cleanup and manifest artifacts; additive route-receipt and original-flow evidence contracts only.

**Testing**: Driver-free ownership corpus, proxy bridge tests, disposable Windows driver matrix with active sentinels, `cargo xtask ci`, exact-head hosted CI and reviews.

**Target Platform**: Windows 11 x64 for live routing, portable driver-free model tests.

**Project Type**: Rust multi-crate CLI and native proxy.

**Performance Goals**: No packet path added to passive Capture; unrelated sentinels retain zero missing responses and the spec's paired p99 bound.

**Constraints**: No target process memory access, hooks, game-file edits, global proxy setting, unbounded queue, private target fixture, false inspection claim, or visible child console.

**Scale/Scope**: Three issue closures through one end-to-end controlled route and bridge; CLI and calibration selection follow in #583.

## Constitution Check

**Pre-design**: P-1 permits only an explicitly selected, bounded, signed WinDivert Deep Capture effect after authorization. P-2/P-3 keep packet routing, core data, facade orchestration, and native protocol handling in their owning layers. P-4/P-9 require exact packet loss and owner uncertainty. P-5/P-6 preserve versioned artifact and vocabulary contracts. P-7/P-8 require complete tests, hidden Windows child execution and UTF-8 output. P-11 forbids calling a routed encrypted flow inspected without proxy evidence. Constitution lines 478-480 require separate review and explicit approval for the exact 0BSD dependencies `smoltcp` and `managed` before their inclusion. No general allowlist widening is planned.

**Post-design**: The typed receipt and original-flow ledger separate helper and game ownership. A dynamic narrow packet handle avoids routing unrelated traffic through the helper queue. A missed first packet is a truthful bypass. WFP ALE redirection was compared but would require a different driver. The owner approved the exact two-crate 0BSD exception and the WinDivert queue-loss limitation: exact accounting starts at adapter receipt, pre-receive loss is unmeasurable, and active sentinels provide the end-to-end loss gate. The Windows matrix remains the implementation gate.

## Project Structure

### Documentation

`specs/200-direct-socket-route/` contains specification, checklists, research, plan, data model, internal contract, quickstart, tasks and analysis. Update the master specification, plan index, security review and dated changelog when behavior is proven.

### Source

`crates/fragcap/src/deep_capture/routing.rs`, a Windows-only direct-route adapter beside it, `crates/fragcap/src/deep_capture/native.rs`, `crates/fragcap-proxy/src/runtime.rs`, `crates/fragcap-proxy/src/socks5.rs`, `crates/fragcap/tests/`, `crates/fragcap-cli/tests/`, and the disposable Windows integration workflow. The facade owns route orchestration and receipt admission; the proxy retains protocol handling.

## Implementation Sequence

1. Freeze the fixture oracle, route dispositions, receipt contract, and security/uncertainty cases in driver-free tests.
2. Review and pin the packet-stack dependency or prove a smaller equivalent; enforce exact supply-chain and MSRV compatibility.
3. Add the pre-effect direct component and socket observer. Open signed WinDivert dynamically only after authorization and close every handle on all exits.
4. Add narrow TCP and UDP packet admission, bounded queues, packet accounting, cancellation and exact original-flow proof.
5. Bridge selected streams and associations through authenticated native proxy admission, preserve trusted original owner and actual protocol outcomes.
6. Run the full disposable Windows matrix with live direct clients and active unrelated sentinels. Repair every failure before claiming #578/#581/#582.
7. Reconcile docs, security inventory, changelog and Spec-Kit analysis; run local and hosted gates, push and open the official PR, handle every CI and bot review within two review rounds, then hand off for owner merge.

## Complexity Tracking

The local stack and receipt ledger are necessary because WinDivert provides packets separately from process events and the native proxy receives streams. A separately verified 0BSD exception is narrower than changing the Rust dependency policy globally. Live timing and isolation remain unproven until the disposable Windows matrix passes.
