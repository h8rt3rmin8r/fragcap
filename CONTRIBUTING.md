# Contributing to fragcap

Thanks for looking. This document is the practical how; the governing rules
live in `.specify/memory/constitution.md`, and the mechanical ones in
`CONVENTIONS.md`.

Contributions are accepted under the Apache-2.0 inbound-equals-outbound model
described in section 5 of the license. No separate contributor license
agreement is required. Contributed game profiles are data covered by the same
license.

## Before anything else: what fragcap will not accept

fragcap has two shipped modes under constitution principle P-1. Capture is passive process-attributed packet capture. Deep Capture is explicit, target-scoped, reversible local proxy inspection for authorized sessions. Neither mode permits covert target instrumentation.

P-1 prohibits, absolutely:

- Packet interception or filtering drivers
- Code injection into any target process
- Function hooking
- Process handles carrying memory-read rights against a target
- Layered service providers or Winsock catalog modification
- Executable image modification
- Target TLS key extraction

A pull request using any of these will be declined regardless of how well it works or what it enables. This is not negotiable and is not a judgment about the contributor. If a capability seems to require one, open an issue rather than an implementation so the requirement can be checked against the architecture.

Deep Capture may start a session-owned local proxy, apply target-scoped launch configuration, manage a purpose-specific local certificate authority after explicit confirmation, and export proxy-owned TLS key-log material. It may not fall back silently to system-wide proxy settings, trust a certificate silently, bypass certificate pinning, or leave cleanup residue unreported.

Copyleft-licensed dependencies are also declined. See constitution, licensing
section.

## Current state

v0.8.0 is the current release. The Rust workspace ships target discovery, passive process-attributed Capture, explicit Deep Capture for known-compatible stored targets, analyzer integration, Windows packaging, and the documentation site. [`CHANGELOG.md`](CHANGELOG.md) records the chronological release history, `specs/` records every completed work slice, and the open [GitHub milestones](https://github.com/h8rt3rmin8r/fragcap/milestones) show the current workstreams.

Deep Capture is functional but incomplete. Its current CLI path uses external
mitmdump and certutil. [Issue #278](https://github.com/h8rt3rmin8r/fragcap/issues/278)
is the native-completion authority; contributors must not describe the feature
as native, self-contained, or feature-complete until #334 closes. The S102
native crate is a bounded lifecycle foundation, not a protocol-inspection or
CLI-cutover claim.

Npcap remains a separately installed prerequisite for live packet capture. fragcap never bundles, hosts, caches as its own, or redistributes Npcap or its installer. After explicit interactive confirmation, the shipped `fragcap doctor --fix` opens the official download page. A source build with the optional `net` feature may instead fetch and launch the vendor's signed installer. The default workspace build and offline tests require neither Npcap nor administrative privilege.

## The rule

- **All changes reach `main` only through a pull request that a human reviews
  and approves.** No one pushes commits directly to `main`, and no agent merges
  its own pull request.
- The operator (`@h8rt3rmin8r`) is the reviewer, approver, and merger.
- Continuous integration must be green before a pull request is merged.
- A release is cut only by pushing a `vX.Y.Z` tag, and that tag push is the
  operator's.

## Workflow

1. Branch off `main` (for example `feat/<slug>` or `fix/<slug>`).
2. If the change implements a feature, it goes through the spec-kit sequence
   first. Every feature traces to `docs/fragcap-specification.md` and lands as
   a numbered `specs/NNN-slug/` slice. See `AGENTS.md` for the full cycle. Bug
   fixes and documentation corrections do not need a slice.
3. Make the change. Follow the constitution, `CONVENTIONS.md`, and the existing patterns. Do not modify pinned artifacts (`.github/workflows/**`, `rust-toolchain.toml`, `release.toml`, `scripts/**`, release documentation) without a dated `changelog.d/*.decisions.md` fragment.
4. Add or update tests. Run verification in the foreground and watch it finish:

   ```sh
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all --locked
   ```

5. Add a changelog **fragment**: a new `changelog.d/<key>.<section>.md` file.
   **Do not edit `CHANGELOG.md` directly.** It conflicts with every other
   concurrent pull request. See `changelog.d/README.md`.
6. Open a pull request. Put `Closes #N` in the body so the issue auto-closes on
   merge, summarize the change, and state how you verified it.
7. Do not merge your own pull request.

Two files are deliberately never edited by a pull request: `CHANGELOG.md` (use
a fragment) and `.specify/feature.json` (local per-branch spec-kit state, and
gitignored).

If a change is ambiguous or underspecified, open the pull request as a draft
and use the body to ask the specific questions rather than guessing.

## Testing

The testing strategy exists so that most of the project is verifiable with no
capture driver, no elevation, and no game running. A replay `PacketSource`
backed by capture fixtures and a scripted `FlowAttributor` make the whole
pipeline testable offline. That property is worth protecting: if a change makes
a previously offline-testable component require live hardware, that is a design
problem, not a testing inconvenience.

Three tiers:

| Tier | What it covers | Runs in CI |
| --- | --- | --- |
| Unit | Individual components in isolation | yes |
| Pipeline integration | End to end over fixtures, no driver, no game | yes |
| Live smoke | Real capture against a real title | no, manual |

Test fixtures under `fixtures/` are the one place capture files are committed.
They are reviewed before they land and MUST NOT contain account identifiers,
session tokens, or addresses attributable to a real operator.

## Documentation and the glossary

Constitution principle P-6: a term introduced in code or documentation gets a
glossary entry in the same change that introduces it. No term appears in a
project document without an entry existing first.

This is enforced by the documentation linter, so a pull request introducing
vocabulary without an entry fails continuous integration rather than review.
The glossary is what lets the prose stay precise without condescending, and it
only works if it never falls behind.

## Reporting a problem

Use the issue templates. For a capture defect, the useful report includes the
`fragcap doctor` output, the fragcap version, the npcap version, the profile in
use, and the capture statistics.

**Scrub anything you attach.** Capture files carry addresses and, in some
titles, session identifiers. Do not attach a raw capture to a public issue.
Findings and fixtures are recorded without account identifiers, session tokens,
or addresses attributable to the reporter.

## Security

If you believe you have found a security problem, or a way fragcap could be
made to do something the constitution prohibits, do not open a public issue.
Report it privately through the repository's security advisory process.
