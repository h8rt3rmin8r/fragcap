# Contract: scripts/lint-docs.sh

The documentation linter, built to the ShruggieTech Bash standard, enforcing the
specification section 4.6 checks and regenerating the glossary index.

## Invocation

```text
bash scripts/lint-docs.sh <mode>
bash scripts/lint-docs.sh -h | --help
```

The script stays non-executable (mode 100644, like `scripts/fragcap.sh`) and is
invoked through `bash`, matching how continuous integration and the `wrappers`
gate call the repository's shell scripts.

Modes:

| Mode | Purpose | Exit |
| --- | --- | --- |
| `check` | validate the glossary and the term inventory; report all failures | 0 clean, 1 failures found, 2 could not run |
| `fix` | regenerate `docs/glossary/index.md` from the category pages in place | 0 written, 2 could not run |
| `link` | verify every external reference URL responds (weekly) | 0 all live, 1 dead links found, 2 could not run |

`--help` prints the self-parsing man-page help and exits 0 without validating.

## check mode (the section-4.6 checks)

1. **Entry completeness**: every entry on every `docs/glossary/<category>.md` page
   carries a prose blurb or detail (not merely metadata markers such as "Also
   known as" or "See also"), and a `**References:**` section or a matters callout,
   where present, is not empty. A references section is not mandated on every
   entry: much of the glossary is fragcap's own internal vocabulary (for example
   "Sink thread") for which no primary source exists, and fabricating one would
   violate P-9. Cross-links are repository-relative sibling paths
   (`<category>.md#<anchor>`) so entries resolve on disk and on GitHub.
2. **Cross-link resolution**: every internal cross-link (a `See also` entry or an
   in-body glossary link) resolves to an existing entry anchor, within a page and
   across pages.
3. **Glossary reference / the Undefined Term Rule**: every glossary reference in a
   canonical document (a Markdown link into the glossary) names a defined term.
   The documents scanned follow specification section 4.2 (README, top-level docs,
   and the glossary itself; website copy joins with the site in S18c-2), not
   source-code identifiers. A bare prose word that is not referenced as a glossary
   term is not scanned: no sound rule distinguishes it from ordinary English, so
   the enforced mechanism is the glossary reference the documents actually use to
   mark a term.
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
