# Data Model: Data-driven detection signatures

Derived from the spec's Key Entities and the Phase 0 decisions. Types are named for
orientation; exact Rust identifiers are settled in implementation.

## Signature

One detection rule. A row in the `signature` table in `catalog.db`.

| Field | Type | Notes |
| --- | --- | --- |
| `id` | integer, primary key | Row identity. |
| `category` | text | `engine` \| `anti-cheat` \| `drm` (D6). |
| `kind` | text | `filename` \| `directory-shape` \| `pe-version-string` \| `binary-marker`. First three implemented this slice; `binary-marker` is carried but inert (D3). |
| `pattern` | text, non-empty | The match pattern, interpreted per `kind` (see below). |
| `product` | text, non-empty | The product named, for example `Unity`, `Easy Anti-Cheat`, `Denuvo`. |
| `confidence` | text | One of `definitive` or `heuristic` (a small closed enum, CHECK-constrained). Drives the fidelity a match stamps (D4). |

### Confidence to fidelity

- `definitive` (a named, engine-specific marker such as `UnityPlayer.dll`,
  `GameAssembly.dll`, `vgk.sys`) maps a match to `FidelityTier::Verified`. This is
  the case FR-008 names: a locally detected engine from a definitive marker is
  `verified`.
- `heuristic` (a generic shape that a non-target could also present, such as a lone
  `*.pak`) maps a match to `FidelityTier::HeuristicUnverified`, so a weak local
  signal does not over-claim. It still outranks nothing remote by itself; the
  local-outranks-remote rule (FR-009) applies to `verified` local engine matches.

Every Appendix B engine marker in the seed is `definitive` except where the signal
is a bare container extension; the seed author sets the value per row.

### Pattern interpretation by kind

- **filename**: matches when a file whose name (or relative path) matches `pattern`
  exists in the scanned directory subtree. Example: `UnityPlayer.dll`,
  `EasyAntiCheat*.dll`, `vgk.sys`.
- **directory-shape**: matches on the presence and arrangement of directories and
  files by name, for example a `*_Data/` directory beside the executable, an
  `Engine/Binaries/` tree, or a `bin/win64/` directory with `steam_api64.dll`
  adjacent. A directory-shape match is the one that stops descent in the classifier
  (FR-007).
- **pe-version-string**: matches `pattern` against a string field of a candidate
  binary's PE version resource (`CompanyName`, `ProductName`, `FileDescription`,
  and similar). Requires reading the binary's on-disk bytes (D3), never process
  memory.
- **binary-marker** (inert this slice): would match a byte or section marker inside
  an executable. Seeded for Denuvo, Arxan, VMProtect but not applied; counted and
  surfaced as not-yet-matchable (D3, FR-013).

### Validation

- `pattern` and `product` are non-empty; `category` and `kind` are within their
  enumerations (a CHECK constraint plus a load-time guard).
- A malformed filename/directory-shape pattern (one the match engine rejects) is
  rejected at load with a surfaced diagnostic and does not disable the rest of the
  table (P-4).

## SignatureSet (loaded)

The in-memory product of loading the table: the applied signatures (implemented
kinds with valid patterns) plus the accounting of what was not applied.

| Field | Meaning |
| --- | --- |
| applied | Signatures of an implemented kind that compiled. |
| inert | Signatures of an unimplemented kind (binary-marker), counted and surfaced. |
| skipped | Signatures rejected at load (malformed pattern), each naming its product. |

Invariant: `applied + inert + skipped == total loaded`. Surfaced so reduced
coverage is visible, mirroring the existing `compiled + skipped == total` invariant
the detector already carries.

## DetectionFinding

The result of a signature matching during a scan.

| Field | Meaning |
| --- | --- |
| `category` | The matched signature's category. |
| `product` | The product named. |
| `evidence` | The matched relative path or version-string field: the auditable marker. |
| `fidelity` | Derived from the signature's confidence (D4): `verified` for a definitive local marker. |

Neutral by construction (D9): a finding carries no status, risk, gate, or color
value. Nothing about a finding characterizes a title as off limits.

## Detected engine attribution

For the classifier path, the engine a directory's shape identifies, attached to the
emitted candidate at `verified` fidelity. Distinct from and outranking any remote
catalog engine attribution (`heuristic-unverified`) for the same candidate (FR-008,
FR-009). When both exist, the local `verified` value is presented.

## ClassifierVerdict (S052 seam, now real)

The S052 `DirectoryClassifier::classify` decision, unchanged in shape:

- `Hit { classification }`: the directory matched an engine signature; emit one
  candidate, stamp the detected engine `verified`, stop descent.
- `Miss`: no engine signature matched; the directory is considered-not-a-game in the
  S052 discovery account.

Non-engine findings (anti-cheat, DRM) matched on a `Hit` directory are recorded as
additional evidence on the candidate; they do not change the verdict.

## Discovery account (S052, extended surfacing)

The per-directory account outcomes are unchanged (produced, parse_failed,
declined_by_user, considered_not_a_game, volume_skipped, access_error) and stay
conserved (P-4). S053 adds set-level surfacing that is a property of the loaded
`SignatureSet`, not a per-directory outcome: the inert count and the skipped count
are reported once so a not-yet-matchable kind or a malformed row is never silently
absent.

## Schema change

Additive migration `MIGRATE_4_TO_5`, applied transactionally, bumping the shared
schema `SCHEMA_VERSION` from 4 to 5. Creates the `signature` table with the columns
above and CHECK constraints on `category` and `kind`. `catalog.db` populates it via
the seed; `local.db` leaves it empty (D5). The migration chain stays sequential
(v1->2->3->4->5).

## Seed document

`fragcap-targets/assets/signatures.json`: the bundled Appendix B signature set, one
entry per (product, signal), loaded by `seed_signatures` into the table. All 16
Appendix B products are represented (SC-001); the three content-marker-only DRM
products carry `binary-marker` rows that seed but stay inert.
