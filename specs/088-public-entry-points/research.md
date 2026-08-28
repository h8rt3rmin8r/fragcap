# Phase 0 Research: Public Entry Point Reconciliation

## Shared Product Definition

**Decision**: Every first-contact surface will distinguish passive Capture from shipped Deep Capture using the constitution and master specification as the factual authority. Deep Capture wording will include explicit activation, target scope, reversibility, authorized use, and compatibility limits where the surface has room.

**Rationale**: Calling the whole product passive is false after v0.7.0. Calling Deep Capture a general decryption path is also false. The mode-specific definition is accurate at both extremes and already governs the implementation.

**Alternatives considered**:

- Describe only Capture on short surfaces: rejected because it repeats the current omission.
- Lead every surface with Deep Capture details: rejected because first-contact text needs the bounded product distinction, not the full compatibility reference.
- Copy one long paragraph everywhere: rejected because the surfaces have different jobs and would drift as soon as one needed a local qualification.

## Master Specification Scope Addition

**Decision**: Correct stale present-tense v0.7.0 status claims in specification sections 1, 2.1, 19, 27.3, and 28 as part of S088. Preserve historical revision rows and completed slice records unchanged.

**Rationale**: Planning found that the architecture of record declares `Applies-To: 0.7.0` while saying current software ends at v0.4.0, v0.5.0 is in progress, and Deep Capture has not shipped. P-11 classifies that as a defect. The correction is narrow and directly supports the public entry points that link to the specification.

**Alternatives considered**:

- Leave the master specification for issue #247: rejected because #247 owns the public architecture page and diagrams, while P-11 requires the architecture of record to describe the current release now.
- Rewrite all of sections 19 and 28: rejected as disproportionate. Only stale current-status statements and release rows need correction.
- Rewrite the old specification outline: rejected because the master specification explicitly supersedes it and this issue does not scope an outline refresh.

## Contributor Information Architecture

**Decision**: Keep `CONTRIBUTING.md` as the canonical practical workflow and make the site contributing page a concise, current summary that links to it. Both surfaces will use the same P-1 mode boundary and current repository state.

**Rationale**: Two full contributor guides would duplicate workflow details and recreate the drift being fixed. The current link relationship already establishes the repository file as canonical.

**Alternatives considered**:

- Duplicate the full repository guide into the site: rejected because it creates two maintenance authorities.
- Reduce the site page to a bare link: rejected because public readers need the security boundary and check set before leaving the site.

## Npcap Acquisition Wording

**Decision**: State the complete current rule: Npcap is installed separately and never bundled, hosted, cached as fragcap's own, or redistributed; `doctor --fix` may fetch and launch the vendor's own signed installer only after explicit interactive confirmation.

**Rationale**: Saying fragcap never downloads Npcap contradicts constitution 1.3.0 and shipped behavior. Saying fragcap installs Npcap would overstate the carve-out and blur the licensing boundary.

**Alternatives considered**:

- Say only "Npcap is required": rejected because the acquisition and redistribution boundary is material to users and contributors.
- Describe `doctor --fix` as an installer: rejected because fragcap fetches and launches the vendor installer but does not own or redistribute it.

## Issue Form Contract

**Decision**: Use `fragcap capture 1 --duration 30m` as the bug-form reproduction example, expect the reporter's actual `fragcap --version`, retain only WinPcap API-compatible mode as the Npcap installation-option question, and update feature-request confirmations to permit constitution-approved Deep Capture without permitting denylisted techniques.

**Rationale**: `run --profile` is retired, loopback support is automatic in current Npcap, and a blanket "never modify traffic" checkbox incorrectly excludes the shipped local proxy mode. The issue forms should collect facts that current triage can use.

**Alternatives considered**:

- Use a Deep Capture reproduction by default: rejected because Capture remains the broadest baseline and Deep Capture-specific failures can state their exact command.
- Remove Npcap information entirely: rejected because live Capture failures still depend on driver presence and compatibility mode.
- Keep the old traffic-modification checkbox: rejected because it conflicts with the explicitly permitted local proxy posture.

## Repository Description

**Decision**: Use `Passive process-attributed Capture and explicit, target-scoped Deep Capture for Windows game traffic.` as the GitHub repository description.

**Rationale**: The sentence fits the short metadata surface, distinguishes the modes, names the target platform, and does not promise universal attribution or inspection.

**Alternatives considered**:

- Add protocol and artifact details: rejected as too dense for repository metadata.
- Use "network capture and decryption": rejected because Deep Capture support is compatibility-dependent and does not decrypt every flow.

## Verification Strategy

**Decision**: Combine deterministic phrase audits with current help comparison, YAML parsing, link and documentation checks, a production site build, the specification currency gate, and the full repository gate.

**Rationale**: Phrase audits catch the exact regressions in issue #244, while the existing gates protect structure, links, site generation, encoding, punctuation, and repository-wide integration. No new permanent checker is justified because issue #246 owns mechanical CLI-reference gating.

**Alternatives considered**:

- Add a new cross-document generator: rejected as over-engineering for six prose files and one metadata field.
- Rely on manual review only: rejected because the production build and stale-phrase absences are objectively testable.
