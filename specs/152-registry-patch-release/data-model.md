# S152 Data Model

## Effective Approval Policy

Environment record identity is `name=crates-io`, id 21294112472. Exactly one required-reviewer rule names `User` id 46768484 and login `h8rt3rmin8r`, with self-review prevention false. Administrator bypass is explicitly true at the operator's direction. Custom deployment policies are enabled and protected-branches mode is disabled. A separate complete allowance record has `total_count=1` and exactly one positive-id rule with `type=tag,name=v*`. Additional API metadata is not policy authority; unknown protection kinds and incomplete records are rejected. Existing harmless wait timers are preserved.

## Verification Evidence

Fresh environment and allowance documents are bounded to 1 MiB each. Their inventory count must equal the supplied rule count and fit the requested 100-entry page. The validator emits policy findings and a configuration-verified message, never raw metadata or secrets and never approval. Before/after evidence carries observation time and immutable environment identity. Subsequent live configuration drift invalidates current verification.

## Release Identity

Candidate source, ten product crates, writer product strings, output goldens, conformance identity and not-started independent-review candidate use 0.10.1. `docs/published-release.json` remains actual 0.10.0 from source `787739edfa8d748e25cb4b5c4965f5c936850d16`; all current published markers remain 0.10.0. Candidate changelog and short highlights are preparation records, not publication evidence. No output schema, product protocol, target store or fact authority changes.

## Lifecycle

Specified, clarified and checklist-reviewed work advances through planned tasks and blocking analyze to controlled implementation, verified configuration, local candidate checks, official PR and satisfied current-head review. Human merge, explicit release tag, actual registry approval or deliberate owner bypass, publication verification and optional field measurements are separate later states. S152 performs none of the latter actions.
