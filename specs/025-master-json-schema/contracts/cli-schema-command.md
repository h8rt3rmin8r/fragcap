# Contract: `fragcap schema` command surface

A new subcommand group `schema` with two subcommands. Thin wrappers over
`fragcap-profile` library functions (P-7); no output parsing.

## `fragcap schema validate <file>`

Validate any JSON file against the embedded master schema.

- **Input**: a path to a JSON file.
- **Behavior**: parse the file; if it is not syntactically valid JSON, report the
  syntax error (distinct from a schema violation) and exit non-zero. Otherwise
  determine the variant from `kind`, run structural validation, and report every
  violation found in one pass, each located by JSON pointer.
- **Exit code**: `0` when the file is valid; non-zero when any violation exists
  or the file cannot be read or parsed.
- **Output**: on success, a single confirmation line. On failure, one line per
  violation, each naming the JSON-pointer location and the problem; and, when the
  input is a hint or export missing fidelity or provenance, that specific refusal.
- **Determinism**: violation ordering is stable (document order by JSON pointer)
  so output is diffable across runs.

### Acceptance

| Given | Then |
| --- | --- |
| a file with four independent structural violations | exactly four violations reported, one run, exit non-zero |
| a valid file of any variant | no violations, exit zero |
| a file with an unknown key | that key reported as a violation |
| a hint with no `fidelity` | refused, naming the missing field |
| a syntactically broken JSON file | syntax error reported, distinguished from a schema violation, exit non-zero |
| a file whose `kind` is absent or unknown | reported as undetermined variant, exit non-zero |
| a file declaring an unsupported `schema` version | refused, naming the supported version |

## `fragcap schema print`

Emit the exact schema the binary enforces.

- **Input**: none.
- **Behavior**: write the embedded master JSON Schema document to stdout,
  byte-for-byte identical to the embedded asset.
- **Exit code**: `0`.
- **Use**: lets a contributor, an editor, or an agent obtain the canonical schema,
  and backs the mechanical drift check that the embedded, published-repo, and
  docs-site copies agree.

### Acceptance

| Given | Then |
| --- | --- |
| a running binary | `schema print` emits the embedded schema exactly |
| the emitted schema vs the repository-published copy | identical; a drift is caught by an automated check |
