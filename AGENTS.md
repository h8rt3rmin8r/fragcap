# Agent guide (provider-agnostic)

This is the canonical, agent-neutral instruction file for this repository. Any
coding agent that reads `AGENTS.md` (Codex, Cursor, opencode, and others)
should treat this as the source of truth. Claude Code reads it through
`CLAUDE.md`, which imports this file.

These instructions OVERRIDE any default behavior. Follow them exactly.

## What fragcap is

fragcap is a passive, process-attributed network capture tool for Windows,
written in Rust. Packet capture is a solved problem; attribution is not.
Standard tooling captures at the network driver, below the socket layer, where
the association between a packet and the process that produced it has already
been discarded. fragcap reconstructs that association for game clients launched
indirectly through platform and publisher launchers, and writes it into an
extended pcapng profile that unmodified analyzers still read as ordinary
pcapng.

It observes. It does not modify traffic, and it does not reach inside the
processes it names. That distinction is the whole security posture, and
principle P-1 makes it absolute.

## Reference documents

Read these before acting. They are ordered by authority.

- **Constitution** (governing principles, versioned):
  `.specify/memory/constitution.md`
- **Master specification** (architecture of record):
  `docs/fragcap-specification.md`. Every feature traces to it. Section
  references in the constitution and in slice specs point here.
- **Specification outline** (a map of the above, useful for navigation):
  `docs/fragcap-spec-outline.md`
- **Slice ordering and dependencies**: `docs/plans/README.md`
- **Repository mechanical rules**: `CONVENTIONS.md`
- **Contributor workflow**: `CONTRIBUTING.md`
- **Active feature directory**: recorded in `.specify/feature.json`
  (`feature_directory`). Read that feature's `plan.md` before implementing; it
  carries the technologies, project structure, shell commands, and
  slice-specific context for the current work.

## Current state

Slices S01 and S02 are complete. The Cargo workspace exists with the eight
crates from the architecture of record, a task runner carrying the repository's
own checks, and six workflow files. `fragcap-core` carries the type and trait
vocabulary from specification sections 8.4 and 8.5.

**There is still no behavior.** Nothing captures, attributes, parses, or writes
anything. S02 fixed the shape of the seams; the slices that fill them start at
S03. Each crate's module documentation names the slice that fills it.

The workspace has one external dependency, `bytes`. `fragcap-core` may depend
only on crates named in the allowlist in `xtask/src/deps.rs`, which is checked
mechanically.

The remote is `origin`, at `https://github.com/h8rt3rmin8r/fragcap`. S01
integrated through pull request #1.

Two things are scaffolded but not exercised, and must not be reported as
passing checks:

- **Half the workflow matrix has never completed.** The first runs landed
  during the GitHub incident of 2026-08-06. `check (ubuntu-latest)` and
  `check (windows-latest)` passed; `minimum supported toolchain`, `core builds
  without a capture backend`, `platform`, and `audit` never acquired a runner
  and are red for that reason, not for a code reason. Re-run them before
  treating any of the four as green.
- **The minimum-toolchain check now runs for real.** Until S02 it built with
  the pinned toolchain and reported success, which said nothing about the
  declared minimum. It now builds through `rustup run 1.82` and exits 2 when
  that toolchain is absent, so a check that did not run can no longer look like
  one that passed.
- **The npcap SDK acquisition step is unexercised.** No crate links against the
  capture library until S09.
- **`cargo deny` has never run.** The `audit` workflow owns it and is weekly
  and dispatch-only. The dependency graph is no longer empty, so the check now
  has a subject; nobody has watched it pass.

## Spec-driven development workflow (spec-kit)

Every slice MUST be spec'd through the spec-kit framework before
implementation. The slice ordering document scopes a slice but never
substitutes for its spec.

The engine is shared and agent-neutral; drive it the same way regardless of
which agent you are:

- Templates: `.specify/templates/` (`spec-template.md`, `plan-template.md`,
  `tasks-template.md`, `checklist-template.md`, `constitution-template.md`)
- Scripts: `.specify/scripts/bash/` (`create-new-feature.sh`, `setup-plan.sh`,
  `setup-tasks.sh`, `check-prerequisites.sh`, `common.sh`)
- Workflow registry: `.specify/workflows/workflow-registry.json`
- Constitution (the gate every phase checks against):
  `.specify/memory/constitution.md`

The full cycle, run end to end per slice:

1. **specify** - create or update the feature spec from the slice intent.
2. **clarify** - resolve underspecified areas; encode answers back into the
   spec.
3. **checklist** - generate a slice-appropriate quality checklist.
4. **plan** - produce design artifacts into the feature directory.
5. **tasks** - generate a dependency-ordered `tasks.md`.
6. **analyze** - non-destructive cross-artifact consistency check. This gate
   MUST pass and MUST NOT be weakened or skipped.
7. **implement** - execute `tasks.md`.
8. **verify** - run the full gate set in the foreground (see below).
9. **commit** - stage only the slice's files, add a changelog fragment under
   `changelog.d/`, and commit. `.specify/feature.json` is local, gitignored
   state; never stage it.

