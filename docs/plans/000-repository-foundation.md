# 000: Repository foundation

**Status: complete.** 2026-08-06.

This document records what the repository foundation established and why. It
is not a slice; slices are spec'd through spec-kit and live in `specs/`. The
foundation is the layer beneath that, holding everything which must exist
before the first slice can run.

## Scope boundary

The deliberate line: **the foundation is what a slice cannot create, and
nothing a slice owns.**

Specification section 27.3 names S01 as "workspace scaffold, licensing, CI
skeleton". That overlaps with repository setup, so the boundary was drawn
explicitly rather than allowed to blur:

| In the foundation | Deferred to S01 |
| --- | --- |
| Licensing files (LICENSE, NOTICE) | Cargo workspace manifest |
| Text hygiene and conventions | The eight crates |
| Constitution and Spec Kit itself | `rust-toolchain.toml` |
| Agent context and skills | `xtask` runner |
| Plan and reference documents | Continuous integration workflows |
| Contributor workflow | `profiles/`, `fixtures/`, `scripts/` |

The licensing row is the one exception where the foundation reaches into S01's
territory, and it is deliberate: a repository without a license is ambiguous
from its first commit, and the npcap obligation shapes S01's own CI design, so
S01 needs it as an input rather than an output.

Nothing else was scaffolded by hand. Building the workspace outside the
spec-kit sequence would bypass the workflow that every later slice inherits,
which is precisely the habit the constitution exists to prevent.

## Decisions

### Spec Kit runs four agent surfaces over one engine

Installed: Claude Code, Codex, Cursor, opencode.

There is no agent-agnostic mode to select. Agnosticism is structural: the
`.specify/` directory (templates, scripts, workflow registry, constitution) is
the engine, and each agent's command directory is a thin convenience over it.
An agent with no wrapper runs the same eight phases directly against the
templates and scripts and gets the same result.

Four surfaces means four directories to regenerate when spec-kit upgrades.
That was accepted rather than privileging one agent, since the whole point is
that the repository does not assume which agent a contributor brings.

### Helper scripts are POSIX `sh`, not PowerShell

Deviates from the sibling repositories, which use PowerShell.

Continuous integration runs on both Linux and Windows (specification section
24.3), and `sh` runs on every contributor platform including Git Bash and
WSL2 without requiring a PowerShell 7 install first. Since the repository was
being set up for agent and platform agnosticism, requiring a shell install
before a contributor can create a feature slice worked against that.

This does not touch specification section 18, which still requires both a Bash
and a PowerShell 7 shell wrapper for the tool itself. That is a product
requirement; this is a development-time helper.

### Vendored skills live in `.agents/skills/`

Three tiers, and the split matters:

| Tier | Path | Committed |
| --- | --- | --- |
| First-party skills authored here | `skills/` | yes |
| Vendored portable third-party content | `.agents/skills/` | yes |
| Provenance and integrity | `skills-lock.json` | yes |
| Per-agent views (machine-local symlinks) | `.claude/skills/`, `.cursor/skills/` | **no** |

`.agents/skills/` won on tooling agreement rather than aesthetics: Codex reads
it natively, the skills CLI targets it, and spec-kit's own Codex integration
writes there. A bespoke directory would have required path overrides in all
three.

The per-agent symlinks carry absolute paths and are meaningless on another
machine, so they are gitignored and regenerated locally. Spec-kit's own
`speckit-*` skills are real files and stay tracked in every surface, which is
why the gitignore excludes directory *contents* with a `speckit-*` exception
rather than the directories themselves.

### Two skill categories were deliberately excluded

A sibling repository carries skills covering antivirus evasion and sandbox
escape. Both were left out.

They contradict constitution P-1 directly. They also misrepresent the project:
specification section 23.3 makes "reads as laboratory equipment, not cheat
tooling" a load-bearing requirement rather than a matter of taste, because
identity that reads as cheat tooling attracts platform removal, security
software heuristics, and community moderation regardless of what the software
actually does. A public packet-capture repository shipping evasion tooling in
its agent configuration would undercut that on the first look by anyone
auditing it.

`AGENTS.md` records this as a standing prohibition so it does not get
reintroduced by a later bulk skill sync.

### Git is local only

No remote was created. The README, the disclaimer, the constitution, and the
full specification are reviewed in place before anything is published.

Open question Q-9 (crate name reservation) becomes time-sensitive once the
name is public, so it is flagged in `docs/plans/README.md` to be handled
alongside S01.

## Known gaps

**The house Bash standard skill is missing.** Specification section 18.3
requires Bash wrappers built to it, and section 22.5 specifies
`scripts/lint-docs.sh` as built to it. The skill is not on local disk and was
not vendored. A different Bash skill was deliberately not substituted, since
silently swapping the standard would be worse than recording the gap. This
must be resolved before S18.

**Brand identity is a placeholder.** Guardrails are captured in
`docs/brand/README.md`, and open questions Q-7 and Q-8 remain. Sufficient to
build against; not sufficient to ship a site.

## What comes next

1. Review this foundation, especially the disclaimer and the constitution.
2. Run the reconnaissance session (`reconnaissance.md`). It gates S09, S10,
   and S17, and it is the only work that can invalidate completed work.
3. S01, through the spec-kit sequence, including crate name reservation.

The natural instinct is to start the constitution-to-code pipeline immediately.
The reconnaissance is worth doing first: it is cheap, it needs no code, and
Appendix D is already sitting there waiting for the findings.
