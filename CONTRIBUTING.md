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

Published baseline: [v0.10.11](https://github.com/h8rt3rmin8r/fragcap/releases/tag/v0.10.11).

v0.10.11 is the current release. The Rust workspace ships target discovery, passive process-attributed Capture, explicit native Deep Capture for known-compatible stored targets, analyzer integration, Windows packaging, and the documentation site. [`CHANGELOG.md`](CHANGELOG.md) records the chronological release history, `specs/` records every completed slice, and the open [GitHub milestones](https://github.com/h8rt3rmin8r/fragcap/milestones) show the current workstreams.

Deep Capture is native and shipped for known-compatible stored targets. Protocols, scoped routing, managed launch, artifacts, recovery, conformance, package certification, stable API and guided calibration shipped in v0.10.0 on 2026-09-15. v0.10.2 added the complete documentation contract, reproducible independent-review intake, deterministic certification and Windows observation corrections. v0.10.3 added QUIC observation headroom and the interactive CLI input correction. v0.10.4 added guided first-run Steam client setup, exact cold platform-root binding, and causal calibration failure reporting. v0.10.5 corrects the managed route-owner lifetime after a Deep Capture attempt. v0.10.6 publishes terminal display alignment, bounded acquisition and exact attempt correlation, recipient-context artifact access, finite generated-session retention, and dependency-alert corrections from S167-S171. v0.10.7 corrects automatic same-account elevated output access, retires historical permission repair, and includes seven TLS/QUIC advisory fixes. v0.10.8 enforces Capture scope and metadata-only output, adds bounded managed shutdown, improves first-use and CLI result guidance, and updates rustls-webpki. v0.10.9 includes the corrected Tokio dependency graph and fixes for relocated libraries, calibration evidence timing, managed shutdown, final proxy counters, and clean Windows startup. v0.10.10 adds one installation CA for all Windows users with first-install removal of identified pre-S185 state, and fixes managed route release and ordinary Deep Capture observation timing. v0.10.11 corrects ordinary-user installation CA health reporting, HTTPS CONNECT default-port authority handling, and HTTPS compatibility evidence. The [native product contract](https://fragcap.com/docs/reference/native-documentation) states the supported boundary. Independent security review and optional field testing are not claimed as performed. A failure observed in an active release is filed as a concrete defect against that release.

Npcap remains a separately installed prerequisite for live packet capture. fragcap never bundles, hosts, caches as its own, or redistributes Npcap or its installer. After explicit interactive confirmation, the shipped `fragcap doctor --fix` opens the official download page. A source build with the optional `net` feature may instead fetch and launch the vendor's signed installer. The default workspace build and offline tests require neither Npcap nor administrative privilege.

## The rule

- **All changes reach `main` only through a pull request that a human reviews
  and approves.** No one pushes commits directly to `main`, and no agent merges
  its own pull request.
- The operator (`@h8rt3rmin8r`) is the reviewer, approver, and merger.
- Continuous integration must be green before a pull request is merged.
- A release is cut only by pushing a `vX.Y.Z` tag, and that tag push is the
  operator's.
- Before that tag is pushed, an AI synthesizes the user-relevant highlights into `release-notes/vX.Y.Z.md`; `cargo xtask notes X.Y.Z` must validate the short summary and its link to the complete tagged changelog.

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

Human multi-column output must use the shared display-cell layout with exactly four spaces between measured column anchors. Include headings, optional/indexed keys, Unicode, and ANSI styling in the actual emitted row inventory. Add controlled narrow/plain/color regressions that preserve exact values and continuation anchors; do not shorten the gap or truncate tokens to fit. An exception requires explicit operator permission. Structured output is the automation contract and must retain its existing identities and fields.

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
| Live compatibility | Optional operator-owned observation of an already published release against a real title | no, manual |

Test fixtures under `fixtures/` are the one place capture files are committed.
They are reviewed before they land and MUST NOT contain account identifiers,
session tokens, or addresses attributable to a real operator.

Implementation acceptance never requires an agent to launch a real game, mutate the real trust store, or run sensitive live capture. Guided calibration uses a checked controlled-evidence registry in ordinary CI. A real-game run is optional compatibility evidence owned by the operator, is permitted only against already published product bytes, and does not block or reopen completed implementation work. If it reveals a failure, file a concrete defect against the affected release. Retain raw evidence privately and publish only a scrubbed summary.

## Installed-product field testing

An installed-product field test uses Fragcap's documented default target stores, generated session bundles, retention, and cleanup when it claims the default user journey. A test request authorizes those native outputs, not a separate agent audit directory. Do not silently pass `--bundle`, `--local-db`, `--catalog-db`, a profile-root flag, `--retain-bundle`, `FRAGCAP_SESSION_DIR`, store or profile environment overrides, or redirected application-data roots for convenience. A technically necessary exception requires the owner's agreement to the exact path and changed behavior before use; review its observation on its own terms and do not count it as proof of default storage, retention, or cleanup. Isolated roots in controlled engineering or package certification test those controlled paths only.

### Worked default-path example

For a previously registered target with current compatible Deep Capture facts, the reviewer uses the installed CLI for the initiating user with the privileges required by the selected mode and follows these steps without creating an agent-owned report directory:

1. Confirm that the installed CLI inherits no `FRAGCAP_*` storage override or agent-supplied application-data redirect. Run `fragcap targets show <HANDLE>` without a store override. The nominal target store is `%APPDATA%\fragcap\local.db`; verify the effective canonical file path from actual product and filesystem readback rather than substituting the environment-variable spelling. A packaged agent host may redirect writes into its package `LocalCache` tree while `%APPDATA%` and `%LOCALAPPDATA%` appear ordinary, so report the effective physical path as well as the logical default.
2. Run `fragcap deep-capture <HANDLE> --launch` and review its normal authorization plan. Omit `--bundle`, `--retain-bundle`, all store and profile path overrides, and session-root redirects. The product creates a generated bundle under its nominal `%APPDATA%\fragcap\sessions` root and applies its default generated-history policy of 30 days, 20 completed sessions, and 2 GiB of logical contents. A refusal or partial session is recorded as such; it is not converted into a success by changing paths.
3. Read the terminal bundle path and `manifest.json`, resolve the effective canonical bundle location, and list only the native artifacts actually present. Keep those artifacts in place under the declared retention and recovery rules. Record findings in the conversation; file a detailed privacy-scrubbed issue for a concrete defect. Do not write a duplicate report, transcript, log, helper, download, or build output into Fragcap's data directories or an agent-chosen sidecar.
4. Run read-only `fragcap doctor` and `fragcap bundle collect` preview to inspect cleanup and retention. Apply only a supported, exact reviewed product operation when its ownership and recovery conditions are satisfied. Before deleting older agent scratch, prove the exact path is not a native bundle or capture and has no active owner or recovery obligation. Preserve manifests, journals, recipient access, Doctor inventory, and the only evidence of any unresolved session. If cleanup is rejected or native output cannot be safely relocated, report the residual and do not try a different mechanism or silently move it.

The handoff for this example names the effective canonical `local.db`, session-registry, and generated bundle paths, each artifact actually retained, the route and recovery outcome reported by Fragcap, any product cleanup applied, the transient agent files removed, and any material still present. If the physical location cannot be proven, default-path acceptance remains unverified. Keep private target identity, traffic, account paths, and raw captures out of public issues. A run with an explicitly authorized custom output directory can support its observed capture or protocol result, but a reviewer must mark default bundle placement and default retention as untested.

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
