# Contract: Public Entry Points

## Shared Mode Boundary

Every in-scope first-contact surface must preserve these facts:

1. Capture is shipped passive process-attributed packet capture.
2. Deep Capture is shipped explicit, target-scoped, reversible local proxy inspection for authorized sessions.
3. Deep Capture observes application semantics only for compatible traffic that reaches the proxy and accepts the configured trust path.
4. Neither mode injects code, hooks target functions, reads target memory, modifies target executables, changes the Winsock catalog, uses a packet interception driver, extracts target TLS keys, or bypasses certificate pinning.
5. No surface claims that every packet is attributable or every encrypted flow is inspectable.

Exact prose is intentionally not contractual except for the repository description below.

## Repository Description

The GitHub repository description is exactly:

```text
Passive process-attributed Capture and explicit, target-scoped Deep Capture for Windows game traffic.
```

## Surface Responsibilities

| Surface | Required content | Deliberately deferred |
| --- | --- | --- |
| `README.md` | Product value, both modes, v0.7.0 status, current command overview, Npcap policy, security boundary, current repository map | Full first-run walkthrough, exhaustive CLI, bundle artifact reference |
| `CONTRIBUTING.md` | Current workspace state, P-1 boundary, workflow, checks, Npcap dependency rule | User walkthrough and architecture diagrams |
| `site/content/docs/index.mdx` | Both-mode introduction, prerequisites, start links, concise security posture | Detailed compatibility matrix and bundle inventory |
| `site/content/docs/contributing.mdx` | Concise current contributor summary linked to canonical guide | Duplicate full workflow |
| Bug issue form | Current reproduction example, actual version request, current Npcap option, scrub guidance | Command reference prose |
| Feature issue form | Current P-1 scope checks and current planning pointers | Roadmap duplication |

## Issue Form Values

- Bug reproduction placeholder: `fragcap capture 1 --duration 30m`.
- Version placeholder: `fragcap --version output` rather than a frozen old version.
- Npcap installation choices: WinPcap API-compatible mode enabled, not enabled, or not sure.
- Feature requests may propose explicit local proxy behavior within P-1. They may not require a denylisted technique or silent system-wide proxying.

## Npcap Boundary

Npcap remains a separate installation and is never bundled or redistributed. Public surfaces may state that `fragcap doctor --fix`, after explicit interactive confirmation, fetches and launches the vendor's own signed installer from the official location. They must not describe that action as fragcap distributing or silently installing Npcap.

## Historical Integrity

Historical release records, revision-history rows, and completed slice artifacts remain unchanged. Present-tense master-spec statements and the release table must describe v0.7.0 accurately.
