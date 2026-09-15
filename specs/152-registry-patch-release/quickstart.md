# S152 Validation Guide

## Controlled Checks

From the repository root, use the verified hidden Windows launcher for non-Git tooling. These commands do not run installed sensitive software:

```sh
cargo test -p xtask --locked release_guard
cargo xtask release-guard
cargo xtask notes 0.10.1
cargo xtask spec
cargo xtask ci
cargo xtask msrv
```

Expected results are green fixture/wiring checks, valid prepared short highlights, candidate applicability 0.10.1 and actual publication 0.10.0. Hosted PR package certification must validate final candidate bytes and the isolated Windows installer lifecycle. Do not install candidate packages or invoke production Doctor on the operator machine.

## Current Environment Verification

Use fresh read-only records, not a committed snapshot:

```sh
gh api --method GET repos/h8rt3rmin8r/fragcap/environments/crates-io > environment.json
gh api --method GET 'repos/h8rt3rmin8r/fragcap/environments/crates-io/deployment-branch-policies?per_page=100' > policies.json
cargo xtask release-guard environment.json policies.json
```

Keep temporary records outside tracked source. Failed fetches, missing reviewer/tag policy, unknown or mismatched administrator-bypass state and incomplete facts must fail closed. Administrator bypass remains enabled by explicit operator direction. Success verifies configuration only; an operator-selected bypass is distinct from reviewer approval. Never approve or bypass a pending deployment as an agent.

## Human Handoff

The official PR must have current-head green CI and all comment dispositions, with at most one manual second review request. The operator performs final review and merge. Tagging, registry approval and publication require a separate explicit release request. #372 field measurements and independent #333/#413 acceptance remain pending; no operator-only evidence is replaced by these checks.
