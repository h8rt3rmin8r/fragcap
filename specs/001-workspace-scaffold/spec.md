# Feature Specification: Workspace Scaffold, Licensing, and CI Skeleton

**Feature Branch**: `feat/workspace-scaffold`

**Created**: 2026-08-06

**Status**: Draft

**Slice**: S01 (specification sections 20, 21, 24; Appendix A; section 8.3)

**Input**: Establish the Cargo workspace and the continuous integration
skeleton that every later slice builds on.

## Overview

This is the first implementation slice. The repository today holds the
specification, the constitution, plan documents, and agent configuration, and
no Rust code whatsoever. S01 creates the structure into which every later
slice writes.

The value delivered is not a feature a user runs. It is the property that
seventeen subsequent slices can each start by writing code into a place that
already exists, compiles, and is checked. The audience is therefore a
contributor, and the measure of success is what a contributor can do on their
first day rather than what an operator can capture.

## Clarifications

### Session 2026-08-06

- Q: How is the declared minimum supported toolchain reconciled with the
  toolchain the project builds with? → A: Pin the build channel to the current
  toolchain, declare the minimum separately, and verify the minimum in a
  dedicated check.
- Q: Where does the repository conventions linter live, and in what language?
  → A: A subcommand of the repository task runner, written in the project's own
  language.

Both were resolved under the autopilot decision policy rather than escalated.
Rationale for each is recorded below and carried into `plan.md`.

**Toolchain.** Two distinct things were conflated in the architecture of
record: the version a consumer must have (a compatibility claim) and the
version the project builds with (a reproducibility control). Pinning the build
channel at the minimum would force every later slice to hold its dependencies
back to versions compatible with a toolchain from 2024, which trades real
capability for a claim that can be checked more cheaply another way. Pinning at
the current toolchain and adding a check that builds at the minimum keeps both
properties and honors FR-012.

Recorded honestly: with no external dependencies yet, that check passes
trivially. It becomes meaningful at S02, when dependencies first enter the
graph. It is scaffolded now precisely so that it is already in place when it
starts to mean something, and its current vacuity is stated rather than
implied to be a verified minimum.

**Conventions linter.** Three considerations pointed the same way. The house
shell standard the architecture of record assumes is a known missing gap
recorded in the reconnaissance notes, so building a required check against an
unavailable standard would block this slice on something outside it. The task
runner is specified as requiring nothing beyond the language toolchain, while a
shell linter needs a shell present on every runner. And a check written in the
project's own language is itself testable, where a shell script is not.

This does not affect the documentation linter, which the architecture of record
separately specifies as a shell script and which belongs to a later slice. The
repository conventions linter and the documentation linter are different
checks.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Build from a clean clone (Priority: P1)

Someone clones the repository, runs one command, and gets a working build
without hunting for a toolchain version, an environment variable, or an
undocumented prerequisite.

**Why this priority**: Nothing else in the slice matters if the workspace does
not build. Every later slice, and every other story here, depends on this
holding.

**Independent Test**: Clone to an empty directory on a machine with `rustup`
and nothing else installed, run the documented build command, and observe a
successful build with no manual intervention.

**Acceptance Scenarios**:

1. **Given** a clean clone and no project-specific tooling installed,
   **When** the contributor runs the documented build command,
   **Then** the toolchain named by the repository is selected automatically and
   the whole workspace compiles.
2. **Given** a built workspace,
   **When** the contributor runs the documented test command,
   **Then** tests execute and pass, including with a locked dependency set.
3. **Given** a clean clone,
   **When** the contributor looks for where a given kind of code belongs,
   **Then** every directory named in the repository layout exists and explains
   its own purpose.

---

### User Story 2 - Mistakes caught before review (Priority: P1)

Someone opens a change that breaks formatting, introduces a lint, breaks the
platform-neutrality rule, or adds a dependency under a forbidden license. The
project tells them, mechanically, rather than a reviewer noticing.

**Why this priority**: Constitution P-8 states that consistency depending on a
reviewer noticing is consistency that decays. The checks are worth more than
the code they check at this stage, because they constrain seventeen slices of
future work.

