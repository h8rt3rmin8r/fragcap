# Data Model: Public Entry Point Reconciliation

This slice changes documentation and repository metadata rather than runtime data. The model below captures the claim relationships that the implementation must keep consistent.

## Product Mode Claim

| Field | Capture | Deep Capture |
| --- | --- | --- |
| Availability | Shipped in v0.7.0 | Shipped in v0.7.0 |
| Activation | Normal capture selection | Explicit operator selection |
| Data path | Passive observation | Selected target routed through a local proxy |
| Scope | Selected target and observed flows | Selected target session and compatible proxy-routed traffic |
| Cleanup | Capture session cleanup | Proxy, trust, bundle, and sensitive-artifact cleanup is visible and auditable |
| Limits | Attribution can remain unresolved | Inspection can be partial or unsupported; no pinning bypass or target key extraction |

## Public Entry Point

| Attribute | Meaning |
| --- | --- |
| Surface | Repository landing page, contributor guide, documentation index, site contributor page, issue form, or repository description |
| Audience | Prospective user, contributor, reporter, or repository browser |
| Required claims | The subset of product, security, acquisition, workflow, and release facts necessary for that audience |
| Canonical authority | Constitution for boundaries, master specification for architecture, current CLI for commands, and current release for availability |
| Exclusions | Details owned by later focused pages or issues |

## Npcap Policy Claim

| Attribute | Required value |
| --- | --- |
| Runtime relationship | Separate prerequisite for live packet capture |
| Distribution | Never bundled, hosted, cached as fragcap's own, or redistributed |
| Shipped assisted acquisition | `doctor --fix` opens the official download page after explicit interactive confirmation |
| Optional assisted acquisition | A `net`-enabled source build may fetch and launch the vendor installer after explicit interactive confirmation |
| Non-interactive behavior | Report the official location without fetching or launching |
| Installation option | WinPcap API-compatible mode remains required; loopback support is automatic in current Npcap |

## Relationships

- Each public entry point projects claims from the same product-mode and Npcap-policy authorities.
- `CONTRIBUTING.md` is the canonical contributor workflow; the site contributor page summarizes and links to it.
- The master specification records current architecture and release state; historical slice artifacts remain immutable provenance.
- The repository description is an external metadata projection of the short product definition.
