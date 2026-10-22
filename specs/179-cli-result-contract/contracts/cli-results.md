# Contract: CLI Results and Global Flags

## Flag precedence

| Selection | Ordinary stdout result | Optional stderr progress and warnings | Required actions and errors | Selected JSON stream |
| --- | --- | --- | --- | --- |
| default | human | shown | shown | absent |
| `--quiet` | human | progress hidden; warnings shown | shown | absent |
| `--silent` | hidden for success | hidden | shown | absent |
| `--json` | structured or existing machine artifact | human text hidden; structured warnings shown | structured where protocol permits | shown |
| `--json --silent` | structured or existing machine artifact | optional warnings hidden | structured where protocol permits | shown |

## Channels

| Family | Machine result under `--json` | Channel | Compatibility requirement |
| --- | --- | --- | --- |
| Targets | Typed list, selection, discovery, registration, import, remove, or reconciliation result; export remains target-entry JSON array | stdout | Stable id and handle, exact no-match and ambiguous outcome |
| Catalog | Typed import and per-tier seed result; export remains catalog JSON document | stdout | Counts and skip reasons reflect work actually performed |
| Schema | `diagnostic` lines plus terminal `summary`; print remains embedded JSON Schema | stdout | Separate code, path, line, column, message; exit 2 for invalid |
| Technologies | Typed findings and coverage facts | stdout | No absent finding invented for incomplete scan |
| Doctor | Existing check records | stdout | JSON retains every check; human silent retains only blocking checks and readiness when the report fails |
| Bundle, Fresh Start, Steam | Bundle collect/access keep their structured records; cleanup/export use typed action records; Fresh Start and Steam keep their structured records | stdout | Existing authorization and effect authority retained |
| Capture, Deep Capture, calibration | Existing lifecycle event stream | stderr | No sink-byte contamination; terminal state retained |
| Extcap, build identity, export and schema print | Existing protocol or JSON document | current protocol channel | No extra envelope or prose |

## Common records

A non-capture result record uses `event` as a stable command-specific discriminator. A structured error on stderr uses the existing emitter's `event:error` shape. Schema validation instead emits all `diagnostic` records and one `summary` on stdout, with the exit class indicating validity.

| Event | Required domain fields |
| --- | --- |
| `targets.list` | `count`, `targets[]` with `stable_id`, `handle`, `name`, classification, fidelity, and available evidence; `ready_count` for a populated store |
| `targets.show` | `status` (`resolved`, `no_match`, or `ambiguous`); resolved `target` and compatibility facts, or explicit selection/matches |
| `targets.add`, `targets.remove`, `targets.import`, `targets.scan`, `targets.discover`, `targets.reconcile.preview`, `targets.reconcile.apply` | Exact action status, observed counts and identities, with scan or discovery omissions when present |
| `catalog.import` | `store`, `imported` |
| `catalog.seed.tier` | `tier`, `status`; a filled tier has fetched/written/excluded/duplicates/failed, a skipped tier has `reason` |
| `catalog.seed.summary` | `store`, `tiers_filled` |
| `schema.diagnostic`, `schema.summary` | Separate defect code/path/line/column/message; terminal validity and count |
| `technologies.detect` | Source path and catalog store, finding array, coverage warnings, inert and skipped signature counts |
| `extcap.install`, `extcap.uninstall` | `status`, exact destination `path` |
| `bundle.cleanup.artifact`, `bundle.cleanup.summary`, `bundle.export` | Per-artifact cleanup status/path/reason and terminal artifact count/completion, or exported manifest path |

```json
{"event":"schema.diagnostic","code":"wrong-type","path":"/name","line":null,"column":null,"message":"expected string"}
{"event":"schema.summary","valid":false,"diagnostics":1}
```

## Atomicity

Writers serialize one complete JSON value before writing a newline. A failure to write a result is not treated as a successful command. Validation and source-preflight errors do not create a destination solely to report a result.
