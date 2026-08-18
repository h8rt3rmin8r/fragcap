# Contract: Observe-mode profile synthesis

## Input

A resolved `TargetEntry` whose launch chain is unresolved
(`launch_is_unresolved(&entry) == true`) and that is **not** Steam-anchored, plus
its observed executable `E = observed_executable(&entry)`.

## Output

A validated `Profile` with two stages:

```json
{
  "schema": 1,
  "kind": "profile",
  "fidelity": "heuristic-unverified",
  "game": { "id": "observe", "name": "launch-and-observe" },
  "stage": [
    { "role": "launcher", "lifecycle": "session", "match": { "exe": "<E>" } },
    { "role": "client", "lifecycle": "session", "terminal": true,
      "match": { "descends_from": "launcher" } }
  ]
}
```

## Guarantees

- **Valid**: passes `Profile::parse` (both stages carry a non-empty predicate; the
  client stage carries no `exe`, so `ambiguous_image_match` does not fire).
- **Binds the holder**: the launcher stage binds `E`; the terminal client stage binds
  any process descending from a `launcher`-bound process. So the socket holder is
  attributed whether it is `E` itself (attributed at the launcher stage) or a child
  (attributed at the terminal client stage).
- **Fidelity heuristic-unverified**: the identity was synthesized, not typed by an
  operator; the profile fidelity is not `authored`. (The stored target's fidelity is
  a separate axis, raised to `verified` on promotion.)
- **No wildcard**: never emits an empty-predicate stage.

## Refusals (unchanged from before this slice)

- An unresolved entry with no observed executable (`observed_executable` is `None`) is
  refused as before ("names no Windows client executable ..."): there is nothing to
  observe from.
- A Steam-anchored entry (even if unresolved) is resolved through the install-layout
  cascade, not this branch.
