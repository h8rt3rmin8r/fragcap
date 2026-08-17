# Contract: The `TargetSource` seam

This contract binds every discovery origin. It is the P-10 seam: one operation,
one candidate shape, across every batch size.

## Trait

```text
trait TargetSource {
    fn name(&self) -> &str;
    fn discover(&self) -> Result<Discovery, TargetsError>;
    fn default_fidelity(&self) -> FidelityTier;
}
```

## Guarantees

1. **One candidate shape.** `discover` yields `CandidateTarget` values regardless
   of source; a consumer (the listing, the authoring step) never branches on which
   source produced a candidate.
2. **Total accounting.** `discover` returns a `DiscoveryAccount` that conserves:
   every item considered lands in exactly one named outcome (see
   [discovery-account.md](discovery-account.md)). A source that examines N items
   and produces K returns an account whose outcomes sum to N.
3. **Hard vs soft failure.** An `Err` is a whole-run failure (e.g. an unreadable
   metadata store). A single item that cannot be parsed, is declined, or is not a
   game is counted in the account, never turned into an `Err` and never dropped
   silently (P-4, P-9).
4. **Unknown is a value.** A candidate whose identity does not join the catalog is
   produced with `classification = unknown`; it is never dropped for being unknown
   (P-9).
5. **No durable write from discovery.** `discover` writes nothing to `local.db`.
   Persistence happens on first use (FR-021). The eligibility table is written by
   the volume machinery, not by `discover` (see
   [volume-eligibility.md](volume-eligibility.md)).
6. **Fidelity is stamped by the source.** `default_fidelity` is the source's stamp
   (Steam -> heuristic-unverified; interactive-accepted -> authored). A consumer
   does not infer fidelity from whether a candidate exists (the S051 rule).

## Adding a new source (SC-006)

Implementing `TargetSource` is the whole cost of adding Epic, GOG, Xbox,
Battle.net, or an emulator ROM directory. The discovery driver, the tiers, the
account, and the entry model do not change. `FixtureSource` in the tests
demonstrates this: a new source is added to a run with no other edit.

## Descent contract (tiers 2/3)

A directory walk MUST test each directory through a `DirectoryClassifier` and stop
descending on a `Hit`, emitting one candidate for that directory. It MUST NOT
enumerate a directory's executables first and then ask whether each is a game
(FR-009, FR-015). The classifier itself is a seam; the signature matcher lands in
S053.
