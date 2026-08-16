# Contracts: check and format surfaces (S049)

The interfaces this slice exposes are repository check commands and two document
formats. All exit codes follow the house 0/1/2 contract (specification 17.4): 0
ran and passed, 1 ran and failed, 2 could not run.

## `cargo xtask spec`

New subcommand. Two assertions, reported independently, aggregated to one exit
code.

| Assertion | Pass (0) | Fail (1) | Could not run (2) |
| --- | --- | --- | --- |
| Version lock-step | `Applies-To` equals the workspace version | they differ | `Applies-To` field absent/unparseable, or the manifest version unreadable |
| Fragment format | every `changelog.d` fragment carries a well-formed `spec-impact` line | one or more are missing or malformed | `changelog.d/` unreadable |

- On failure, both values are reported for the version mismatch, and each
  offending fragment is named for the format failure.
- Runnable standalone (`cargo run --package xtask -- spec`) and as part of the
  aggregate (`cargo xtask ci`).

## `cargo xtask ci` (extended)

The aggregate gains a `spec` step, in the same 0/1/2 propagation style as the
existing `lint`, `deps`, `license`, `wrappers`, and `docs check` steps. Placement
is after `docs check`. A `spec` failure fails `ci` with exit 1; a could-not-run
propagates 2.

## `.github/workflows/ci.yml` (extended)

The `check` job gains a step:

```yaml
- name: Specification version lock-step
  run: cargo run --package xtask -- spec
```

Placed with the other `cargo run --package xtask -- <cmd>` steps. This is a
pinned-artifact change and lands with a dated `changelog.d/*.decisions.md`
fragment.

## `cargo xtask changelog --release <version> <date>` (extended)

Gains a release-gate preflight, run before the existing assembly rewrite and
fragment deletion.

- **Input**: the fragments under `changelog.d/` and the release diff
  (`git diff --name-only <base>..HEAD`, `<base>` = most recent `v*.*.*` tag).
- **Behavior**: if any fragment's `spec-impact` names one or more sections and
  `docs/fragcap-specification.md` is not in the diff, the command fails (exit 1)
  and names the offending fragments; nothing is rewritten and no fragment is
  deleted. If git or the base ref is unavailable, exit 2.
- **Unchanged on the happy path**: when every section-naming fragment is backed
  by a specification edit (or all fragments are `spec-impact: none`), assembly
  proceeds exactly as today.

## `spec-impact` fragment field (format)

- **Where**: first line of every `changelog.d/*.md` fragment except `README.md`.
- **Grammar**: `<!-- spec-impact: none -->`
  or `<!-- spec-impact: <section>[, <section>...] -->`, where `<section>` matches
  `[0-9]+(\.[0-9]+)*`.
- **Stripped** from the body by the changelog assembler; never appears in
  `CHANGELOG.md`.
- **Documented** in `changelog.d/README.md`.

## `Applies-To` specification field (format)

- **Where**: the document-control header block of
  `docs/fragcap-specification.md`.
- **Shape**: `**Applies-To:** <X.Y.Z> \`
- **Invariant**: equals the workspace package version; enforced by
  `cargo xtask spec`.

## Backward compatibility

- Existing fragments must gain a `spec-impact` line for the format check to pass.
  There is one today (`changelog.d/doctor-npcap-delay-load.fixed.md`); it and any
  fragments this slice adds carry the field.
- No public API, output format, or capture behavior changes. Unmodified analyzers
  and existing captures are unaffected (nothing in the capture path is touched).
