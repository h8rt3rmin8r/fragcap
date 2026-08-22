# Phase 1 Data Model: Anti-cheat detection and machine-scope presence

No schema migration. `SCHEMA_VERSION` stays at its current value (8): the new
signature rows use existing valid `kind`s, and every new finding rides the
existing `evidence` JSON column (per target) or is computed fresh and never
persisted (machine-wide).

## Reused, unchanged

- **`DetectionFinding`** (`fragcap-profile::signature`): `{ category,
  product, evidence, fidelity }`. Every new producer in this slice (the
  launch-entry classifier) constructs this type directly; nothing new is
  added to it.
- **`SignatureCategory::AntiCheat`**: unchanged; every new finding in this
  slice uses this existing category.

## New: `merge_finding` (extracted, not new behavior)

```text
pub fn merge_finding(findings: &mut Vec<DetectionFinding>, candidate: DetectionFinding)
```

Lives in `fragcap-profile::signature`, extracted verbatim from
`SignatureSet::detect`'s existing inline block. Same-`(category, product)`
findings collapse to one, keeping the stronger fidelity. `detect` calls it
internally (no behavior change); the facade's Steam discovery merge site
(below) calls it a second time.

## New: the launch-entry classifier (`fragcap-steam::anti_cheat`)

```text
pub fn classify_launch_entries(entries: &[SteamLaunchEntry]) -> Vec<DetectionFinding>
```

Pure function, no I/O. For each entry:

- `executable` case-insensitively equals `start_protected_game.exe` -> Easy
  Anti-Cheat, evidence `"launch executable: start_protected_game.exe"`,
  fidelity `HeuristicUnverified` (a launcher shim name, not a byte-exact
  marker).
- `arguments` (if present) contains `-anticheat_settings=` or
  `-force_enable_eac_module` -> Easy Anti-Cheat, evidence is the matched
  flag substring, fidelity `HeuristicUnverified`.
- `description` (if present), normalized (trimmed, lowercased), exactly
  equals `"eac-release"` or `"easy anti cheat"` -> Easy Anti-Cheat, evidence
  is the original (non-normalized) description string, fidelity
  `HeuristicUnverified`.

A single entry may trigger more than one rule (e.g. both the executable and
the arguments rule); the caller merges duplicates via `merge_finding`, so
the classifier itself does not need to dedupe within one entry's matches
before returning, it returns every match, `merge_finding` collapses them.

No rule inspects `os`, `osarch`, `beta_branch`, or `launch_type`; these carry
no anti-cheat signal per the issue's measured data.

## New: `InstalledTitle.anti_cheat` (`fragcap-steam::library`)

```text
pub struct InstalledTitle {
    // ...existing fields...
    pub anti_cheat: Vec<DetectionFinding>,
}
```

Populated in `discover_in`, at the same point `appinfo_index` is built: the
`appinfo_index` value tuple gains a third element,
`(Option<String> /* common_type */, Option<String> /* launch_executable */,
Vec<DetectionFinding> /* anti_cheat */)`, computed via
`classify_launch_entries(&app.launch)` before `app.launch.first()` is read
(a borrow, not a move, so both reads coexist). `read_manifest` destructures
the 3-tuple and carries the third element onto `InstalledTitle` unchanged.
An app_id absent from the index (never seen in appinfo, or the cache itself
absent) yields an empty `Vec`, exactly as `app_type`/`launch_executable`
already default to `None` in that case.

## New: machine-wide probe seam (`fragcap-targets::machine_probe`)

```text
pub struct MachineAntiCheatFinding {
    pub product: String,
    pub evidence: String,
}

pub trait MachineAntiCheatProbe {
    fn detect(&self) -> Vec<MachineAntiCheatFinding>;
}

pub struct FixtureMachineAntiCheatProbe {
    findings: Vec<MachineAntiCheatFinding>,
}
impl FixtureMachineAntiCheatProbe {
    pub fn new(findings: Vec<MachineAntiCheatFinding>) -> Self { .. }
}
impl MachineAntiCheatProbe for FixtureMachineAntiCheatProbe {
    fn detect(&self) -> Vec<MachineAntiCheatFinding> { self.findings.clone() }
}
```

No `FidelityTier` on `MachineAntiCheatFinding`: a machine-wide finding is
never merged into a title's evidence array (FR-007), so it never needs to
compete for fidelity against a title-scope finding; it is rendered as its
own, separately-labeled fact.

`detect(&self)` never returns a `Result`: per FR-008, a probe that cannot
run (non-Windows, a permission failure) returns an empty `Vec`, identical in
shape to "ran and found nothing." The caller's rendering rule (below) is
what keeps that indistinguishable-internally state from being rendered as a
false "confirmed clean" claim: it prints nothing at all when the vec is
empty, in either case, so no code path ever asserts "no anti-cheat found."

## New: `WindowsMachineAntiCheatProbe` (`fragcap::machine_probe`, `#[cfg(windows)]`)

```text
pub struct WindowsMachineAntiCheatProbe;
impl fragcap_targets::MachineAntiCheatProbe for WindowsMachineAntiCheatProbe {
    fn detect(&self) -> Vec<MachineAntiCheatFinding> { .. }
}
```

Checks exactly one known product: Easy Anti-Cheat, via
`service_registered("EasyAntiCheat_EOS")`, an existence-only registry check
(`RegOpenKeyExW` under `HKLM\SYSTEM\CurrentControlSet\Services\EasyAntiCheat_EOS`,
`RegCloseKey` on success, no value read). On success, evidence is
`"service EasyAntiCheat_EOS registered"`.

## CLI rendering contract (informal; the formal contract is in `contracts/`)

`fragcap-cli::commands::targets`'s hero listing calls the probe once (a
`#[cfg(windows)]`-only call site; a non-Windows build never references the
Windows adapter and the section is simply never printed there), and, only
when the result is non-empty, prints:

```text
Machine:
  Easy Anti-Cheat (service EasyAntiCheat_EOS registered)
```

No target row is touched by this. The section's absence on an empty or
failed probe is the entire mechanism satisfying FR-008; there is no
"no anti-cheat products found" text anywhere in this slice.
