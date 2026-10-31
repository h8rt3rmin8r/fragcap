# S184 verification record

## Source and scope

- Branch: `codex/s184-deep-capture-session-lifecycle`.
- Issues: #526 and #527. #528 and #529 remain separate.
- Spec-Kit analysis before implementation found no uncovered requirement, conflicting task, unresolved placeholder, or constitution conflict. The feature selector names this directory.

## Controlled evidence

| Check | Result | Authority |
| --- | --- | --- |
| Exact root and client with an unrelated same-name declared event | Pass | Windows shutdown adapter controls and verifies exit only for the two exact owners; unrelated child remains alive and route release is partial. |
| Same event with a verified exit | Pass | Windows shutdown adapter verifies the exact owner exit and permits complete release. |
| Unresolved route owner after Capture failure | Pass | Facade reports `route-owner` and `target-route` failed, retains the trust lease for exact recovery, and records proxy continuity loss after finite stop. |
| Four-minute ordinary request and real adapter mapping | Pass | CLI deadline/adapter unit checks 240-second plan and 240-second Capture request with a separately budgeted five-second return. Facade preflight preserves 240 seconds and refuses 301 seconds before invoking adapters. |
| Controlled binary-backed duration above one minute | Pass (61.41 seconds) | Hidden `fragcap.exe` process requests 70 seconds, authorizes its emitted JSON plan, keeps a controlled target alive past 60 seconds, then checks terminal and cleanup. |

The controlled pcapng is synthetic test truth with fixed fixture timestamps. It cannot establish a real Npcap frame span or validate an installed MSI against a game. An isolated installed-product run with a real Capture span remains unverified in this source PR. No private title identity or raw traffic is committed.

## Repository gates

| Command | Result |
| --- | --- |
| `cargo xtask spec` | Pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | Pass |
| `cargo test --all --locked` | Pass; the later complete `cargo xtask ci` rechecked the final source. |
| `cargo xtask docs check` | Initial reference-table format drift found and corrected; the complete CI gate passed the final docs check. |
| `cargo xtask ci` | Pass on final source: fmt, all-feature clippy, locked workspace tests, lint, dependencies, licenses, supply chain, package contract, wrappers, skills, docs, spec, guided calibration, threat model, review handoff, review record, fuzz, failure matrix, conformance, performance, and Windows integration authority. |

## PR handoff boundary

The final head SHA, exact-head CI, external review rounds, and review-thread state are GitHub PR evidence. Report their verified values at owner handoff rather than recording a changing remote status in this source file.
