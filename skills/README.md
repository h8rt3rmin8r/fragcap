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
| Per-agent views | `.claude/skills/`, `.cursor/skills/` | **no** | skills CLI |
| Spec-kit commands | `.agents`, `.claude`, `.cursor`, `.opencode` | yes | spec-kit CLI |

`.agents/skills/` is where vendored content goes because it has the most
tooling agreement: Codex reads it natively, the skills CLI targets it, and
spec-kit's Codex integration writes there. Claude Code and Cursor see the same
content through machine-local symlinks, which carry absolute paths and are
therefore gitignored and regenerated per machine.

`skills-lock.json` records the source and a hash of every vendored skill. The
hash covers every file in the skill directory, path-sorted, not `SKILL.md`
alone, so a changed asset or script is detected too.

## What is deliberately not here

**Detection evasion and sandbox escape skills MUST NOT be added.** A sibling
repository carries them; they were left out of this one on purpose.

They contradict constitution principle P-1, which makes passive observation
absolute. They also misrepresent the project. Specification section 23.3 makes
"reads as laboratory equipment, not cheat tooling" a load-bearing requirement,
because an identity that reads as cheat tooling attracts platform removal,
security software heuristics, and community moderation regardless of what the
software does. A public packet-capture repository shipping evasion tooling in
its agent configuration undercuts that on first inspection.

This is recorded in `AGENTS.md` as a standing prohibition so a later bulk skill
sync does not quietly reintroduce it.

Also excluded as irrelevant rather than harmful: Lua, GraphQL, and game
engine skills. Front-end and design skills are held until S18 brings the
documentation site into scope.

## Known gap

The house Bash standard skill is not vendored, because it is not available on
local disk. Specification section 18.3 requires Bash wrappers built to it, and
section 22.5 specifies `scripts/lint-docs.sh` as built to it.

A different Bash skill was deliberately not substituted. Silently swapping the
standard would be worse than an honest gap, since the wrapper compliance
checker in continuous integration validates against the real one. Resolve
before S18.

## Adding a vendored skill

1. Copy the skill directory into `.agents/skills/`.
2. Regenerate `skills-lock.json`.
3. Check it against constitution P-1 before committing. A skill that teaches a
   denylisted technique does not land, whatever else it is useful for.
4. Note it in a `changelog.d/` fragment.

## Conventions

Skills follow the house skill-authoring conventions: `SKILL.md` frontmatter
declares `description` and `disable-model-invocation` explicitly, the body
stays under 500 lines with reference material in supporting files, and content
is AI-facing pure Markdown.

`CONVENTIONS.md` binds these files like every other file in the repository.

Note that several vendored skills use generic names (`debug`, `test`, `plan`,
`review`, `fix`) that can collide with an agent's own bundled skills. That
collision is inherited from upstream and is left as-is for consistency with the
sibling repositories, but a first-party skill written here should use a
distinctive name.
