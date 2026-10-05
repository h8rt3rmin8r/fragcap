<!-- spec-impact: 17.2.1, 28.1 -->
**2026-10-05** A separate two-minute route-owner release budget now precedes proxy shutdown because persistent managed applications can pass inherited session proxy settings to later children. Query-only process absence supports a release claim; a remaining or unknown owner is a partial-cleanup recovery state, and fragcap does not control the target process.

**2026-10-05** PR review found that ASCII-only executable comparison could mistake a still-running non-ASCII image for an absent route owner. S165 uses the target store's Unicode lowercase fold for route-owner deduplication, release checks, and the existing warm-process inventory check.

**2026-10-05** Hosted package certification exposed a closed structured-event bound of 32 records. The controlled smoke now emits 33 because S165 records one additional route-owner cleanup result. The certification parser permits at most 40 records while retaining its 256 KiB byte limit, and the controlled CLI test asserts both limits.
