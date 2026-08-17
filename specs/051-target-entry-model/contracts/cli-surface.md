# Contract: CLI surface (selectors, retired and kept commands)

This is the user- and machine-facing command contract this slice changes. Exact
flag names and help text are settled in `tasks.md`/implementation; the behavior
below is the contract.

## Target selection

A command that acts on a target (capture, show, edit) accepts a selector:

| Input | Resolves as | Exit on failure |
| --- | --- | --- |
| `<token>` positional | exact `handle`, then case-insensitive exact `name` | 0 with "no match" if none; 2 if the name matches > 1 |
| `--id <N>` | exact `stable_id` or a superseded alias | 2 if no such id |
| bare integer `<N>` positional | ephemeral row index over the current listing | 2 if out of range |

Ambiguity contract (FR-017): when a name matches more than one target, the tool
writes each match with its `handle` and `stable_id` to output and exits **2**,
resolving nothing. It never picks one. Example shape:

```text
error: name "portal 2" matches 2 targets; select by handle or --id:
  portal_2     id=4611686018427387… (steam:620)
  portal_2_2   id=1234567890123456… (steam:200)
```

`--id` is the durable, machine-facing contract; a bare integer is documented as
ephemeral (row order may change between invocations).

## Retired surface (this slice)

- `--profile <path>` selector: removed. Targets are rows, not files.
- The AppData profile directory and its env override: removed
  (`paths.rs::user_profile_dir`, `search_path`, and the profile-dir env var).
- `profile validate` (and the `profile` command whose subject was profile files):
  removed. Any target listing it provided is served by the `targets` command.

A retired flag/subcommand is **unrecognized** (clap rejects it), not a silent
no-op, so a script still passing `--profile` fails loudly rather than capturing
the wrong thing.

## Kept surface

- `schema validate <file>`: unchanged. Validates a JSON target document against
  the published master schema; exit 0 conformant, exit 1 non-conformant, exit 2
  usage error. Lives under the separate `schema` command.
- `schema print`: unchanged.
- The two store-path overrides from S050: `--catalog-db` / `--local-db` (and
  `FRAGCAP_CATALOG_DB` / `FRAGCAP_LOCAL_DB`).

## Exit codes (unchanged conventions)

- `0` success (including a clean "no match").
- `1` expected negative outcome (e.g. schema non-conformance).
- `2` configuration/usage error (ambiguous selector, unknown id, bad flag).
