<!-- spec-impact: 15, 17.2.1, 25, 28.1 -->

- Stored target resolution remains authoritative: resolved and ambiguous stored selectors never fall through to discovery, and `--id` remains stored-only.
- Discovery accepts only an exact Steam application identifier or Unicode-aware case-insensitive display-name match. Ambiguity, partial names, executable hints, and path inference never select a target.
- Registration uses a complete, domain-separated plan that binds the candidate, conserved discovery account and warnings, and effective store before default-no or exact structured confirmation. Fresh discovery must reproduce the plan before the existing shared registration operation runs.
- Registration authorization is separate from any later Deep Capture session authorization. Discovery hints are not promoted into launch topology, so a newly registered target can truthfully stop at the existing missing-declaration limitation without starting a session.
