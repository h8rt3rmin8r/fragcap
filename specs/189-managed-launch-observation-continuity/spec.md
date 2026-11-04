# Feature Specification: Managed Launch Observation Continuity

**Slice**: S189

**Branch**: `codex/s189-managed-launch-observation-continuity`

**Date**: 2026-11-04

**Status**: In progress
**Issues**: #552 and #553; #547 records a source fix already shipped in S188/v0.10.13.

## Intended behavior

A managed Deep Capture may encounter a short-lived client before a launcher or anti-cheat descendant starts the lasting client. The first exit must not end an already proven owned handoff or start shutdown. A later client with the declared exact path and causal ancestry can take over the terminal role. If no successor can be proven within the finite acquisition wait, the session reports an incomplete handoff. Ordinary passive Capture retains its current terminal-exit rule.

For managed Deep Capture, `--wait` measures client acquisition from capture arm and `--duration` measures eligible observation from verified client acquisition. A proven successor receives its own full observation interval within the finite attempt budget. The plan states the effective phase values before authorization. An accepted explicit duration is not shortened after authorization.

The S188 change already joins basename-only ETW starts to a same-instance verified executable path for exact stage matching. Behavior observed after release is reported through GitHub issues.

## Requirements

- **FR-001 (#552)**: A terminal client exit with a causally owned live descendant enters a finite provisional handoff and does not itself trigger managed shutdown.
- **FR-002 (#552)**: A replacement terminal client must satisfy the declared stage path and ancestry using its exact process instance. A reused PID, wrong path, escaped parent, missing identity, or ambiguous match cannot inherit the predecessor's role.
- **FR-003 (#552)**: A handoff that does not bind a proven successor by the acquisition deadline reports `handoff-incomplete` and keeps packet, proxy, process, and cleanup facts distinct.
- **FR-004 (#552)**: Packet role stamps are valid only for the bound process lifetime. The first client, provisional exit, successor, and actual terminal reason remain visible in retained process evidence.
- **FR-005 (#552)**: Graceful or force release of managed processes starts after effective observation ends or another actual terminal condition. Process control remains limited to exact authorized owned instances.
- **FR-006 (#553)**: Managed client acquisition and post-acquisition observation have separate finite clock origins and durations. Startup, capture preparation, owner release, proxy shutdown, and cleanup retain separate bounds.
- **FR-007 (#553)**: A client acquired within the accepted wait gets the requested post-acquisition duration unless a named actual terminal condition occurs. A proven successor receives a full interval within the authorized attempt ceiling.
- **FR-008 (#553)**: Simple launch defaults remain 30 seconds of acquisition and 60 seconds of observation. Declared multi-stage launches default to 180 seconds and 120 seconds respectively. Exact supported overrides take precedence.
- **FR-009 (#553)**: Human and JSON authorization plans, terminal output, and manifest state the effective values and phase origins. Runtime refuses a budget that cannot honor an accepted request instead of silently clamping it.
- **FR-010**: Capture remains passive; Deep Capture remains explicitly authorized, target-scoped, finite, and auditable. No target memory access or private installed-title identity is added.

## Issue disposition

S188/v0.10.13 shipped #547's source-level basename/path correction and precise acquisition reason. If an installed release exhibits a remaining or new defect, it is reported as a GitHub issue with its observed behavior and separate evidence claims.
