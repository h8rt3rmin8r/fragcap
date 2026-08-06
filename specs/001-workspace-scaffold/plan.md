# Implementation Plan: Workspace Scaffold, Licensing, and CI Skeleton

**Branch**: `feat/workspace-scaffold` | **Date**: 2026-08-06 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from `specs/001-workspace-scaffold/spec.md`

## Summary

Create the Cargo workspace, eight crate skeletons with the dependency edges the
architecture of record prescribes, a task runner carrying the repository's own
checks, and six workflow files. Nothing here implements a capability. The
deliverable is a structure that compiles, a check set that runs, and an honest
statement of which checks are meaningful yet.

## Technical Context

**Language/Version**: Rust, edition 2021. Build toolchain pinned at 1.96.0;
minimum supported version declared as 1.82 and checked separately. \
**Primary Dependencies**: None in this slice. Crate skeletons take no external
dependencies; Appendix A's dependency assignments land as the slices that need
them arrive. \
**Storage**: N/A \
**Testing**: `cargo test --workspace --locked`, plus tests covering the task
runner's own check logic. \
**Target Platform**: `x86_64-pc-windows-msvc` for the binary; `fragcap-core`
additionally built for a non-Windows target to prove P-2. \
**Project Type**: Cargo workspace, multi-crate library plus one binary. \
**Performance Goals**: N/A \
**Constraints**: No git remote exists, so no workflow can execute. The npcap
software development kit is absent and not required, since no crate links
against the capture library until S09. \
**Scale/Scope**: 8 crates, 1 task runner, 6 workflows.

## Constitution Check

| Principle | Bearing on this slice | Gate |
| --- | --- | --- |
| P-1 Passive observation | No technique is exercised. No dependency in this slice provides a denylisted capability. | Pass, vacuously |
| P-2 Core platform-neutral | Directly exercised. `fragcap-core` takes no dependency, and CI builds it for a non-Windows target. | **Enforced by check** |
| P-3 Capture/attribution separate | The crate split creates the seam. `fragcap-capture` and `fragcap-attr` are distinct crates and neither depends on the other. | Pass by construction |
| P-4 No silent loss | No discard path exists yet. Applies to this slice as reporting discipline: a check that did not run is not reported as passing. | **Enforced by FR-018** |
| P-5 Compatibility | No output format exists yet. | N/A |
| P-6 Glossary first | This slice introduces no new domain vocabulary; all terms come from the architecture of record. | Pass |
| P-7 Wrappers stay thin | No wrappers in this slice. The task runner is not a wrapper; it is a workspace member. | N/A |
| P-8 House standards | Directly exercised. The conventions check is the mechanism. | **Enforced by check** |
| P-9 Instrument does not lie | Binds the reporting of this slice's own verification. See the honesty gate below. | **Enforced by FR-012a, FR-018** |

**Honesty gate.** Three things in this slice are scaffolded rather than
exercised, and each must be labelled as such wherever a reader encounters it:
the six workflows (no remote), the minimum-toolchain check (no dependencies
yet), and the capture library acquisition step (no crate links it yet). Marking
any of them as verified would be the P-9 failure mode applied to the project's
own build.

**Post-design re-evaluation**: no new violations. One deviation is recorded
below and is compliant with the stated rules.

## Key Decisions

**D-1. The facade depends on the core crate directly.** The dependency diagram
in section 8.3 draws edges from the facade to the five mid-level crates and not
to core, but a facade that re-exports core types needs core as a direct
dependency; Rust cannot re-export through a transitive one without routing
through an intermediary's public surface, which is fragile and ugly. The added
edge violates neither stated rule: it is not a dependency on the binary crate,
and it is not a sibling-level dependency. Recorded as a diagram omission to
promote to section 8.3 at the next specification revision.

**D-2. Build toolchain pinned at current, minimum declared separately.**
Resolved in the spec's Clarifications. `rust-toolchain.toml` pins 1.96.0;
`workspace.package.rust-version` declares 1.82; a dedicated check builds at
1.82. Currently vacuous, labelled as such.

**D-3. The conventions check is a task runner subcommand.** Resolved in the
spec's Clarifications. Written in Rust, covered by tests, requires no shell.

**D-4. Dependency direction is verified from workspace metadata, not by
reading source.** The task runner reads `cargo metadata` and asserts the edge
set against a declared expectation. A structural rule enforced by prose decays;
enforced by a test it does not. This also makes the rule legible: the expected
graph is written down in one place.

