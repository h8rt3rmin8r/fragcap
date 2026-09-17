# Data model: S157 complete native documentation contract

## Documentation contract registry

The registry is one versioned JSON document with exact top-level fields:

- `schema_version`: integer `2`.
- `published_baseline`: exact release tag and immutable source revision represented as ordinary strings.
- `current_source_boundary`: closed stable identifiers for the post-release documentation and verification slices that current guidance may describe without calling them published product behavior.
- `topics`: the complete ordered set of documentation contract rows.
- `example_authorities`: the complete ordered set of command and artifact example authorities.
- `completion_boundary`: exact open issue identities whose evidence remains external to S157.

Unknown top-level fields fail. Required arrays are non-empty, have unique stable identifiers, and appear in the canonical registry order.

## Documentation contract row

Each topic row contains:

- `id`: one stable member of the thirteen-topic closed vocabulary.
- `page`: one confined current `site/content/docs/**/*.mdx` regular file outside historical changelog and glossary trees.
- `sections`: a non-empty ordered set of exact level-two or level-three heading texts that must remain present on the page.
- `tests`: one or more exact Cargo-discoverable non-ignored test authorities with explicit feature sets.
- `example_authorities`: zero or more stable identifiers referencing the global example authority set.

The thirteen topic identities are architecture, setup, capture-modes, cli, protocols-routing, refusals, artifacts-correlation, diagnostics-recovery, security-privacy, bounds, packaging-migration, public-api, and completion-status.

## Executable test authority

Each executable authority contains:

- `path`: confined current Rust source under `crates/` or `xtask/src/`.
- `function`: exact test function identifier.
- `features`: unique explicit Cargo features required to discover the test.

The referenced function must be an actual non-ignored test under the declared feature ownership. A comment, helper, disabled configuration, malformed name, absent feature, or unsupported harness owner fails.

## Example authority

Each example authority contains:

- `id`: stable unique identifier.
- `kind`: `command-corpus`, `committed-artifact`, or `controlled-contract`.
- `paths`: zero or more confined committed files or directories appropriate to the kind.
- `tests`: one or more exact executable test authorities.

Command-corpus authority owns all current README and nonhistorical site fragcap examples through the production command parser without dispatch. Committed-artifact authority names exact safe specimens such as manifest examples and packet goldens. Controlled-contract authority names artifacts that are safest and most truthful when produced and reconciled entirely inside controlled tests rather than committed as standalone files.

## Completion boundary

The completion boundary contains the exact open issues whose facts cannot be produced by S157:

- #333, independent whole-product native proxy and trust review.
- #413, independent installed QUIC performance retest.
- #331, parent documentation reconciliation after real findings.
- #334, final feature-completion gate.
- #278, native Deep Capture epic.

Registry validation checks presence and uniqueness. It does not query GitHub or infer issue state during offline CI.

## State transitions

1. Version-one readiness inventory is read only as historical source control and is replaced in current authority by version two.
2. S157 starts with published v0.10.1 plus current source verification boundaries.
3. Passing validation establishes complete agent-runnable documentation engineering, not independent acceptance.
4. Later actual independent findings amend current pages and completion evidence before #331 can close.
5. Only #334 can change public completion language from functional but incomplete.
