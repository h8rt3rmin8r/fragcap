# S168 data model

## Aligned report

An ordered finite row set owns actual sanitized cells and indentation. Column widths include headings and optional labels. Anchors advance by maximum previous display width plus four; styling has zero width. Whole strings use exact-pinned `unicode-width` 0.2.2 after ANSI CSI/OSC removal. Normal ambiguous-width characters are narrow. Wrapping hangs at the value anchor; a newline replaces one separator at a word break, additional spaces remain in the continuation, and exact indivisible tokens may overflow width. Empty trailing cells add no padding.

## Discovery diagnostic

A diagnostic retains its original message and source/root/target/operation provenance. The caller selects command scope through typed identity. Missing-target fallback reports all coverage actually relevant to that lookup; broad output retains the whole inventory.

## History projection

Original Doctor checks retain stable session/resource identity and lifecycle classifications. Human history summary counts distinct sessions and resources only within observed inventory; limitations and actionable records remain separate. The projection never changes recovery eligibility.

## Calibration evidence and decision

Additive optional terminal diagnostics retain finite connection totals, category/code counts, bounded exact failures, exchange totals, correlation populations, and losses. Observation cutoff and owner-release end define eligibility independently from final writer completion. Semantic artifact completeness derives from manifest/trace authority, while written/access status remains independent. Attempted verdict and current exact requested-case readiness have separate reasons. Supported next command is optional; absent remedy names its unresolved blocker. Existing target/workflow identity and append-only facts remain authoritative.

Version-one CompatibilityObservation and TerminalSnapshot fields remain unchanged. PhaseQualifiedObservation and the optional PhaseObservationDrain attach eligibility separately. TerminalDiagnostics carries the finite proxy projection, exact cutoffs and an index-stable observation window vector through an additive Diagnostics event emitted before fact and artifact finalization. Missing window indices are unavailable to explicit window-aware helpers. Legacy raw helper and drain contracts remain available; native producers supply exact windows. CLI JSON publishes deep_capture.diagnostics without changing the original application record constructor.
