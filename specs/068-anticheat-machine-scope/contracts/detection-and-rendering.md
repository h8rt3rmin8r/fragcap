# Contract: anti-cheat detection surfaces

## Directory-scan signature contract

`SignatureSet::detect(root)` behavior is unchanged in shape; only
`signatures.json`'s row set grows. Any consumer of `ScanOutcome` (the
pointed-directory source, `targets add <exe>`, `technologies <path>`)
automatically gains the new rows with no code change, per the S053 property
this contract preserves.

## Launch-entry classifier contract

```text
fn classify_launch_entries(entries: &[SteamLaunchEntry]) -> Vec<DetectionFinding>
```

- Pure, no I/O, no panic on any input including empty `entries` or entries
  with every optional field `None`.
- Returns `DetectionFinding { category: AntiCheat, product: "Easy
  Anti-Cheat", .. }` for any entry matching the rules in `data-model.md`.
- Returns nothing for an entry whose only anti-cheat-adjacent text is a
  `description` containing (but not exactly equal to) an anti-cheat-related
  phrase, or whose `arguments` contains `-no-eac` with no positive-enable
  flag also present.
- Duplicate findings across multiple matching entries for the same product
  are the caller's responsibility to merge (via `merge_finding`), not this
  function's.

## Discovery-merge contract (`SteamSource::discover`)

For each Steam title, the candidate's `evidence` is: the directory scan's
findings, then each of `title.anti_cheat`'s findings merged in via
`merge_finding`, then re-sorted by `(category order, product)`. A product
present in both sources appears once, at the stronger fidelity. A title with
no directory-scan findings and no launch-entry findings has an empty
evidence vec, exactly as today.

## Machine-wide probe and rendering contract

- `MachineAntiCheatProbe::detect(&self) -> Vec<MachineAntiCheatFinding>`
  never fails (no `Result`); "could not run" and "ran and found nothing"
  both yield an empty `Vec`.
- `fragcap targets` (bare `fragcap` and `fragcap targets`) calls the real
  probe once per invocation, on Windows only.
- Non-empty result: a `Machine:` heading followed by one indented line per
  finding (`  <product> (<evidence>)`), printed after the per-target table,
  before the footer/next-command line.
- Empty result: no `Machine:` section at all. No text anywhere asserts a
  negative ("no anti-cheat products found").
- No target row's `SENSITIVITIES`/`ENGINE` cell is affected by the
  machine-wide result under any circumstance.

## Exit codes and streams

Unaffected. The machine-wide probe is informational only; its result never
changes `fragcap targets`'s exit code, and its output goes to the same
stdout stream the hero listing already writes to (not the emitter's
stderr), since it is a result fact, not a diagnostic.
