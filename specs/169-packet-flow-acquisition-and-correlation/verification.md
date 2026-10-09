# S169 verification and review record

## Authority and scope

S169 implements #468 in `specs/169-packet-flow-acquisition-and-correlation/` on `codex/s169-packet-flow-acquisition-and-correlation`, based on human-merged S168 main `7e3e61fb10755d79402e56a1fbb508361a6364e3`. Spec-Kit specify, clarify, checklist, plan, tasks and blocking consistency analysis completed before implementation. The owner explicitly authorized push and an official PR, every review response and at most one additional review round. Merge and release remain owner actions.

## Controlled red and green evidence

- Two fixed-filter and three CLI assembly regressions failed before acquisition corrections. The corrected focused suites passed 25 filter tests, 29 assembly tests and per-interface pipeline filter isolation. Windows all-feature CLI compilation passed.
- Five loopback ownership regressions reproduced three failing canonical-port/competing-owner/creation-boundary cases before the facade correction. All 14 stamping tests pass, covering both families/orientations, retention fidelity, creation time, unrelated traffic and unresolved competing identities.
- `loopback_correlation` passes through real finite endpoint identities, declared platform/client ancestry and socket ownership, synthesized Windows DLT_NULL frames, production parser/stamper/pipeline, packet/JSON writers, shared registry and process trace. Four packets reconcile across platform and client ownership without attributing the proxy server.
- All 15 native tests pass, including real HTTP 200 exchanges for IPv4/IPv6 crossed with client, platform and absent-flow ownership. Final application serialization, phase-qualified eligibility, routing/propagation facts, body and writer conservation are asserted. Existing reused-tuple, ambiguous, withheld/bounded, global buffer history, absent terminal and semantic cutoff regressions remain enabled.
- The diagnosis regression failed before the closed reason projection, then passed. Reason counts reconcile to retained observation-window records; late owner-release detail and unknown raw adapter text cannot enter that population. Missing ownership remains inconclusive even with completed HTTP exchanges.
- Manifest role projection failed with an empty ordinary-session role list before correction, then passed for matched client/platform roles, unavailable ownership and retained partial process-trace loss/completeness.
- Human diagnosis renders reason counts at widths 20, 40, 80 and 160 with measured four-space anchors, omits zero counts, and explains read-only Doctor inspection without claiming ownership or a gameplay remedy.

The first full gate caught a Windows fixture assumption: accepted sockets can inherit a nonblocking listener mode. Both new finite endpoint fixtures explicitly select blocking accepted-stream I/O before their existing timeouts. The corrected native suite passed 15 tests and the full parallel facade library passed 158 tests with one intentional child-process-probe ignore (the child invocation passed).

## Full repository gate

`cargo xtask ci` completed with exit 0 and `ci: all checks passed` on 2026-10-09. It includes format, all-target/all-feature Clippy with warnings denied, locked workspace tests, repository conventions, dependency direction, licensing, supply-chain and release guards, package certification, wrappers, skill provenance, documentation and both curated API variants, specification currency, guided calibration acceptance, native threat model, review handoff/candidate registries, fuzz inventory, failure matrix, HTTP/TLS conformance, performance authority and Windows integration authority. The owning counts include 343 CLI library tests, 311 core library tests, 158 enabled facade library tests, 66 calibration integrations and 30 Deep Capture integrations. Existing dedicated-tier ignores retain their named ownership.

The final accepted-stream blocking-mode correction in `loopback_correlation` and manifest-reader validation assertion each passed their focused test after the combined gate. Final `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features -- -D warnings` passed with exit 0. Strict UTF-8/no-BOM/LF/final-newline/mojibake and punctuation scans passed all 31 changed/new files; `git diff --check` passed. No dependency or pinned artifact changed.

All console tooling uses verified hidden foreground child execution with redirected noninteractive I/O; output is streamed and watched through completion. No game, live Npcap capture, real trust-store mutation or owner artifact modification is performed.

## Evidence limits and external handoff

The controlled TCP exchanges and endpoint identities are real. Packet frames, process events and socket-table ownership are declared repository fixtures. This demonstrates the implementation correction and conservative artifact/fact behavior; it does not establish the historical machine-specific incident cause or universal title compatibility. Ordinary-user access repair #464, storage retention #458 and dependency PR #470 remain separate.

T019 is the post-publication external gate. The official PR conversation records bot-review rounds, every finding/thread disposition, final exact-head hosted checks and the owner handoff after this source checkpoint. Automatic creation review is round one; no more than one explicit second round is allowed.

## First external round and hosted corrections

Official PR #471 was published and attached on head `355730108ca349a365388f7816b3db50adfaee79`. The initial Codex review completed with one finding, 4230186671: arbitrary virtual loopback descriptions satisfied required host-loopback acquisition. The core helper, assembly selection/locality and Doctor regressions failed before correction, then passed after a closed known-Npcap description fallback. All 24 interface tests passed, including ordinary Capture selection. The shared Driver/Doctor helper retains its existing public signature.

Two hosted conformance invocations failed the existing no-ALPN HTTPS fixture, while the ordinary workspace invocation passed and TShark consumption itself passed. The fixture now queues a complete first TLS application flight with TCP_NODELAY and explicitly refuses generic-TLS fallback in its assertions. All seven HTTPS tests and local `cargo xtask conformance` passed. Linux Cargo is unavailable in local WSL; no tooling was installed. The corrected head receives the full local gate, hosted Linux/Windows checks and the single authorized second code/security review request. No third review request is permitted.

The corrected full `cargo xtask ci` completed with exit 0 and `ci: all checks passed` after the first review and hosted fixture correction. The owning counts are 345 CLI library tests, 312 core library tests, 158 enabled facade library tests, 66 calibration integrations, 30 Deep Capture integrations and all seven HTTPS tests. Format, all-feature Clippy, both curated API variants, source/docs/security/architecture and conformance gates pass without new waivers. Text hygiene and diff checks passed all eight correction files. Final exact-head hosted CI and second-round disposition are recorded in the PR conversation.
