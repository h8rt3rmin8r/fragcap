# Contract: Engine-Rule Provider and Module

The public contract this slice adds to `fragcap-profile`. Signatures are the
intended shape; exact Rust is settled in implementation, but these are binding.

## Cascade participation

The `EngineRuleProvider` (existing type, `providers.rs`) reports
`Precedence::EngineRule` and, in this slice, answers instead of always
declining:

```text
impl TargetProvider for EngineRuleProvider {
    fn precedence(&self) -> Precedence { Precedence::EngineRule }
    fn provide(&self, request, notes) -> Result<Option<Target>, ProviderError>;
}
```

`provide` contract:

- Input absent (`request.install_root()` is `None`): return `Ok(None)`.
- No rule matches, or a matched rule's client file is absent: return `Ok(None)`.
- Ambiguous match (more than one candidate client for one rule): record
  `notes.note_engine_rule_ambiguous(engine, candidates)` and return `Ok(None)`.
- Exactly one candidate: return `Ok(Some(target))` where
  - `target.fidelity() == FidelityTier::HeuristicUnverified`
  - `target.provenance().source() == "engine-rule"`
  - `target.origin()` is `TargetOrigin::EngineRule(EngineRuleTarget { .. })`
- Never returns `Err`: an engine rule has no "present but unusable input" class
  the way the profile provider does; every non-answer is a decline, so the
  cascade always continues to the platform walker and runtime observation.

## Request input

```text
impl<'a> ResolutionRequest<'a> {
    pub fn for_install(install_root: &'a Path, search: &'a SearchPath,
                       bundled: &'a BundledSet) -> ResolutionRequest<'a>;
    pub fn install_root(&self) -> Option<&Path>;
}
```

`for_reference` and `for_observation` continue to set `install_root` to `None`.

## Module surface (`engine_rule`)

```text
pub enum Engine { Unreal, Unity, RenPy }
impl Engine { pub fn as_str(&self) -> &'static str; }

pub struct EngineRuleTarget { /* engine, image_name, image_path, identity */ }
impl EngineRuleTarget {
    pub fn engine(&self) -> Engine;
    pub fn image_name(&self) -> &str;
    pub fn image_path(&self) -> &str;
    pub fn identity(&self) -> &MatchPredicates;
}

// Internal to the module (not part of the public API):
enum EngineResolution { Resolved(EngineRuleTarget), NoMatch,
                        Ambiguous { engine: Engine, candidates: usize } }
fn resolve_engine(install_root: &Path) -> EngineResolution;
```

`resolve_engine` contract:

- Evaluates the engine rules in a fixed, total order (Unreal, Unity, Ren'Py) and
  returns the first engine whose layout is present. The order is declared, not
  incidental, so the result is independent of filesystem iteration order.
- Unreal rule: a file named `*-Win64-Shipping.exe` (case-insensitive) under a
  directory ending in `Binaries/Win64` (case-insensitive, either separator). The
  resolved `identity` carries `exe = <that file name>` and
  `path_contains = "Binaries\\Win64"`.
- Unity rule: a `*_Data` directory beside a player executable, with a
  `UnityPlayer.dll` present. The resolved client is the player executable
  matching the `*_Data` stem; `identity` carries its `exe`.
- Ren'Py rule: a `renpy` directory and at least one `.rpa` archive under the
  root. The resolved client is the Ren'Py launcher executable in the root;
  `identity` carries its `exe`.
- A recognized layout whose named client file is absent yields `NoMatch`, never a
  fabricated target.
- More than one candidate client for the winning rule yields
  `Ambiguous { engine, candidates }`.

## Public re-exports (`lib.rs`)

`Engine` and `EngineRuleTarget` are re-exported from `fragcap-profile` (and flow
through the facade re-export, as the other cascade types do), so a caller can
match on `TargetOrigin::EngineRule` and read the resolved client.

## Invariants asserted by tests

- Determinism: the same install tree resolves to the same target across repeated
  runs and across reordered directory contents.
- Precedence: an authored/verified profile targeting the same install outranks
  the engine-rule answer through a `TargetResolver` that holds both providers.
- Fidelity honesty: the resolved target is `HeuristicUnverified` with
  provenance `engine-rule`; no path stamps a higher tier.
- No silent loss: an ambiguous layout records a note observable via `Unresolved`
  when nothing lower resolves.
- P-1: the module names no process handle API; `cargo xtask lint` stays green.
