<!--
Sync Impact Report (1.1.0, 2026-08-06)
- Version change: 1.0.0 -> 1.1.0 (MINOR: principle added)
- Added principle:
  P-9. The Instrument Does Not Lie (NON-NEGOTIABLE)
- Reason: specification revision 0.1.1-draft introduced capture-time redaction
  of process command lines, including irreversible masking at the source and
  withholding observed data from output. That is contrary to the project's
  stated purpose of accurate observation, and the reasoning that produced it
  was the well-intentioned kind that no existing principle blocked. P-4
  covered silent packet loss but not silent alteration. Withdrawn in
  specification 0.1.3-draft; this principle prevents its reintroduction.
- Templates: no changes required. The Constitution Check gate in
  plan-template.md reads this file and picks up the new principle
  automatically.
- Follow-up TODOs: none

Prior report, 1.0.0
- Version change: unversioned template -> 1.0.0
- Ratification: initial adoption (first authored constitution)
- Source: fragcap v0.1.0 Technical Specification section 27.2, which names
  these eight principles as constitution content because each must survive
  every agent session without restatement.
- Modified principles: none (initial authoring; all placeholder tokens
  replaced)
- Added principles:
  P-1. Passive Observation Only (NON-NEGOTIABLE)
  P-2. Core Stays Platform-Neutral
  P-3. Capture And Attribution Stay Separate
  P-4. No Silent Loss
  P-5. Compatibility Outranks Richness
  P-6. Glossary First
  P-7. Wrappers Stay Thin
  P-8. House Standards Apply
- Added sections:
  Licensing And Third-Party Obligations
  Development Workflow And Quality Gates
  Governance
- Removed sections: none
- Templates:
  .specify/templates/plan-template.md ......... aligned (generic Constitution
    Check gate references this file; no edit needed)
  .specify/templates/spec-template.md ......... aligned (no constitution
    references; no edit needed)
  .specify/templates/tasks-template.md ........ aligned (no constitution
    references; no edit needed)
  AGENTS.md, CLAUDE.md ........................ consistent with these
    principles (runtime guidance; no edit needed)
- Follow-up TODOs: none
-->

# fragcap Constitution

fragcap is a passive, process-attributed network capture tool. It observes
traffic and records which process produced it. It does not modify what it
observes, and it does not reach inside the processes it names.

That single sentence is the source of most of what follows. The principles
below are the rules that must hold across every agent session, every feature
slice, and every contributor, without being restated each time. The full
reasoning for each lives in the master specification
(`docs/fragcap-specification.md`); the section references are load-bearing and
are the place to go when a principle needs interpretation.

## Core Principles

### P-1. Passive Observation Only (NON-NEGOTIABLE)

The technique denylist in specification section 19.3 is absolute. No technique
on it is used, regardless of convenience, under time pressure, or when a slice
appears blocked. The denylist:

- Packet interception and filtering drivers. Use the NDIS capture driver.
- Code injection into a target process. No alternative; out of scope.
- Function hooking. Use socket table attribution from outside the process.
- Process handles carrying memory-read rights against a target. Use
  creation-time ancestry from ETW.
- Layered service providers and Winsock catalog modification. Use socket table
  attribution.
- Executable image modification. No alternative; out of scope.

The permitted set is exactly section 19.2: the NDIS capture driver, ETW kernel
providers, IP Helper socket tables, query-only process enumeration, and
ordinary platform protocol handler launches.

Any use of a process handle MUST state its requested access rights explicitly
at the call site. A request carrying memory rights fails review. A dependency
providing a prohibited capability fails the dependency audit.

Rationale: every technique on the denylist is a cheat primitive that detection
systems watch for directly, and not one of them is needed for any capability
fragcap claims. Reaching for one would trade the project's entire security
posture for convenience it does not need. This is also why the project can be
published at all: a passive observer is defensible to a publisher, a security
vendor, and a user in a way that an injector never is.

### P-2. Core Stays Platform-Neutral

`fragcap-core` acquires no platform-specific dependency, no I/O crate, and no
capture library. It compiles for any target supporting the standard library,
and continuous integration proves this by building it for a target where no
capture backend exists.

Dependencies flow one direction only, concrete toward abstract, per
specification section 8.3. No crate depends on `fragcap-cli`. No crate below
the facade depends on a sibling at its own level. This direction is not
violated to expedite a slice.

