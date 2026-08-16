# Research: Specification and constitution reconciliation (S049)

Phase 0 decisions. Each resolves an unknown in the plan's Technical Context.

## R-1: Where the `spec-impact` field lives in a changelog fragment

**Decision**: A single leading HTML comment on the first line of every
`changelog.d/*.md` fragment: `<!-- spec-impact: none -->` or
`<!-- spec-impact: 3.3, 23.1, 27.3 -->`. The changelog assembler strips this
line so it never reaches `CHANGELOG.md`.

**Rationale**: Fragments have no frontmatter today; the section is encoded in the
file name (`.added.md`) and the body is otherwise freeform Markdown copied
verbatim into `CHANGELOG.md`. An HTML comment is invisible in rendered Markdown,
is trivially greppable and parseable, and does not collide with the body's
`### Heading` convention. It is stripped in the same place the assembler already
strips a redundant leading `### Section` header (`strip_leading_section_header`
in `xtask/src/changelog.rs`), so `CHANGELOG.md` stays clean.

**Alternatives considered**:
- A companion file (`<key>.spec-impact`): doubles the file count per change and
  is easy to forget; the field belongs with the fragment it describes.
- A visible `spec-impact:` prose line: leaks into `CHANGELOG.md` unless stripped,
  and reads as content rather than metadata.
- YAML frontmatter: introduces a parser and a second syntax the assembler would
  have to learn for one scalar; disproportionate.

## R-2: Value grammar for `spec-impact`

**Decision**: The value is either the literal `none` or a comma-separated list of
section-number tokens. A section-number token is one or more ASCII digits with
optional `.`-separated numeric parts (`3`, `3.3`, `27.3`, `23.1`). Whitespace
around commas is trimmed. An empty value, a missing comment, or a token that is
not `none` and not a section-number shape is a format error.

**Rationale**: Matches how the specification numbers its sections. Keeping the
grammar to `none | list-of-section-numbers` makes both the format check and the
release gate simple and total. The token shape is validated (digits and dots),
but existence of the named section in the specification is deliberately NOT
validated (see R-4).

## R-3: The version lock-step check (`Applies-To` vs workspace version)

**Decision**: A new `cargo xtask spec` subcommand, backed by `xtask/src/spec.rs`,
that (A) reads the `Applies-To` field from the specification's document-control
block and asserts it equals the workspace package version, and (B) asserts every
`changelog.d` fragment carries a well-formed `spec-impact` line. It follows the
house 0/1/2 exit contract: 0 pass, 1 fail (mismatch or malformed fragment), 2
could-not-run (field or manifest unreadable). It is wired into the `ci`
aggregation in `xtask/src/main.rs` and added as a step in `.github/workflows/ci.yml`.

The workspace version is read with a helper modeled on the existing
`workspace_msrv` in `main.rs`: the first `version = "..."` line under
`[workspace.package]` in the root `Cargo.toml` (currently `0.4.0`).

**Rationale**: One subcommand covers both currency checks (version field and
fragment format), adding a single CI step. The 0/1/2 contract and the
`workspace_msrv` precedent are already established in this crate, so the new code
matches its neighbors. Reading a dedicated `Applies-To` field (not the document's
own `**Version:**` field) avoids conflating the software release the spec
describes with the spec document's own version.

**Alternatives considered**:
- Folding the version check into `lint`: `lint` is text-hygiene and conventions
  over files; a version-equality assertion against the manifest is a different
  concern and deserves its own named step that reads clearly in CI logs.
- Reading the workspace version via `env!("CARGO_PKG_VERSION")` in xtask: that is
  xtask's own version, which equals the workspace version today but couples the
  check to xtask's packaging rather than to the manifest it means to assert.

## R-4: The release gate (`spec-impact` backed by a real edit)

