# Analysis: Deep Capture MVP

## Review-sensitive points

- This is the first implementation slice that places fragcap on a selected session's application data path. The implementation must keep Capture passive while making Deep Capture explicit, scoped, reversible, and auditable.
- The MVP is intentionally narrow: one stored target, one known-compatible scoped launch path, one external `mitmdump` backend, and controlled local verification. Unknown real-target compatibility is a refusal.
- The command must not silently degrade into system-wide proxy settings. That fallback would violate the constitution and would produce misleading compatibility facts.
- Trust mutation is the critical side effect. Tests need an adapter seam that proves no trust change occurs without confirmation and that cleanup does not fabricate success.
- Output success is a bundle, not a proxy log. The manifest, `.fcapng`, application JSONL, optional HAR, proxy/process sidecars, compatibility update sidecar, and cleanup report must agree on session identity and omissions.
- The controlled target harness is mandatory. Real locally installed games can inform manual research, but committed tests and fixtures must not include actual title names, account data, install paths, endpoints, or captured payloads.

## Relationship to issues

- #213 positioned Capture and Deep Capture in the constitution and master specification.
- #214 researched backend options and left native Rust backend selection as follow-on work.
- #215 measured launch inheritance scenarios and established that scoped proxy propagation is compatibility data, not something to assume.
- #216 defined the session bundle and correlation model this MVP must write.
- #217 added local Deep Capture compatibility facts this MVP must consume and update.
- #218 added doctor readiness and cleanup surfaces this MVP should reuse.
- #219 is this MVP.
- #220 should wait until #219 behavior lands, then publish supported traffic type documentation and a generated compatibility matrix.

## Analyze Gate

- **Ambiguity**: The exact command spelling can be decided during implementation, but Deep Capture must be first-class and stored-target-only for MVP. This is acceptable.
- **Coverage**: Requirements cover command, preflight, backend, trust, launch, capture, outputs, status, compatibility facts, cleanup, controlled verification, refusals, and privacy. No known acceptance area is uncovered.
- **Constitution**: P-1, P-4, P-5, P-9, and P-10 are the highest-risk principles. The plan includes explicit gates for each.
- **Dependency risk**: No new Rust dependency is planned. The external `mitmdump` runtime backend is isolated behind an adapter so native Rust backend work can replace it later.
- **Privacy risk**: The task list contains a required scan for PII, real local title names, endpoints, paths, credentials, account material, and payloads before PR review.
- **Implementation readiness**: Ready to implement as one substantial PR after this planning commit.
