# Contract: exit-code consistency

Covers #68. The classes are the master-spec section 17.4 contract, unchanged:

| Code | Class |
| --- | --- |
| 0 | Success (including operator interrupt during capture; passed diagnostics) |
| 1 | Expected failure (target never appeared, driver/backend absent, not found, blocked doctor) |
| 2 | Usage or configuration error (bad arguments, invalid/malformed profile) |

## The fix

Verified against the resolver (research R1): the two subcommands call the same
`resolve(...)` and cannot disagree on identical input. The observed split is
between reference *shapes*: an absent id-slug yields `NotFound` (exit 1) while an
unresolvable path-shaped reference yields `InvalidReference` (exit 2). Both mean
"this reference names no resolvable profile" and MUST agree on **exit 1**.

The minimal change is one line in `crates/fragcap-cli/src/exit.rs`
(`From<ResolveError>`): reclassify `ResolveError::InvalidReference` from
`CliError::Usage` (2) to `CliError::Failure` (1). `Load { LoadError::Invalid }`
(a profile file that exists but fails validation) stays `Usage` (2). No change to
`commands/profile.rs`.

| Condition | Resolver variant | `show` today | `validate` today | Target (both) |
| --- | --- | --- | --- | --- |
| Absent id-slug (valid form, no match) | `NotFound` | 1 | 1 | 1 (unchanged) |
| Unresolvable path-shaped reference | `InvalidReference` | 2 | 2 | **1** |
| Profile file exists but invalid | `Load { Invalid }` | 2 | 2 | 2 (unchanged) |
| Profile found and valid | - | 0 | 0 | 0 (unchanged) |

Invariant the implementation must satisfy: any reference that resolves to no
profile is `Failure` (1); an invalid profile *file* remains `Usage` (2).

## `replay` not-implemented

`replay` currently exits 2 ("not yet implemented"). Section 17.4 places
"unsupported mode, sink, or command" in the usage/configuration class (2), so
`replay` staying at 2 is consistent and is left as-is; only its help copy loses
the internal slice id (#67). This is a deliberate no-change, documented so a
future reader does not "fix" it into 1.

## Documentation (FR-012)

The 0/1/2 contract and the per-condition classification above are stated in
operator-facing material (the section 17 update and/or the CLI help/README), so
the extcap and wrapper surfaces can rely on it.

## Acceptance

- An integration test runs `profile show` and `profile validate` against an
  absent id-slug and asserts both exit 1.
- A test runs both against an unresolvable path-shaped reference (e.g.
  `missing.toml`) and asserts both exit 1 (the reclassified `InvalidReference`).
- A test asserts a profile file that exists but fails validation exits 2 from
  `validate`.
- A test asserts a valid profile exits 0 from both.
