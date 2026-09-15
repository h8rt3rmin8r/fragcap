# Research: S149 Session Presentation

## 2026-09-15 Authority Audit

Decision: Retain canonical JSON and add a readable consequence summary. Rationale: S134 hashes complete target, launch, route, trust, deadlines, artifacts, and cleanup scope, and existing tests fail closed on write failure. Alternative: replace JSON with a short plan, rejected because undisplayed authority would weaken informed consent.

Decision: Project production typed events, leaving structured schemas unchanged. Rationale: `LibraryEventAdapter` already receives proxy, trust, launch, observation, cleanup, and terminal facts, but human mode discards several events. Alternative: add new lifecycle policy or another event vocabulary, rejected as unnecessary parallel authority.

Decision: Maintain fixed inspection counters and emit the first and each hundredth observation. Rationale: bounded memory and bounded-per-observation telemetry preserve responsiveness without a line for every application record. Alternative: per-record prose, rejected as flooding; silence until terminal, rejected as inadequate live guidance.

Decision: Terminal guidance is a verbosity-gated human summary, with errors still emitted by the existing command. Rationale: quiet must retain the outcome, while silent must not receive optional output. Artifact existence and result state must be checked before claiming retained evidence. Alternative: treat proxy stop as complete or cleanup failure as data loss, rejected because independent evidence authorities must not collapse.

Decision: Use existing display-cell wrapping and controlled tests. Rationale: narrow output and Unicode values already have a shared CLI solution, and contributor policy forbids agent live sensitive execution for implementation acceptance. External technology research is unnecessary because no technology or dependency is selected.

Implementation clarification: Compatibility observations are emitted after collection and reconciliation, not directly from the live proxy. The new display therefore counts delivered observations without calling them live. Existing Capture packet counters remain the live authority; process ownership and application loss reconcile through terminal artifacts. The canonical six-state S120 classification replaces the older four-state compatibility projection only for the new human counters, preserving opaque and decrypted-unknown states without altering structured records.

Final display audit: Generic prose wrapping normalizes whitespace. Exact target names, bundle paths, and cleanup resource identities instead use complete value lines, preserving repeated spaces and allowing an exact value to exceed a narrow width. A failing regression test established this boundary before the correction; shared Doctor wrapping behavior remains unchanged.
