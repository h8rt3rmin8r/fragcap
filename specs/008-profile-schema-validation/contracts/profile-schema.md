# Contract: Profile Schema Version 1

**Slice**: S05 | **Date**: 2026-08-09 | **Spec**: [../spec.md](../spec.md)

The TOML surface of a schema version 1 profile, and the library surface that
reads one. This is the contract a profile author writes against and the one S12
and S14 consume.

Specification section 15.2 is the authority. Where this document is more
specific, it is filling in what section 15.2 left to the implementation; where
it would contradict section 15.2, section 15.2 wins and this document is the
defect.

## The file

```toml
schema = 1

[game]
id        = "eso"                        # required, slug
name      = "The Elder Scrolls Online"   # required
platform  = "steam"                      # optional
app_id    = "306130"                     # optional

[capture]                                # optional table
mode      = "file"                       # file | stream | ring
duration  = "30m"                         # duration literal
roles     = ["launcher", "client"]        # declared role names
loopback  = true
payload   = true

[[stage]]                                # one or more
role      = "launcher"                   # required, unique
lifecycle = "transient"                  # transient | session | service
terminal  = false                        # optional, default false
match     = { exe = "*Launcher.exe" }    # required, one or more predicates
```

## Value forms

Every TOML value form schema version 1 can contain is accepted: basic and
literal strings, single-line and multi-line, integers, booleans, arrays, inline
tables, dotted and quoted keys, and arrays of tables.

A Windows path is written as a literal string so its backslashes need no
doubling, and it is stored exactly as written:

```toml
match = { path_contains = 'C:\Program Files\Zenimax Online' }
```

**One known divergence.** TOML datetimes (`1979-05-27T07:32:00Z` and the local
date and time forms) are refused by the parser this crate uses, as a syntax
fault. No key in schema version 1 has a datetime type, so a datetime can appear
only in a profile that is already invalid, and the effect is on the message
rather than on the verdict: a syntax diagnostic rather than a `WrongType`
diagnostic located at the key. Research R-1 records the measurement and why the
alternative parser is unavailable at this workspace's minimum toolchain.

## Accepted keys

Every table is closed. A key outside its accepted set is an `UnknownKey`
diagnostic naming the key and the set, because ignoring it would silently give
the author a profile narrower or wider than the one they wrote.

| Table | Accepted keys |
| --- | --- |
| top level | `schema`, `game`, `capture`, `stage` |
| `game` | `id`, `name`, `platform`, `app_id` |
| `capture` | `mode`, `duration`, `roles`, `loopback`, `payload` |
| `stage` | `role`, `lifecycle`, `terminal`, `match` |
| `stage.match` | `exe`, `path_contains`, `path_regex`, `cmdline_contains`, `descends_from` |

## Types and domains

| Key | Type | Domain |
| --- | --- | --- |
| `schema` | integer | Exactly `1`. Any other integer is `UnsupportedSchema` and suppresses the semantic diagnostics. |
| `game.id` | string | Non-empty; lowercase ASCII alphanumerics, `-`, `_`. |
| `game.name` | string | Non-empty. |
| `game.platform` | string | Unconstrained in this schema version. |
| `game.app_id` | string | Unconstrained. A platform identifier is a string even when it looks numeric, because Steam's are and leading zeros matter. |
| `capture.mode` | string | `file`, `stream`, `ring`. |
| `capture.duration` | string | Duration literal, below. |
| `capture.roles` | array of string | Non-empty; every entry declared by a stage. |
| `capture.loopback` | boolean | |
| `capture.payload` | boolean | |
| `stage.role` | string | Non-empty, unique within the profile. |
| `stage.lifecycle` | string | `transient`, `session`, `service`. |
| `stage.terminal` | boolean | At most one stage true; that stage's lifecycle must be `session`. |
| `stage.match` | table | At least one predicate. |
| `match.exe` | string | Image name glob, below. Non-empty. |
| `match.path_contains` | string | Substring, case-insensitive at evaluation. |
| `match.path_regex` | string | Must compile. |
| `match.cmdline_contains` | string | Substring. |
| `match.descends_from` | string | A role declared in the same profile. |

