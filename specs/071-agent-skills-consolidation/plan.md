# Implementation Plan: Agent skills consolidation

**Branch**: `071-agent-skills-consolidation` | **Date**: 2026-08-22 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/071-agent-skills-consolidation/spec.md`

## Summary

Reduce `.agents/skills/` from 36 vendored skills to the four the constitution
actually binds, taken pristine from one tagged upstream; delete the 33 that
nothing binds or executes; correct four instruction files that describe a
repository which no longer exists (and, in one case, never did); anchor the
`.gitignore` pattern that has silently excluded a vendored skill since the
founding commit; and add `cargo xtask skills`, a structural check that fails
when disk, git, and `skills-lock.json` disagree.

The highest-risk step was measured before planning rather than after. E-12
establishes that the refreshed PowerShell checker, which gates continuous
integration, passes `scripts/Invoke-FragCap.ps1` with output identical to the
stale copy it replaces. That retires the one failure mode that could have forced
a mid-slice halt.

## Technical Context

**Language/Version**: Rust 1.82 (workspace MSRV) for the new xtask module; Markdown and JSON elsewhere
**Primary Dependencies**: None added. The gate uses `std` plus the `git` binary already required by the repository's workflow
**Storage**: `skills-lock.json`, a 4-entry JSON object after this slice
**Testing**: `cargo xtask ci` in full; `cargo xtask wrappers` as the load-bearing member for the vendoring step; `cargo xtask lint` for the prose changes; unit tests in the new module
**Target Platform**: Any host that can run the task runner; the gate is platform-neutral
**Project Type**: Repository tooling and governance surface, not shipped software
**Performance Goals**: Not applicable. The gate reads a directory tree and one `git ls-files` invocation
**Constraints**: `.github/workflows/ci.yml` is a pinned artifact; the lock's entry schema is owned by an external tool absent from this repository
**Scale/Scope**: 33 directories and 348 tracked files removed, 4 trees vendored (32 files), 4 instruction files corrected, 1 xtask module added

## Phase 0: Research (complete)

Four questions were open at kickoff. All are answered, three of them by
measurement rather than reasoning.

**1. Does refreshing the PowerShell standard break the gate?** No. Both the
currently vendored checker and the v1.11.0 checker report
`scripts/Invoke-FragCap.ps1` compliant, with byte-identical output and exit code
0. The 48-line and 141-line drifts between the two checkers are elsewhere. This
was run before any file in the repository was touched, so the result is
attributable. Recorded as E-12.

**2. Is the upstream acceptable under the constitution's licensing section?**
Yes. `shruggietech/skills` is Apache-2.0, which is fragcap's own licence and is
inside the allowlist. It is public, actively maintained, and cuts tagged
releases with per-skill archives and a published `SHA256SUMS.txt`. All four
archives this slice vendors were verified against that file and reported `OK`
(E-20).

**3. Does pristine vendoring conflict with `CONVENTIONS.md`?** No, and this is
the finding that resolves a five-slice-old inconsistency. Every file in all four
archives is already free of em-dashes, en-dashes, CRLF, and byte-order marks
(E-21). The original 2026-08-06 vendoring hand-edited its copies for exactly
this reason, and that editing is the most likely cause of the three
never-reproducing lock hashes (E-06). Taking current upstream unmodified removes
both the need and the divergence.

**4. Is `traffic-analysis-pcap` admissible on domain relevance?** No. It is a
capture-the-flag forensics playbook: credential harvesting, TLS decryption,
covert-channel detection, DNS-tunneling heuristics. It routes to four skills
never vendored here, so four of its own cross-references are broken. fragcap
writes pcapng and attributes flows; it has no analysis surface this serves. It
also carries em-dashes in its heading, which only survived because `xtask lint`
excludes the vendored trees (E-08, E-09).

## Constitution Check

*GATE: passed before Phase 0. Re-checked after design; still passing.*

| Principle | Bearing | Verdict |
| --- | --- | --- |
| P-1 Passive Observation Only | Four skills are admitted to the repository and must be checked against the technique denylist before landing, as `AGENTS.md` requires of any vendored skill. Three are authoring standards for shell and Markdown; one is a workflow protocol. None teaches interception, injection, hooking, or image modification. | **Gated** by FR-004 |
| P-2 Core Stays Platform-Neutral | No crate changes. The new xtask module is not in the dependency graph `cargo xtask deps` governs. | Not engaged |
| P-3 Capture And Attribution Stay Separate | No capture or attribution code touched. | Not engaged |
| P-4 No Silent Loss | Reinterpreted for this surface, as S061 did: 33 skills are removed, and each is enumerated by name in the changelog rather than summarized away. A removal that is not recorded is the governance form of an uncounted discard. | **Satisfied by design** |
| P-5 Compatibility Outranks Richness | The lock's four-field entry schema is owned by an external tool this repository does not contain. The release tag is carried inside the existing `source` string rather than in a new field, so a tool that reads the file still parses it. | **Satisfied by design**, FR-006 |
| P-6 Glossary First | No new user-facing term. "Vendored skill", "lock entry", and "CLI-owned skill" are defined in the spec's Key Entities and are internal to this surface. | Satisfied |
| P-7 Wrappers Stay Thin | `scripts/fragcap.sh` and `scripts/Invoke-FragCap.ps1` are unchanged. The refreshed checker was measured against both before planning. | Satisfied |
| P-8 House Standards Apply | **The principle this slice exists to serve.** P-8 binds Bash, PowerShell, and Markdown symmetrically. Before this slice PowerShell's standard was stale, Markdown's was stale, and Bash's was absent despite a written commitment to vendor it. All three are now present at current upstream. | **Primary driver** |
| P-9 The Instrument Does Not Lie | Applied to the instruction surface, as S061 established. `AGENTS.md` and `CLAUDE.md` assert a symlink mechanism this checkout does not have; `docs/plans/000-repository-foundation.md` asserts a count that is wrong; `skills/README.md` asserts a deadline that expired 50 slices ago. Each is corrected against a measured claim, not rewritten from memory. | **Primary driver** |
| P-10 One Path To A Target | No target model changes. | Not engaged |
| P-11 The Specification Describes What Shipped | No specification section describes the agent skills mechanism, so every fragment carries `spec-impact: none` and the version lock-step is undisturbed (OOS-002). | Satisfied |

Pinned artifacts: `.github/workflows/ci.yml` gains one step (FR-010). The
constitution permits this only with a dated decision recorded in the changelog;
the fragment is task T032. `scripts/**` is untouched.

No violations. Complexity Tracking omitted.

## Design

### 1. The admission test, stated

> `.agents/skills/` carries the ShruggieTech house standards this repository's
> constitution binds, sourced from one upstream, and nothing else. `speckit-*`
> is CLI-owned and out of scope.

Operationally: a skill is admitted only if a named constitution principle binds
this repository to it, or a repository gate executes it. Both conditions are
checkable by a reader against a file and a line, which is what makes the test
usable on a skill nobody has seen before.

Applied, this yields four:

| Skill | Admitted by | Action |
| --- | --- | --- |
| `shruggie-powershell` | P-8 names PowerShell; `xtask/src/wrappers.rs:226-243` executes its checker | Re-vendor |
| `shruggie-markdown` | P-8 names Markdown | Re-vendor |
| `shruggie-bash` | P-8 names Bash | Vendor new |
| `shruggie-speckit` | Drives the mandated spec-kit workflow; cited at `specs/066-steam-identity-presence/spec.md:29` | Re-vendor for provenance; content already current (E-13) |

`shruggie-docs`, `shruggie-graph-memory`, and `shruggie-html` are in the same
upstream and the same brand, and are declined. No principle binds them and no
gate runs them. Admitting on brand alone is the failure this slice corrects,
only with a different brand than last time.

### 2. Vendoring is pristine, and that is load-bearing

Archives are downloaded from the v1.11.0 release, verified against its
`SHA256SUMS.txt` **before** extraction, and copied in unmodified.

The rule matters because of what happened last time. The 2026-08-06 vendoring
edited its copies to satisfy text hygiene, and three lock hashes have never
reproduced since. An edited vendored copy is no longer the upstream standard it
claims to be, and the hash mismatch is the symptom rather than the disease. E-21
shows the edit is unnecessary now, so the rule costs nothing to adopt.

Consequence worth naming: dropping `rust-skills` and `shruggie-html` removes two
of the three divergences, and re-vendoring `shruggie-powershell` from canonical
bytes resolves the third. All three are gone after this slice. That is a
side effect, not a goal, and FR-016 requires it be recorded so a later reader
does not read the disappearance as a tidy-up of inconvenient evidence.

### 3. The lock file

36 entries to 4. Each surviving entry keeps its four fields and takes
`source: "shruggietech/skills@v1.11.0"`.

**Resolves CHK011.** `computedHash` is recomputed for all four, because all four
sets of bytes change or are new. The algorithm is the one a prior session
derived empirically: SHA-256 over each file in the skill directory, sorted by
relative path, hashing the relative path bytes followed by the content bytes,
with CRLF normalized to LF for text and binary content hashed raw. That session
reproduced 32 of the 35 entries then present, across single-file,
nested-directory, mixed-case, binary-carrying, and CRLF-on-disk shapes.

The confidence this carries, stated plainly: it is empirical, not authoritative.
The tool that normally writes the file is not present in this repository and is
never named by it. Three things bound the risk. The gate does not read hashes
(FR-009), so a wrong value cannot produce a false green. The real integrity
anchor for this slice is the publisher's `SHA256SUMS.txt`, verified at download
and recorded in the decisions fragment with the digests. And what would falsify
the algorithm is specific and cheap to run: the external skills CLI regenerating
the file and producing different values for these four entries. If that happens,
the CLI's values are correct and ours are replaced.

### 4. The gate

New `xtask/src/skills.rs`, exposing `run(&Path) -> io::Result<usize>`, the same
shape as `wrappers::run`. It returns the count of failed checks; the caller maps
`Ok(0)` to success, `Ok(n)` to exit 1, and `Err` to exit 2, which is the house
contract documented at `xtask/src/main.rs:9-14`: passed, failed, and could not
run are three different facts.

Three assertions, each tied to a drift class observed in this repository rather
than an imagined one:

1. **Every lock entry has a directory**, and that directory contains the file
   `skillPath` names. Catches an entry left behind by a deletion.
2. **Every non-`speckit-*` directory has a lock entry.** Catches a skill copied
   in without provenance.
3. **Every file under every vendored skill is tracked by git.** Catches E-04:
   `.agents/skills/debug/` has been on disk and in the lock, and uncommitted,
   since the founding commit, and neither a person nor a check noticed for the
   life of the project.

**Resolves CHK016.** Assertion 3 asks git, not `.gitignore`. It runs
`git ls-files -- .agents/skills` and compares that set against a directory walk.
Reparsing ignore rules would reimplement precedence rules that git already
implements and that this repository has already been caught getting wrong. More
importantly, the index is the thing that actually determines what a clone
receives, which is the property under test. If git is not on `PATH`, or the
command fails, or the directory is not a work tree, the function returns `Err`
and the caller exits 2. It never degrades to a pass. This is the same posture as
`neutral` and `msrv`, which exit 2 rather than 0 when they cannot run, and it is
the direct application of the module-level rule that a check which did not run
must not look like one that passed.

**Resolves CHK017 without a stale list.** CLI-owned directories are recognized
by the `speckit-` name prefix, which is the same rule `skills-lock.json`'s own
`note` field states and which the spec-kit CLI's manifests already use. No
enumeration to go stale.

Reporting mirrors `wrappers`: `println!("skills: OK   ...")` per passing
assertion, `eprintln!("skills: FAIL ...")` naming the offending entry, directory,
or file. A gate that fails without naming what failed is a gate someone disables.

Unit tests cover the three assertions against constructed fixtures plus the
`speckit-` exclusion, so the logic is testable without mutating the real tree.
FR-011 additionally requires each assertion be demonstrated failing against the
**real** tree by a deliberate, reverted regression, because a test over a fixture
proves the function and not the wiring. This is the discipline S070 established.

### 5. Wiring

`xtask/src/main.rs` gains `mod skills;`, a `"skills"` dispatch arm modelled on
`"wrappers"` (lines 137-151), a `ci` step modelled on lines 340-351, and a
`USAGE` line. Placed adjacent to `wrappers` throughout, since the two check
neighbouring properties.

`.github/workflows/ci.yml` gains one step mirroring line 76. That file is a
pinned artifact; the dated decision is T024.

### 6. The `.gitignore` fix

Line 3's bare `debug` becomes `/debug`. The pattern arrived from a Rust
`.gitignore` template written for an older Cargo layout; today Cargo writes
`target/debug`, and `target` on line 4 already covers it. Unanchored, it excludes
any path segment named `debug` anywhere in the tree, which is how a vendored
skill went uncommitted for the life of the project.

Anchoring rather than deleting: the line is vestigial for current Cargo, but
proving that for every Cargo configuration is not this slice's job, and `/debug`
preserves the original intent while removing the collateral damage. The latent
bug outlives the dropped skill, so the fix is not made moot by the prune.

### 7. Instruction surface

**Resolves CHK021.** Three files describe this mechanism at three depths, and
the fix is to give each one job rather than to write a fourth partial
description.

| File | Role after this slice |
| --- | --- |
| `skills/README.md` | **The procedure.** Owns the admission test, the add procedure it already has, and the new removal procedure. The one place a contributor is sent. |
| `AGENTS.md` / `CLAUDE.md` | **The policy summary.** Two paragraphs: what is vendored and why, the P-1 filter, and a pointer to `skills/README.md`. No procedure duplicated. |
| `docs/plans/000-repository-foundation.md` | **The historical record.** Corrected in place with a note naming what superseded it. Not updated to describe the current state; it is a record of a decision made on 2026-08-06. |

The removal procedure belongs in `skills/README.md` because that file already
owns the add procedure, and a remove step that lives anywhere else guarantees
the two drift. Issue #197 asked for exactly this symmetry.

`AGENTS.md` and `CLAUDE.md` also stop asserting that `.claude/skills/` and
`.cursor/skills/` are populated with symlinks to the vendored set. They are not,
on this checkout, and `git ls-files -s` finds no symlink anywhere in the
repository (E-17). The corrected text describes the mechanism as the intended
one, states that a checkout may carry no per-agent views at all, and does not
claim the repository generates them. Making the mechanism actually run is
OOS-006.

### 8. What needs no fix, and why that is recorded

The two broken cross-references issue #197 reported (E-14) are in
`legacy-code-safety` and `gh-fix-ci`, both of which are dropped. The issue's
item 3 is therefore discharged by deletion rather than by repair. This is stated
explicitly in the spec and the changelog so the item is visibly closed rather
than quietly skipped, which is the P-4 reading this slice applies to governance.

### 9. Ordering, and why it is not arbitrary

Vendor before deleting, and run `cargo xtask wrappers` between the two. The
PowerShell checker is live infrastructure: for the window between deleting the
old copy and landing the new one, the gate has no checker. Vendoring first means
that window never opens, and running `wrappers` immediately after means a
regression is attributable to the refresh alone rather than to a tree that has
also lost 33 directories.

## Project Structure

### Documentation (this feature)

```text
specs/071-agent-skills-consolidation/
├── spec.md
├── plan.md
├── tasks.md
└── checklists/
    ├── requirements.md
    └── consolidation.md
```

No separate `research.md`: Phase 0 produced four answers short enough to live in
this file, matching the precedent of S062 and S070. No `data-model.md` or
`contracts/`: the only structured artifact is a 4-entry lock file whose schema is
unchanged and documented in the spec's Key Entities. No `quickstart.md`: the
Verification section below is the runnable guide.

### Files changed

```text
.agents/skills/                      33 directories removed, 4 vendored
skills-lock.json                     36 entries -> 4, source re-pointed
xtask/src/skills.rs                  new
xtask/src/main.rs                    mod, dispatch arm, ci step, USAGE line
.github/workflows/ci.yml             one step (pinned artifact)
.gitignore                           line 3, debug -> /debug
skills/README.md                     admission test, removal procedure
AGENTS.md                            Skills section
CLAUDE.md                            Claude Code specifics, skills paragraph
docs/plans/000-repository-foundation.md   count correction
changelog.d/S071-*.{removed,changed,fixed,decisions}.md
```

**Structure Decision**: The gate lives in `xtask` beside `wrappers`, not in a
shell script, because the repository's checks live in the task runner by design
(`xtask/src/main.rs:3-7`) and because a check that needs nothing installed beyond
the toolchain is one every contributor can actually run.

## Verification

Foreground, watched to completion. Never backgrounded; a buffered runner makes a
background run indistinguishable from a dead one.

```bash
cargo xtask ci
```

Load-bearing members, in order of the risk each carries:

1. **`cargo xtask wrappers`**, run immediately after vendoring and before any
   deletion. E-12 predicts it passes; the run is what confirms it on the real
   tree. A failure here is attributable to the refresh alone.
2. **`cargo xtask skills`**, with each of its three assertions demonstrated
   failing against the real tree for its own reason, then reverted and confirmed
   passing (FR-011).
3. **`cargo xtask lint`**, the mechanical gate that applies to a prose change.
4. Everything else, which must stay green to prove nothing else moved.

Then the structural read-back:

```bash
ls -d .agents/skills/*/ | wc -l                    # 14
ls -d .agents/skills/*/ | grep -vc speckit-        # 4
git ls-files .agents/skills | wc -l                # 4 trees + 10 speckit files
git status --porcelain                             # clean
```

And the byte-level one: each vendored tree compared against its extracted
v1.11.0 archive, expecting no differences (SC-003).

Finally the manual read-back, which is the S061 discipline: every rewritten
sentence in `skills/README.md`, `AGENTS.md`, `CLAUDE.md`, and
`docs/plans/000-repository-foundation.md` checked against the spec's Evidence
table at commit time, not against the tree as it was when the spec was written
(SC-007). A claim that no longer holds is corrected in place rather than shipped.
