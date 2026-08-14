# Contract: doctor output (human and machine-readable)

`fragcap doctor` produces a human report by default and a machine-readable stream
under `--json`. This slice extends both. The contract below is what downstream
consumers (a user reading the report, a script parsing `--json`, the golden
tests) may rely on.

## Section order (both forms)

1. `Identity` (new): version, binary path, profile dir, hint-db path.
2. `Platform`: os, subsystem, privilege.
3. `Capture driver`: npcap, loopback adapter, winpcap api mode, live backend,
   socket-table backend.
4. `Tracing`: process events.
5. `Interfaces`: adapters (now real) or the one warning row when empty.
6. `Integration`: analyzer extcap.
7. `Profiles`: profiles.

## Human form (`render_human`, plain)

- Each section is preceded by a blank line except the first.
- Each check renders as `  <name padded> <status> <detail>`; the status column
  width is preserved.
- Detail or remediation longer than 80 columns wraps to an indented continuation
  aligned under the detail column; no default-content line exceeds 80 columns.
- The final line is the readiness verdict (unchanged wording), and the exit
  status is unchanged: non-zero when any check is a blocking failure.
- This function emits no color control codes. It is the byte-exact golden
  subject.

## Human form (terminal presentation, `commands/doctor.rs`)

- When stdout is an interactive terminal and `NO_COLOR` is unset, the presentation
  layer colors each status word by severity (ok green, warn yellow, skip dim,
  fail red) and bolds section headings, wrapping the plain `render_human` output.
- When stdout is not a terminal, or `NO_COLOR` is set, output is exactly the plain
  `render_human` bytes.

## Machine-readable form (`render_json`)

- One JSON object per line, one line per check, in the section order above.
- Each object carries at least `section`, `name`, `detail`, `status`, and
  `remediation` when present. The identity facts are ordinary check records in
  the `Identity` section, not a separate object.
- Never colorized.
- `lines == report.checks.len()` continues to hold
  (`the_json_form_is_one_record_per_check`).

## Invariants

- Exit status distinguishes ready (0) from a blocking problem (non-zero),
  unchanged by identity and presentation additions.
- Identity and loopback-undetermined rows are non-blocking.