## Duration literal

```text
duration := integer unit
integer  := 1*DIGIT           ; no sign, no fraction, no separator
unit     := "ms" / "s" / "m" / "h"
```

Accepted: `500ms`, `30s`, `30m`, `2h`.

Refused: `30` (no unit), `30 m` (whitespace), `1.5h` (fraction), `-5m` (sign),
`0s` (zero), `1h30m` (compound, not in this schema version), `30d` (unknown
unit), and any value that overflows the duration representation.

Widening this grammar later keeps every profile written against schema 1 valid,
which is why the narrow form is the one that ships first.

## Image name glob

```text
*   matches any run of characters, including none
?   matches exactly one character
```

Every other character is a literal. There is no escape sequence, because Windows
forbids `*` and `?` in a file name, so an occurrence always means the wildcard.
Comparison is case-insensitive, per section 10.3. The pattern is stored as the
author wrote it; case folding happens on copies at comparison time.

Every non-empty string is a well-formed pattern. The empty pattern is refused
because it matches only the empty image name.

## The ambiguity rule

Validation refuses a profile when two stages could bind to the same process
because they can match a common image name and at least one of them has nothing
else to distinguish it.

Precisely: for every unordered pair of stages whose `exe` patterns have a
non-empty intersection, the pair is refused unless both stages carry at least
one predicate other than `exe`.

```toml
# Refused: both stages rely on exe alone and both can match ESOLauncher.exe.
[[stage]]
role  = "a"
match = { exe = "*Launcher.exe" }
[[stage]]
role  = "b"
match = { exe = "ESOLauncher.exe" }
```

```toml
# Accepted: the shared image name is pinned by ancestry on both sides.
# This is the section 15.2 profile for the second focal title.
[[stage]]
role  = "client"
match = { exe = "TheDivision2.exe", descends_from = "anticheat" }
[[stage]]
role  = "helper"
match = { exe = "TheDivision2.exe", path_contains = "helper" }
```

The diagnostic names both roles and states that a further predicate resolves it.
Section 15.4's other half, the runtime warning when a stage matching on `exe`
alone binds a process that transmits nothing, belongs to S12.

## Resolution

Section 15.3's order, first match winning:

1. `reference` names an existing regular file.
2. `<reference>.toml` in a command line profile directory, in the order given.
3. `<reference>.toml` in the user profile directory.
4. A bundled profile whose `game.id` equals `reference`.

Steps 2 through 4 require `reference` to be a valid slug and refuse it before
any path is joined. Step 1 does not, because an operator naming a path has named
a file.

A search directory that is absent or unreadable is skipped. A candidate file
that is present and unreadable is an error, not a skip: it has already won its
step, and falling through would silently substitute a different profile.

The search locations are supplied by the caller. This crate consults no
environment variable and no platform configuration location.

## Library surface

```rust
// Pure, no filesystem. Tier 0.
impl Profile {
    pub fn parse(text: &str) -> Result<Profile, Diagnostics>;
}

// Reads with the size limit applied before the contents.
pub fn load(path: &Path) -> Result<Profile, LoadError>;

// Section 15.3.
pub fn resolve(
    reference: &str,
    search: &SearchPath,
    bundled: &BundledSet,
) -> Result<Resolved, ResolveError>;
```

Guarantees the surface makes:

- A `Profile` exists only if every check passed. There is no public constructor,
  no public field mutation, and no `Default`.
- A failed parse yields a non-empty, deterministically ordered `Diagnostics`.
- A successful resolution yields the profile and which of the four steps
  supplied it.
- Nothing in this crate opens a process handle, enumerates processes, or reads a
  socket table. A profile describes processes; reading the description observes
  nothing.

## Stability

The `DiagnosticCode` enumeration is stable surface: adding a variant is a
visible change, and S14's output and the tests both key on it.

A diagnostic's `location` string and `message` are not stable surface. They
exist to point an author at a line in their own file, and committing to their
shape would freeze a formatting choice for a consumer that does not exist.

The schema version is what makes the file format growable. A profile declaring a
version this build does not support gets one diagnostic naming the supported
version, never a list of complaints produced by reading it under the wrong
rules.
