# Data Model: CLI Result Output

## Output policy

One parsed invocation selects `format` (`human` or `json`) and `verbosity` (`normal`, `quiet`, or `silent`). `quiet` suppresses optional progress. Human `silent` also suppresses optional warnings and ordinary success prose. JSON mode emits the selected machine result or lifecycle stream even when silent is set. Errors and required consent or action text retain their existing mandatory channel.

## Result record

Each non-capture machine result line is a JSON object with a stable `event` discriminator and typed command facts. A terminal `summary` or command-specific result names the actual outcome and count. A no-match or zero-row result is explicit, and a failed effect cannot be labeled complete. The writer appends exactly one newline after each complete record.

## Schema diagnostic

One `diagnostic` record carries `code`, `path`, `line`, `column`, and `message`. `path` is the JSON pointer or document root. Structural defects have null source coordinates because the validator has not observed those positions. Malformed syntax uses parser-observed coordinates where available. The terminal `summary` carries `valid` and `diagnostics` count. The command exits 2 when the count is nonzero.

## Existing machine artifact

Target and catalog exports are JSON documents, schema print is the embedded JSON Schema, build identity is an internal machine document, and extcap emits its analyzer protocol. The global JSON option does not wrap these outputs or append a second result record. Capture-family lifecycle events remain on stderr; Doctor's structured report stays on stdout.

## State and failure rules

- A command may emit progress or required authorization before an effect, but it emits a success result only after the effect or read completes and the result is known.
- A usage error has exit 2 and a runtime failure has exit 1; JSON diagnostics and generic errors remain parseable on their designated channels.
- A partial source or scan result names skipped, unavailable, or incomplete coverage instead of implying completion.
- Output writing failures do not convert a failed or incomplete record into a success claim.