**Independent Test**: Introduce each violation deliberately in a scratch
commit, run the check set, and confirm each is reported and fails.

**Acceptance Scenarios**:

1. **Given** a change with misformatted code,
   **When** the check set runs,
   **Then** it fails and names the offending file.
2. **Given** a change that makes the core library depend on something
   platform-specific,
   **When** the check set runs,
   **Then** the platform-neutrality check fails.
3. **Given** a change adding a dependency under a copyleft license,
   **When** the audit runs,
   **Then** it fails and names the dependency and its license.
4. **Given** a source file with no license identifier,
   **When** the conventions check runs,
   **Then** it fails and names the file.

---

### User Story 3 - See what will ship (Priority: P2)

Someone preparing a release can see which crates exist, in what order they
must be published, and what a release artifact would contain, without reading
the specification.

**Why this priority**: Release mechanics are not exercised in this slice and
publishing is explicitly out of scope, but the structure that determines them
is created here. Getting the crate graph wrong now is expensive to correct
later, because every later slice writes against it.

**Independent Test**: Inspect the workspace metadata and confirm the crate set
and dependency edges match the architecture of record exactly.

**Acceptance Scenarios**:

1. **Given** the workspace,
   **When** its dependency graph is inspected,
   **Then** it matches the direction rule in the architecture of record, with
   no crate depending on the binary crate and no crate below the facade
   depending on a sibling at its own level.
2. **Given** any crate in the workspace,
   **When** its manifest is inspected,
   **Then** it declares the project license.

### Edge Cases

- **The declared minimum toolchain differs from the toolchain being built
  with.** A minimum-version claim that is never exercised is an unverified
  claim. Either it is checked mechanically or it is not made.
- **A workflow cannot run.** There is no git remote, so no workflow can
  execute during this slice. Workflow correctness is established by static
  validation only, and the distinction must be stated rather than implied.
- **A capture prerequisite is absent.** No crate in this slice links against
  the capture library, so the software development kit that later slices need
  is scaffolded for but not exercised.
- **A placeholder directory is empty.** Version control does not track empty
  directories, so each must carry content that explains its own purpose.
- **The facade re-exports nothing yet.** The facade crate exists before the
  crates it will re-export have any content, and must still compile and pass
  lints.

## Requirements *(mandatory)*

### Functional Requirements

**Workspace structure**

- **FR-001**: The repository MUST define a single workspace whose members are
  every crate under the crates directory plus the repository task runner.
- **FR-002**: The workspace MUST centrally declare the version, edition,
  license, repository, authors, and minimum supported toolchain, and every
  member MUST inherit them rather than restating them.
- **FR-003**: The workspace MUST contain exactly the eight crates named in the
  architecture of record, each with the purpose recorded there.
- **FR-004**: Dependency edges between crates MUST match the direction rule in
  the architecture of record. No crate depends on the binary crate; no crate
  below the facade depends on a sibling at its own level.
- **FR-005**: The core crate MUST depend on nothing platform-specific, no
  input or output library, and no capture library.
- **FR-006**: The repository MUST provide a task runner requiring nothing
  installed beyond the language toolchain, with its task surface declared even
  where individual tasks are not yet implemented.

**Licensing and conventions**

- **FR-007**: Every crate manifest MUST declare the project license.
- **FR-008**: Every source file MUST carry a license identifier as its first
  line, in the comment syntax of its language.
- **FR-009**: The repository MUST declare the permitted dependency license set
  and reject any dependency outside it.
- **FR-010**: No build artifact, distribution artifact, or committed file MUST
  contain the capture driver or its software development kit.

**Toolchain**

- **FR-011**: The repository MUST pin the toolchain it builds with, at the
  current version, so that a local build and an automated build are identical.
- **FR-012**: The minimum supported toolchain version MUST be declared
  separately from the pinned build toolchain, and MUST be verified by a check
  that builds against it.
- **FR-012a**: Where that check cannot yet be meaningful, its status MUST be
  stated where a reader will see it rather than counted as a passing
  verification.

**Checks**

- **FR-013**: The check set MUST verify formatting, lints with warnings
  treated as errors, and tests against a locked dependency set.
