# Phase 0 Research: Workspace Scaffold

No `NEEDS CLARIFICATION` markers survived the specify and clarify phases. This
document records the decisions those phases produced, plus the alternatives
considered, so a later reader can see what was weighed rather than only what
was chosen.

## R-1: Toolchain pinning versus minimum supported version

**Decision.** Pin the build toolchain at 1.96.0 in `rust-toolchain.toml`.
Declare `rust-version = "1.82"` in the workspace package table. Add a check
that builds the workspace with 1.82.

**Rationale.** Section 24.1 asks for a pinned toolchain so that local and
automated builds are identical, and section 21.4 declares a minimum supported
version of 1.82. These are different properties and the specification's wording
runs them together. Pinning the build channel gives reproducibility; declaring
the minimum gives a compatibility promise; only the check makes the promise
true.

**Alternatives considered.**

*Pin the build channel at 1.82.* Guarantees the minimum by construction, and
removes the need for a separate check. Rejected because it forces every later
slice to hold its dependencies to versions compatible with a toolchain from
late 2024. Several crates named in Appendix A already require newer, so this
would trade real capability for a claim obtainable another way.

*Declare the minimum as 1.96 and drop the older claim.* Simplest and trivially
true. Rejected because it narrows who can consume the library for no benefit,
and it discards a decision the architecture of record made deliberately.

**Known limitation, recorded rather than glossed.** With no external
dependencies in the workspace, the minimum-version check passes for any value.
It is vacuous until S02. It is built now so it is already in place when it
starts to constrain something, and its status is labelled in the workflow, in
the plan, and in the slice's completion report.

## R-2: Where the conventions check lives

**Decision.** A subcommand of the repository task runner, written in Rust,
covered by unit tests.

**Rationale.** Section 21.5 requires the conventions to be enforced by a
linter in continuous integration and does not name a language. Three
considerations favor Rust. The house shell standard the project's scripts are
meant to follow is a known missing gap recorded in the reconnaissance notes, so
a shell implementation would block this slice on an unavailable dependency.
Section 21.3 specifies the task runner as requiring nothing installed beyond
the language toolchain, and a shell linter needs a shell on every runner. And a
Rust check can be unit tested against fixture inputs, which matters because a
linter that silently matches nothing is indistinguishable from a clean
repository.

That last point is the deciding one. A check that cannot fail is worse than no
check, because it manufactures confidence. Tests that feed it known-bad input
and require a failure are the only thing that distinguishes the two.

**Alternatives considered.**

*A shell script under `scripts/`.* Matches section 22.5's treatment of the
documentation linter and keeps all checks in one place. Rejected on the three
grounds above; the missing house standard is the blocking one.

*Both, with the shell script delegating.* Rejected as strictly worse than
either alone: two artifacts to keep in step, no additional capability.

**Scope note.** This decision covers the repository conventions linter only.
The documentation linter in section 22.5 remains a shell script owned by S18,
and the missing house standard still blocks it.

## R-3: How dependency direction is enforced

**Decision.** The task runner reads `cargo metadata` and asserts the workspace
edge set against an expectation declared in one place.

**Rationale.** Section 8.3's direction rule is the kind of constraint that
survives exactly as long as everyone remembers it. Encoding the expected graph
makes both violation and intent legible: a contributor who adds an edge sees a
named failure, and a reader who wants to know the architecture reads a list
rather than eight manifests.

**Alternatives considered.**

*Review attention.* This is the status quo the constitution's P-8 rationale
explicitly rejects for mechanical rules.

*A third-party dependency-policy tool.* Rejected for this slice: it adds a
dependency and a configuration format to enforce eight edges. Worth revisiting
if the graph grows.

## R-4: Proving platform neutrality

**Decision.** Build `fragcap-core` on the Linux continuous integration job,
natively. Offer a local cross-target check in the task runner that reports
plainly when the target is not installed.

**Rationale.** Section 9.3 requires core to build where no capture backend
exists, and section 24.3 makes it a blocking gate. The CI matrix already
includes Linux, so the proof costs one job step and no cross-compilation
setup. The local variant exists so a contributor can check before pushing, and
it says "target not installed" rather than passing quietly, because a check
that skips silently is a check that reports success it did not earn.

**Alternatives considered.**

*Cross-compile from Windows only.* Requires every contributor to install a
target they otherwise never use, and would still not exercise a real Linux
build.

*Trust the dependency direction check to imply neutrality.* Rejected: a
platform-specific dependency can enter through a feature flag or a transitive
edge that the direction check, which looks at workspace crates, would not see.

## R-5: Whether skeleton crates take their eventual dependencies now

**Decision.** No. Crates take no external dependencies in this slice.

**Rationale.** Appendix A assigns dependencies per crate, but adding them
before the code that uses them produces unused-dependency warnings, a lockfile
recording choices nothing justifies, and a license audit over crates the
project does not use. Each slice adds what it needs, which also keeps the
audit's output meaningful.

**Consequence.** The workspace has an empty external dependency graph, which is
why R-1's minimum-version check is vacuous for now. The two facts are the same
fact.
