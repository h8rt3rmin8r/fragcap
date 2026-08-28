# Phase 0 Research: Verified First Capture and Deep Capture Journeys

## Guide Structure

**Decision**: Replace the existing Capture-only sequence with two connected paths on one page. The first path ends at an opened `.fcapng`; the second begins with a read-only compatibility check and proceeds only for a stored target with current evidence for the supported launch case.

**Rationale**: Issue #245 requires a first Capture and a known-compatible Deep Capture journey. Keeping them connected establishes Capture as the packet-truth baseline and makes the extra Deep Capture prerequisites visible without presenting the modes as interchangeable.

**Alternatives considered**:

- Split the paths into separate guides: rejected because the issue asks for two connected first-run paths and the documentation index already routes new users to this page.
- Lead with Deep Capture: rejected because Capture is the general baseline and remains useful when proxy inspection is unsupported.
- Retain the old Capture guide and append one command: rejected because the existing doctor output, target listing, database-path requirements, and encryption claims are already stale.

## Output Specimen Authorities

**Decision**: Derive the doctor specimen from `crates/fragcap-cli/tests/goldens/doctor-ready.txt`, replacing its test-only version token with `0.7.0` and retaining synthetic reserved addresses and paths. Derive the target listing from the current renderer and `cli_targets.rs` assertions, including `CAPTURE`, `ENGINE`, `SENSITIVITIES`, and `Next command:`.

**Rationale**: These committed synthetic contracts exercise the same human renderers as the CLI. They provide stronger evidence than copying output from the operator's machine and avoid leaking local state.

**Alternatives considered**:

- Capture live host output: rejected because it contains machine-local paths, interfaces, versions, and readiness state.
- Hand-author an abbreviated layout without checking the renderer: rejected because that is how the retired `KNOWN` column survived.
- Add a second doctor golden only for documentation: rejected because the existing golden already supplies the required shape.

## Command Validation Boundary

**Decision**: Validate the guide's commands against current clap help and the focused `cli_args`, `cli_help`, `cli_targets`, and `cli_deep_capture` suites during this slice. Do not add a permanent page parser or command-tree documentation gate.

**Rationale**: S090 must prove its own examples, but issue #246 explicitly owns a deterministic cross-reference gate between the public CLI reference and the command tree. Building part of that system here would duplicate its design work and widen this documentation slice.

**Alternatives considered**:

- Add a guide-specific command extractor: rejected as a second validation mechanism immediately before #246 establishes the shared one.
- Trust prose review alone: rejected because clap and the focused suites can objectively reject stale flags and command forms.
- Execute every copied command end to end: rejected because Capture and Deep Capture intentionally require live capture, elevation, a stored target, and explicit side effects. Parser and controlled integration coverage prove grammar without manufacturing a real game or trust change.

## Synthetic Target Convention

**Decision**: Use `sample-target` and `sample.exe` as the only target and executable identities. Use listing row `1` for the first Capture command and the stable handle for `targets show` and Deep Capture examples.

**Rationale**: The compatibility reference already uses these placeholders, so the guide stays consistent with a published synthetic convention. A row number demonstrates the labelled listing hint; the handle avoids implying that a row snapshot is durable across later commands.

**Alternatives considered**:

- Name a real game: rejected by issue #245's privacy boundary and because one title would look like a compatibility endorsement.
- Use only metavariables such as `<TARGET>`: rejected because copied command specimens should be concrete enough to parse and compare.
- Use a numeric Steam app identifier: rejected because even an invented number may map to a real catalog entry and is unnecessary to explain the journey.

## Deep Capture Eligibility and Refusal

**Decision**: Make `fragcap targets show sample-target` the eligibility checkpoint. Proceed only when current local facts show `proxy-routing = reached-client` and `proxy-propagation = confirmed` for `steam-protocol-cold`. Treat inspectability as observed session evidence rather than an eligibility gate. Stop at the compatibility reference when routing evidence is absent, stale, conflicting, or for a different launch case.

**Rationale**: v0.7.0 has no supported compatibility-bootstrap command. The preflight refusal is correct, and the guide must not invent a path around it while issue #251 remains open.

**Alternatives considered**:

- Tell users to confirm compatibility manually: rejected because it would turn belief into stored evidence and undermine P-9.
- Present the closed #215 research harness as an end-user path: rejected because it is not a supported product command.
- Omit the limitation: rejected because most new targets would then fail after the guide implied readiness.

## Traffic and Artifact Depth

**Decision**: State the first-run traffic and artifact subset directly and link to the compatibility reference for the current traffic matrix. The guide names packet truth, application JSONL, optional HAR, optional proxy-owned key logs, proxy logs, process traces, compatibility updates, manifest state, and cleanup status. It does not direct operators to the output-format page for Deep Capture handling until issue #248 corrects that page's false two-equivalent-formats claim.

**Rationale**: Operators need enough information to handle a first bundle safely, but issues #247 and #248 own the full architecture and artifact-reference corrections. A stale destination cannot supply missing safety guidance. Keeping the minimum handling contract in the guide preserves those issue boundaries without endorsing the output page's known false claim.

**Alternatives considered**:

- Reproduce the complete traffic and artifact tables: rejected because duplicated contracts drift and would absorb #248.
- Link the current output-format page as the bundle authority: rejected because it says both outputs carry the same facts and does not document Deep Capture bundles.
- Mention only the bundle directory: rejected because application observations and key logs are sensitive and differ in authority from packet truth.
- Call all bundle contents capture output: rejected because it falsely collapses packet and proxy observation authority.

## Production Validation

**Decision**: Run focused CLI suites, stale-phrase and privacy audits, documentation checks, the production site build, and the full repository gate. Defer interactive route, responsive, search, keyboard, and accessibility auditing to issue #249.

**Rationale**: The selected checks prove source correctness and buildability. Claiming the full rendered audit here would preempt a separately scoped issue whose acceptance criteria require production browsing at multiple viewports.

**Alternatives considered**:

- Skip the production export because only one page changes: rejected because MDX imports, links, and markup can fail only at build time.
- Perform the full site audit now: rejected as disproportionate and explicitly owned by #249.
