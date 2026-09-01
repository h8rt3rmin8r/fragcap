# TLS and Sensitive Artifact Contract

## Key logging

- An absent request creates no file and no manifest claim.
- A present request creates and protects the final path before proxy traffic.
- Only the client-facing rustls server configuration receives the logger.
- Every accepted line is a complete NSS label, client-random, secret triple and is flushed live.
- Final status reports exact record count, failure, emptiness, and retention outcome.

## Client identity

- Certificate and private-key inputs are paired and explicit.
- The facade reads only those paths; the proxy parses and configures the identity.
- Accepted keys are PKCS#1, PKCS#8, or SEC1 and must match the leaf certificate.
- No API discovers or extracts target-owned private keys.
- Secret bytes are redacted and zeroized when ownership ends.

## TLS refusals

- Categories are `client-certificate-required`, `client-certificate-rejected`, `certificate-validation`, `protocol-mismatch`, `client-trust-rejection`, or `unknown`.
- Machine-readable evidence comes from rustls variants and alerts, never display-string parsing.
- Only explicit hash-certificate rejection may claim pinning; other trust refusal is unknown.

## Sensitive artifacts

- Bundle protection completes before the first writer or proxy starts.
- Cleanup reads only normalized, contained, manifest-declared sensitive paths.
- Each destructive action has a synced intent before its effect and a synced result after it.
- Missing files are an idempotent success; unrelated ordinary evidence is never selected.
- Sharing writes a sibling staging tree, excludes secret-adjacent roles, writes a complete transformation manifest, and publishes by rename.
- Source bundle bytes never change during sharing.

## Platform workflow

- Pull requests run the Windows platform job without a path filter.
- Pushes run it only on `main`, without a path filter.
- Repository lint rejects a future `paths` or `paths-ignore` filter in the trigger block.
