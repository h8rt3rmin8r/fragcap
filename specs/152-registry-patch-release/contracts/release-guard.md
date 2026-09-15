# S152 Release Guard Contract

## Repository Command

`cargo xtask release-guard` checks repository workflow wiring offline. `cargo xtask release-guard <environment.json> <policies.json>` additionally checks supplied authoritative metadata; both files must be freshly fetched immediately before the protected effect in automation. The command never configures an environment, contacts a registry, approves a deployment or executes product effects. Valid checks return 0, policy findings return 1, unavailable inputs or invalid invocation return 2. No failure prints complete input JSON.

## Exact Policy

Require immutable `crates-io` id 21294112472, `can_admins_bypass=true`, exactly one required-reviewer rule with self-review false and only the owner User id 46768484/login `h8rt3rmin8r`; custom-only branch policy and one tag `v*` allowance with complete count. Missing fields, duplicates, wider allowances, branch-vs-tag confusion, unknown protection kinds and unapprovable reviewer settings refuse. Harmless valid wait timers are accepted without removal. Both documents are at most 1 MiB, and at most 100 allowance entries may constitute a complete page.

## Workflow Boundaries

The existing identity job first verifies exact release tag/workspace agreement, then fetches fresh environment and tag-allowance records using read-only GitHub API calls and validates them. Package certification depends on identity. GitHub release creation depends on successful certification and independently fetches and validates current policy immediately before `gh release create`, rather than relying on the potentially 45-minute-old identity check. The existing publish job still depends on the official release and references `environment: crates-io`; after normal owner approval or a deliberate owner bypass, it independently fetches and validates fresh policy immediately before `publish --execute`. Failed fetching or validation prevents all subsequent steps. Approval and bypass remain distinct GitHub operator actions; neither is inferred from validation success.

Offline regression validation checks dependency and environment fields at their actual job indentation, not matching text inside a step. Identity, certification, release and publication jobs must not use a job-level `continue-on-error` override; guard steps also refuse conditional or nonblocking mutations. This is a bounded check of the pinned workflow shape, not a general YAML policy engine.

## Preparation Boundary

No `v0.10.1` tag, release asset or registry upload is created during S152. The actual publication record remains v0.10.0. New patch notes and assembled changelog identify prepared content and retain all independent completion limits. Operator-installed product validation is neither simulated nor agent-executed.