**D-5. Crate skeletons take no external dependencies.** Appendix A assigns
dependencies per crate, but pulling them in before the code that uses them
would produce an unused-dependency surface, an unjustified lockfile, and a
license audit over crates the project does not yet use. Each slice adds what it
needs.

**D-6. Platform neutrality is proved on a Linux runner natively rather than by
cross-compiling.** CI already runs on both Linux and Windows. Building
`fragcap-core` on the Linux job is the same proof with no cross-toolchain
setup. The task runner additionally offers a local cross-target check for
contributors who have the target installed, and says plainly when they do not
rather than passing silently.

**D-7. Platform neutrality needs two checks, not one.** Discovered while
demonstrating that the checks can fail, which is the only reason it was caught.

`cargo xtask neutral` builds `fragcap-core` for a target with no capture
backend. That is what section 9.3 literally asks for, and it passes. But adding
`windows-sys` to core as a dependency, conditionally or unconditionally, **does
not make that build fail**: platform crates are themselves internally
cfg-gated and compile to nothing off-platform. The build succeeds while P-2 has
been violated.

So the build check proves core *compiles* portably. It does not prove core has
no platform-specific dependency, which is what FR-005 and data model rule V-4
actually require. The stronger property needs a manifest check, and
`cargo xtask deps` now asserts that `fragcap-core` has no dependencies of any
kind, across `[dependencies]` and every `[target.'cfg(...)'.dependencies]`
table.

Both checks are retained because they fail on different things. Recorded here
because the gap is invisible from the outside: a reader seeing a green
neutrality check would reasonably conclude core was dependency-free, and until
this was tested against known-bad input, so would this plan.

## Project Structure

### Documentation (this feature)

```text
specs/001-workspace-scaffold/
├── spec.md
├── plan.md              This file
├── research.md          Phase 0: decisions and alternatives
├── data-model.md        Phase 1: the crate graph as the structural model
├── contracts/
│   └── xtask-cli.md     Phase 1: the task runner's command surface
├── quickstart.md        Phase 1: how to validate this slice
├── tasks.md             Phase 2 (generated by /speckit-tasks)
└── checklists/
    └── requirements.md
```

### Source Code (repository root)

```text
Cargo.toml               Workspace manifest, section 21.4
Cargo.lock               Committed; the workspace is a shipped artifact
rust-toolchain.toml      Pinned build toolchain, section 24.1
deny.toml                Dependency license allowlist, section 20.4
crates/
├── fragcap-core/        Types, traits, pipeline. No dependencies.
├── fragcap-profile/     Profile schema and matching
├── fragcap-capture/     Acquisition backends
├── fragcap-attr/        Attribution and process watching
├── fragcap-sink/        Sinks and transports
├── fragcap-steam/       Steam integration
├── fragcap/             Facade
└── fragcap-cli/         Binary
xtask/                   Repository task runner
profiles/                Bundled game profiles (README only in this slice)
fixtures/                Test capture corpus (README only in this slice)
scripts/                 Shell wrappers and linters (README only in this slice)
.github/workflows/
├── ci.yml               Format, lint, test, conventions, platform neutrality
├── platform.yml         Windows capture-dependent tests
├── audit.yml            Vulnerabilities and licenses
├── docs.yml             Site build and deploy
├── links.yml            External reference verification
└── release.yml          Artifacts and publication
```

## Dependency Graph

The edge set the task runner asserts. Anything not listed is a violation.

```text
fragcap-cli     -> fragcap
fragcap         -> fragcap-core, fragcap-profile, fragcap-capture,
                   fragcap-attr, fragcap-sink, fragcap-steam
fragcap-capture -> fragcap-core
fragcap-attr    -> fragcap-core
fragcap-sink    -> fragcap-core
fragcap-profile -> fragcap-core
fragcap-steam   -> fragcap-profile
fragcap-core    -> (nothing)
```

## Complexity Tracking

| Item | Why it is here | Simpler alternative rejected because |
| --- | --- | --- |
| Eight crates before any code | The graph is the architecture, and every later slice writes against it. | One crate split later means rewriting every import in seventeen slices. |
| A task runner in slice one | Two checks (conventions, dependency direction) have no other home, and both must exist before code arrives to violate them. | A shell script needs a house standard that is a known missing gap, and cannot be unit tested. |
| Six workflows, none runnable | Section 24.2 names them; writing five later means five chances to forget one. | Writing only `ci` now would leave the other five undated and unclaimed. |
