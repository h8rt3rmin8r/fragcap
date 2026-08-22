# Phase 0 Research: Anti-cheat detection and machine-scope presence

## Decision: fix the in-tree signature set first, as pure data

**Decision**: Add five new rows to `crates/fragcap-targets/assets/signatures.json`
under the existing `anti-cheat` category, using only the two `kind`s already
in the schema's CHECK set: `filename` (`EasyAntiCheat*.exe`, `EACLaunch.exe`,
`AntiCheatInstaller.exe`, `start_protected_game.exe`) and `directory-shape`
(`EasyAntiCheat/`, `EasyAntiCheat_EOS/`).

**Rationale**: The issue measured these exact artifacts on a real machine
against the existing (2-row) EAC signature set and showed none of them
match. This alone, with no new code, is enough to satisfy SC-001 (both
measured titles report Easy Anti-Cheat) via the existing directory-scan
machinery, `all_markers_tree` fixture, and rendering path (readiness.rs's
`sensitivities_summary` already renders any `anti-cheat`-category finding
with zero code change, per S065).

**Alternatives considered**: A new signature `kind` for a bootstrapper
"shape" that bundles the directory and its known children in one rule was
considered and rejected: the existing `filename`/`directory-shape` kinds
already express every artifact the issue measured, so a new kind would add
matching-engine surface for no new expressive power.

## Decision: the launch-entry classifier is a pure function in `fragcap-steam`

**Decision**: `fragcap-steam` already depends on `fragcap-profile` (for
other reasons, per its `Cargo.toml`), so `classify_launch_entries(entries:
&[SteamLaunchEntry]) -> Vec<DetectionFinding>` can live directly in a new
`crates/fragcap-steam/src/anti_cheat.rs`, producing the same
`fragcap_profile::signature::DetectionFinding` type the directory scanner
produces, with no new cross-crate dependency edge.

**Rationale**: `SteamLaunchEntry::arguments`, `::description`, and
`::executable` are already parsed and held in memory (verified: `appinfo.rs`
lines 46-68 and 467-476); no new appinfo-tree navigation or fixture-encoder
change is needed for this source, only a new matching function over data
already extracted.

**Alternatives considered**: Placing the classifier in the `fragcap` facade
(alongside the Windows-specific adapters) was considered, since the facade
is the usual home for cross-cutting composition in this codebase. Rejected
because `fragcap-steam` already has the direct dependency edge to
`fragcap-profile` that this classifier needs, and the classifier is pure
(platform-neutral), placing it in `fragcap-steam` keeps it testable with no
Windows dependency at all, and keeps the facade's role limited to genuinely
platform-specific adapters, matching the codebase's stated placement
argument for `SteamSource`/`WindowsVolumeInventory`.

## Decision: the classifier matches specific tokens, never a broad substring

**Decision**: The classifier matches exactly: `arguments` containing
`-anticheat_settings=` or `-force_enable_eac_module`; `executable`
case-insensitively equal to `start_protected_game.exe`; `description`
case-insensitively equal to `eac-release` (exact, not substring). It does
**not** match any broader token like "anti-cheat" or "anti cheat" appearing
anywhere in `description`.

**Rationale**: The issue's own measured data contains a direct counter-example:
a Halo: MCC launch entry whose `arguments` includes `-no-eac` and whose
`description` reads "Halo: MCC Anti-Cheat Disabled (Mods and Limited
Services)". A substring match on "anti-cheat" in `description` would report
this explicitly-disabled variant as evidence *for* anti-cheat, which is the
exact class of false positive the issue devotes a whole section to warning
against for `EOSSDK-Win64-Shipping.dll`. Every token actually matched is
either an enable-flag unique to a protected launch, the canonical EAC
launcher shim's exact filename, or an exact (not partial) description value
observed only on a genuinely protected entry.

**Alternatives considered**: Matching `description` for the substring "easy
anti cheat" (case/space-insensitive) was considered, since that is one of
the issue's own measured values. Rejected in favor of an exact match after
noticing the MCC counter-example: a title could plausibly ship a
description like "Easy Anti-Cheat: Disabled for This Mode" that a substring
match would still catch. An exact-string match on the two measured positive
values (`"easy anti cheat"`, `"eac-release"`, normalized) closes that
opening while still matching every value the issue measured.

## Decision: same-product, same-title findings merge via an extracted, shared function

**Decision**: Extract the inline dedup-keep-strongest-fidelity block
currently inside `SignatureSet::detect` (`crates/fragcap-profile/src/signature.rs`,
lines ~440-451) into a standalone `pub fn merge_finding(findings: &mut
Vec<DetectionFinding>, candidate: DetectionFinding)`. `SignatureSet::detect`
calls it internally (no behavior change, verified by the existing test
suite passing unchanged); `fragcap::discovery::SteamSource::discover` calls
it again after `detect_evidence` to merge in `title.anti_cheat`, then
re-sorts by the same `(category order, product)` key `detect` already uses.

**Rationale**: FR-005 requires the exact same "same product, keep strongest
fidelity" rule at a second call site. Duplicating the match block would
create two implementations of one invariant that could silently diverge, the
exact failure class this codebase's own history warns against (S067's
`SteamListingIdentity` decision recorded the identical reasoning for a
different pair of renderers).

