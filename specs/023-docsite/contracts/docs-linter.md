# Contract: scripts/lint-docs.sh

The documentation linter, built to the ShruggieTech Bash standard, enforcing the
specification section 4.6 checks and regenerating the glossary index.

## Invocation

```text
lint-docs.sh <mode>
lint-docs.sh -h | --help
```

Modes:

| Mode | Purpose | Exit |
| --- | --- | --- |
| `check` | validate the glossary and the term inventory; report all failures | 0 clean, 1 failures found, 2 could not run |
| `fix` | regenerate `docs/glossary/index.md` from the category pages in place | 0 written, 2 could not run |
| `link` | verify every external reference URL responds (weekly) | 0 all live, 1 dead links found, 2 could not run |

`--help` prints the self-parsing man-page help and exits 0 without validating.

## check mode (the four section-4.6 checks)

1. **Entry completeness**: every entry on every `docs/glossary/<category>.md` page
   carries a blurb, a detail paragraph, and a references section; an entry that
   influences a design decision also carries the "why it matters here" note.
2. **Cross-link resolution**: every internal cross-link (a `See also` entry or an
   in-body glossary link) resolves to an existing entry anchor, within a page and
   across pages.
3. **Term inventory (the Undefined Term Rule)**: every term appearing in project
   documentation resolves to a glossary entry. The term list and the documents
   scanned follow specification section 4.2 (spec, README, architecture docs,
   human-facing comments, website copy), not source-code identifiers.
4. **Index reproducibility**: the committed `docs/glossary/index.md` is identical
   to the freshly generated index; any difference is a failure directing the user
   to run `fix`.

check reports every failure it finds (it does not stop at the first), so one run
tells the author everything to correct. It exits 1 if any check fails, 0 if all
pass, 2 if a required tool is absent.

## fix mode

Regenerates `docs/glossary/index.md` from the category pages: an alphabetical list
of every term, each linking to its entry anchor on the owning category page. fix
changes nothing else. On a clean tree, running fix is a no-op (the committed index
already matches), which is what check's reproducibility check asserts.

## link mode

Requests every external reference URL and reports those that do not respond. It
runs on the weekly `links.yml` schedule, not per commit, because link liveness
depends on third parties rather than on the commit. A dead link is exit 1.

## House standard (ShruggieTech Bash)

The script carries `#!/usr/bin/env bash` on line 1, an SPDX identifier on line 2,
`set -euo pipefail` with an explicit `IFS`, a man-page help block with `# NAME`
and `# SYNOPSIS`, the `print_help` / `has_cmd` / `safe_run` / `log_*` fixtures, and
the four ordered 80-column section headers (`# Declare Functions`, `# Declare
Variables and Arrays`, `# Execute Operations`, `# End of script`) with `# End of
script` last. It is UTF-8 without BOM, LF, no emoji, no em or en dashes. It passes
the repository's Bash compliance checker (`cargo xtask wrappers`, file list
extended to include it).

## Wiring

`cargo xtask docs check` runs `lint-docs.sh check`. That check is in the `cargo
xtask ci` aggregate and a named step in `ci.yml`, so P-6 is enforced on every
push. `links.yml` runs `lint-docs.sh link` weekly.
