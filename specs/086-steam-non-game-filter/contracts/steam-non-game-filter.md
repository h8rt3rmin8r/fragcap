# Contract: Steam Non-Game Filter

## Human Contract

`fragcap targets discover` and the hero `fragcap targets` command must not show Steam app ids whose appinfo type is one of:

- `Music`
- `Tool`
- `Application`
- `Config`
- `Video`

They may still show Steam app ids whose appinfo type is:

- `Game`
- `Demo`
- absent or unreadable

## Discovery Contract

For every excluded Steam app:

- No `CandidateTarget` is emitted.
- `DiscoveryAccount.considered` includes the app.
- `DiscoveryAccount.considered_not_a_game` includes the app.
- `DiscoveryAccount::is_conserved()` remains true.

For every preserved Steam app:

- Existing candidate fields remain governed by the prior Steam discovery contract.
- Catalog classification, evidence detection, fidelity, install root, folder name, and executable hint behavior are unchanged.

## Non-Contract

This slice does not define a public machine-readable list of app types, does not add a CLI flag for overriding the filter, and does not add name-based filtering for appinfo-less titles.
