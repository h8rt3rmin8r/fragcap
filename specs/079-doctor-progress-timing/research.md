# Research: Doctor Progress And Timing

## Decision: Observe Probe Gathering, Not Report Rendering

Doctor silence comes from work that happens before the report exists:
`probe::gather()` performs local environment checks and only then the command
classifies and renders. Progress must therefore attach to probe gathering rather
than to the final report renderer.

**Rationale**: The report renderer is intentionally pure and golden-tested. If
progress is mixed into report construction, redirected output and `--json` become
harder to keep byte-stable. A probe observer gives immediate feedback while
leaving readiness classification unchanged.

**Alternatives rejected**:

- Rewrite the report incrementally: rejected because it makes the renderer
  stateful and risks changing stable output.
- Print a startup banner only: rejected because it proves liveness once but does
  not identify the slow probe.

## Decision: Enable Progress Only For Interactive Human Reports

Progress is enabled only when the doctor command is rendering the human report
to a terminal, is not running `--json`, is not running `--fix`, and the emitter is
not quiet or silent. The stream is stderr; stdout remains the report surface.

**Rationale**: The issue is an interactive operator problem. Automation relies on
stable stdout, and existing JSON consumers must not receive additional records.

**Alternatives rejected**:

- Always write progress to stderr: rejected because redirected human output and
  quiet/silent invocations must stay noise-free.
- Add progress JSON records: rejected because `doctor --json` is already a
  stable machine surface.

## Decision: Add Hidden `--timings` To The Interactive Progress Surface

`fragcap doctor --timings` displays elapsed time per probe in the same
interactive progress stream. It does not change the final human report or JSON
schema and is intentionally hidden from the public help surface for now.

**Rationale**: Issue #202 requires measurement before optimizing suspected
costs. Hidden timings give maintainers local evidence without making timing
values a stable automation contract.

**Alternatives rejected**:

- Add timings to final report: rejected because existing doctor goldens and
  user-facing report shape should stay unchanged.
- Add timings to JSON: rejected because that would change the machine contract.

## Decision: Use Coarse, Stable Probe Labels

The observer names the user-meaningful probe groups: identity, platform, capture
driver and interfaces, process event tracing, analyzer integration, target
stores, Deep Capture readiness, and report rendering.

**Rationale**: These labels are granular enough to isolate the suspected #203
and #204 costs without exposing internal function names or brittle platform
implementation details.

**Alternatives rejected**:

- One timing per low-level helper: rejected because it makes the output noisy and
  couples progress to private implementation details.
- One broad "checking system" item: rejected because it cannot identify the slow
  probe.

## Decision: Keep #203 And #204 As Separate Follow-up Fixes

This slice measures the duplicate Npcap enumeration and ETW watcher startup
costs but does not optimize them.

**Rationale**: Issue #202 is about visible progress and evidence. Optimizing the
probes before timing them risks solving the wrong problem and blurs review of
behavioral output preservation.

**Alternatives rejected**:

- Fold #203 and #204 into S079: rejected because those are behavior-preserving
  optimization slices that should be justified by measured dominant costs.
