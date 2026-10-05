<!-- spec-impact: 17.2.1, 28.1 -->
**2026-10-05** A separate two-minute route-owner release budget now precedes proxy shutdown because persistent managed applications can pass inherited session proxy settings to later children. Query-only process absence supports a release claim; a remaining or unknown owner is a partial-cleanup recovery state, and fragcap does not control the target process.

**2026-10-05** PR review found that ASCII-only executable comparison could mistake a still-running non-ASCII image for an absent route owner. S165 uses the target store's Unicode lowercase fold for route-owner deduplication, release checks, and the existing warm-process inventory check.
