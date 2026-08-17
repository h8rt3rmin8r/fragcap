# Contract: The discovery account (P-4 conservation)

Every `discover()` returns a `DiscoveryAccount` alongside its candidates. The
account is the mechanical guard against silent loss: a discard path added later
with no counter fails the conservation test rather than passing quietly. It is the
same discipline `SeedSummary::is_conserved` already enforces for the catalog
seeder.

## Named outcomes

Every item a source considers lands in exactly one of:

| Outcome | Meaning |
| --- | --- |
| `produced` | Emitted as a `CandidateTarget`. |
| `parse_failed` | Metadata present but unparseable (e.g. a bad appinfo section). |
| `declined_by_user` | A human rejected the candidate at the interactive step. |
| `considered_not_a_game` | A directory matched no signature and no known-root rule. |
| `volume_skipped` | Not examined because its volume was ineligible. |
| `access_error` | Not examined due to a permission or I/O error; named per root/volume. |

## Invariant

```text
produced + parse_failed + declined_by_user
    + considered_not_a_game + volume_skipped + access_error == considered
```

`DiscoveryAccount::is_conserved()` returns this equality. Every source test
asserts it. Adding a new non-produced path without a counter breaks a compile (a
new enum arm) or the invariant (an uncounted item), never ships silently.

## Surfacing

The account is surfaced to the user in the discovery/listing output the same way
the seeder's summary is surfaced: the counts are reported, so "found 3 games,
skipped 1 volume, 2 directories were not games" is statable rather than a bare
list. An excluded volume's skip is visible (it is not silent), which is what makes
the eligibility decision auditable (FR-017).