Rationale: the Linux and macOS backends in section 28 are only reachable if
core never grew Windows assumptions. Platform leakage into core is cheap to
introduce and expensive to remove, and it is invisible until the second
platform is attempted.

### P-3. Capture And Attribution Stay Separate

No implementation merges `PacketSource` and `FlowAttributor`. No attribution
logic enters a packet source, and no packet acquisition enters an attributor.

Rationale: the two have different platform requirements, different failure
modes, and different upgrade paths. Keeping them apart is also what makes the
testing strategy in section 25 work at all: a replay `PacketSource` and a
scripted `FlowAttributor` make the whole pipeline testable offline, with no
capture driver, no elevation, and no game running. Merging them would collapse
that seam and force every test onto live hardware.

### P-4. No Silent Loss

Every discarded packet is counted in a named counter and surfaced in
statistics. Adding a discard path without a counter is a defect, not an
oversight.

Backpressure is a bounded ring with drop-oldest semantics, counted and
reported. Unattributed packets are retained and marked, never dropped.

Rationale: a capture tool that loses data without saying so produces
conclusions that are wrong in a way the user cannot detect. Every other defect
in this project is recoverable by reading the output. This one corrupts the
output's meaning.

### P-5. Compatibility Outranks Richness

Output format changes that would require a plugin to read are rejected in
favor of changes readable by unmodified tooling. A `.fcapng` file MUST open
cleanly in an unmodified analyzer, which ignores the annotations it does not
understand.

Rationale: the attribution data is worth having only if the file remains a
capture file. A format that needs bespoke tooling to open has traded the entire
existing analysis ecosystem for a feature, which is a bad trade at any richness.

### P-6. Glossary First

A term introduced in code or documentation gets a glossary entry in the same
change that introduces it. No term appears in any project document without a
glossary entry existing first. Entries follow the template in specification
section 4.3 and carry primary-source references.

Rationale: fragcap sits at the intersection of five domains, and a contributor
arriving from any one of them will not share vocabulary with the others. The
glossary is what lets the prose stay precise without condescending, and it only
works if it is never allowed to fall behind.

### P-7. Wrappers Stay Thin

Shell wrappers handle privilege checks, capture driver presence detection,
interface enumeration, path translation, and output templating. They contain no
parsing and no capture logic.

A wrapper that needs to parse capture output indicates a missing capability in
the Rust binary. The fix is to add that capability, not to grow the wrapper.

Rationale: logic in a wrapper is logic that is untested, duplicated across two
shells, and invisible to the type system. Treating wrapper growth as a defect
signal keeps capability where it can be verified.

### P-8. House Standards Apply

Bash and PowerShell follow the ShruggieTech scripting standards. Markdown
follows the house authoring standard. `CONVENTIONS.md` binds every file in the
repository, including generated ones.

These are enforced by the repository linter in continuous integration, not by
review attention.

Rationale: mechanical consistency that depends on a reviewer noticing is
consistency that decays. Automating it frees review for the questions that
actually need judgment.

### P-9. The Instrument Does Not Lie (NON-NEGOTIABLE)

What fragcap reports is what fragcap observed. No capture path alters,
masks, truncates, reorders, or withholds an observation in order to produce a
safer, tidier, or more comfortable result.

This binds specifically against the well-intentioned version, which is the one
that actually gets written: redacting a field that looked sensitive, dropping a
record that looked like noise, normalizing a value that looked malformed,
suppressing an anomaly that looked like a bug. Each is a small, defensible,
local decision. Together they are a tool whose output no longer means what it
says.

Three things this does not prohibit, because they preserve the observation:

**Scope.** The operator decides what to watch. Not observing something is a
choice they make and can see in their own invocation. It is not the same act as
observing it and altering the record.

**Downstream transformation.** Preparing a capture for publication is a
distinct operation, applied to a copy, reporting exactly what it changed,
leaving the original intact. Never a default, never implicit, never during
capture.

**Declared omission.** If fragcap does drop something, P-4 applies: it is
counted in a named counter and surfaced. A silent redaction is the same defect
class as a silent packet drop.

Rationale: the project's stated purpose is to observe accurately where network
theory and shipped game networking diverge. The shape of that divergence is not
fragcap's concern; recording it faithfully is the entire product. A tool that
sanitizes on the operator's behalf has substituted its judgment for theirs
about their own machine, and has done it invisibly. Every downstream conclusion
then inherits a distortion the researcher cannot see or correct for. Section
2.3 already ranks fidelity of observation above throughput and completeness;
this principle makes that ranking binding on implementation rather than
advisory.

