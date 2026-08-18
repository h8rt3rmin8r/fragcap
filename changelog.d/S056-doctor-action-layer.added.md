<!-- spec-impact: 26.3 -->

### doctor gains an action layer (`doctor --fix`, slice S056)

`fragcap doctor` is unchanged: it stays a read-only classifier that probes the
environment, reports each check, names a remediation for every blocking failure,
and exits 1 when anything blocks. A new `fragcap doctor --fix` adds an action
layer above that same classifier. It prints the same report, then offers to
perform the remediations the report named, one at a time, under the operator's
confirmation. It acts only on remediations `doctor` already printed: each
actionable check now carries a structured action alongside its human-readable
remediation, constructed together so the two cannot drift, and `--fix` offers only
the actions carried by a check present in the current report.

`--fix` is interactive and confirmation-driven, so it is refused (usage error,
exit 2) when combined with `--json`, when stdout is not an interactive terminal,
and, without `--yes`, when stdin is not a terminal. `--yes` pre-confirms every
offered action for unattended interactive use but still requires a terminal
stdout; `--yes` without `--fix` is a usage error. After the action phase, the
classifier is re-run and the updated verdict is printed. Every action reports its
honest outcome (performed, skipped, degraded, or failed); a failed action is never
reported as success.

The actions offered map to the findings `doctor` already reports: obtain npcap
(fetch and launch the vendor's own signed installer in a `net`-capable build, or
open the official download page otherwise), relaunch the npcap installer for the
WinPcap API mode, relaunch elevated (offered first so escalation precedes
privilege-gated work; the elevated child re-checks and the parent stops), register
the analyzer extcap integration, fetch the published catalog, and run discovery
(tiers 1 and 2). The catalog and npcap fetch actions are gated on the `net`
capability and degrade in a default build (the catalog finding becomes guidance
naming `catalog update`; the npcap finding opens or names the download page). The
extcap, catalog, and discovery actions reuse the existing `extcap install`,
`catalog update`, and discovery-composition paths, so there is one path to each
effect.

Two findings the classifier did not previously name are added as new, additive
pure checks so `--fix` can act on them: a missing catalog store (warns, carries
the fetch action) and no registered target entries (warns, carries the discovery
action). Both are warnings, never blocking failures, so a ready machine, one with
a catalog and at least one target entry, still passes and exits 0 with unchanged
output. The classifier remains a pure function from an injected `Inputs` to a
`Report`; the new target-entry count enters through the thin probe, and the action
layer lives entirely in `fragcap-cli` above the classifier.

This change amends constitution Licensing rule 2 to permit the user-confirmed
vendor-installer fetch; see the decisions fragment for the npcap license
determination and the amendment.
