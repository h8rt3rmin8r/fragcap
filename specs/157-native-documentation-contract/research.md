# Research: S157 complete native documentation contract

**Date**: 2026-09-17 UTC

## Documentation baseline

**Decision**: Treat published v0.10.1 as the downloadable product baseline and current main as the documentation and verification baseline. State both when post-release S154 through S156 work matters, without implying that a new product release exists.

**Rationale**: The publication record and fourteen current markers identify v0.10.1 source `a7d24962999d38d7ff130722859d473543864862`. S154 adds documentation traceability and test reliability, S155 adds independent-review intake and published-build replay, and S156 corrects package-certification authority. None changes the shipped native proxy runtime or creates a later release. P-11 requires exact release truth.

**Alternatives considered**: Describe only the release and omit current verification work, which makes current source guidance incomplete; describe main as shipped, which violates P-11; prepare a new release, which is outside the selected documentation slice.

## Contract inventory

**Decision**: Replace the eleven-topic readiness inventory with a version-two complete contract containing thirteen closed topics, exact required section headings, executable test authorities, and separate example-authority references.

**Rationale**: The existing inventory proves only that a page exists and one or more tests exist. It does not prove that the page still contains the contract sections a reader needs, it omits the dedicated Capture modes page, and it does not own the reader-facing contract/status page. Version two can reject section deletion, stale status framing, and example authority drift without parsing prose meaning.

**Alternatives considered**: Expand the hard-coded version-one topic list only, which leaves section and example gaps; introduce a general prose semantic checker, which would overclaim mechanical proof and duplicate review; create one registry per page, which fragments the closure and makes missing cross-page topics harder to detect.

## Command examples

**Decision**: Keep the existing no-dispatch production-parser corpus as the command authority and bind it explicitly in the version-two registry. Preserve both default and network-capable command-tree runs.

**Rationale**: The current parser already discovers fragcap invocations in every fenced block except Mermaid across README and nonhistorical site pages, handles shell continuations and comments, normalizes documented placeholders, rejects retired Deep Capture consent inputs, and dispatches nothing. Replacing it would create a weaker approximate parser.

**Alternatives considered**: Restrict examples to PowerShell fences, which misses plain-text transcripts; run commands, which violates the host-execution boundary; write a second command grammar, which can drift from the executable.

## Artifact examples

**Decision**: Register committed manifest specimens and packet-output goldens against their existing actual readers, and register the complete native bundle, application stream, HAR, process trace, cleanup, correlation, and public API examples against exact non-ignored executable contracts when no safe standalone public specimen exists.

**Rationale**: A shortened synthetic artifact can look plausible while violating reconciliation. Existing product readers and complete controlled-session tests already own these contracts. The inventory should expose that authority instead of adding approximate JSON validation or unsafe real-session artifacts.

**Alternatives considered**: Add handcrafted snippets for every artifact, which creates unauthoritative examples; publish a real bundle, which risks sensitive data; treat JSON syntax as validation, which cannot prove schema, lifecycle, loss, or cross-artifact reconciliation.

## Reader-facing contract

**Decision**: Keep the stable `/docs/reference/native-documentation` route but retitle and rewrite it as the native product contract index. It will distinguish published bytes, current-source verification, engineering completion, and pending external reconciliation.

**Rationale**: Existing links and search already point to the route. Renaming the file would add redirect and broken-link risk without reader value. The present page incorrectly calls merged S154 changes a candidate and centers readiness rather than the product contract.

**Alternatives considered**: Delete the page and rely on navigation, which removes the closed contract view; create a second page, which duplicates status authority; close #331 from this slice, which would fabricate the missing independent-review reconciliation.

## Verification boundary

**Decision**: Extend existing xtask, parser, site, accessibility, search, anchor, and full-CI gates only. Do not run an installed product, real game, live capture, real trust mutation, or independent-review scenario.

**Rationale**: Every S157 requirement is verifiable from static content, controlled readers, or existing synthetic contracts. Actual reviewer independence and the installed QUIC retest remain external facts under #333 and #413.

**Alternatives considered**: Owner-host installed validation, which the operator prohibited; implementation-agent security approval, which is not independent; defer all documentation until #333, which leaves the agent-runnable baseline unfinished.

## Authority

The governing authorities are constitution 1.4.0, master specification sections 19, 22, 25, and 28, issue #427 under #331, published v0.10.1 identity, current source and tests, and S154 through S156 records. No unresolved research choice requires operator input.
