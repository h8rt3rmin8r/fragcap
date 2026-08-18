# Feature Specification: doctor gains an action layer (--fix)

**Feature Branch**: `056-doctor-action-layer`

**Created**: 2026-08-18

**Status**: Draft

**Input**: Slice S056 (issue #143, milestone v0.5.0). Source:
fragcap-v0.5.0-UX-Handoff-Plan.md sections 12 (S057 placeholder), 3.8, 10.
Depends on S055 (merged). Renumbered to specs ordinal 056.

## Overview

`fragcap doctor` today answers one question: can this machine capture packets. It
is a read-only classifier that probes the environment, reports each check, names
a remediation for every blocking failure, and exits 1 when anything blocks. It
installs nothing and changes nothing. But the user's actual first task is not
diagnosis, it is preparing an environment: install the driver, register the
analyzer integration, seed the catalog, elevate the session. Today the operator
reads the remediations and performs each one by hand.

This slice adds an action layer above the existing classifier. `fragcap doctor`
stays exactly as it is. A new `fragcap doctor --fix` runs the same classifier,
then offers to carry out the remediations the report already named, one at a
time, with the operator's confirmation. Environment preparation is built under
`doctor`; there is no sibling `setup` command, because the only actions offered
are the ones `doctor` already diagnosed and named aloud.

The load-bearing constraint is that the action layer sits strictly above the
classifier and never inside it. The classifier remains a pure function from an
injected `Inputs` struct to a `Report`, over a read-only probe that installs
nothing. That purity is why the entire section 26.3 matrix is testable on a CI
runner with no capture driver and no game, and it is preserved unchanged: this
slice adds no probe, mutates no existing check logic, and leaves every existing
classifier test unmodified. `--fix` consumes the `Report` and may act only on
remediations the classifier already produced. It cannot surprise the operator
with an action `doctor` did not first print, which matters in a tool that may be
running elevated.

## Clarifications

### Session 2026-08-18

- Q: Does `--fix` act only on what the report named, or may it take actions the
  report did not print? A: Only on what the report named. Each actionable `Check`
  carries an optional structured `action` alongside its human-readable
  `remediation` string, so the printed remediation and the offered action cannot
  drift. An action with no corresponding actionable check in the current report
  is never offered or performed (FR-003, the load-bearing safety invariant).
- Q: When is `--fix` refused outright? A: When combined with `--json`, and when
  the process stdout is not an interactive terminal. `--fix` is an interactive,
  confirmation-driven mode; a machine-readable or non-interactive context is a
  usage error (exit 2), not a silent no-op (FR-007, FR-008).
- Q: How does `--fix` run unattended? A: With `--yes`, which pre-confirms every
  offered action. `--yes` has no meaning without `--fix`; `--fix --yes` still
  requires an interactive stdout (it refuses when stdout is not a terminal), so
  `--yes` removes the per-action prompt but not the terminal requirement
  (FR-009). This keeps an elevated, unattended run from acting against a
  redirected or piped session.
- Q: Do the network-dependent actions (fetch the npcap installer, fetch the
  published catalog) run in a default build? A: No. They are gated on the same
  `net` capability the rest of the project gates live network behavior on
  (`catalog update` already needs it). In a default build the npcap action
  degrades to opening or printing the vendor download page, and the catalog-fetch
  action is offered only when the fetch capability is present. A degraded action
  still tells the operator exactly what to do (FR-013, FR-016).
- Q: Is the npcap-absent finding a single confirm prompt or a nested sub-menu of
  installer choices (Wireshark bundle / npcap-alone / links)? A: A single primary
  action per finding, so the confirmation seam stays a flat list of yes/no prompts
  (the same shape as the existing interactive confirm seam) rather than a nested
  menu. The primary action is: in a `net`-capable build, fetch and launch the
  Wireshark installer (which also provides npcap); in a default build, open the
  vendor download page. The alternative source (the npcap-alone installer) is named
  in the printed guidance text, not offered as a separate confirm prompt (FR-012).
- Q: What does the "relaunch elevated" action do, given elevation gates other
  actions in the same run? A: It relaunches the same `fragcap doctor --fix`
  invocation elevated. The elevated child re-runs the classifier in its elevated
  context and offers the now-unblocked actions; the original non-elevated process
  reports the handoff and stops offering further actions rather than continuing to
  act without the privilege it just escalated for (FR-014).
- Q: Does this slice detect a stale catalog, or only a missing one? A: Only a
  missing one. The classifier carries no staleness input today (`catalog_db_present`
  is a bool), and inventing a staleness signal is out of scope. The catalog action
  is offered when the store is absent. Stale-catalog detection is a documented
  follow-up (FR-015, Assumptions).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Fix what doctor named (Priority: P1)

A new user runs `fragcap doctor` and sees several blocking failures: npcap is not
installed, the analyzer integration is not registered, and there are no target
entries yet. Rather than performing each remediation by hand, they run
`fragcap doctor --fix`. The same report prints. Then, for each check that carries
an action, `--fix` names the action it is about to take and asks the operator to
confirm. On yes, it performs the action and reports the outcome; on no, it skips
to the next. When the run finishes, `--fix` reruns the classifier and prints the
updated readiness verdict so the operator sees what changed.

**Why this priority**: This is the whole point of the slice: turning a read-only
diagnosis into a guided preparation. Without it there is no action layer. It is
independently valuable even with only the no-network, in-process actions wired
(register the extcap integration, run discovery, relaunch elevated, open or print
the npcap download page).

**Independent Test**: With the classifier fed a report containing actionable
checks, the action layer offers exactly the actions those checks carry, in report
order, and performs each confirmed action; the decision-and-offer logic is
testable with a scripted confirmation double and injected report, with no capture
driver, no elevation, and no network. The side-effecting actions that genuinely
need the platform (launch an installer, relaunch elevated) are demonstrated at
Tier 2, stated not hidden, per the S010 precedent.

**Acceptance Scenarios**:

1. **Given** a report where the npcap check failed and carries an action, **When**
   the operator runs `doctor --fix` and confirms, **Then** the npcap action runs
   (in a default build, opening or printing the vendor download page) and its
   outcome is reported.
2. **Given** a report where the extcap integration check warned and carries an
   `extcap install` action, **When** the operator confirms, **Then** the
   registration runs at the chosen scope and the outcome is reported.
3. **Given** a report with no target entries, **When** the operator confirms the
   discovery action, **Then** discovery tiers 1 and 2 run and register found
   titles.
4. **Given** any offered action, **When** the operator declines it, **Then** it is
   skipped and the next action is offered, and declining changes nothing.
5. **Given** a run that performed one or more actions, **When** the run finishes,
   **Then** the classifier is rerun and the updated verdict is printed.

---

### User Story 2 - It can only do what it said (Priority: P2)

An operator, possibly running elevated, must be able to trust that `--fix` will
never take an action `doctor` did not first name. The action offered for a check
is bound to that check's structured action, and an action whose check is not
present (or not actionable) in the current report is never offered. `--fix` is
refused with `--json` and when stdout is not a terminal. Unattended use requires
`--yes`, which pre-confirms the per-action prompts but still refuses a non-terminal
stdout.

**Why this priority**: These are the guardrails that make an elevated action layer
safe to ship. They are independently testable and independently valuable: a
`--fix` that could act beyond the report, or act silently into a pipe, would be a
liability regardless of which actions are wired.

**Independent Test**: Purely testable. `doctor --fix --json` and `doctor --fix`
with a non-terminal stdout both exit 2 with a usage error and perform no action.
An action whose check is absent from an injected report is never offered. `--yes`
without `--fix` is a usage error. All checked without any side effect.

**Acceptance Scenarios**:

1. **Given** `--fix` combined with `--json`, **When** the command runs, **Then** it
   exits 2 with a usage error and takes no action.
2. **Given** `--fix` with stdout not a terminal (piped or redirected), **When** the
   command runs, **Then** it is refused (exit 2) and takes no action, even with
   `--yes`.
3. **Given** an action definition whose corresponding check is not actionable in the
   current report, **When** `--fix` runs, **Then** that action is neither offered nor
   performed.
4. **Given** `--yes` without `--fix`, **When** the command runs, **Then** it exits 2
   with a usage error.
5. **Given** `--fix --yes` in an interactive terminal, **When** it runs, **Then**
   each offered action is performed without a per-action prompt.

---

### User Story 3 - The action catalog (Priority: P3)

Each finding `doctor` already reports has a matched action, so the operator can
resolve the machine end to end from one command:

| Finding | Action |
| --- | --- |
| npcap absent | Fetch and launch the Wireshark installer (which provides npcap), or the npcap installer alone, or open/print the download links |
| WinPcap API mode off | Explain, then offer to relaunch the npcap installer |
| Not elevated | Offer to relaunch fragcap elevated |
| extcap not registered | Run `extcap install`, at user or machine scope |
| catalog.db missing | Fetch the current published catalog |
| No target entries yet | Run discovery tiers 1 and 2 |

The network-dependent actions (npcap installer fetch, catalog fetch) are gated on
the `net` capability; without it the npcap action degrades to opening or printing
the vendor download page, and the catalog action is offered only when the fetch
capability is present.

**Why this priority**: The catalog completes the value, but each entry is
additive over the US1 mechanism and can land incrementally. Wiring one action
does not depend on wiring another.

**Independent Test**: Each action's selection and presentation is testable from an
injected report (the right action is offered for the right finding, degraded
correctly when the capability is absent). The side effects are Tier 2.

**Acceptance Scenarios**:

1. **Given** an npcap-absent report in a default build, **When** the operator picks
   the npcap action, **Then** the vendor download page is opened or its links
   printed, and nothing is downloaded.
2. **Given** an npcap-absent report in a `net`-capable build, **When** the operator
   picks the fetch action and the npcap license permits fetching the vendor
   installer, **Then** the installer is fetched from the vendor and launched.
3. **Given** a not-elevated report, **When** the operator confirms, **Then** fragcap
   is relaunched elevated.
4. **Given** a missing catalog store in a `net`-capable build, **When** the operator
   confirms, **Then** the published catalog is fetched into the store.

---

### Edge Cases

- What happens when a report has no actionable checks (a ready machine)? `--fix`
  prints the report, states there is nothing to fix, and exits 0.
- What happens when an action fails (the installer process cannot start, discovery
  errors)? The failure is reported for that action, the run continues to the next
  action, and the final verdict reflects reality. An action failure is never
  presented as success (P-9).
- What happens when the operator declines every action? Nothing is changed and the
  verdict is unchanged from the initial report.
- What happens when `--fix` is passed on a non-Windows build where the probe cannot
  gather? The same refusal and reporting rules apply; actions that cannot run on the
  platform are not offered.
- What happens if the npcap license determination is ambiguous? Implementation
  stops and the operator is asked before the fetch action ships; until then the
  npcap action degrades to opening the download page (section 14, and the
  Assumptions below).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `fragcap doctor` (no `--fix`) MUST remain read-only and behaviorally
  unchanged: same checks, same rendering, same exit code (1 when any check blocks,
  0 otherwise), installing and changing nothing.
- **FR-002**: The classifier MUST remain a pure function from `Inputs` to `Report`.
  This slice MUST NOT change any existing check's status logic and MUST leave every
  existing classifier test unmodified. Adding a NEW pure check (a new function from
  the existing or an additively-extended `Inputs` to a `Check`) is permitted and is
  the only sanctioned way to surface a new actionable finding; it MUST NOT read the
  environment from inside the classifier (any new fact enters through `Inputs`,
  gathered by the thin probe). "Unmodified" scopes to existing assertions and check
  logic: a mechanical addition of the new `Inputs` field to a shared test fixture
  (for example `ready_inputs()`) is permitted and expected, and MUST NOT change any
  existing assertion or existing check's classification.
- **FR-003**: Each `Check` MUST be able to carry an optional structured action
  alongside its human-readable `remediation`, and `--fix` MUST offer and perform
  only actions carried by a check present and actionable in the current report. An
  action with no such check MUST NOT be offered or performed. (The safety
  invariant: `--fix` acts only on what `doctor` named.)
- **FR-004**: The structured action and the human-readable remediation for a check
  MUST be derived so they cannot drift: a check that names a remediation to the
  operator and offers a different action to `--fix` is a defect.
- **FR-005**: `fragcap doctor --fix` MUST first run the same classifier and print
  the same report, then enter the action phase.
- **FR-006**: In the action phase, `--fix` MUST offer each actionable check's action
  in report order, naming the action before performing it, and MUST perform an
  action only after the operator confirms it (or `--yes` pre-confirms it).
- **FR-007**: `--fix` combined with `--json` MUST be refused as a usage error (exit
  2), taking no action.
- **FR-008**: `--fix` MUST be refused as a usage error (exit 2) when the process
  stdout is not an interactive terminal, taking no action. Because the confirmation
  prompt reads stdin, `--fix` (without `--yes`) MUST also require stdin to be an
  interactive terminal, so a terminal stdout paired with a redirected stdin is
  refused rather than silently reading end-of-file and skipping every action. (`--yes`
  supplies the confirmations and does not read stdin, so it needs only the stdout
  terminal of this requirement.)
- **FR-009**: `--fix` MUST accept `--yes` to pre-confirm every offered action for
  unattended use; `--yes` MUST still refuse a non-terminal stdout (FR-008), and
  `--yes` without `--fix` MUST be a usage error (exit 2).
- **FR-010**: After performing one or more actions, `--fix` MUST rerun the
  classifier and print the updated readiness verdict.
- **FR-011**: An action that fails MUST be reported as failed for that action; the
  run MUST continue to the remaining actions, and a failed action MUST NOT be
  reported as succeeded (P-9).
- **FR-012**: The npcap-absent finding MUST offer a single primary action to obtain
  npcap (not a nested sub-menu), keeping the confirmation seam a flat list of yes/no
  prompts. The primary action is: in a `net`-capable build, fetch and launch the
  Wireshark installer (which provides npcap); in a default build, open the vendor
  download page. The npcap-alone installer alternative MUST be named in the printed
  guidance text rather than offered as a separate prompt. fragcap MUST NOT bundle,
  vendor, or host npcap; it only fetches the vendor's own installer or points at it
  (the Licensing non-negotiable).
- **FR-013**: The npcap fetch action MUST NOT be implemented until the npcap license
  text has been read and confirmed to permit downloading the vendor's installer from
  the vendor and launching it; the determination MUST be recorded in a
  `changelog.d/` decisions fragment. If the license does not permit it, the action
  MUST degrade to opening the download page. If the determination is ambiguous,
  implementation MUST stop and ask the operator (section 14).
- **FR-014**: The not-elevated finding MUST offer an action to relaunch the same
  `doctor --fix` invocation elevated; on confirmation the elevated child re-runs the
  classifier and offers the now-unblocked actions, and the original non-elevated
  process reports the handoff and stops offering further actions. The
  WinPcap-API-mode-off finding MUST offer an action to relaunch the npcap installer;
  the extcap-not-registered finding MUST offer an `extcap install` action that asks
  or accepts a user-or-machine scope. When a `RelaunchElevated` action is among the
  offered actions, it MUST be offered first, so escalation precedes any
  privilege-gated action rather than running after actions the parent already
  performed.
- **FR-015**: The no-target-entries finding MUST offer an action to run discovery
  tiers 1 and 2 (the same discovery the S055 hero listing runs), registering found
  titles.
- **FR-016**: The missing-catalog-store finding MUST offer an action to fetch the
  current published catalog. The actionable fetch (a confirm prompt that runs the
  fetch) is offered only in a build with the fetch capability; in a default build the
  finding surfaces guidance only (the printed remediation naming the manual command),
  with no confirm prompt for a step it cannot perform. The finding is never silently
  dropped.
- **FR-017**: The decision logic of the action layer (which action for which
  finding, the refusal rules, the confirmation gating, the ordering, the
  capability-degraded presentation) MUST be testable without a capture driver,
  without elevation, and without the network; any part that genuinely requires the
  platform MUST be marked Tier 2 and stated, not silently skipped.
- **FR-018**: A new term introduced by this slice (for example "action layer",
  "structured action") MUST get a glossary entry in the same change (P-6).
- **FR-019**: Because `--fix` may act only on findings the report named (FR-003),
  the catalog-store-missing finding and the no-target-entries finding MUST each be
  surfaced by the classifier as an actionable check carrying its action. Today the
  catalog store's presence is only an informational identity note and there is no
  target-entry signal at all; this slice MUST add these as new, additive pure checks
  (FR-002), fed by `Inputs` (the existing `catalog_db_present`, and a new
  target-entry-count fact gathered by the probe). A ready machine with a catalog and
  at least one target entry MUST NOT be pushed to a failing verdict by these checks:
  their absence is a warning that carries an action, not a blocking failure, so
  `doctor` (without `--fix`) still exits 0 on an otherwise ready machine (FR-001).

### Key Entities *(include if feature involves data)*

- **Structured action**: The machine-facing counterpart of a check's remediation
  string. Identifies which remediation `--fix` can perform for a check and carries
  what it needs to perform it. Optional on a check (informational checks carry
  none). Bound to the check so the printed remediation and the offered action
  cannot diverge.
- **Action outcome**: The result of attempting one action: performed, skipped
  (declined), degraded (a capability-limited fallback ran), or failed. Reported per
  action and never misreported (P-9).
- **Confirmation seam**: The interface through which `--fix` asks the operator to
  confirm an action, with a console implementation and a scripted test double (the
  same shape as the existing interactive confirm seam), so the decision logic is
  driven from tests.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `fragcap doctor` output and exit codes are byte-for-byte and
  code-for-code identical to before this slice (the existing goldens and classifier
  tests pass unmodified).
- **SC-002**: From a blocked machine, an operator can reach a ready (or maximally
  prepared) state by running one command, `fragcap doctor --fix`, and confirming
  the offered actions, without typing any remediation command by hand for the wired
  actions.
- **SC-003**: `--fix` never performs an action absent from the report it printed;
  demonstrated by a test in which an action definition whose check is not present is
  never offered.
- **SC-004**: `--fix` with `--json` and `--fix` with a non-terminal stdout are both
  refused with a usage error and take no action; demonstrated by tests.
- **SC-005**: The npcap license determination is recorded in a `changelog.d/`
  decisions fragment before any fetch action ships.
- **SC-006**: The action layer's selection, refusal, and gating logic is covered by
  tests that run on a CI runner with no capture driver, no elevation, and no
  network.

## Assumptions

- The action layer lives entirely in `fragcap-cli`, alongside the existing `doctor`
  classifier and command shell; no core, capture, attribution, sink, or parser
  change is needed. The wired actions reuse existing capabilities (`extcap install`,
  the S055 discovery composition, `catalog update`) rather than reimplementing them.
- Network-dependent actions (npcap installer fetch, catalog fetch) are gated on the
  existing `net` capability, consistent with `catalog update`. A default build
  offers the degraded, no-network form (open or print links, name the manual
  command). This mirrors how `pcap`/`live` and `http_req`/`net` are already gated,
  and keeps the default build and the 1.82 MSRV gate free of the network stack.
- "catalog.db stale" from the handoff plan is narrowed to "catalog.db missing" for
  this slice, because the classifier has no staleness input today and adding one is
  out of scope. Stale-catalog detection is an additive follow-up.
- The confirmation seam follows the existing interactive seam pattern (a console
  implementation plus a scripted double), so the interactive path is exercised in
  tests without a real terminal.
- Relaunching elevated and launching an installer are platform side effects
  demonstrated at Tier 2 (not run in CI), consistent with the S009/S010 precedent
  for platform-gated behavior; their selection and presentation are tested at
  Tier 1.
- The npcap installer fetch/launch presentation intersects the installer exit-dialog
  confusion tracked in issue #133; this slice presents the action clearly but does
  not own that issue's resolution.
- Constitution P-1 (passive observation) and P-9 (the instrument does not lie) both
  bind: `--fix` modifies the local environment only through named, confirmed
  actions, never traffic and never a target process, and never misreports an
  outcome.
