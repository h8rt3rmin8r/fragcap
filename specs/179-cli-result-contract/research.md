# Research: Consistent CLI Result Output

## Existing boundaries

- `run_with_authorization` selects a shared stderr `Emitter` for progress, warnings, errors, and Capture-family events. Non-capture handlers receive stdout directly; Doctor, Bundle, Fresh Start, and Steam already receive a JSON selection, while Targets, Catalog, Schema, and Technologies do not.
- The `Emitter` already keeps JSON lifecycle events under `--json --silent`, preserves errors, and suppresses optional warnings. Its channel ownership is authoritative for Capture, Deep Capture, calibration, and extcap diagnostics.
- Targets and Catalog have domain values before writing their human result; their JSON branches can serialize those values without scraping formatted tables. Exports and schema print already produce authoritative JSON documents.
- `SchemaDiagnostics` retains stable `code`, `pointer`, and `message` for every structural defect. It has no source coordinates, so structural records must state unavailable coordinates rather than infer them. A malformed JSON parse can obtain actual line and column from `serde_json::Error`.

## Decisions

1. **Machine precedence**: `--json` chooses the machine stream even with `--silent`; human silent suppresses ordinary successful prose and optional warnings. This preserves explicit automation output, terminal state, and required consent while correcting the misleading global help. Alternative rejected: make silent erase JSON successes and Capture events, producing an exit with no machine result.
2. **Domain projection**: Add a small shared result writer for one JSON line and command-local typed projections for missing handlers. Preserve raw JSON exports, embedded schema, build identity, and extcap protocol bytes. Alternative rejected: serialize the human renderer into a JSON string, which would still require text scraping.
3. **Failure shape**: Keep exit 0/1/2 and stderr `Emitter` error records. Schema validation is a special multi-diagnostic stdout result with one record per defect and a summary; avoid additionally flattening those diagnostics into a single generic error. Alternative rejected: continue `Diagnostics::to_string()` as the JSON error message.
4. **Missing source positions**: Structural schema defects carry `line:null` and `column:null`; malformed JSON uses parser coordinates when available. Alternative rejected: guessed positions from pointer searches, which would be wrong with duplicate keys or escapes.
5. **Human silence**: Gate ordinary stdout result rendering in the command path, preserving explicit prompts, required action reports, and machine artifacts. For existing command-specific JSON implementations, only add the missing human silent gate. Alternative rejected: globally replace stdout with a sink, which would swallow interactive prompts and protocol output.
6. **Surface coverage**: Audit every top-level command and record its machine protocol or result shape in one contract matrix. Any truly unsupported combination must refuse before effects; existing machine modes remain supported. Alternative rejected: blanket `--json` rejection for entire command families.
7. **Storage and security**: No schema migration, new dependency, target process right, capture payload transform, or new secret field. Structured output exposes only facts already selected for display or export. Alternative rejected: include raw store rows or capture evidence in generic result envelopes.

## Verification strategy

- Write a controlled command/flag matrix before handler changes, using temporary stores and injected writers. Assert parseable JSON lines, channel ownership, exact counters, quiet/silent behavior, empty and failure paths, and existing Capture/Doctor protocol identities.
- Run targeted CLI tests, the repository CI parity gate, text hygiene, then hosted checks on the final PR head. Live game and owner bundle observations are outside this slice.
