# Data Model: CLI Reference Gate

S093 adds no persisted runtime data model. The following values exist only while the documentation test runs.

## Command Contract

| Field | Meaning | Validation |
| --- | --- | --- |
| `path` | Root-relative public command path, such as `targets add` | Nonempty, unique, recursively derived from visible clap commands |
| `availability` | `default` or `net` command-tree variant | The active test variant selects matching reference rows |
| `options` | Locally owned public named options | Compared as an exact set for the owning path |

The root command has an empty path internally and owns the global-option contract. Every visible subcommand, including a parent with no local options, has one reference section.

## Option Contract

| Field | Meaning | Validation |
| --- | --- | --- |
| `long` | Canonical long flag without leading hyphens | Required and unique within the owning command |
| `short` | Optional one-character alias | Must agree with clap |
| `values` | Ordered or normalized finite value set | Must agree with clap possible values; open strings use the documented sentinel |
| `default` | Parser-declared default values | Must agree with clap; application fallbacks are explanatory prose, not parser defaults |
| `availability` | Variant in which the row exists | Must include the active feature tree and exclude inactive conditional rows |

Generated help and version controls are classified by clap action and excluded through the documented generated-control policy. Hidden arguments are excluded through clap visibility.

## Reference Section

| Field | Meaning | Validation |
| --- | --- | --- |
| `path` | Code-formatted command path in the heading | Exactly one section for every public path and no stale path |
| `heading_level` | Level that encodes top-level versus nested command | Must match the reference convention |
| `option_rows` | Human-visible table records | Parsed into option contracts for exact comparison |
| `source_line` | One-based location in `cli.mdx` | Included in malformed-contract diagnostics |

## Worked Invocation

| Field | Meaning | Validation |
| --- | --- | --- |
| `source_line` | First line of the invocation | Included in parser failures |
| `shell` | Fence language and continuation convention | Limited to forms used by the page |
| `text` | Logical invocation after continuation and comment handling | Must begin with executable `fragcap` |
| `argv` | Tokenized command arguments | Passed only to clap parsing, never dispatch |

## Sink Grammar Contract

| Field | Meaning | Validation |
| --- | --- | --- |
| `schemes` | Accepted canonical schemes and aliases | Extracted from sink parser match arms and exactly matched by the reference |
| `modifiers` | Accepted modifier keys | Extracted from parser match arms and exactly matched by the reference |
| `constraints` | Transport or platform interpretation | Human-visible contract checks require the relevant scheme associations |

## Validation Result

Failures are grouped into four categories: invalid reference contract, command-tree drift, sink-grammar drift, and invalid worked invocation. Each failure identifies the owning command or source line and shows the compared values when applicable.