**Alternatives considered**: Giving `SignatureSet::detect` a second
parameter (`extra_findings: &[DetectionFinding]`) so the whole merge stays
inside one function was considered. Rejected: `detect` is specifically
"scan this install root," and threading Steam-specific, appinfo-derived
findings through a directory-scanning function's signature would blur what
the function does for every non-Steam caller (the pointed-directory source,
`targets add <exe>`, `technologies <path>`), none of which have an appinfo
cache to draw from.

## Decision: machine-wide probe is a `fragcap-targets` trait, Windows adapter in the facade

**Decision**: `pub trait MachineAntiCheatProbe { fn detect(&self) ->
Vec<MachineAntiCheatFinding>; }` and `pub struct MachineAntiCheatFinding {
pub product: String, pub evidence: String }` live in a new
`crates/fragcap-targets/src/machine_probe.rs`, alongside a
`FixtureMachineAntiCheatProbe` for tests (mirroring `VolumeInventory` /
`FixtureInventory`, `crates/fragcap-targets/src/volume.rs` lines 132-147).
The one real implementation, `WindowsMachineAntiCheatProbe`, lives in a new
`crates/fragcap/src/machine_probe.rs`, `#[cfg(windows)]`-gated, reading
`HKLM\SYSTEM\CurrentControlSet\Services\<name>` key existence via
`RegOpenKeyExW`/`RegCloseKey` (no value read needed, unlike the existing
`read_reg_sz`, since existence alone is the fact).

**Rationale**: This is the exact placement precedent `crates/fragcap/src/discovery.rs`'s
own module doc states for `WindowsVolumeInventory`: "the seam and model" in
`fragcap-targets` (which has zero platform code and zero `windows-sys`
dependency today, and should stay that way), "the platform adapter" in the
facade (already the one crate depending on both `fragcap-steam` and
`fragcap-targets`, and already carrying a `#[cfg(windows)]`
`[target.'cfg(windows)'.dependencies] windows-sys = { workspace = true }`
line that needs only an added `features = ["Win32_System_Registry"]`, an
additive feature on the already-resolved 0.36 pin: verified
`Win32_System_Registry = ["Win32_System"]` exists in the pinned
`windows-sys-0.36.1` and is already enabled by `fragcap-steam` and
`fragcap-cli`, so this adds no `Cargo.lock` package).

**Alternatives considered**: Putting the Windows registry read directly in
`fragcap-steam` (which already has the registry-reading precedent and the
`windows-sys` feature already enabled) was considered, since it would touch
one fewer crate. Rejected: a machine-wide anti-cheat probe is not a Steam
concern (BattlEye and Vanguard, named in the issue as future candidates,
have nothing to do with Steam), and `fragcap-steam`'s own module doc scopes
it to "Steam platform integration." The facade is the crate whose stated
role already covers exactly this kind of cross-cutting platform adapter.

## Decision: only Easy Anti-Cheat is implemented, machine-wide and in launch-entry sources

**Decision**: `KNOWN_MACHINE_PRODUCTS` (or equivalent) names exactly one
entry: Easy Anti-Cheat, service name `EasyAntiCheat_EOS`. The launch-entry
classifier likewise only recognizes Easy Anti-Cheat's tokens.

**Rationale**: The issue's own text is explicit that BattlEye and Vanguard's
machine-wide deployment claims are "not installed on this machine, so those
two claims come from the deployment model rather than from a local
measurement, and should be verified before being relied on." Shipping an
unverified probe entry would itself be the kind of unmeasured claim this
project's practice (and P-9) exists to avoid, even though a wrong service
name would fail closed (no finding) rather than lie. The trait and the
product table are both structured as an extensible list specifically so a
future issue can add a verified entry without a structural change.

**Alternatives considered**: Including BattlEye (`BEService`) and Vanguard
(`vgc`) as best-effort entries "since a wrong probe just fails closed" was
considered and rejected on the same P-9 reasoning the issue itself states,
not because it would be technically unsafe.

## Decision: source D (Steam Deck compatibility tokens) is out of scope

**Decision**: This slice does not read `common/steam_deck_compatibility` or
any `#SteamDeckVerified_TestResult_UnsupportedAntiCheat*` token.

**Rationale**: The issue frames source D as "worth investigating," distinct
from its numbered "Proposed work" list (items 1-6, none of which mention
it), and states plainly that the token set is "product-agnostic... no `_EAC`
or `_BattlEye` variant," so it could corroborate presence but never name a
product, it would not, by itself, satisfy the issue's headline acceptance
criterion that the two measured titles report *Easy Anti-Cheat* by name.

**Alternatives considered**: None seriously; the issue's own framing already
settles this as a follow-on rather than part of this slice.

## Decision: machine-scope rendering lives in the `fragcap targets` hero listing only

**Decision**: The machine-wide probe runs once in
`crates/fragcap-cli/src/commands/targets.rs`'s hero-listing path (bare
`fragcap` and `fragcap targets`), printing a `Machine:` section after the
per-target table only when the probe returns at least one finding.

**Rationale**: Every acceptance scenario in the spec (User Story 3) is
phrased against "`fragcap targets` runs." Wiring the same probe into
`targets discover` or other surfaces is not required by any acceptance
criterion, and doing so would multiply the surface this slice needs to test
without a corresponding requirement.

**Alternatives considered**: A dedicated `fragcap technologies --machine`
subcommand flag was considered. Rejected as scope the issue does not ask
for: the issue's request is that `fragcap targets`'s existing output stop
losing this information, not a new inspection surface.
