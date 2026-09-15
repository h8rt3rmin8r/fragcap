# S151 Research and Decisions

**Date**: 2026-09-15

## Diagnostic Ownership

Decision: one scoped, explicitly joined worker runs eligible gather/classify/render, while borrowed non-Send Emitter/output remain on the caller thread. A bounded sixteen-event channel preserves serial phase order. Create receiver inside scope closure so coordinator unwind disconnects before automatic joining; disconnected sends are best effort. Join then propagate worker panic, never fabricate readiness. Suppressed and fix paths remain synchronous.

Rationale: existing synchronous `observe` cannot update while work blocks. Official [scope](https://doc.rust-lang.org/std/thread/fn.scope.html), [scoped join](https://doc.rust-lang.org/std/thread/struct.ScopedJoinHandle.html) and [bounded channel](https://doc.rust-lang.org/std/sync/mpsc/fn.sync_channel.html) contracts support owned lifetimes. Explicit join covers worker destruction, unlike a detached timer.

Alternatives considered: detached work risks surviving ownership-sensitive cleanup; moving borrowed writers requires broader Send contracts; per-probe workers expand concurrency; timeout-to-unavailable violates P-9. None is needed.

## Late Boundary Attribution

Decision: retain coarse phases and add fixed readiness leaves for native residue inventory, manifest/artifact scan, manifest CA identities, separate current-user/machine root stores and IPv4/IPv6 readiness. Emit one-second pending elapsed diagnostics and automatic slow completion durations; visible `--timings` covers fast completions too. Two active slots cover parent and leaf.

Rationale: these existing calls are distinct potentially blocking boundaries. Native residue inventory remains an honest aggregate including its own ownership, journal and certificate reads. Current `EtwWatcher::probe_session` starts one session, enables its provider and drops it before return; teardown API errors are currently ignored. [StartTraceW](https://learn.microsoft.com/en-us/windows/win32/api/evntrace/nf-evntrace-starttracew) documents finite session resources and prompt stopping. S151 preserves that behavior and measures the aggregate availability boundary, not a particular historical API cost or newly verified cleanup.

Alternatives considered: optimizing remembered ETW labels guesses the cause; storing timings in final report changes contracts; faster output adds noise. Controlled channel release tests prove waiting-before-completion without real host probes.

## Published State

Decision: add reviewed actual publication identity and exact current baseline markers to the currency gate. Preserve historical tables, S150 preparation evidence, tagged changelog and release notes; explicitly mark S151 diagnostics unreleased. Correct the release handoff's unconfigured registry-approval claim without changing GitHub settings.

Rationale: [v0.10.0](https://github.com/h8rt3rmin8r/fragcap/releases/tag/v0.10.0) was published 2026-09-15 from `787739edfa8d748e25cb4b5c4965f5c936850d16`, with all jobs green in [34990387334](https://github.com/h8rt3rmin8r/fragcap/actions/runs/34990387334). The `crates-io` environment protection rules were empty. Workspace version proves candidate identity, not publication or independent audit.

Alternatives considered: Cargo-derived publication falsely publishes candidates; network-dependent offline CI adds availability/credential dependence; global older-version bans rewrite history. A finite reviewed record and negative tests avoid those defects.

## Scope and Tracking

Decision: #414 tracks engineering delivery under #372 with #331 traceability. Actual elevated first-run/repeat timing, installed independent review/retest #333/#413 and final #334/#278 remain open. Human owns final merge. No release, settings, dependency, storage-schema, workload/budget or immutable artifact change is included.
