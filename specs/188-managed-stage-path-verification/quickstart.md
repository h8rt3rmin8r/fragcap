# Quickstart: S188 controlled validation

1. Confirm the branch is `codex/s188-managed-stage-path-verification` and `.specify/feature.json` selects `specs/188-managed-stage-path-verification` before every Spec-Kit phase.
2. Run focused process-tree and matcher tests, then the synthesized-platform-profile acquisition test. The basename-only exact path must bind only with a verified path joined to the event; wrong path, later PID generation, missing path, and escaped parent must not bind.
3. Run the controlled Windows production-path fixture on an isolated CI desktop with Npcap and ETW access. Its synthetic owned Steam chain must send loopback traffic, open scoped capture, and produce at least one target-owned pcapng frame readable by unmodified TShark. Run the ordered publisher variant and all refusal rows there or in the same production orchestrator seam.
4. Run the composed Deep Capture timeout fixture. Assert human and structured `acquisitiontimeout` and independently read back partial packet, proxy application, manifest, and cleanup facts.
5. Run `cargo xtask ci` in the foreground, the applicable isolated Windows/package gate, and the UTF-8 and mojibake checks. After the authorized push, verify exact-head CI and all review threads; request no more than one second `@Codex` round.

Controlled source and isolated package fixtures may use disposable roots and do not prove a private installed-title retest. The private native bundle reported in #547 remains in the product's default history under its own retention rules and is never copied into this slice.
