# Contract: Repository Task Runner Command Surface

The task runner is the only interface this slice exposes. It is invoked through
the language toolchain and requires nothing else installed.

```text
cargo xtask <command>
```

## Commands

| Command | Status in S01 | Purpose |
| --- | --- | --- |
| `lint` | implemented | Repository conventions check |
| `deps` | implemented | Dependency direction check |
| `ci` | implemented | Run the full local check set in order |
| `msrv` | implemented | Build at the declared minimum toolchain |
| `neutral` | implemented | Build the core crate for a non-host target |
| `docs` | stub | Documentation site, owned by S18 |
| `publish` | stub | Registry publication in dependency order, owned by release |

A stub prints what it will do, states the slice that owns it, and exits with
the usage code. It does not exit zero, because a caller cannot distinguish a
successful no-op from a successful run.

## Exit codes

Follows the house convention in section 17.4.

| Code | Meaning |
| --- | --- |
| 0 | The check ran and passed |
| 1 | The check ran and failed |
| 2 | The check could not run: bad usage, missing prerequisite, unimplemented |

The distinction between 1 and 2 is load-bearing. "The conventions are violated"
and "the conventions could not be checked" are different facts, and collapsing
them lets a broken check masquerade as a clean repository.

## `cargo xtask lint`

Walks tracked files and asserts the rules in `CONVENTIONS.md`.

| Check | Rule |
| --- | --- |
| encoding | UTF-8, no byte order mark |
| line endings | LF only, no CR bytes |
| trailing whitespace | none on any line |
| final newline | exactly one |
| dashes | no em-dash or en-dash in any file |
| license identifier | present as the first line of every source file |

Binary files are skipped by content sniffing, not by extension. Vendored
directories are excluded by an explicit list, so the exclusion is visible.

**Output**: one line per violation, `path:line: rule: detail`. A summary count.
Exit 1 if any violation, 0 otherwise.

**Testable property**: given a fixture file violating exactly one rule, the
check reports exactly that rule. This is the test that distinguishes a working
linter from one whose matcher never fires.

## `cargo xtask deps`

Reads `cargo metadata` and compares the workspace edge set against the
expectation in `data-model.md`, encoded in one place in the source.

**Output**: unexpected edges and missing edges, each named. Exit 1 if the sets
differ.

**Testable property**: given a synthetic metadata document with an added edge,
the check reports that edge.

## `cargo xtask msrv`

Builds the workspace with the declared minimum supported toolchain.

**Reports its own vacuity.** While the workspace has no external dependencies,
the check prints that the result does not yet constrain anything and why. It
still exits 0 on success, because it did run and did pass; the caveat is in the
output where a reader sees it, not hidden in an exit code.

## `cargo xtask neutral`

Builds `fragcap-core` for a target that has no capture backend.

If the target is not installed, exits 2 with the `rustup` command that would
install it. It does not exit 0. A skipped check that reports success is the
defect this project's constitution names most often.

## `cargo xtask ci`

Runs, in order: `fmt --check`, `clippy -D warnings`, `test --locked`, `lint`,
`deps`. Stops at the first failure and propagates its exit code.

This is the command a contributor runs before opening a change, and it is the
same set the automated check set runs, so the two cannot drift.
