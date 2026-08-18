# Phase 0 Research: doctor gains an action layer (--fix)

This resolves the unknowns and governance questions the plan depends on. Two are
decisive (the constitution carve-out and the npcap license determination); the rest
are technical decisions grounded in the existing code.

## D-1: npcap license determination (FR-013, the required gate)

**Decision**: A tool that fetches the official npcap (or Wireshark) installer from
the vendor's own official location over HTTPS and launches it on the user's machine,
embedding/copying/hosting/bundling nothing, does NOT redistribute npcap and is
permitted by the npcap license.

**Evidence** (npcap LICENSE, https://github.com/nmap/npcap/blob/master/LICENSE, and
https://npcap.com/):

- Free use is granted: the license "entitles you to install and use five (5) copies
  of the Software" (unlimited when used only with Nmap, Wireshark, or Microsoft
  Defender for Identity). An individual may download and run the official installer
  at no cost.
- The prohibition is on redistribution and transfer: npcap "is not open source
  software and may not be redistributed or used in other software without special
  permission"; a licensee "may not ... redistribute, encumber, sell, rent, lease,
  sublicense, or otherwise transfer your rights to the Software Product." The OEM
  Redistribution License exists precisely for companies that "wish to distribute
  Npcap OEM within their products," which the free edition does not allow.
