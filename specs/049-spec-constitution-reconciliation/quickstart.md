# Quickstart: validating S049

Runnable validation scenarios that prove the reconciliation and its gates work.
Run from the repository root. See [contracts/checks.md](contracts/checks.md) for
exit-code contracts and [data-model.md](data-model.md) for the field formats.

## Prerequisites

- The pinned toolchain (via `rust-toolchain.toml`).
- `git` on PATH (the release gate resolves the base tag through it).

## 1. The specification describes shipped reality

```bash
grep -n '^\*\*Applies-To:\*\*' docs/fragcap-specification.md
grep -rn 'v0\.2\.0' docs/fragcap-specification.md
```

Expected: the first prints an `Applies-To` line naming the workspace version
(`0.4.0`). The second returns only references that are correct in context (for
example historical release-history rows), and none that present v0.2.0 as the
first or current functional release. The document title no longer reads
"v0.1.0", and section 28's heading no longer reads "Roadmap Beyond v0.2.0".

## 2. The durable rules are in the constitution

```bash
grep -n 'P-10' .specify/memory/constitution.md
grep -n 'P-11' .specify/memory/constitution.md
grep -n '^\*\*Version\*\*: 1\.2\.0' .specify/memory/constitution.md
```

Expected: P-10 (One Path To A Target) and P-11 (The Specification Describes What
Shipped) are present, and the version footer reads `1.2.0`.

## 3. The version lock-step check passes when aligned, fails when not

```bash
cargo run --package xtask -- spec
```

Expected: exit 0 on the reconciled tree ("spec: Applies-To matches the workspace
version" and "every fragment carries a spec-impact line").

Divergence check (do not commit): temporarily change the `Applies-To` value to a
different version, re-run `cargo run --package xtask -- spec`, and confirm it
exits 1 and reports both values. Restore the value.

## 4. The check runs inside the aggregate and in CI

```bash
cargo run --package xtask -- ci
```

Expected: the run reaches "ci: running spec" and the whole set passes. The same
step (`Specification version lock-step`) appears in `.github/workflows/ci.yml`.

## 5. Fragment format is enforced

```bash
cargo run --package xtask -- spec
```

Expected: passes only when every `changelog.d/*.md` fragment (except `README.md`)
begins with a well-formed `spec-impact` comment. To see the failure path,
temporarily remove the `spec-impact` line from a fragment and re-run; it exits 1
and names the fragment. Restore the line.

## 6. The release gate rejects an unbacked section claim

This is exercised by unit tests over the pure decision function (no git needed):

```bash
cargo test --package xtask changelog
cargo test --package xtask spec
```

Expected tests:
- A fragment with `spec-impact: 23.1` plus a changed-file set that does NOT
  include `docs/fragcap-specification.md` yields a violation.
- The same fragment with the specification path present in the changed set yields
  no violation.
- A `spec-impact: none` fragment never constrains the specification.
- `SpecImpact` parsing accepts `none` and `3.3, 27.3`, and rejects an empty value
  and a non-numeric token.

## 7. Whole gate set is green

```bash
cargo run --package xtask -- ci
```

Expected: `ci: all checks passed`, including `fmt`, `clippy`, `test`, `lint`,
`deps`, `license`, `wrappers`, `docs check`, and the new `spec` step. The
documentation linter passes with the two new glossary entries (`Applies-To`,
`spec-impact`) and the regenerated index.
