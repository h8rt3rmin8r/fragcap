# Contract: v0.10.3 release publication

## Candidate

The candidate moves the complete workspace and generated version-bearing surfaces to 0.10.3, assembles all unreleased fragments in chronological order, and presents bounded highlights. It leaves actual published-state records at v0.10.2. An operator-reviewed PR is the only route to main.

## Tag

After operator merge, fetch and verify exact clean main, compare candidate and merged source, and reject an absent or conflicting version identity. Create annotated v0.10.3 only when no conflicting local or remote tag exists. The tag must peel to exact merged source before push.

## Hosted release

The existing workflow checks identity and registry protection, certifies Windows packages, publishes GitHub files, and publishes crates in dependency order. If `crates-io` waits for approval, the operator acts through GitHub; the agent does not approve or bypass that environment.

## Completion

Four jobs must pass; the release must be public, non-draft, and non-prerelease; six files and three sidecars must reconcile; certification must bind the tag source; and ten crate version records must be non-yanked with checksums.

## Records

Only a later records-only operator-reviewed PR may advance current publication markers. It changes no tag, public file, package byte, workflow, dependency, or environment policy.
