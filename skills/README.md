# First-party skills

Skills authored for this repository live here, one directory per skill, named
to match the invocation slug.

```text
skills/<skill-name>/
  SKILL.md           Required
  README.md          Optional, human-facing notes
  assets/            Optional, templates and reference material
  scripts/           Optional, executable helpers
```

There are none yet. Write one when a procedure in this repository is run often
enough that describing it every time costs more than encoding it, and specific
enough that no vendored skill covers it. Likely candidates as the project
matures: the reconnaissance protocol, `.fcapng` annotation validation, and
profile authoring for a new title.

## Where the other skills are

Three tiers, and the split is deliberate:

| Tier | Path | Committed | Who writes it |
| --- | --- | --- | --- |
| First-party | `skills/` | yes | us |
| Vendored portable content | `.agents/skills/` | yes | upstream, copied in |
| Provenance and integrity | `skills-lock.json` | yes | generated |
| Per-agent views | `.claude/skills/`, `.cursor/skills/` | **no** | an external skills CLI |
| Spec-kit commands | `.agents`, `.claude`, `.cursor`, `.opencode` | yes | spec-kit CLI |

`.agents/skills/` is where vendored content goes because it has the most
tooling agreement: Codex reads it natively and spec-kit's Codex integration
writes there.

Claude Code and Cursor read their own directories instead. An external skills
CLI populates those with machine-local symlinks carrying absolute paths, which
is why they are gitignored. That CLI is not part of this repository, and a
checkout may therefore carry no per-agent views at all beyond the tracked
`speckit-*` directories. If yours does not, the vendored content is still there
under `.agents/skills/`; nothing in this repository depends on the symlinks
existing.

`skills-lock.json` records the source and a hash of every vendored skill. The
hash covers every file in the skill directory, path-sorted, not `SKILL.md`
alone, so a changed asset or script is detected too. Note that the hash is
recorded, not verified: `cargo xtask skills` checks structure, not content.
See "The gate" below.

## What is admitted, and what is not

The vendored set is the ShruggieTech house standards this repository's
constitution binds, taken from one upstream, and nothing else.

**The admission test.** A skill is admitted only if a named constitution
principle binds this repository to it, or a repository gate executes it. Both
conditions are checkable against a file and a line, which is what makes the
test usable on a skill nobody here has seen before. Plausible usefulness is not
a qualification, and neither is sharing a brand with something that is.

That test currently admits four:

| Skill | Admitted by |
| --- | --- |
| `shruggie-bash` | P-8 names Bash |
| `shruggie-markdown` | P-8 names Markdown |
| `shruggie-powershell` | P-8 names PowerShell, and `cargo xtask wrappers` executes its compliance checker |
| `shruggie-speckit` | Drives the spec-kit workflow `AGENTS.md` makes mandatory |

**The upstream** is <https://github.com/shruggietech/skills>, Apache-2.0. Every
lock entry's `source` names it together with the release tag the content came
from, so a reader can resolve exactly what was vendored. Taking a skill from
anywhere else means the set no longer has one authority, which is the state
slice S071 exists to have ended.

## The gate

`cargo xtask skills` runs in the ordinary check set and asserts three things:

1. Every `skills-lock.json` entry has a directory and the `SKILL.md` its
   `skillPath` names.
2. Every vendored directory has a lock entry. `speckit-*` is excluded by
   prefix; the spec-kit CLI owns those.
3. Every file under a vendored skill is tracked by git.

The third exists because of a real defect rather than a hypothetical one.
`.agents/skills/debug/` sat on disk and in the lock, and uncommitted, from the
founding commit until S071, because `.gitignore` carried a bare `debug` pattern
inherited from a Cargo template. Nothing noticed, because until S071 nothing
read this file at all.

The gate does not verify hashes. The tool that writes them is not part of this
repository and its algorithm is reproduced here empirically rather than from a
specification, so a hash check could fail against correct content. If you are
tempted to add one, read the S071 decisions fragment first.

## Adding a vendored skill

1. Check it against the admission test above. If no principle binds it and no
   gate runs it, it does not go in, however useful it looks.
2. Download it from the upstream release and verify the archive against that
   release's published `SHA256SUMS.txt` **before** extracting.
3. Copy the directory into `.agents/skills/` **unmodified**. Do not hand-edit it
   for text hygiene; see "Why vendored content is never edited" below.
4. Check it against constitution P-1 before committing. A skill that teaches a
   denylisted technique does not land, whatever else it is useful for.
5. Add its `skills-lock.json` entry, with `source` naming the upstream and the
   release tag.
6. Run `cargo xtask skills` and `git add` the tree, in that order if you want
   the gate to demonstrate itself.
7. Note it in a `changelog.d/` fragment.

## Removing a vendored skill

The mirror of the above, and it is written down because its absence is how the
set grew to 36 entries that no one could account for.

1. Establish that nothing depends on it. `cargo xtask wrappers` executes a
   script inside `shruggie-powershell`, so that one cannot simply be deleted;
   check for others the same way, by searching the repository for the skill's
   path rather than its name.
2. Delete the directory from `.agents/skills/`.
3. Delete its entry from `skills-lock.json`. Do not recompute any other entry's
   hash while you are in the file.
4. Run `cargo xtask skills`. It fails if the two halves disagree.
5. Note the removal in a `changelog.d/` fragment, naming the skill. A removal
   that is not recorded is the governance form of an uncounted discard.

## Why vendored content is never edited

The first vendoring, in the founding commit, hand-edited its copies to satisfy
the text hygiene rules in `CONVENTIONS.md`. Three lock hashes never reproduced
again, and because nothing verified the file, that went unnoticed for the life
of the project until S071.

An edited vendored copy is no longer the upstream standard it claims to be, and
a hash that does not reproduce is the symptom rather than the disease. Current
upstream ships clean under `CONVENTIONS.md` already, so the edit buys nothing.
If a future upstream does not, raise it upstream rather than patching it here.

## Conventions

Skills follow the house skill-authoring conventions: `SKILL.md` frontmatter
declares `description` and `disable-model-invocation` explicitly, the body
stays under 500 lines with reference material in supporting files, and content
is AI-facing pure Markdown.

`CONVENTIONS.md` binds files in this repository, but `xtask/src/lint.rs`
deliberately excludes the vendored trees from the linter, because vendored
content is upstream's to fix. That exclusion is what let an em-dash sit in a
vendored file for the life of the project. It is tolerable only while the
admission test keeps the set small and the upstream single; it is not a licence
to vendor anything that would fail the linter.

A first-party skill written here should use a distinctive name, so it cannot
collide with an agent's own bundled skills.
