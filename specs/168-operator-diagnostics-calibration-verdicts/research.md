# S168 design research

## Shared layout and Doctor

Decision: Extend existing display.rs with ANSI-aware actual-cell ColumnLayout and render_fields, a constant four-space gap, hanging continuations preserving repeated whitespace, and no token truncation. Migrate every inventoried human multicolumn family including live status and bundle cleanup. Add Doctor --history-details; retain original checks and JSON while summarizing healthy history only in human projection. Preserve observed findings when inventory also reports limitations.

Rationale: Existing helpers and renderers already own the data; one projection API replaces literal padding without a new output framework. Terminal width must not relax the owner rule. Completed resource rows are not session inactivity evidence.

Alternatives: Patch targets only (violates #461); keep compact smaller gaps (not authorized); delete old history (outside #458); aggregate machine records (breaks stable identities).

Dependency decision (2026-10-09): Replace incomplete handwritten Unicode width ranges with exact-pinned `unicode-width` 0.2.2, default features off, as a CLI runtime dependency. This adds exactly one lock package, no required transitive package, and no platform edge. The [official manifest](https://raw.githubusercontent.com/unicode-rs/unicode-width/v0.2.2/Cargo.toml) declares Rust 1.66 and MIT OR Apache-2.0; the [official API documentation](https://docs.rs/unicode-width/0.2.2/unicode_width/) covers whole-string emoji and script ligatures. ANSI CSI/OSC removal precedes normal Unicode width calculation. Hand-maintaining more character ranges was rejected because combining scripts and composed emoji require sequence-aware measurement. See [renderer-inventory.md](renderer-inventory.md) for the complete command-family migration and font-independent width contract.

## Selected target preparation

Decision: Add exact Steam discovery/refresh APIs that read only the selected manifest/appinfo identity and declared library metadata. Retain diagnostic provenance alongside existing message fields. Perform query-only process observation first, then minimum identity/workflow applicability lookup; refuse or enter explicit warm restart before expensive work. Keep the launch-time recheck.

Rationale: Current broad discovery and client scan precede running-Steam observation. Exact identities need no unrelated known-root sweep. Typed relevance avoids string matching and preserves true limitations.

Alternatives: Hide warning strings (loses provenance); add progress around unchanged delayed check (fails #463); block all targets when Steam exists (unapproved eligibility change).

Integration correction (2026-10-09): A Steam manifest display-name match cannot establish applicability because a direct target from another source may share that name. The first process snapshot remains before discovery. Stored Steam anchors and exact Steam app selectors apply the guard immediately; uncertain names first resolve the existing cross-source candidate authority, then resolved Steam candidates get a fresh guard before registration confirmation, recursive client setup or workflow/session effects. Exact direct candidate choices remain available while unrelated Steam is running. The controlled one-second refusal budget is specifically for established exact identities. Guessing topology from a title name was rejected because it changed the existing ambiguity and non-Steam eligibility behavior.

## Calibration evidence and verdicts

Decision: Add bounded typed proxy diagnostics through the native lease and additive terminal snapshot fields, including connection causes, completed exchanges, losses, phase windows, and correlation populations. Filter phase eligibility against the observed cutoff before selecting compatibility facts. Reuse exact stored applicability for current readiness, separate from attempted outcome. Human phase/result blocks project those typed authorities.

Rationale: Current diagnosis parses accepted totals from cleanup prose, and aggregate connection failures can hide completed exchanges. Artifact written status is not semantic completeness. Current missing-flow correlation is still #468.

Alternatives: Infer routing from listener accepts or HTTP 200 (incorrect ownership); derive readiness from invocation count (incorrect identity); generic gameplay retry (unsupported remedy); weaken conservative correlation (contrary to accurate observation).

## Process and publication

Decision: Execute installed Spec-Kit command workflows and scripts. The checklist prerequisite requires a plan file, so setup-plan first copied only its template; checklist validation precedes substantive design. There are no extension hooks. Use a temporary out-of-repository launcher with CreateNoWindow, redirected output, and disabled interactive stdin for Bash/Cargo. Direct Git/GitHub commands remain direct.

Rationale: S168 is explicitly authorized through official PR publication and at most two total review rounds. Final merge and release remain owner actions. Runtime effects and existing private bundles are not required for acceptance.
