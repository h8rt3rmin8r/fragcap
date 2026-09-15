# Contract: Native Documentation and Independent Review Handoff

## Acceptance boundary

S150 provides parser-checked current guidance, valid artifact specimens, a complete static review mapping, explicit finding workflow, and version-ready release preparation. It cannot report independent audit completion, installed-build pass, Doctor reproduction, published bytes, live compatibility, or final feature completion.

## Reproducibility

An authorized reviewer records `git rev-parse HEAD`, the actual release tag and source relationship, SHA-256 of Cargo.lock, exact certified package checksums and hidden build identity, supported environment, tool versions, selected methods, results, and unperformed checks. Review source identity is recorded after final merge/tag rather than making a document refer to its own future commit. Package certification and supply-chain reports identify transferred bytes independently.

## Validation interfaces

`cargo test -p fragcap-cli --test cli_reference --locked` parses current command examples without dispatch. `cargo xtask review-handoff` checks review-scope and the workspace-version-bound not-started template without host effects. Test references declare exact package features, require Git-tracked Cargo ownership, and must appear in compiled harness `--list` output with defaults disabled; discovery does not execute those tests. Existing `published_examples_match_the_versioned_reader` requires `--features deep-capture` and validates manifest specimens through the product reader. `cargo xtask ci`, MSRV, docs build/check, and production accessibility remain required implementation evidence.

## Operator release and external review

After green CI, resolved PR reviews, and operator merge, only the operator creates and pushes the v0.10.0 tag and approves registry publication. Independently authorized reviewers then follow installed-build procedures under #333. #331 reconciles their documentation findings; #334 alone controls final completion. Doctor #372 measurements use the published release and remain separate from speculative optimization.

## Authorized #413 remediation

Runtime correction must preserve every observation, the finite 64-event pending writer limit, timeout/final flush, exact failed-storage reconciliation, and nonblocking queue admission. The S128 registry and workload remain immutable. Both original reports remain truthful historical evidence. Two fresh fixed-code Windows short campaigns must individually pass the canonical report validator and agree under the existing comparability contract before release verification clears. A successful retry of unchanged failing code is not remediation. Independent #333 review, operator testing, merge, and publication remain external acceptance.