The honest researcher is the user. Treating them as someone to be protected
from their own data is both a product failure and, given the domain, a
credibility failure.

## Licensing And Third-Party Obligations

fragcap is licensed under Apache-2.0. Every source file carries an SPDX
identifier. Crate manifests declare `license = "Apache-2.0"`. The deviation
from the Rust ecosystem's conventional `MIT OR Apache-2.0` dual license is
deliberate and recorded in specification section 20.1; it is not relitigated.

**npcap is not redistributable, and this is a product constraint rather than a
paperwork one.** Four rules follow, and all four are binding:

1. **No bundling.** No distribution artifact contains npcap binaries,
   installers, or driver files. This includes release archives, installers, and
   container images.
2. **Detection, not installation.** fragcap detects npcap's presence and
   version at runtime and reports absence with the official download location.
   It never downloads, installs, or invokes an installer.
3. **Documented prerequisite.** Installation documentation states npcap as a
   required separate installation, with its required non-default options, ahead
   of every usage instruction.
4. **No vendoring of the SDK.** Continuous integration acquires the npcap
   Software Development Kit at build time. It is never committed.

Dependency licenses are restricted to MIT, Apache-2.0, BSD two-clause and
three-clause, ISC, Unicode-DFS, and Zlib. Copyleft licenses are excluded. This
applies with particular force to packet interception libraries, which are
commonly copyleft licensed and which P-1 prohibits on independent grounds.

## Development Workflow And Quality Gates

**Spec-driven development is mandatory.** Every feature traces to the master
specification and is built through the full spec-kit sequence before any
implementation code: specify, clarify, checklist, plan, tasks, analyze, then
implement. Work lands as a numbered `specs/NNN-slug/` slice. The slice ordering
lives in `docs/plans/README.md`; the specification supplies technical scope. The
specification scopes a slice but never substitutes for its spec. The analyze
gate MUST pass and MUST NOT be weakened or skipped.

**Deviations get recorded.** Any divergence from the specification discovered
during implementation is recorded in the slice and promoted to specification
section 29 at the next version. Silent divergence between the specification and
the code is a defect in both.

**Verification runs in the foreground, watched to completion.** Never
backgrounded, never assumed. Once the workspace exists, the gates are
`cargo fmt --all -- --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, and
`cargo test --all --locked`, plus the repository conventions linter, the
documentation linter in check mode, and both shell wrapper compliance checkers.

**Claims require evidence.** Do not report a slice complete, a test passing, or
a defect fixed without having run the command and read the output. Reporting an
unverified success is worse than reporting a known failure, because it removes
the operator's ability to trust any other report.

**Pinned artifacts change only with a dated decision recorded in
`CHANGELOG.md`:** `.github/workflows/**`, `rust-toolchain.toml`, `release.toml`,
`scripts/**`, and the release documentation.

**Text hygiene is absolute.** All text files are UTF-8 without BOM with LF line
endings, no trailing whitespace, and a single trailing newline. No em-dashes and
no en-dashes anywhere, including code comments; use commas, parentheses, or
standard hyphens.

## Governance

This constitution supersedes all other practices. Where it conflicts with a
habit, a convenience, or an agent's default behavior, the constitution wins.
Where it conflicts with the master specification, that is a defect in one of
them, and the conflict is resolved explicitly rather than by picking a side
silently.

**Amendments** require an explicit change to this file with the Sync Impact
Report header updated, a version bump under the policy below, and a statement
of what the amendment changes and why. Principles are not amended mid-slice to
unblock that slice; that is the situation the principles exist for.

**Versioning policy.** MAJOR for a principle removed or materially redefined.
MINOR for a principle or section added, or an existing one materially expanded.
PATCH for clarifications and wording that do not change meaning.

**Compliance.** Every plan carries a Constitution Check gate. Every pull request
verifies compliance. Complexity that appears to require violating a principle
must be justified in writing in the slice, and the justification is reviewed by
the operator rather than accepted by the author.

**Escalation.** An agent that believes a principle blocks correct work halts and
raises it, rather than proceeding under an interpretation that weakens it. P-1
in particular is never reinterpreted; a slice that appears to need a denylisted
technique is a slice that has been scoped wrong.

**Version**: 1.1.0 | **Ratified**: 2026-08-06 | **Last Amended**: 2026-08-06