**Decision**: The release gate runs inside `cargo xtask changelog --release`, as
a preflight before any rewrite or fragment deletion. It parses the `spec-impact`
of every fragment about to be consumed; if any names one or more sections, it
asserts that `docs/fragcap-specification.md` appears in the release diff. The
release diff is `git diff --name-only <base>..HEAD`, where `<base>` is the most
recent release tag (`git describe --tags --abbrev=0 --match 'v*.*.*'`). If git or
the base ref is unavailable the gate exits 2 (could-not-run), never 0.

The check is **file-level**: a fragment naming any section requires the
specification file to have changed in the diff. It does not verify that the
specific named section changed, and it does not verify the section exists. Both
are deliberate, documented limitations that keep the gate total and cheap and
match the handoff plan's wording ("a fragment naming a section with no
specification change in the diff fails the release gate").

**Rationale**: Fragments are consumed at changelog assembly, which is the last
moment they exist; the claim "this change touched section X" must be validated
then, not at tag time (by which the fragments are gone). Locating the gate in
`changelog --release` keeps it out of the pinned `release.yml`, so only `ci.yml`
incurs a pinned-artifact decision fragment. The pure decision (given fragments
and a changed-file set, produce violations) is a separate function so it is unit
tested without touching git, satisfying the acceptance test directly.

**Alternatives considered**:
- A step in `release.yml`: `release.yml` is pinned (a second decisions fragment)
  and its `release` job checks out shallow, so it would need `fetch-depth: 0`;
  more moving parts for no gain, and the fragments are already consumed by then.
- Per-section line-range validation: brittle (section boundaries move), and the
  handoff plan asks only for a specification change, not a specific-section diff.
- Validating that a named section exists: turns the gate into a section-index
  parser; out of scope and orthogonal to the "backed by a real edit" contract.

## R-5: Glossary obligation under P-6

**Decision**: Add glossary entries for the two terms this slice introduces into
documentation, `Applies-To` (specification field) and `spec-impact` (changelog
fragment field), and regenerate the glossary index.

**Rationale**: P-6 requires a glossary entry in the same change that introduces a
term, and the precedent is that repository-process terms are glossaried here:
`xtask` and `msrv` both have entries under `docs/glossary/rust-and-tooling.md`.
The documentation linter (`scripts/lint-docs.sh`, wired into `cargo xtask ci`)
enforces entry completeness and index reproducibility, so omitting them risks a
CI failure as well as a principle violation. The natural category is
`rust-and-tooling.md` (build and process tooling), consistent with `xtask` and
`msrv`.

**Alternatives considered**:
- Treating them as exempt process jargon: contradicted by the `xtask`/`msrv`
  precedent and by P-6's plain text.

## R-6: The `Applies-To` field placement and the document's own version

**Decision**: Add `**Applies-To:** 0.4.0 \` to the specification's
document-control header block (the run of `**Field:** value \` lines at the top),
naming the released version the document describes. The document's own
`**Version:**` and title framing are corrected by the FR-004 currency sweep as a
separate concern; the lock-step check reads only the `Applies-To` field.

**Rationale**: A dedicated field gives the check an unambiguous anchor and keeps
"which software release does this describe" distinct from "which revision of this
document is this," which the current title (`v0.1.0`) and section 1 framing
(`v0.2.0`) already muddle. Setting it to `0.4.0` satisfies the clarified rule
that `Applies-To` equals the workspace version during development.

## R-7: Constitution amendment mechanics

**Decision**: Insert P-10 and P-11 verbatim from the handoff plan Appendix C after
P-9 in `.specify/memory/constitution.md`; prepend a new Sync Impact Report block
recording the change; bump the version 1.1.0 -> 1.2.0 (MINOR: two principles
added); update the footer `Version` and `Last Amended` line to the amendment
date (2026-08-16). The plan-template Constitution Check reads the file live, so
no template edit is required (consistent with the 1.1.0 amendment note).

**Rationale**: This mirrors the existing 1.0.0 -> 1.1.0 amendment exactly, which
is the in-repo precedent for how principles are added. MINOR is the correct bump
per the versioning policy ("a principle or section added").
