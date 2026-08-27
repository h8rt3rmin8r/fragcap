# Research: Steam Non-Game Filter

## Decision 1: Exclude only observed non-capturable app types by appinfo type

**Decision**: Treat `Music`, `Tool`, `Application`, `Config`, and `Video` as non-capturable Steam app types in target discovery.

**Rationale**: `Music` was already settled by S066. Issue #212 observed Steamworks Common Redistributables, a Steam `Tool`, being offered as a ready target. `Tool`, `Application`, `Config`, and `Video` are Steam app classes that do not name a playable game client for fragcap to capture. Counting them as not-a-game preserves the discovery account without adding a new bucket.

**Alternatives considered**:

- Exclude only `Tool`: fixes the observed redistributable but leaves the same defect for other non-game app classes.
- Exclude every non-`Game` type: too broad, because `Demo` can be playable and capturable.
- Name or folder denylist: less reliable than appinfo type when the type is present, and likely to create false negatives for games with installer-like words in their names.

## Decision 2: Keep `Demo` and unknown app types eligible

**Decision**: `Demo`, `Game`, and absent app types remain eligible for Steam discovery candidates.

**Rationale**: A demo can be a real playable title with network behavior. An absent type means the local appinfo observation is incomplete, often because appinfo is missing or unreadable. P-9 requires fragcap to report what it observed, not turn absence into an unstated non-game claim.

**Alternatives considered**:

- Filter unknown app types by name: possible future fallback, but out of scope because it would need its own false-positive and false-negative policy.
- Treat absent type as non-game: silently loses possible games and breaks existing fixture expectations.

## Decision 3: Keep the logic in the discovery adapter

**Decision**: Implement a small predicate in `crates/fragcap/src/discovery.rs` and test it through `SteamSource`.

**Rationale**: `fragcap-steam` is the low-level library walker. It should continue to enumerate installed Steam records and expose local appinfo facts. Whether an installed record is a capture target is the discovery seam's job, where candidates and conservation accounting live.

**Alternatives considered**:

- Filter in `fragcap-steam::discover_in`: would hide installed records from other Steam consumers and move target-discovery accounting out of the discovery seam.
- Filter in CLI rendering: too late, because the bad candidate could still register through other target paths.
