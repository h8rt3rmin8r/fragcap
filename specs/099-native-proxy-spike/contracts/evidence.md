# Evidence Contract

## Result Vocabulary

Every proof point uses one status: `complete`, `empty`, `bounded`, `truncated`, `unsupported`, `failed`, or `not-measured`. `complete` means the expected observation was captured in full and matched by byte length and digest where a body or message exists. No other status counts as parity.

## Required Authorities

- The controlled scenario owns expected request, response, protocol, and message values.
- The backend adapter owns emitted application observations and lifecycle events.
- The harness owns byte accounting, stable digests, deadlines, normalization, and sanitization.
- The dependency tools own package, feature, license, advisory, source, and Rust-version data.
- The final decision record owns the selected follow-up, not product shipping state.

## Sanitization

Committed evidence may contain loopback addresses without ephemeral ports, fixed synthetic paths, package identities, public tool versions, relative durations, byte counts, and stable digests of synthetic payloads. It must not contain private keys, raw captured traffic, credentials, tokens, cookies, user names, home paths, ephemeral ports, process identifiers, or non-loopback addresses.

## Completeness Invariant

For each backend, every scenario and required proof point has exactly one result row. A missing event becomes `not-measured`, never an absent row. Every non-complete row carries a reason. Candidate and baseline rows sort by the same scenario and proof-point keys.
