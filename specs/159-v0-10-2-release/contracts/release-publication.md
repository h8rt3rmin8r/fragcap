# Contract: v0.10.2 release publication

## Candidate preparation

The candidate MUST move the complete workspace and every generated version-bearing surface to 0.10.2. It MUST assemble all unreleased changelog fragments in chronological order and provide bounded release highlights.

The candidate MUST leave actual-publication records at v0.10.1. A human-reviewed pull request is the only route to main.

## Tag publication

After human merge, the publisher MUST fetch main, verify a clean exact match with origin, resolve the merged candidate source and create v0.10.2 only when no conflicting local or remote tag exists.

The tag MUST peel to that source. Pushing it is the release trigger authorized by the operator's request.

## Hosted release

The release workflow MUST validate tag and workspace agreement and registry protection before certification. It MUST build and certify the Windows distribution, independently revalidate certified bytes, render the reviewed highlights and create the public GitHub release.

Registry publication MUST remain behind the existing `crates-io` environment. If GitHub requests approval, only the owner may approve or deliberately bypass it. The agent does neither.

## Completion

Publication is complete only when identity, package certification, release creation and registry publication jobs are green; the public release is non-draft and non-prerelease; all six files reconcile; and all ten registry packages are exact and non-yanked.

## Records reconciliation

The later records branch MUST contain only verified publication facts, source-neutral documentation updates, S159 artifacts and project evidence. It MUST NOT alter the tag, published files, package bytes, workflow, dependencies or environment policy.

Independent security review, installed QUIC acceptance, Doctor field measurements and final Deep Capture completion remain separate outcomes.