- **FR-014**: The check set MUST verify that the core crate builds for a
  target where no capture backend exists, which is how platform neutrality is
  proven rather than asserted.
- **FR-015**: The check set MUST verify the repository conventions: encoding,
  line endings, trailing whitespace, terminal newline, absence of the
  prohibited dash characters, and presence of license identifiers.
- **FR-015a**: The conventions check MUST be implemented as a subcommand of
  the repository task runner, requiring nothing installed beyond the language
  toolchain, and MUST itself be covered by tests.
- **FR-016**: The workflow set MUST cover the six purposes named in the
  architecture of record: the standard check set, platform-dependent checks,
  dependency vulnerability and license audit, documentation, external link
  verification, and release.
- **FR-017**: Workflows whose subject does not yet exist MUST be skeletons
  that declare their trigger and purpose without pretending to verify
  something absent.

**Honesty of reporting**

- **FR-018**: Any check that is scaffolded but not exercised MUST say so where
  a reader will see it, rather than being presented as a passing check.

### Key Entities

- **Workspace**: The single unit of build and dependency resolution. Owns the
  shared metadata and the locked dependency set.
- **Crate**: One of eight units with a single purpose, a declared position in
  the dependency direction, and inherited shared metadata.
- **Task runner**: A member of the workspace that implements repository-wide
  operations, invoked through the language toolchain.
- **Workflow**: A named automated procedure with a trigger and a purpose. Six
  exist; each is either exercised or explicitly a skeleton.
- **Check**: One verifiable assertion about the repository, runnable locally
  and automatically by the same command.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A contributor with only the language installer present goes from
  clone to a completed build with a single command and no manual steps.
- **SC-002**: Every directory named in the repository layout exists and
  explains its own purpose to a first-time reader.
- **SC-003**: The complete check set runs locally with one command, and its
  result matches what the automated equivalent would report.
- **SC-004**: Each of the four deliberately introduced violations, one per
  check category, is detected and reported by name.
- **SC-005**: The crate set and every dependency edge match the architecture
  of record exactly, verified by inspecting workspace metadata rather than by
  reading source.
- **SC-006**: The core crate builds for a platform other than the development
  platform.
- **SC-007**: Every claim the slice makes about verification is either
  demonstrated by a command whose output is recorded, or is explicitly
  labelled as unexercised.

## Assumptions

- **A-S01-1**: The minimum supported toolchain named in the architecture of
  record is a compatibility claim about consumers, distinct from the toolchain
  the project builds with. Both are declared, and the former is checked.
  Resolved in Clarifications; the check is vacuous until dependencies exist at
  S02, which is stated rather than glossed.
- **A-S01-5**: The house shell standard that the architecture of record assumes
  for repository scripts is a known missing gap. This slice routes around it by
  implementing its one required check in the project's own language. The gap
  still blocks the documentation linter at S18 and is not closed here.
- **A-S01-2**: Placeholder directories carry a short document explaining their
  purpose, which both makes them trackable and serves the reader.
- **A-S01-3**: The absence of a git remote is a temporary condition of this
  slice, not a permanent property. Workflows are written to be correct when a
  remote exists, and their unexercised status is recorded.
- **A-S01-4**: Skeleton crates contain the minimum needed to compile and pass
  lints, and no speculative structure. Later slices own their contents.

## Out of Scope

- Core types and traits, header parsing, and every downstream capability.
- Any capture, attribution, profile, sink, or platform integration logic.
- The documentation site build.
- Publishing to any package registry, including reserving names.
- Executing any workflow, which requires a remote that does not exist.

## Dependencies

- Depends on: nothing. This is the first slice.
- Depended on by: every subsequent slice.

## Constitution Alignment

- **P-2** is the principle this slice exists to make mechanical. FR-005 and
  FR-014 turn it from a rule into a check.
- **P-4** binds FR-018 and SC-007: a check that is claimed but not run is the
  reporting equivalent of a silent drop.
- **P-8** binds FR-008 and FR-015: conventions are enforced by the linter, not
  by review attention.
- **P-9** binds FR-018: the slice reports what it verified, not what it
  intended to verify.
- The licensing obligations bind FR-007 through FR-010.
