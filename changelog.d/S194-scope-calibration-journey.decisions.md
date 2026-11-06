<!-- spec-impact: 15.7, 17.4, 19.3 -->
# S194 Decisions (2026-11-05)

1. Seal the public scope vocabulary as `auto|game|launch-chain`. Resolve `auto` before effects, using exact-game inspection with blind relay for launcher-mediated targets and launch-chain inspection for direct targets.
2. Make requested scope, effective scope, middleware handling, and policy identity exact applicability dimensions. Store version 14 preserves legacy facts with absent dimensions, so they remain readable but inapplicable to new exact cases.
3. Keep fallback authority in the initial immutable workflow and select recovery from the typed inspection result independently from lifecycle completion. One unsuccessful launch-chain inspection may produce one fresh exact-game successor only when that authority was reviewed and fresh resolution reproduces its target and resolved launch authority apart from the scope transition. Packet-only fallback retains its actual passive observation without creating a positive Deep Capture fact only after capture output, required artifacts, event delivery, and cleanup reconcile; otherwise the original failure remains authoritative.
4. Certify the parser and exact-game authorization contract from the packaged executable in CI. Installed-product and private-title observations occur after release and any defect returns as a GitHub issue.
