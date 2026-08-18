# Contract: The corrected `fragcap doctor` report

The contract the getting-started sample mirrors after the companion code change
(FR-017, FR-018, FR-019). Only the profile surface changes; every other row and
the exit-status semantics are unchanged.

## Identity section (changed)

Before: `version`, `binary`, `profile dir`, `catalog db`, `local db`.

After: `version`, `binary`, `catalog db`, `local db`. The `profile dir` row is
removed.

## `Profiles` section (removed)

Before: a `Profiles` section with one row `profiles  ok  bundled: N, user: M`.

After: the section does not exist. The `PROFILES` section constant, the
`profiles(inputs)` check, and its `push` in the assembly are removed.

## Sections that remain (unchanged, in order)

`Identity`, `Platform`, `Capture driver`, `Tracing`, `Interfaces`, `Integration`,
and the S056 preparation rows (`catalog store` / `target entries`, surfaced only
when absent). The closing `Ready to capture.` line is unchanged.

## Invariants

- Exit status is unchanged for every input (the removed rows were all `ok`,
  informational, and never affected exit status).
- `--json` output loses the `profile dir` record and the `profiles` record and
  nothing else.
- The `Inputs` struct loses `profile_dir`, `bundled_count`, `user_count`; no other
  field changes. The probe stops computing them; no other probe output changes.
- The internal `Profile` / `BundledSet` / `SearchPath` types are NOT removed
  (capture still synthesizes a one-stage profile). Only doctor's reporting of a
  profile directory and profile counts is removed.

## Test expectations

- `checks.rs` tests asserting the identity row list are updated to
  `version, binary, catalog db, local db`.
- Any test constructing `Inputs` drops the three removed fields.
- A test asserts the report contains no section named `Profiles` and no row
  labeled `profile dir` (guards against reintroduction).
