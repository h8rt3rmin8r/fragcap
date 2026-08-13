# Phase 1 Data Model: Engine-Rule Provider

The entities this slice adds or extends, all in `fragcap-profile`. Types are
described by role and fields, not by their eventual Rust signatures beyond what
pins the design.

## New types

### `Engine` (enum)

The set of engines the provider recognizes. The markers align with the open
`SteamDatabase/FileDetectionRuleSets` ruleset (the filename subset that also
names the client executable).

- Variants: `Unreal`, `Unity`, `Godot`, `RenPy`.
- Derives: `Clone, Copy, Debug, PartialEq, Eq`.
- `as_str()` yields a stable lower-case label (`"unreal"`, `"unity"`, `"godot"`,
  `"renpy"`) for diagnostics and tests. This label is not the provenance source;
  the provenance source is always the single string `engine-rule` (all engines
  share one provider and one provenance).

### `EngineRuleTarget` (struct)

What an engine-rule answer carries. Analogous to `ObservedTarget`, but naming a
file on disk rather than a live process.

- `engine: Engine` -- which rule matched.
- `image_name: String` -- the resolved client's file name (for example
  `MyGame-Win64-Shipping.exe`).
- `image_path: String` -- the resolved client's full path on disk.
- `identity: MatchPredicates` -- the match rules the pipeline binds the process
  by once it appears (exe pattern plus the path anchor the rule keyed on).
- Derives: `Clone, Debug, PartialEq, Eq`.
- Accessors: `engine()`, `image_name()`, `image_path()`, `identity()`.
- Invariant: `image_path` names a file that exists on disk at resolution time
  (the provider does not fabricate a target for an absent file, per the spec edge
  cases). `identity` is non-empty (carries at least an `exe` pattern).

### `EngineResolution` (module-internal enum, `engine_rule` module)

The result of evaluating the rule set against an install directory. Internal to
the module; the provider maps it onto the cascade contract.

- `Resolved(Box<EngineRuleTarget>)` -- exactly one rule matched with exactly one
  candidate client. Boxed because it is much larger than the other variants.
- `NoMatch` -- no rule matched, or a matched rule's client file was absent.
- `Ambiguous { engine: Engine, candidates: usize }` -- a rule matched more than
  one candidate client; the count and engine are carried for the note.
- `Unreadable { path: PathBuf }` -- a filesystem error left a scan incomplete;
  the unreadable path is carried so the decline is observable and an inaccessible
  install is never resolved from a partial view (FR-009).

## Extended types

### `TargetOrigin` (enum, `target.rs`) -- add one variant

- Add `EngineRule(EngineRuleTarget)` alongside the existing `Profile(Profile)`
  and `Observed(ObservedTarget)`.
- `Target::profile()` and `Target::into_profile()` return `None` for the new
  variant (only a profile origin yields a profile), matching how they already
  treat `Observed`.

### `ResolutionRequest` (struct, `resolver.rs`) -- add one input

- Add field `install_root: Option<&'a Path>`.
- Add constructor `for_install(install_root: &'a Path, search, bundled)` that
  sets `reference: None`, `identity: None`, `tree: None`, and
  `install_root: Some(...)`.
- The existing `for_reference` and `for_observation` constructors set
  `install_root: None`, so the engine-rule provider declines on them and no
  existing caller changes.
- Add accessor `install_root() -> Option<&Path>`.

### `ResolutionNotes` and `Unresolved` (`resolver.rs`) -- add an ambiguity note

- `ResolutionNotes` gains `engine_rule_ambiguous: Option<EngineRuleAmbiguity>`
  and a recorder `note_engine_rule_ambiguous(engine, candidates)`, plus
  `engine_rule_unreadable: Option<PathBuf>` and a recorder
  `note_engine_rule_unreadable(path)` (first path wins).
- `EngineRuleAmbiguity` is a small record `{ engine: Engine, candidates: usize }`
  (or an inline pair), enough to explain the decline.
- `Unresolved` carries both notes through and exposes them via
  `engine_rule_ambiguous()` and `engine_rule_unreadable()`, alongside the
  existing `profile_not_found()`, so a caller that resolves nothing at all can
  report that the engine rule saw an ambiguous layout, or could not read the
  install, and stepped aside. The `Unresolved` display names the unreadable path.

## Fidelity and provenance (unchanged types, fixed values)

- Fidelity: every engine-rule answer is stamped `FidelityTier::HeuristicUnverified`
  (`schema.rs`), never higher (FR-003).
- Provenance: `Provenance::new("engine-rule".to_string(), None)` on every answer.
  `engine-rule` is the value already named in the master schema and glossary.

## Relationships

```text
TargetResolver
  └─ providers: [ProfileProvider, HintProvider, EngineRuleProvider,
                 PlatformWalkerProvider, ObservationProvider]   (by Precedence)

EngineRuleProvider (Precedence::EngineRule)
  ├─ reads  ResolutionRequest.install_root
  ├─ calls  engine_rule::resolve_engine(install_root) -> EngineResolution
  └─ maps   Resolved  -> Ok(Some(Target{HeuristicUnverified, "engine-rule",
                                        TargetOrigin::EngineRule(..)}))
            NoMatch   -> Ok(None)
            Ambiguous -> notes.note_engine_rule_ambiguous(..); Ok(None)
            absent input -> Ok(None)

Target
  ├─ fidelity: FidelityTier::HeuristicUnverified
  ├─ provenance: Provenance("engine-rule")
  └─ origin: TargetOrigin::EngineRule(EngineRuleTarget{engine, image_name,
                                                       image_path, identity})
```

## Validation and determinism rules

- Rule evaluation order across engines is fixed and total (a declared order over
  `Engine`), so a directory that could match more than one engine resolves the
  same way every run (FR-006).
- Candidate selection within a rule collects all matches, then decides: one is
  `Resolved`, zero is `NoMatch`, more than one is `Ambiguous`. The decision does
  not depend on directory-iteration order because it is a function of the
  collected set's size, not of which entry was seen first (FR-006).
- The resolved `image_path` must exist as a file; a rule that recognizes a layout
  but finds its named client absent yields `NoMatch`, not a fabricated target.
