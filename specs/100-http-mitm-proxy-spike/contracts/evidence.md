# Evidence Contract

Each row is uniquely identified by `(backend, scenario, proof_point)`. The harness seeds every required fallback row as `not measured` before execution and replaces it at most once. Comparison joins those rows to S099 by `(scenario, proof_point)`.

Statuses are complete, partial, empty, bounded, truncated, unsupported, failed, and not measured. Complete payload rows include protocol, length, and digest. Every negative state names a bounded reason.

Parity is false unless both rows are complete. Payload parity additionally requires identical protocol, length, and digest. Missing and negative rows never become parity through omission.

Committed evidence may include fixed fixture data, stable digests, package names, versions, licenses, loopback labels, tool versions, and aggregate timings. It excludes private keys, credentials, tokens, raw operator traffic, absolute machine paths, usernames, ephemeral ports, and operator-attributable addresses.

The record is complete only when every S099 proof point has one fallback result, every issue criterion links to evidence, and all three backends appear in the comparison.