- The license contains no clause restricting a user (or a tool acting on the user's
  behalf on the user's machine) from downloading the official installer from the
  vendor or executing it.

**Why this is not ambiguous for our action**: fragcap redistributes nothing. It
downloads the vendor's signed installer from the vendor each time and launches it;
npcap arrives on the user's machine through the vendor's own installer, never
through a fragcap artifact. This is functionally the user clicking the vendor's
download link, automated on their machine under an explicit per-action
confirmation. The redistribution/transfer prohibitions do not reach it. The
operator's Option B decision named exactly this action ("the vendor's own installer,
not bundling/redistributing"), so the intent is settled as well as the license.

**Guardrails carried into the design**: fetch only from the vendor's official
location; never store the installer in any fragcap artifact or cache it as
fragcap's own; verify it is the vendor's signed installer where practical; only on
explicit interactive confirmation. This determination is recorded in a
`changelog.d/` decisions fragment (SC-005).

## D-2: constitution Licensing rule 2 carve-out (operator decision, 2026-08-18)

**Decision**: Amend constitution Licensing rule 2 from an absolute ("It never
downloads, installs, or invokes an installer") to a narrow, user-confirmed carve-out
that permits fetching and launching the vendor's own installer. Rules 1 (no
bundling), 3 (documented prerequisite), and 4 (no SDK vendoring) stay absolute, and
P-1/P-9 are untouched. Version bump 1.2.0 -> 1.3.0 (MINOR: a scoped expansion of an
existing section's guidance, no core principle removed or redefined).

**Proposed amended rule 2 text** (applied as a task in this slice):

> 2. Detection, and user-confirmed fetch of the vendor installer. fragcap detects
>    npcap's presence and version at runtime and reports absence with the official
>    download location. It never bundles, hosts, embeds, caches as its own, or
>    redistributes npcap or its installer (rules 1 and 4 remain absolute). It may,
>    only under an explicit interactive user confirmation (as in `doctor --fix`),
>    download the vendor's own signed installer from the official location and launch
>    it, storing nothing in any fragcap artifact and redistributing nothing. Absent
>    that confirmation, and in every non-interactive or machine-readable context, it
>    reports the download location and neither fetches nor launches.

**Rationale**: The operator chose to enable the fetch action (issue #143's action
table lists it) rather than ship link-only or defer. The carve-out is the minimum
change that enables it while keeping every redistribution and passivity guarantee
intact. It is recorded in the constitution's Sync Impact Report and a dated
`changelog.d/` decisions fragment.

**Alternatives considered**: (a) keep rule 2 absolute, npcap action opens the page
only, offered and declined by the operator; (b) defer the npcap action entirely,
offered and declined. Both are strictly simpler but do not deliver the action the
operator wants.

**Sequencing**: The unamended rule already permits the degraded default (open the
page), so no code path depends on the carve-out before it lands. The amendment task
runs with the slice.

## D-3: the classifier stays pure; new findings enter as additive checks

**Decision**: Preserve the classifier as a pure `Inputs -> Report` function.
Existing checks and their tests are unmodified. The two findings the action table
needs that the classifier does not yet name are added as NEW pure checks fed by
`Inputs`:

- catalog store missing: today `catalog_db_present` is only an informational
  identity note. Add a check (section Integration or a new Catalog section) that
  warns and carries a fetch action when the store is absent.
- no target entries: today there is no target-entry signal in `Inputs` at all. Add a
  probe fact (a target-entry count read from `local.db`) and a check that warns and
  carries a discovery action when the count is zero.

Both are warnings, never blocking failures: a machine with npcap, the backend, and
interfaces is still "Ready to capture" and `doctor` (no `--fix`) still exits 0
(FR-001, FR-019). This keeps SC-001 (byte-identical `doctor` output) true for the
ready case, because a ready machine already has a catalog and at least one entry;
the new rows appear only when the corresponding fact is missing, and the existing
goldens are for ready machines.

**Rationale**: FR-003 (act only on what the report named) requires the report to
name these findings before `--fix` can act on them. Adding a probe read inside the
classifier would break purity (P-2, FR-002); the fact is gathered by the thin probe
and passed in, exactly as every other input is.

**Note on goldens**: any golden that exercises an absent catalog or zero entries
gains two rows; goldens for ready machines are unchanged. The drift is confined to
new fixtures the slice adds.

## D-4: the confirm seam

**Decision**: Introduce an `ActionConfirm` seam in `fragcap-cli` modeled on the
existing `fragcap-targets::sources::interactive::Confirm` seam: a trait with a
console implementation (reads a yes/no from stdin, as `prompt_socket_holder` already
does in `commands/targets.rs`) and a scripted double for tests. The seam confirms a
described action (its human name), not a candidate target, so it is a small distinct
trait rather than a reuse of the targets one.

**Rationale**: This is the established pattern for making an interactive path
testable without a console (S052/S055, FR-017). A scripted double drives the confirm
loop in `cli_doctor.rs` with no terminal.

## D-5: refusal gating (--json, non-TTY, --yes)

**Decision**: `--fix` refusal and gating live in the command shell
(`commands/doctor.rs`), before any action runs:

- `--fix` with `--json`: usage error, exit 2. `doctor` uses `Exit`/`CliError`;
  the exit-2 usage path already exists for other commands.
- `--fix` when stdout is not a terminal: usage error, exit 2. Keyed on
  `std::io::stdout().is_terminal()`, consistent with how `doctor` already decides
  colorization (`use_color`). The visible interaction surface is stdout; the prompt
  reads stdin.
- `--yes` without `--fix`: usage error, exit 2 (enforced at the arg layer or the
  shell).
- `--yes` with `--fix`: pre-confirms each action but still requires an interactive
  stdout (does not bypass the non-TTY refusal).

**Rationale**: FR-007, FR-008, FR-009. Keeping the gates in the shell means the pure
selection logic in `action.rs` never has to know about terminals.

## D-6: the six actions, how each is performed, and net gating

**Decision**: Each finding maps to one action; the performing code reuses existing
capabilities:

| Finding | ActionKind | How performed | Gating |
| --- | --- | --- | --- |
| npcap absent | ObtainNpcap | net: fetch vendor installer + launch; default: open download page | net feature; else degrade |
| WinPcap API mode off | RelaunchNpcapInstaller | explain, then launch the npcap installer (same fetch/launch path) | net feature; else degrade to page |
| Not elevated | RelaunchElevated | relaunch the same `doctor --fix` elevated; parent reports handoff and stops | Windows side effect (Tier 2) |
| extcap not registered | InstallExtcap | reuse `extcap install` at chosen scope (user/machine) | none |
| catalog store missing | FetchCatalog | reuse `catalog update` | net feature; else degrade to naming the command |
| no target entries | RunDiscovery | reuse the S055 discovery composition (tiers 1 and 2) | none |

Network actions are gated on the existing `net` feature, consistent with
`catalog update` (which already refuses without `net` and names the manual step).
A default build offers the degraded form and still tells the operator what to do
(FR-012, FR-016, FR-019).

**Rationale**: P-10 (one path to a target) and no reimplementation: the discovery,
extcap-install, and catalog-fetch actions call the same code the standalone commands
call.

## D-7: elevation relaunch semantics

**Decision**: The RelaunchElevated action relaunches the same `fragcap doctor --fix`
invocation elevated (Windows `runas`), then the original non-elevated process reports
the handoff and stops offering further actions; the elevated child re-runs the
classifier and offers the now-unblocked actions.

**Rationale**: elevation gates other actions (tracing, some interfaces); continuing
to act in the non-elevated parent after escalating would act without the privilege
just requested. Re-running in the child is the honest model (Clarifications, FR-014).
The relaunch itself is a Tier 2 side effect; its selection and the handoff message
are Tier 1.

## D-8: testability tiers

**Decision**: Tier 1 (CI, no driver/elevation/network): the pure selection of offered
actions from a Report, the refusal rules, the confirm loop driven by a scripted
double, the degraded presentation in a default build, the two new checks over
hand-built `Inputs`, and the outcome reporting. Tier 2 (not in CI, stated): actually
launching an installer, actually relaunching elevated, actually fetching over the
network. This mirrors the S009/S010 precedent where platform side effects are
demonstrated out of CI and named, not hidden (FR-017, SC-006).
