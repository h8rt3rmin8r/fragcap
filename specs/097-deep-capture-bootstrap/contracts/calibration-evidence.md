# Contract: Calibration Evidence And Audit

## Observation Rules

- Correlated final-client proxy traffic may produce `proxy-routing=reached-client`.
- Routing does not produce `proxy-propagation=confirmed` without independent non-invasive evidence.
- The controlled target may confirm propagation by reporting its own inherited proxy environment.
- Launcher-only, escaped-tree, and no-proxy outcomes require corresponding positive process or packet evidence.
- Silence alone is inconclusive or no relevant traffic.
- Generic TLS failure does not prove certificate pinning.
- Phase outcomes are never stored as aggregate compatibility verdicts.

## Fact Persistence

Facts append through the existing target-owned store and carry launch case, observed-run provenance, timestamp, fragcap version, backend identity/version/mode, observed owner context, and a fresh stale marker. Each attempted append has an individual audit result. Failure to write one row does not erase another successful row.

## Bundle Authority

`compatibility.json` owns the displayed plan, confirmation state, phase outcome, direct observations, omissions, proposed fact rows, and per-row write results. `cleanup.json` owns per-resource cleanup. `manifest.json` indexes both and records complete, partial, or failed session state. Missing artifacts and unavailable observations use explicit omission reasons.

## Privacy

Local facts and bundles are not published automatically. Automated fixtures contain only controlled target names, loopback endpoints, and synthetic observations. Credentials, real titles, account identifiers, local install paths, public endpoints, captured third-party payloads, and private CA material are never committed.
