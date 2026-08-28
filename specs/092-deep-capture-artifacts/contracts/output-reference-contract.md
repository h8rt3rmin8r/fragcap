# Public Output Reference Contract

## Page Authority

- `site/content/docs/reference/output-formats.mdx` is the single public contract for ordinary Capture formats and Deep Capture bundle artifacts.
- The page keeps ordinary Capture pcapng and packet JSON Lines visibly separate from Deep Capture application JSON Lines and sidecars.
- Existing inbound links remain valid because the page path does not change.

## Required Page Sequence

1. Introduce the two output families and reject the retired equivalent-facts claim.
2. Document ordinary Capture pcapng and packet JSON Lines, payload handling, and finalization accounting.
3. Introduce the Deep Capture session bundle and `manifest.json` read-first rule.
4. Define the three finalized manifest states and the early-cleanup-without-manifest exception.
5. Present the complete artifact authority, sensitivity, lifetime, and production matrix.
6. Explain application JSON Lines, HAR projection limits, and proxy-owned TLS key-log lifecycle.
7. Define current manifest omissions separately from application reasons and cleanup statuses.
8. Explain correlation anchors and absent-anchor semantics.
9. Provide one synthetic manifest example and private-by-default handling guidance.
10. Link to compatibility, CLI, glossary, and getting-started material without duplicating their concerns.

## Token Coverage

The page must contain all artifact roles or their exact filenames:

- `manifest.json`
- `capture.fcapng`
- `application.jsonl`
- `http.har`
- `tls-keylog.log`
- `proxy.jsonl`
- `process-trace.jsonl`
- `compatibility.json`
- `cleanup.json`

The page must contain all finalized states:

- `complete`
- `partial`
- `failed`

The page must contain all current manifest omission reasons:

- `writer-failed`
- `no-http-semantics`
- `not-requested`
- `not-produced`

## Security Language

- `tls-keylog.log` is always described as proxy-owned, secret-adjacent session material and an analyzer aid.
- The page states that no key is extracted from a target process and that certificate pinning is not bypassed.
- The exact manifest sensitivity labels are preserved, but `ordinary` is never defined as safe to publish.
- Sharing guidance preserves the original bundle and directs the operator to a reviewed scrubbed copy.
- Normal session cleanup and later confirmation-gated `doctor --fix` residue cleanup are distinguished.

## Cross-Page Contract

- `site/content/docs/reference/deep-capture-compatibility.mdx` links to the output page for the complete artifact and omission contract.
- `site/content/docs/reference/cli.mdx` links to the output page beside the `deep-capture` command description.
- Neither inbound page carries a second complete artifact matrix.

## Validation Contract

- Focused source checks find every role, state, reason, and inbound link.
- No phrase claims that both output families carry the same facts.
- No synthetic example contains a drive-letter path, user profile, real title, account identifier, access token, private endpoint, host identifier, or TLS secret line.
- `cargo xtask docs check`, `cargo xtask docs build`, repository lint, and `cargo xtask ci` pass.
