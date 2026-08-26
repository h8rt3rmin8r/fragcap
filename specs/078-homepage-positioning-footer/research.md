# Phase 0 Research: Homepage Positioning And Next-Command Footer

## Homepage Claim Structure

**Decision**: Lead with the outcome, explain correlation precisely, then distinguish Capture and Deep Capture in a separate paragraph.

**Rationale**: The visitor first needs to understand the value, then the reason standard captures lack process ownership, then the mode boundary. This order remains instrument documentation while avoiding the current scolding packet-count thought experiment.

**Alternatives considered**:

- Keep the packet-count opener and edit its second sentence: rejected because packet count is arbitrary, frames attribution at the wrong granularity, and keeps the ambiguous subject.
- Lead with proxy inspection: rejected because Capture remains the shipped foundation and both modes should read as one product.
- Use a feature grid: rejected by specification section 23.1 and the existing visual language.

## Attribution Wording

**Decision**: Say that packet records do not ordinarily preserve process ownership and fragcap correlates captured flows with separate Windows socket and process-lifecycle observations.

**Rationale**: This matches the implemented socket-table and ETW model. It does not claim the operating system destroyed information, and it leaves room for unresolved or retained attribution fidelity.

**Alternatives considered**:

- Say the link is reconstructed after being discarded: rejected because Windows still exposes separate observations and the capture artifact, not the entire operating system, lacks ownership.
- Say every flow is attributed: rejected because unresolved outcomes are intentionally retained.

## Mode And Dependency Wording

**Decision**: Describe Capture as passive and Deep Capture as explicit target-scoped local proxy inspection for compatible targets. State that Npcap is required for live packet capture, Wireshark is recommended for analysis, and `fragcap doctor` reports readiness for both modes.

**Rationale**: This matches the constitution, S075 MVP, S076 support reference, and Npcap licensing rules. It avoids treating Wireshark as required or Deep Capture warnings as Capture blockers.

**Alternatives considered**:

- Continue calling the whole product passive: rejected because Deep Capture is intentionally active.
- List every Deep Capture prerequisite in the homepage callout: rejected because compatibility is target and launch-case specific; `doctor` is the stable authority.

## Next-Command Shape

**Decision**: Render one blank line followed by `Next command:  fragcap capture <row>`.

**Rationale**: The exact label cannot be mistaken for a machine observation, aligns with the labelled empty-state suggestions, and keeps the recommendation on one compact line. The two spaces after the colon visibly separate label from command without introducing a new layout helper.

**Alternatives considered**:

- `Next:`: rejected as less explicit in a dense diagnostic output.
- A two-line heading and indented command: rejected as unnecessary vertical expansion for one command.
- Color or terminal styling: rejected because redirected and machine-adjacent human output must remain plain and deterministic.

## Homepage Specimen

**Decision**: Keep a hand-authored two-row specimen, but use synthetic handles, the current four columns, placeholder technology values, and the exact next-command label.

**Rationale**: A live CLI invocation would require a local database and discovery environment during the static site build. A generated fixture would add machinery disproportionate to two rows. Binding the specimen through source assertions and the production build provides sufficient drift detection for this slice.

**Alternatives considered**:

- Invoke `fragcap targets` during site generation: rejected because output depends on local state and platform availability.
- Embed real title examples: rejected because they imply compatibility facts and violate the slice's public-data rule.
- Add a new cross-language fixture generator: rejected as over-engineering for a small static specimen.

## Historical Specification

**Decision**: Leave S057 byte-for-byte historical and record in the S078 specification and master specification that its verbatim homepage copy is superseded.

**Rationale**: Slice artifacts record what was decided and implemented at that time. Rewriting them would erase provenance. The current architecture of record is the correct correction site.
