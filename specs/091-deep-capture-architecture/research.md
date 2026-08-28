# Phase 0 Research: Deep Capture Architecture and Trust Boundaries

## Page Structure

**Decision**: Organize the page around two separate left-to-right Mermaid diagrams followed by trust-boundary, output-authority, dependency, and security-limit sections.

**Rationale**: Issue #247 asks readers to distinguish two shipped modes and requires both mobile and desktop readability. Separate diagrams prevent proxy and trust steps from appearing in passive Capture, while short nodes and explanatory prose keep the diagrams scannable.

**Alternatives considered**:

- One combined diagram with optional branches: rejected because it visually makes active Deep Capture behavior part of every capture.
- One large trust-boundary diagram: rejected because the artifact and refusal detail would exceed the twelve-node readability budget.
- Prose without diagrams: rejected because execution order and component ownership are the central acceptance criteria.

## Capture Scope and Packet Truth

**Decision**: Show Npcap acquisition, packet parsing, external socket and process evidence, scope accounting, `.fcapng`, and an unmodified analyzer. State that the default `target` scope omits unattributed and other-process packets with named counters, while `--scope all` retains all observed traffic with attribution state.

**Rationale**: Current Capture behavior is scoped output, not a promise that every acquired packet is written by default. The distinction is needed to preserve P-4 and avoid repeating the public-library coverage overclaim corrected in S088.

**Alternatives considered**:

- Say all observed packets are retained: rejected because that is true only under `--scope all`.
- Omit scope from the diagram: rejected because scoped retention is part of the shipped data path and a reader could mistake absence for packet loss.
- Describe scope exclusions as kernel filtering: rejected because scope is a userspace decision after attribution.

## Deep Capture Eligibility and Execution

**Decision**: Begin Deep Capture with a stored target and read-only compatibility preflight. Permit the current real-target path only with current `proxy-routing = reached-client` and `proxy-propagation = confirmed` facts for `steam-protocol-cold`; refuse unknown, stale, conflicting, wrong-launch, warm-Steam, and direct-executable cases before session side effects.

**Rationale**: `require_known_compatibility` and `require_supported_launch_case` enforce this exact gate. v0.7.0 has no supported end-user compatibility-bootstrap command, so architecture prose must not imply that any stored target can proceed.

**Alternatives considered**:

- Generalize the gate to known compatible targets: rejected because it hides the exact evidence and launch case.
- Show compatibility as a post-run observation only: rejected because compatibility is enforced at the read-only boundary before session side effects.
- Describe direct execution as a fallback: rejected because the implementation refuses it.

## Trust Authorization and Ownership

**Decision**: State that `--trust-ca` is the explicit authorization for the run's fragcap-owned CA entry in the current-user Root store and that no second interactive trust prompt follows. Describe `--yes` as broader unattended preconfirmation, and show cleanup and its record as part of the session.

**Rationale**: The command rejects a run unless `--trust-ca` or `--yes` is present. Adding `--trust-ca` already authorizes the scoped change. Promising another prompt would be false and could cause an operator to approve a command under the wrong expectation.

**Alternatives considered**:

- Say every trust change prompts interactively: rejected because the flag is the confirmation.
- Present `--yes` as equivalent first-run guidance: rejected because it preconfirms more than the trust action.
- Describe the CA as mitmproxy-owned: rejected because fragcap provisions and tracks purpose-specific session material and exact trust identity.

## Artifact Authority

**Decision**: Group bundle contents by authority: `.fcapng` packet truth; `application.jsonl` and optional `http.har` proxy observations; optional `tls-keylog.log` proxy-owned analyzer material; proxy/process/compatibility sidecars as correlation and diagnostic evidence; `manifest.json` and `cleanup.json` as audit records.

**Rationale**: The manifest itself assigns distinct authorities and sensitivity classes. Collapsing these files into one equivalent output set would overstate what proxy observations prove and conceal the special handling required by application data and key material.

**Alternatives considered**:

- List filenames without authority: rejected because a reader could infer equivalent truth.
- Reproduce issue #248's full artifact matrix: rejected because that separate slice owns omission, lifetime, and handling detail.
- Link the current output-format page as the Deep Capture authority: rejected because that page still describes two equivalent Capture formats and issue #248 owns its correction.

## Correlation Limits

**Decision**: Explain that proxy observations carry session identity and proxy connection identity, and receive packet flow identifiers only when the live flow registry can match observed endpoints. Preserve absent correlation as absent.

**Rationale**: `correlate_observations` enriches observations from structured anchors. The architecture must describe this as a bounded join rather than implying that time proximity or application content creates a match.

**Alternatives considered**:

- Claim every application event maps to packets: rejected because no match may exist.
- Describe timestamp-only correlation: rejected because the implementation uses structured connection and flow evidence.
- Hide uncorrelated observations: rejected because absence is meaningful evidence.

## Npcap Acquisition

**Decision**: State that fragcap never bundles, hosts, caches as its own, or redistributes Npcap or its installer. After per-action confirmation, the published build opens the official acquisition page. A source build with optional `net` support may instead fetch the vendor's signed installer to a uniquely named temporary path and launch it.

**Rationale**: `doctor --fix` selects the degraded page-opening action without `net` and the vendor fetch-and-launch action with it. Both are offered and confirmed before execution. This is the current constitution carveout and corrects the architecture page's obsolete never-download wording without implying redistribution.

**Alternatives considered**:

- Retain “never downloads”: rejected because it is false for `net` source builds.
- Say every build fetches the installer: rejected because the published build opens the official page.
- Describe Npcap as bundled with fragcap: rejected by licensing policy and actual distribution behavior.

## Dependency Boundaries

**Decision**: Present Npcap as the live packet backend shared by both modes, mitmdump as the shipped Deep Capture proxy child only, and Wireshark or another unmodified pcapng analyzer as a downstream consumer.

**Rationale**: This prevents an absent mitmdump backend from appearing to disable ordinary Capture and prevents the analyzer from appearing inside the acquisition engine.

**Alternatives considered**:

- Put mitmdump in the Capture diagram: rejected because Capture has no proxy dependency.
- Treat Wireshark as required to record packets: rejected because it consumes output rather than producing it.

## Validation Boundary

**Decision**: Run phrase, link, node-count, privacy, encoding, punctuation, documentation, static-export, and full repository gates. Defer the exhaustive artifact matrix to issue #248 and interactive responsive/accessibility testing to issue #249.

**Rationale**: These checks prove this page's source correctness and production buildability without duplicating adjacent planned slices.
