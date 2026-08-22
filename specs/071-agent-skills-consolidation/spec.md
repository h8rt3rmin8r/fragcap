# Feature Specification: Agent skills consolidation

**Feature Branch**: `071-agent-skills-consolidation`

**Created**: 2026-08-22

**Status**: Implemented

**Input**: User description: "S071: consolidate the vendored agent skills onto
one upstream, closing issue #197. Prune `.agents/skills/` against the test of
whether a skill is bound by a named constitution principle or executed by a
gate, re-source what survives from one canonical upstream, refresh the stale
house standards, close the unresolved P-8 Bash gap, and add a structural gate so
the set cannot drift unobserved again."

## Context

`.agents/skills/` was populated once, in the founding commit `24c2ef1`, and
extended exactly once after (`2082c42`, pull request #4). Nothing has touched it
since. It holds 46 directories: 10 CLI-owned `speckit-*`, which the spec-kit CLI
regenerates and which are out of scope here, and 36 vendored skills recorded in
`skills-lock.json`.

Thirty-five of those 36 carry `source: "h8rt3rmin8r/eso-weave"`, an unrelated
personal repository. The 36th says `"h8rt3rmin8r (local skill library)"`. No
entry names a versioned, resolvable upstream, and nothing in the repository
verifies the file, so the set has drifted in three independent directions
without anyone noticing.

Issue #197 framed this as a prune. The prune is the visible half. The defect
underneath is that there is no stated test for what `.agents/skills/` is *for*,
which is how a capture-the-flag forensics playbook, twelve skills whose names
collide with the agent's own built-ins, and a 283-file third-party Rust rulebook
all came to be presented as fragcap house standards vetted for this project.

This slice states the test, applies it, and gates it.

## Evidence

Every claim was measured against the working tree or the upstream release on
2026-08-22. A claim about something observed once carries its date; a claim
about how a check behaves names how to reproduce it.

**This table records the state the slice found, not the state it left.** Rows
E-04, E-06, E-08 through E-11, and E-14 through E-18 describe defects this slice
corrected, so they read as false against the tree after it. That is the point of
keeping them: a fix whose prior state is not written down is indistinguishable
later from a thing that was always fine. Re-verified at commit time; each row
either still holds or was corrected by a change recorded in `changelog.d/`.

| # | Claim | How it was established |
| --- | --- | --- |
| E-01 | `cargo xtask wrappers` executes `.agents/skills/shruggie-powershell/scripts/test-script-compliance.sh`. Its `else` arm counts a **failure**, so deleting that skill turns the gate red rather than skipping it. | `xtask/src/wrappers.rs:187,193,226-243`; reached from `xtask/src/main.rs:137,341` and `.github/workflows/ci.yml:76` |
| E-02 | `shruggie-speckit` is the autopilot protocol this slice runs under, and is already cited as spec provenance. | `specs/066-steam-identity-presence/spec.md:29` |
| E-03 | 33 of the 36 vendored skills have no reference anywhere outside their own directory. The other three are `shruggie-powershell` (E-01), `shruggie-speckit` (E-02), and `shruggie-graph-memory`, whose only mention is the changelog line recording that it was vendored. | Repository-wide search excluding the skill trees; generic names (`debug`, `test`, `plan`, `review`, `fix`, and the rest) checked separately for slash-invocation form to exclude ordinary-English false positives |
| E-04 | `.agents/skills/debug/` is present on disk and recorded in `skills-lock.json` but **has never been committed**. `.gitignore` line 3's bare `debug`, inherited from a Cargo build-artifact template, silently excludes it. | `git check-ignore -v .agents/skills/debug` reports `.gitignore:3:debug`; the directory is absent from `git ls-files` |
| E-05 | Nothing verifies `skills-lock.json`. There is no xtask subcommand, script, or workflow step that reads it. | Search for `skills-lock` finds only prose in `AGENTS.md`, `CLAUDE.md`, `CHANGELOG.md`, `skills/README.md`, `docs/plans/000-repository-foundation.md`, and unrelated spec analogies |
| E-06 | Three lock hashes have never reproduced: `rust-skills`, `shruggie-html`, `shruggie-powershell`. All three were committed in `24c2ef1` together with the lock file, so this predates any later change. They were deliberately not recomputed, because overwriting a non-matching hash with whatever is on disk destroys the signal the lock exists to carry. | Established by a prior session that reverse-engineered the hash algorithm and reproduced 32 of the 35 entries then present |
| E-07 | `rust-skills` is 283 of the 382 files tracked under `.agents/skills/`, or 74 percent of the tree. It is a third-party rulebook (265 rules, MIT, author `leonardomso`). | `git ls-files` per directory; `.agents/skills/rust-skills/SKILL.md:1-12` |
| E-08 | `traffic-analysis-pcap` is a capture-the-flag forensics playbook covering credential harvesting, TLS decryption, and covert-channel detection, and it routes to four skills that were never vendored here. fragcap writes pcapng; it does not analyse captures for exfiltration. | `.agents/skills/traffic-analysis-pcap/SKILL.md:9-18` |
| E-09 | The same file carries em-dashes in its own heading, which `CONVENTIONS.md` prohibits absolutely. It survived because `xtask lint` excludes the vendored trees. | `.agents/skills/traffic-analysis-pcap/SKILL.md:7`; `xtask/src/lint.rs:26-38` |
| E-10 | Both bound house standards are stale against current upstream: `shruggie-markdown` 231 lines against 295, `shruggie-powershell` 329 against 364. | Line counts against the v1.11.0 release |
| E-11 | The vendored PowerShell **checker that gates continuous integration** has itself drifted: 48 differing lines in the POSIX twin, 141 in the PowerShell original. | `diff` against the v1.11.0 release |
| E-12 | Refreshing that checker does **not** break the gate. Both the currently vendored checker and the v1.11.0 checker report `scripts/Invoke-FragCap.ps1` compliant, with identical output and exit code 0. Measured 2026-08-22, before any file was changed. | Both checkers run directly against the wrapper |
| E-13 | `shruggie-speckit` is already at upstream content; the refresh changes provenance only. | `diff -rq` reports no differences |
| E-14 | Two broken cross-references exist, as issue #197 reported, though it overstated one. `legacy-code-safety/SKILL.md:423-427` names three skills that do not exist. `gh-fix-ci/SKILL.md:54` names `create-plan`, which does not exist; line 12 of the same file is conditional (`If a plan-oriented skill ... is available`) and is **not** broken. | Both files read directly |
| E-15 | `docs/plans/000-repository-foundation.md:91` states "Thirty-five skills were carried". The lock has 36 entries; `shruggie-graph-memory` was added later. | Lock entry count |
| E-16 | `skills/README.md` documents adding a vendored skill in four steps. No removal procedure exists anywhere in the repository. | `skills/README.md:60-66`; searched `AGENTS.md`, `CONTRIBUTING.md`, `CONVENTIONS.md` |
| E-17 | `AGENTS.md` and `CLAUDE.md` both state that `.claude/skills/` and `.cursor/skills/` are populated with machine-local symlinks to the vendored set. On this checkout both contain only the 10 `speckit-*` directories, and `git ls-files -s` finds no symlink in the repository at all. The described mechanism is not operating here. | Directory listing; `git ls-files -s` filtered for mode `120000` |
| E-18 | `skills/README.md` records a known gap: the house Bash standard was not vendored because it was not on local disk, and "must be resolved before S18". S18 shipped roughly 50 slices ago and the gap was never closed. The skill is now available. | `skills/README.md`; the v1.11.0 release ships `shruggie-bash` |
| E-19 | The canonical upstream is `https://github.com/shruggietech/skills`: public, Apache-2.0 (fragcap's own licence, inside the constitution's allowlist), released as **v1.11.0** at commit `46ba297d`, published 2026-08-22T08:22:03Z. It holds exactly seven skills. | GitHub API |
| E-20 | The four skills this slice vendors were downloaded from that release and verified against its published `SHA256SUMS.txt`: `shruggie-bash` `0351b062...8071`, `shruggie-markdown` `a68a25bf...fa17`, `shruggie-powershell` `1e043b5e...e246`, `shruggie-speckit` `9c00c1db...b22c`. All four reported `OK`. Measured 2026-08-22. | `sha256sum -c SHA256SUMS.txt` |
| E-21 | All four are already clean under `CONVENTIONS.md` as shipped: no em-dashes or en-dashes, no CRLF, no byte-order mark. No local edit is needed to make them comply. | Byte scan of every file in each extracted archive |
| E-22 | Every dropped skill remains available to this operator from the user-global skills directory (551 skills) and from its own upstream. Nothing dropped is fragcap-specific. | Directory listing of the user-global skills path |

## Clarifications

### Session 2026-08-22

Answered under the autopilot decision policy except where the operator was
consulted; those are marked.

- **Q: What is the admission test for `.agents/skills/`?** A: A skill is
  admitted only if a named constitution principle binds this repository to it,
  or a repository gate executes it. Plausible usefulness is not a qualification.
  This is the test issue #197 proposed in its own item 1, stated positively.
- **Q: How aggressive should the prune be?** A: **Operator decision.** Strict:
  keep four. Three of the existing 36 lock entries survive, 33 are dropped, and `shruggie-bash` is added new.
- **Q: Should the surviving set come from one upstream?** A: **Operator
  decision.** Yes. One consolidated pile, one brand, no duplicates. This
  replaces `h8rt3rmin8r/eso-weave` as the provenance for every entry.
- **Q: Is `traffic-analysis-pcap` admitted on domain relevance?** A: No.
  **Operator challenged the assumption and was correct.** See E-08 and E-09. It
  is a security-forensics playbook, not a capture-authoring reference, and
  fragcap has no analysis surface for it to serve.
- **Q: Should `shruggie-docs`, `shruggie-graph-memory`, and `shruggie-html` be
  admitted, being in-brand and in the same upstream?** A: No. No principle binds
  them and no gate executes them. Admitting on brand alone would restate the
  failure this slice exists to correct. Recorded as a decision.
- **Q: Should the stale house standards be refreshed in this slice?** A:
  **Operator decision.** Yes, both are stale (E-10) and must be taken from
  current upstream.
- **Q: Should the P-8 Bash gap be closed?** A: **Operator decision.** Yes,
  vendor `shruggie-bash`.
- **Q: Should vendored content be hand-edited to satisfy text hygiene, as the
  original copies were?** A: No. E-21 shows no edit is needed, and E-06 shows
  that editing after vendoring is what broke three hashes. An edited vendored
  copy is no longer the upstream standard it claims to be.
- **Q: Should the new gate verify hashes?** A: **Operator decision.** No.
  Structural assertions only. The hash algorithm available here is
  reverse-engineered rather than authoritative (E-06), and encoding it wrong in
  a gate would produce false failures against correct content.
- **Q: Should the vendored Bash checker be wired into `cargo xtask wrappers`,
  now that `scripts/fragcap.sh` is measured to pass it cleanly?** A: Not in this
  slice. It is a real improvement and it is scope growth; recorded as OOS-004
  and filed as a follow-up issue so the measurement is not lost.

## User Scenarios & Testing *(mandatory)*

The users here are the people and agents who read this repository's instruction
surface: contributors, reviewers, and the coding agents that act on it.

### User Story 1 - A contributor can tell what the vendored set is for (Priority: P1)

A contributor opening `.agents/skills/` sees a small set whose presence is
explained by a stated rule, rather than a pile they must reverse-engineer.

**Why this priority**: This is the defect. Every other symptom in the Evidence
table follows from there being no stated admission test.

**Independent Test**: Read `skills/README.md` and `.agents/skills/`. The rule is
stated, and every directory present satisfies it.

**Acceptance Scenarios**:

1. **Given** the consolidated repository, **When** a contributor lists
   `.agents/skills/`, **Then** they find exactly the four bound house standards
   plus the CLI-owned `speckit-*` directories.
2. **Given** `skills/README.md`, **When** a contributor asks why a skill is
   present, **Then** the admission test answers it without recourse to history.
3. **Given** `skills-lock.json`, **When** a contributor asks where a skill came
   from, **Then** the `source` field names a public repository and a release tag
   that resolves.
4. **Given** a contributor wanting to remove a vendored skill, **When** they
   consult `skills/README.md`, **Then** a removal procedure exists.

### User Story 2 - The instruction surface stops asserting things that are not true (Priority: P1)

`AGENTS.md`, `CLAUDE.md`, and `docs/plans/000-repository-foundation.md` describe
the skills mechanism accurately, including where the description no longer
matches what a checkout actually contains.

**Why this priority**: Same class of defect as slice S061, and the same
principle governs it. An instruction file that misstates the repository is a
P-9 defect on the surface agents read first.

**Independent Test**: Read each rewritten passage against the Evidence table.
Every claim is either verifiable on a fresh checkout or is explicitly marked as
a historical record.

**Acceptance Scenarios**:

1. **Given** `AGENTS.md` and `CLAUDE.md`, **When** a reader follows their
   description of the per-agent symlink views, **Then** what they find matches
   what was described, including the case where no symlinks exist.
2. **Given** `docs/plans/000-repository-foundation.md`, **When** a reader
   reaches the "Thirty-five skills" sentence, **Then** it is corrected and
   states what superseded it.
3. **Given** `skills/README.md`, **When** a reader reaches the known gap, **Then**
   it does not assert an unmet deadline that passed 50 slices ago.

### User Story 3 - The set cannot drift unobserved again (Priority: P1)

A structural gate runs in the ordinary check set and fails when disk, git, and
the lock file disagree.

**Why this priority**: Without it the prune re-rots. E-04 is the proof: a skill
sat in the lock, on disk, and uncommitted since the founding commit, and no
person and no check noticed for the whole life of the project.

**Independent Test**: Introduce each of the three disagreements in turn and
observe the gate fail for the right reason, then revert and observe it pass.

**Acceptance Scenarios**:

1. **Given** a lock entry with no directory, **When** the gate runs, **Then** it
   fails and names the entry.
2. **Given** a vendored directory with no lock entry, **When** the gate runs,
   **Then** it fails and names the directory.
3. **Given** a vendored file that git does not track, **When** the gate runs,
   **Then** it fails and names the file. This is the E-04 case.
4. **Given** a vendored file git tracks that is absent from the working tree,
   **When** the gate runs, **Then** it fails and names the file.
5. **Given** a consistent tree, **When** the gate runs, **Then** it passes and
   reports what it checked.
6. **Given** the `speckit-*` directories, **When** the gate runs, **Then** it
   does not require lock entries for them, and does not report them as absent
   from the working tree it never walked.
7. **Given** a `skills-lock.json` with trailing content after the document,
   **When** the gate runs, **Then** it exits 2 rather than reading the leading
   object and passing.

### User Story 4 - The bound standards are the current ones (Priority: P2)

The three standards constitution P-8 binds are present, current, and pristine.

**Why this priority**: P-8 binds Bash, PowerShell, and Markdown symmetrically.
Before this slice, PowerShell's standard was vendored but stale, Markdown's was
vendored but stale, and Bash's was absent entirely.

**Independent Test**: Each vendored tree is byte-identical to the corresponding
v1.11.0 release archive.

**Acceptance Scenarios**:

1. **Given** the vendored set, **When** compared against the v1.11.0 archives,
   **Then** every file matches byte for byte.
2. **Given** the refreshed PowerShell checker, **When** `cargo xtask wrappers`
   runs, **Then** it passes, as E-12 measured in advance.
3. **Given** P-8's three named languages, **When** a reader looks for each
   standard, **Then** all three are present.

### Edge Cases

- A vendored skill directory containing no `SKILL.md` is malformed; the gate
  treats a missing `SKILL.md` as a failure, because `skillPath` names it.
- The gate must not fail on a repository where `.agents/skills/` is absent
  entirely; that is a different, louder problem, and reporting it as lock drift
  would misdirect.
- Assertion 3 uses git's own index rather than `.gitignore` parsing, so it
  catches exclusion by any mechanism, not only the one that caused E-04.
- Dropping `shruggie-html` and `rust-skills` removes two of the three
  never-reproducing hashes, and re-vendoring `shruggie-powershell` from
  canonical bytes resolves the third. This is a consequence, not a goal, and it
  is recorded so a later reader does not mistake the disappearance for a
  cover-up.

## Requirements *(mandatory)*

### The admission test and its result

A skill is admitted to `.agents/skills/` only if a named constitution principle
binds this repository to it, or a repository gate executes it. Both conditions
are checkable against a file and a line, which is what makes the test usable on
a skill nobody has seen before. Plausible usefulness is not a qualification.

Applied to the 36 entries in `skills-lock.json`, three survive and one is added,
for a final set of four.

| Skill | Admitted by | Action |
| --- | --- | --- |
| `shruggie-powershell` | P-8 names PowerShell; `xtask/src/wrappers.rs:226-243` executes its checker | Re-vendor |
| `shruggie-markdown` | P-8 names Markdown | Re-vendor |
| `shruggie-bash` | P-8 names Bash; closes the gap of E-18 | Vendor new |
| `shruggie-speckit` | Drives the mandated spec-kit workflow; cited at `specs/066-steam-identity-presence/spec.md:29` | Re-vendor for provenance |

The remaining 33 are removed. Each is named here rather than summarized,
because a removal that is not recorded is the governance form of an uncounted
discard:

```text
architect  baseline-restorer  boy-scout-rule
code-review  debug  develop
document  explain  explicit-configuration
find-skills  fix  gh-fix-ci
legacy-code-safety  optimize  orthogonality-principle
plan  professional-honesty  project-memory
proof-of-work  refactor  review
rust-best-practices  rust-design-review  rust-skills
shruggie-graph-memory  shruggie-html  silent-execution
simplicity-principles  solid-principles  structural-design-principles
test  token-efficiency  traffic-analysis-pcap
```

`shruggie-docs`, `shruggie-graph-memory`, and `shruggie-html` are in the same
upstream and the same brand and are declined: no principle binds them and no
gate runs them. Admitting on brand alone would restate the failure this slice
corrects, with a different brand than last time.

### Functional Requirements

- **FR-001**: `.agents/skills/` MUST contain exactly four vendored skills after
  this slice: `shruggie-bash`, `shruggie-markdown`, `shruggie-powershell`, and
  `shruggie-speckit`, plus the CLI-owned `speckit-*` directories, untouched.
- **FR-002**: Each of the four MUST be byte-identical to its archive in the
  `shruggietech/skills` v1.11.0 release, verified against that release's
  published `SHA256SUMS.txt` at the point of download.
- **FR-003**: No vendored file may be hand-edited after vendoring, including for
  text hygiene. E-21 establishes that none is needed.
- **FR-004**: Each of the four MUST be checked against constitution P-1 before
  it lands, and the check recorded, as `AGENTS.md` requires of any vendored
  skill.
- **FR-005**: The 33 remaining vendored skills MUST be removed from
  `.agents/skills/` and from `skills-lock.json`.
- **FR-006**: Every surviving `skills-lock.json` entry MUST carry
  `source: "shruggietech/skills@v1.11.0"`. The four-field entry schema MUST NOT
  gain or lose a field, because it is owned by an external tool this repository
  does not contain.
- **FR-007**: `skills-lock.json` `computedHash` MUST be recomputed for all four
  entries, since all four sets of bytes change or are new.
- **FR-008**: A new `cargo xtask skills` subcommand MUST assert that every lock
  entry has a directory; that every non-`speckit-*` directory under
  `.agents/skills/` has a lock entry; and that the working tree and git's index
  carry the same vendored files.
- **FR-008a**: The working-tree/index comparison MUST run in **both**
  directions. Present-but-untracked means a clone would not receive a file the
  author has; tracked-but-absent means a clone receives a file the author no
  longer has. Both are real drift and only the first was specified originally.
  Added after review of pull request #200, which demonstrated the gate passing
  a tree carrying the second while reporting a silently smaller file count as
  agreement.
- **FR-008b**: The two views MUST cover the same set of paths, so the
  comparison cannot invent a disagreement. `speckit-*` is excluded from both,
  and a file sitting directly in `.agents/skills/` is included in both.
- **FR-008c**: The lock reader MUST require end of input after the document. A
  lock followed by a second value or by trailing garbage is a file no JSON
  reader would accept, and accepting it contradicts the fail-closed contract
  FR-009's rationale rests on. Added after review of pull request #200.
- **FR-009**: `cargo xtask skills` MUST NOT verify hashes.
- **FR-010**: `cargo xtask skills` MUST be reachable from `cargo xtask ci` and
  from `.github/workflows/ci.yml`, following the existing per-subcommand step
  pattern.
- **FR-011**: Each of the three assertions in FR-008 MUST be demonstrated
  failing for a real reason, via a deliberate and reverted regression, before
  being reported as passing.
- **FR-012**: `.gitignore` line 3's bare `debug` MUST be anchored so it cannot
  exclude a path segment named `debug` elsewhere in the tree.
- **FR-013**: `skills/README.md` MUST state the admission test, document a
  removal procedure, name the single upstream, and drop both the
  generic-name-collision paragraph and the expired known gap, neither of which
  describes the repository any longer.
- **FR-014**: `AGENTS.md` and `CLAUDE.md` MUST describe the per-agent view
  mechanism truthfully, including that a checkout may carry no symlinks at all.
- **FR-015**: `docs/plans/000-repository-foundation.md` MUST correct its skill
  count and note what superseded the original vendoring decision, as a
  historical record rather than a silent edit.
- **FR-016**: The slice MUST record, in `changelog.d/`, that the three
  long-standing lock-hash divergences are resolved and by what mechanism.

### Key Entities

- **Vendored skill**: a directory under `.agents/skills/` containing `SKILL.md`
  and optional assets and scripts, admitted under the test in FR-001's rationale
  and recorded in `skills-lock.json`.
- **Lock entry**: the four fields `source`, `sourceType`, `skillPath`,
  `computedHash`, keyed by skill name.
- **CLI-owned skill**: a `speckit-*` directory, regenerated by the spec-kit CLI,
  recorded in `.specify/integrations/*.manifest.json`, and out of scope for both
  the prune and the gate.

### Out of scope

- **OOS-001**: The 10 `speckit-*` directories. The spec-kit CLI owns them, and
  `skills-lock.json` says so in its own `note` field.
- **OOS-002**: `docs/fragcap-specification.md`. No specification section
  describes the agent skills mechanism, so every changelog fragment here carries
  `spec-impact: none` and the P-11 version lock-step is undisturbed.
- **OOS-003**: Hash verification in the gate, and any recomputation of the
  dropped entries' hashes. See the clarification and E-06.
- **OOS-004**: Wiring the vendored Bash checker into `cargo xtask wrappers` to
  replace the hand-rolled `check_bash` in `xtask/src/wrappers.rs`. Measured to
  be safe (`scripts/fragcap.sh` passes it cleanly, ShellCheck included, on
  2026-08-22) but it is a change to what the gate enforces, not to what this
  slice consolidates. Filed as issue #199 with that measurement attached.
- **OOS-005**: `fragcap.code-workspace:25`, which hides `**/.agents/skills` from
  the editor's file explorer. Reasonable at 46 directories and arguably
  unnecessary at 14, but it is one contributor's editor configuration and
  changing it serves nobody's correctness.
- **OOS-006**: Restoring or generating the per-agent symlink views that
  `AGENTS.md` describes. This slice makes the description truthful; making the
  mechanism run is a separate question about tooling this repository does not
  contain.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `.agents/skills/` contains 14 directories: 4 vendored, 10
  `speckit-*`. `skills-lock.json` contains exactly 4 entries.
- **SC-002**: `git ls-files .agents/skills` and the on-disk tree agree exactly.
  No file is present but untracked, and none is tracked but absent.
- **SC-003**: Every one of the four vendored trees is byte-identical to its
  v1.11.0 archive.
- **SC-004**: `cargo xtask ci` passes in full, in the foreground, watched to
  completion.
- **SC-005**: `cargo xtask wrappers` passes with the refreshed checker, as E-12
  predicted.
- **SC-006**: Each `cargo xtask skills` assertion has been observed failing for
  its own reason and then passing, in both directions of the working-tree/index
  comparison, plus the fail-closed exit-2 path.
- **SC-007**: Every claim in the Evidence table is either still true at commit
  time or has been corrected in place, with no claim carried forward on memory.
- **SC-008**: The tracked file count under `.agents/skills/` falls from 382 to 42: 32 across
  the four vendored trees, plus the 10 `speckit-*` files. 348 tracked files are
  removed.

## Assumptions

- The `shruggietech/skills` release assets are the authoritative distribution of
  those skills. The published `SHA256SUMS.txt` is trusted as the integrity
  reference; this is the same trust model already applied to every crate the
  workspace resolves from a registry.
- Pinning the lock's `source` to a release tag is an improvement over an
  untagged repository name and does not conflict with the external tool that
  owns the schema, because the tag is carried inside the existing `source`
  string rather than in a new field.
- Codex, which reads `.agents/skills/` directly, loses access to 33 skills. This
  is accepted: none was fragcap-specific, all remain available from their own
  upstreams, and their presence here was never the product of a decision about
  this project.
