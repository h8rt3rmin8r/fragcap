# Contract: the action catalog

The mapping from a `doctor` finding to the action `--fix` offers, and each action's
degraded form. This is exhaustive: every finding below is surfaced by a check
carrying its action, and every `ActionKind` maps back to exactly one finding.

## Findings and actions

| Finding (check) | Status | ActionKind | Primary action (net-capable build) | Degraded action (default build) |
| --- | --- | --- | --- | --- |
| npcap absent | Fail | ObtainNpcap | Fetch the vendor's signed Wireshark installer (provides npcap) and launch it | Open the official download page; name the npcap-alone alternative in text |
| WinPcap API mode off | Fail | RelaunchNpcapInstaller | Explain, then fetch+launch the npcap installer | Explain, open the download page |
| Not elevated | Warn | RelaunchElevated | Relaunch `doctor --fix` elevated; parent reports handoff and stops | (no network) same behavior |
| extcap not registered | Warn | InstallExtcap{scope} | Run `extcap install` at the chosen user/machine scope | (no network) same behavior |
| catalog store missing | Warn (NEW check) | FetchCatalog | Fetch the published catalog (reuse `catalog update`) | Name the manual `catalog update` command |
| no target entries | Warn (NEW check) | RunDiscovery | Run discovery tiers 1 and 2 (reuse S055 composition), register found titles | (no network) same behavior |

## Rules

- **Single primary action per finding (FR-012)**: the npcap finding offers one
  confirm prompt, not a nested sub-menu. The npcap-alone installer alternative is
  named in the printed guidance, not a separate prompt.
- **Subset invariant (FR-003)**: `--fix` offers exactly the actions carried by checks
  present in the current report. A finding not present yields no action.
- **Elevation offered first (FR-014, L1)**: when `RelaunchElevated` is among the
  offered actions, it is offered before the others, so escalation precedes any
  privilege-gated action.
- **Degraded catalog is guidance only (FR-016)**: in a default build the
  catalog-missing finding surfaces the printed remediation naming `catalog update`
  and is not offered as a confirm prompt; the actionable fetch prompt appears only in
  a `net`-capable build.
- **Degradation (FR-016, FR-019)**: when `net` is absent, a net-required action is
  offered in its degraded form and still tells the operator what to do; it is never
  silently dropped. The catalog action, whose only useful form needs the fetch, is
  offered as "run `catalog update` (needs a net-enabled build)" guidance in a default
  build.
- **Licensing (amended rule 2, D-1/D-2)**: the npcap actions fetch only the vendor's
  own signed installer from the official location, under explicit confirmation,
  storing nothing in any fragcap artifact and redistributing nothing. In any
  non-interactive or `--json` context the npcap action never fetches (it is refused
  along with all of `--fix`).
- **Reuse (P-10)**: InstallExtcap, FetchCatalog, and RunDiscovery call the same code
  as the standalone `extcap install`, `catalog update`, and the S055 discovery
  composition; they add no second path.
- **Honesty (P-9, FR-011)**: each action reports Performed, Skipped, Degraded, or
  Failed; a failed action is never reported as performed, and the final verdict
  reflects reality.
