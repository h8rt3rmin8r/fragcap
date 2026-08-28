# Contract: Deep Capture Architecture Page

## Capture Diagram Contract

- The diagram begins with selected interfaces and Npcap, not a target process hook.
- Raw packet acquisition and external socket/process evidence remain separate until attribution.
- Scope is a userspace output decision after attribution. The default `target` scope omits unattributed and other-process packets with named counters; `--scope all` retains all acquired traffic.
- `.fcapng` is the packet-truth output and opens in an unmodified pcapng analyzer.
- No proxy, CA trust, application semantics, or mitmdump node appears in the Capture diagram.

## Deep Capture Diagram Contract

- The diagram begins with one stored target and a read-only compatibility preflight.
- The only shipped real-target success path is current `reached-client` and `confirmed` evidence for `steam-protocol-cold`.
- Preflight refusal occurs before proxy, trust, managed launch, and bundle side effects.
- Preflight validates Capture configuration but does not claim that Npcap, elevation, or interface opening has succeeded. Those live checks occur after session resources may have started, and their failures flow through cleanup and bundle state.
- The eligible path prepares ordinary Capture, a fragcap-owned loopback mitmdump child, optional current-user Root trust, and launch-scoped proxy configuration before the target launch.
- Packet Capture and proxy observations remain distinct inputs to correlation and the session bundle.
- Cleanup is a first-class execution stage whose per-resource result is retained in audit evidence.
- The diagram uses at most twelve primary nodes and keeps detailed refusal and artifact lists in prose or tables.

## Trust and Security Contract

- `--trust-ca` is itself the explicit authorization for the fragcap-owned current-user Root change. No second trust prompt is promised.
- `--yes` is identified as broader unattended preconfirmation, not safer first-run guidance.
- Deep Capture never silently changes system-wide proxy settings and has no wider fallback.
- Traffic that bypasses the proxy, rejects the CA, uses certificate pinning, or uses unsupported protocols does not yield claimed application semantics.
- Neither mode injects code, hooks target functions, reads target memory, modifies target executables, changes the Winsock catalog, installs a packet interception driver, extracts target TLS keys, or bypasses pinning.

## Output Authority Contract

- `capture.fcapng` is packet truth.
- `application.jsonl` and optional `http.har` are proxy observations and can be partial.
- Optional `tls-keylog.log` contains proxy-owned analyzer material and is sensitive.
- `proxy.jsonl`, `process-trace.jsonl`, and `compatibility.json` carry diagnostic and correlation evidence.
- `manifest.json` indexes artifacts, omissions, state, and trust; `cleanup.json` records per-resource cleanup.
- Missing structured correlation remains missing rather than being inferred or fabricated.
- The page summarizes these classes but does not absorb issue #248's exhaustive handling and lifetime matrix.

## Npcap Contract

- fragcap never bundles, hosts, caches as its own, or redistributes Npcap or its installer.
- The published `doctor --fix` opens the official acquisition page only after its action is confirmed.
- A source build with optional `net` support may fetch the vendor's signed installer to a uniquely named temporary path and launch it only after confirmation. The page makes no unsupported claim that the launched temporary file is deleted afterward.
- Npcap supplies live packet acquisition. mitmdump supplies only the shipped Deep Capture proxy backend. An unmodified analyzer consumes `.fcapng` and is not part of the capture engine.

## Navigation and Content Contract

- Link the master specification, getting-started guide, Deep Capture compatibility reference, Capture output-format reference, CLI reference, and relevant glossary entries.
- Use only generic components, `sample-target`, documentation addresses, and synthetic paths if examples are needed.
- Do not use real titles, accounts, endpoints, hosts, identifiers, or captured material.