Agents with native spec-kit command wrappers may invoke those. Four surfaces
are installed and all drive the same `.specify/` engine:

| Agent | Command surface |
| --- | --- |
| Claude Code | `.claude/skills/speckit-*` |
| Codex | `.agents/skills/speckit-*` |
| Cursor | `.cursor/skills/speckit-*` |
| opencode | `.opencode/commands/speckit.*` |

Agents without a wrapper should follow the phases above directly against the
templates and scripts. The result is identical; the wrappers are convenience,
not capability.

Do not re-point or hand-edit `.specify/integration.json` or
`.specify/init-options.json`. Those record the generated command surfaces and
are regenerated by the spec-kit CLI.

## Skills

Portable skill content is vendored in `.agents/skills/` and committed, with
provenance and integrity hashes in `skills-lock.json`. First-party skills
authored for this repository live in `skills/`.

Codex reads `.agents/skills/` directly. Claude Code and Cursor read their own
directories, populated with machine-local symlinks by the skills CLI; those
symlinks are gitignored because they carry absolute paths. Spec-kit's own
generated `speckit-*` skills are tracked in every surface.

A skill is checked against P-1 before it is vendored. A skill that teaches a
denylisted technique does not land here, whatever else it is useful for.

## Non-negotiables

These restate the constitution's sharpest edges. The constitution is
authoritative; this list is the one to keep in working memory.

- **The technique denylist is absolute.** No packet interception drivers, no
  code injection, no function hooking, no process handles carrying memory-read
  rights against a target, no layered service providers, no executable image
  modification. A slice that appears to need one has been scoped wrong; halt
  and raise it.
- **Any process handle states its access rights explicitly at the call site.**
  A request carrying memory rights fails review.
- **`fragcap-core` takes no platform-specific dependency.** Dependencies flow
  concrete toward abstract, and continuous integration proves core builds for
  a target with no capture backend.
- **Every discard path has a named counter.** A dropped packet that is not
  counted and surfaced is a defect.
- **npcap is never bundled, never downloaded, never installed by fragcap, and
  its SDK is never vendored.** Detection only.
- **Compatibility outranks richness.** Output stays readable by unmodified
  analyzers.
- **A new term gets a glossary entry in the same change that introduces it.**
- **Wrappers stay thin.** A wrapper that needs to parse output means a missing
  capability in Rust.
- **Pinned artifacts change only with a dated decision recorded in
  `CHANGELOG.md`:** `.github/workflows/**`, `rust-toolchain.toml`,
  `release.toml`, `scripts/**`, and release documentation. Write the decision
  as a `changelog.d/<key>.decisions.md` fragment; `CHANGELOG.md` is assembled
  from those fragments at release time, and editing it from a feature branch
  conflicts with every other concurrent pull request. `release.toml` does not
  exist yet; it arrives with the release process. The rule binds it from the
  moment it lands.
- **All text files are UTF-8 without BOM with LF line endings. No em-dashes or
  en-dashes anywhere, including code comments.**

## Verification discipline

Run verification in the foreground and watch it to completion. Never background
it, never infer a result you did not read.

The gate set, all of which `cargo xtask ci` runs in order:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --locked
cargo xtask lint          # repository conventions, CONVENTIONS.md
cargo xtask deps          # dependency direction, specification section 8.3
cargo xtask license       # per-crate license text for registry publication
```

Two further checks are not in `ci` because they need a target or a toolchain
the runner may not have: `cargo xtask neutral` (constitution P-2) and
`cargo xtask msrv`. Both exit 2 rather than 0 when they cannot run.

The documentation linter and the shell wrapper compliance checkers arrive with
the slices that own them.

**Claims require evidence.** Do not report a slice complete, a test passing, or
a defect fixed without having run the command and read its output. If tests
fail, say so and include the output. If a step was skipped, say that. Reporting
an unverified success is worse than reporting a known failure, because it
removes the operator's ability to trust any other report.

## Deciding versus asking

Default to deciding: enumerate the alternatives, evaluate them against the
constitution, the master specification, and the slice scope, pick the best,
proceed, and record the rationale in the slice.

Halt to the operator only when no option is clearly best on an irreversible or
architecture-defining choice, the slice intent is genuinely ambiguous, or a
constitution conflict needs a human call. A P-1 conflict is always a halt.

## Reconnaissance gate

**Closed.** Open questions Q-1 through Q-6 (specification section 29) are
resolved. The findings are recorded in Appendix D and were applied to the
specification; the protocol that produced them is
`docs/plans/reconnaissance.md`.

Slices S09, S10, and S17 were gated on those answers and are now unblocked.
Q-7 and Q-8 remain open and gate S18.

## Integration workflow

Work integrates through pull requests reviewed by the operator
(`@h8rt3rmin8r`). Never push directly to `main`, and never merge your own pull
request. See `CONTRIBUTING.md` for the full workflow.

Never push, tag, cut a release, or publish a crate without explicit
authorization.
