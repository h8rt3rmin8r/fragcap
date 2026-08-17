# Phase 1 Data Model: The target entry model

The authoritative store is `local.db`. Types below live in `fragcap-targets`
(`entry.rs`, `handle.rs`, `identifier.rs`, `selector.rs`) and are persisted by
`store.rs` under schema version 3. DDL is illustrative; the committed DDL lives in
`schema.rs`.

## Entity: TargetEntry

One capture target. One row in the `targets` table.

| Field | Type | Rules |
| --- | --- | --- |
| `id` | INTEGER PK | Row identity, autoincrement. Also the durable target key inside a store. |
| `stable_id` | INTEGER UNIQUE | The 63-bit stable identifier (D-1). Anchored: BLAKE3(canonical anchor) low 63 bits. Unanchored: random 63-bit. Selected by `--id`. |
| `handle` | TEXT UNIQUE | Normalized slug (FR-004). `UNIQUE`; CHECK not purely numeric; collision auto-increments the new item. |
| `name` | TEXT | Display name; source of the derived handle. Non-empty. |
| `classification` | TEXT | Enum {game, launcher, tool, mod, emulator, unknown}. CHECK-constrained. `unknown` is first-class (P-9). |
| `classification_source` | TEXT | Enum {catalog, engine-signature, platform, user, unset}. CHECK-constrained. |
| `fidelity` | TEXT | Enum {authored, verified, heuristic-unverified, observed}. CHECK-constrained. The resolver's ordering key. |
| `provenance` | TEXT (JSON) | How this entry was produced (source, timestamp). Carried whole. |
| `anchor` | TEXT NULL | Canonical anchor string (`steam:<appid>`, `epic:<catalogItemId>`, `gog:<productId>`) or NULL. Sole input to an anchored `stable_id`. |
| `launch_entries` | TEXT (JSON) | The launch entries, carried whole (not decomposed into the existing `launch_entries` catalog table). |
| `install_root` | TEXT NULL | Filesystem root or NULL. Never an input to `stable_id` (FR-011). |
| `evidence` | TEXT (JSON) | Supporting facts (matched signatures, observed processes). Carried whole. |

Relationships: `TargetEntry` is self-contained (launch entries and evidence are
carried as JSON, not foreign-keyed to the catalog `games` tables), so a target
row is meaningful in `local.db` independent of any `catalog.db` row. A
`target_id_aliases` row may reference a `TargetEntry` by `id`.

### Illustrative DDL (schema v3, added by MIGRATE_2_TO_3)

```sql
CREATE TABLE targets (
    id                    INTEGER PRIMARY KEY,
    stable_id             INTEGER NOT NULL UNIQUE,
    handle                TEXT NOT NULL UNIQUE
                            CHECK (length(handle) > 0 AND handle GLOB '*[^0-9]*'),
    name                  TEXT NOT NULL CHECK (length(name) > 0),
    classification        TEXT NOT NULL CHECK (classification IN
                            ('game','launcher','tool','mod','emulator','unknown')),
    classification_source TEXT NOT NULL CHECK (classification_source IN
                            ('catalog','engine-signature','platform','user','unset')),
    fidelity              TEXT NOT NULL CHECK (fidelity IN
                            ('authored','verified','heuristic-unverified','observed')),
    provenance            TEXT,
    anchor                TEXT,
    launch_entries        TEXT,
    install_root          TEXT,
    evidence              TEXT
);

CREATE TABLE target_id_aliases (
    alias_stable_id INTEGER PRIMARY KEY,   -- a superseded 63-bit id
    target_id       INTEGER NOT NULL REFERENCES targets(id) ON DELETE CASCADE
);
```

The `handle GLOB '*[^0-9]*'` CHECK makes a purely numeric handle unstorable at the
layer below the Rust path (FR-006 defense in depth). `stable_id` is a signed
64-bit SQLite integer with the sign bit clear (63-bit value), so it is always
non-negative and comparisons are well defined.

## Enum: Classification

`game | launcher | tool | mod | emulator | unknown`. `unknown` is a real, frequent
state; the resolver and UI treat it as data, not as a missing value (P-9,
FR-002).

## Enum: ClassificationSource

`catalog | engine-signature | platform | user | unset`. Records what assigned the
classification, so a later, higher-authority source can overwrite a lower one
without guessing.

## Enum: Fidelity (ordered)

`authored > verified > heuristic-unverified > observed`. The ordering is the
resolver's selection key (FR-020). It is CHECK-constrained in storage and
represented as an ordered Rust enum so `>` is a type-level fact, not a string
comparison.

## Value: Handle

Derived by the FR-004 algorithm, applied in exactly this order:

1. strip Unicode `So`, `Sk`, `Cf`
2. NFKD
3. strip `Mn`
4. lowercase
5. delete apostrophes and quotation marks outright
6. replace each run outside `[a-z0-9]` with a single `_`
7. trim leading/trailing `_`
8. truncate to 64, then trim any trailing `_`

Constraints and fallback:
- `UNIQUE`; not purely numeric (FR-006).
- Empty/invalid -> executable stem -> `target_<n>` (FR-007). Terminates always.
- Collision -> append `_2`, `_3`, ... on the new item; existing entry untouched
  (FR-008).
- User override allowed under the same rules (FR-009).

## Value: Anchor and StableIdentifier

- **Anchor** canonical form: `<platform>:<platform-id>`, lowercase platform prefix.
  Steam is the only platform a source populates today; `epic:` and `gog:` forms
  are fixed now for forward compatibility (identifier stability must predate the
  second platform).
- **Anchored StableIdentifier** = low 63 bits of `BLAKE3(canonicalize(anchor))`,
  where canonicalization lowercases the platform prefix and trims whitespace.
  Deterministic; independent registrations of one anchor collide and merge
  (FR-010). Derived only from the anchor (FR-011).
- **Unanchored StableIdentifier** = random 63-bit value (FR-012). No bit is
  reserved; an entry is anchored iff its `anchor` column is non-null.

### State transition: unanchored -> anchored

```text
[unanchored entry]  stable_id = R (random 63-bit), anchor = NULL
        |
        |  later matched to anchor A
        v
[anchored entry]    stable_id = BLAKE3(canonicalize(A)) low 63 bits (active)
                    target_id_aliases += R   (superseded, never reissued)
```

An export or machine reference that captured `R` still resolves to the merged
entry via `target_id_aliases`; the active identifier is the anchored one
(FR-013). Import merges on the active identifier and consults aliases (FR-014).

## Value: Selector (resolution input)

| Form | Meaning | Durability |
| --- | --- | --- |
| bare integer `N` | row index over the current listing | ephemeral |
| token | exact `handle`, then case-insensitive exact `name` | durable (handle) |
| `--id N` | exact `stable_id` (or a superseded alias) | durable, machine-facing |

Resolution outcomes: exactly one match resolves; zero matches report no match; a
name matching more than one row lists the matches (handle + `stable_id`) and
exits 2 without resolving (FR-016, FR-017, FR-018).

## Validation rules (consolidated)

- Storage-layer CHECKs on `fidelity`, `classification`, `classification_source`,
  non-empty `name`/`handle`, and non-numeric `handle`.
- `handle` and `stable_id` `UNIQUE`.
- Handle derivation and fallback never error and never loop (FR-007).
- `stable_id` is anchor-only when anchored; never derived from name/handle/path.
- The four resolution declines (sparse, engine-only, launcher-mediated, multi-exe)
  remain declines, expressed as fidelity-aware query conditions (FR-021).
