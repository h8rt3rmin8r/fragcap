# Data model: Specification and constitution reconciliation (S049)

This slice's "data" is a set of document fields and the small parsed values the
checks operate over. No runtime data structures, no storage.

## Document fields

### `Applies-To` (specification document-control field)

- **Location**: the header block of `docs/fragcap-specification.md`, among the
  `**Field:** value \` lines.
- **Shape**: `**Applies-To:** <X.Y.Z> \`
- **Meaning**: the released software version the specification currently
  describes.
- **Invariant**: equals the workspace package version (`[workspace.package]
  version` in the root `Cargo.toml`) at all times. Both move together in a
  release commit.
- **Validation**: the `spec` check parses `<X.Y.Z>` and compares it to the
  workspace version. Absent field or unparseable value is a could-not-run (exit
  2); present-but-unequal is a failure (exit 1).

### `spec-impact` (changelog fragment field)

- **Location**: the first line of every `changelog.d/*.md` fragment (except
  `README.md`), as an HTML comment.
- **Shape**: `<!-- spec-impact: none -->` or
  `<!-- spec-impact: <section>[, <section>...] -->`
- **Meaning**: which specification sections the change described by the fragment
  modified, or `none`.
- **Validation**: the `spec` check asserts the comment is present and its value
  is well-formed (see the SpecImpact value below). The changelog assembler strips
  this line from the body before assembly.

## Parsed values

### `SpecImpact`

The parse of a `spec-impact` value.

- `None`: the literal `none`.
- `Sections(Vec<String>)`: a non-empty list of section-number tokens.

Constraints:
- A section-number token matches `[0-9]+(\.[0-9]+)*` (for example `3`, `3.3`,
  `27.3`).
- An empty list, a missing comment, or a token that is neither `none` nor a
  section-number shape is a parse error (drives exit 1 in the format check).

### `WorkspaceVersion` / `AppliesToVersion`

- Both are `X.Y.Z` strings compared for exact string equality. No semantic
  version ordering is needed; the invariant is equality, not precedence.

## Release-gate inputs (pure decision)

The release gate's testable core is a pure function over values, no git, no I/O:

- **Input**: a list of `(fragment_name, SpecImpact)` and the set of file paths
  changed in the release diff.
- **Rule**: for each fragment whose `SpecImpact` is `Sections(_)`, the
  specification path (`docs/fragcap-specification.md`) must be present in the
  changed set.
- **Output**: a list of violations, each naming the offending fragment and the
  sections it claimed. Empty list means the gate passes.

The impure wrapper supplies the changed set from
`git diff --name-only <last-release-tag>..HEAD` and the fragment list from
`changelog.d/`.

## Constitution entities (documents, not code)

- **Principle P-10 (One Path To A Target)** and **P-11 (The Specification
  Describes What Shipped)**: verbatim Appendix C text added to
  `.specify/memory/constitution.md`.
- **Sync Impact Report**: the amendment header block prepended to the
  constitution, recording version `1.1.0 -> 1.2.0`, the two added principles, and
  the reason.

## Glossary entries (documents, not code)

- **`Applies-To`** and **`spec-impact`**: new entries under
  `docs/glossary/rust-and-tooling.md`, each following the section 4.3 entry
  template with a primary-source reference, plus the regenerated
  `docs/glossary/index.md`.
