# Data model: S183 release candidate

| Entity | Identity | Required relationship |
| --- | --- | --- |
| Candidate source | Branch plus immutable commit SHA and tree | Version, changelog, notes, generated evidence and graph digests agree on it. |
| Dependency graph | Lockfile and exact resolved packages/features | Product and isolated graphs are separately audited. |
| Qualification receipt | Candidate SHA, command or physical test, timestamp, outcome | May support only the exact source or binary measured. |
| PR review receipt | PR number, head SHA, check status, bot round, thread status | A changed head requires renewed checks and comment resolution. |
| Published release | Existing public v0.10.8 tag and assets | Remains the current marker until a later approved release and reconciliation. |

The candidate moves from prepared to checked to review-ready. Merge, tag, publication and current-record updates are later states outside this slice.
