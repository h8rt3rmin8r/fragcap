# Contract: CLI Action Results

**Applies to**: S178 command-specific behavior for `bundle collect/cleanup/export`, `targets export/show/remove/discover/scan`, and `catalog seed`. Global JSON/silent semantics remain owned by #488.

| Command case | Exit | Human result | Structured or file effect |
| --- | --- | --- | --- |
| Bundle collect, complete with eligible content | 0 preview | State deletion scope and exact Apply command | Existing proposal and eligibility JSON remain available; no deletion on preview |
| Bundle collect, missing/unowned/incomplete/zero work | 0 preview | Concrete reason and read-only next step, no Apply command | Existing proposal may remain; JSON has equivalent reason/detail; no deletion |
| Bundle cleanup/export, missing path | Nonzero | Name supplied path and suitable read-only correction | No cleanup, share copy, or `--yes` recommendation |
| Targets export, positional plus `--id` | 2 | Name mutually exclusive selectors | No target JSON or store change |
| Targets export, no selector or one selector | Existing result | Export-all or exact selected target | Existing JSON array and no-match semantics remain |
| Catalog seed, invalid source or explicit launch | Nonzero | Name bad source/tier and accepted next action | Fresh destination remains absent |
| Catalog seed, duplicate supported tier | 0 | One fill and count of one | One tier seeded; no duplicate effect |
| Discover first run | Existing result | State that no candidate is registered but eligibility may be stored | Local store may be created/updated for eligibility |
| Scan first run | Existing result | State registration/update of candidate | Local target store reflects registered findings |

Collection human fields continue using the shared measured display layout. A `detail` value is shown only when the existing inspection provides it; it must not invent a path, owner, or capability. Seed messages may mention only flags present in the active build's parser.
