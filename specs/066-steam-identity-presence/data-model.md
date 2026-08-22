# Phase 1 Data Model: S066

## `InstalledTitle` (`crates/fragcap-steam/src/library.rs`)

Three fields added, all populated in `read_manifest` from the same manifest/appinfo
read, none derived from another:

| Field | Type | Source | Notes |
| --- | --- | --- | --- |
| `installdir` | `String` | `AppState.installdir` in `appmanifest_*.acf`, verbatim | Already read today to build `install_dir`; now also kept raw. Required (the manifest already requires it to resolve at all). |
| `app_type` | `Option<String>` | `common.type` in `appcache/appinfo.vdf`, verbatim | `None` when no appinfo entry exists for the appid, the cache is absent, or the value could not be read. Case is preserved as observed; comparisons against it are case-insensitive. |
| `launch_executable` | `Option<String>` | The first `config/launch` entry's `executable`, from the same appinfo section, verbatim | `None` when no launch entries are present. Used only as a findability hint (see `TargetEntry::executable_hint`); never promotes to a resolved capture chain. |

`install_dir: PathBuf` (existing field) keeps its current meaning: the resolved,
absolute install directory. Its join subdirectory changes from unconditionally
`common` to `music` when `app_type` case-insensitively equals `"Music"`, else stays
`common` (R-2, R-3).

## `CandidateTarget` (`crates/fragcap-targets/src/source.rs`)

Two fields added:

| Field | Type | Notes |
| --- | --- | --- |
| `folder_name` | `Option<String>` | The raw installdir/folder-identifying value, verbatim. `Some` for a Steam candidate (from `InstalledTitle::installdir`); `None` for a known-roots or directory-scan candidate, which has no separate installdir concept from its path-derived `display_name` (documented assumption, unchanged). |
| `executable_hint` | `Option<String>` | The raw observed launch executable name, verbatim. `Some` when the producing source observed one (a Steam candidate with a launch entry); `None` otherwise. |

`SteamSource::discover` (`crates/fragcap/src/discovery.rs`) additionally: skips a
title whose `app_type` case-insensitively equals `"Music"` entirely, counting it
under `account.considered_not_a_game` (R-6) rather than constructing a candidate for
it.

## `TargetEntry` / schema (`crates/fragcap-targets/src/entry.rs`, `schema.rs`)

Two nullable columns added to `targets`, schema version 7 → 8:

```sql
ALTER TABLE targets ADD COLUMN folder_name TEXT;
ALTER TABLE targets ADD COLUMN executable_hint TEXT;
```

Backward-safe by construction: an existing row reads both as `NULL`, which is
exactly "not recorded" (matching the `detection_scan` precedent from schema version
6→7). `TargetEntry` gains matching `folder_name: Option<String>` and
`executable_hint: Option<String>` fields. `register_candidate`
(`crates/fragcap-targets/src/register.rs`) copies `CandidateTarget::folder_name` and
`::executable_hint` onto the new entry fields unchanged. The user-authored `targets
add` path (`crates/fragcap-cli/src/commands/targets.rs::add`) leaves both `None`: a
manually authored target has no separate platform installdir or launch-hint
observation distinct from what the user already supplied as `--exe`/`--name`.

`store.rs` (`Store::insert_target`, `Store::targets`, `Store::target_by_handle`,
`Store::target_by_stable_id`, `Store::target`, and any other row reader) is extended
to select, bind, and read the two new columns alongside the existing ones.

## Selector resolution (`crates/fragcap-targets/src/selector.rs`)

`resolve_positional`'s non-row-index path becomes three tiers (R-8), in order,
stopping at the first tier that produces any match:

1. Exact handle (`Store::target_by_handle`, unchanged).
2. Case-insensitive exact name (`Store::targets_by_name`, unchanged): 0 → fall
   through to tier 3; 1 → `Selection::Resolved`; >1 → `Selection::Ambiguous`
   (unchanged ambiguity semantics for an existing exact-name collision).
3. Case-insensitive substring match against `name`, `folder_name`, and
   `executable_hint` (a new `Store` method, e.g. `targets_by_substring`), each
   target counted once even if it matches on more than one field: 0 → `NoMatch`;
   1 → `Resolved`; >1 → `Ambiguous`.

`export.rs`/`targets_export.rs` add `folder_name` and `executable_hint` as optional
export keys, following the existing precedent (`detection_scan`'s S065
introduction): emitted only when present, so an existing consumer reading the array
is unaffected (P-5-adjacent compatibility posture already established for this
export shape).

## Presence and divergence (derived, never stored)

Two purely presentational derivations, colocated with the existing `readiness.rs`
derivations (which already document themselves as "derived from a `TargetEntry` at
listing time and stored nowhere"):

```text
enum InstallPresence { Present, Missing, NotRecorded }

fn install_presence(entry: &TargetEntry) -> InstallPresence {
    match &entry.install_root {
        None => NotRecorded,
        Some(root) => if Path::new(root).exists() { Present } else { Missing },
    }
}

enum NameDivergence { None, Cosmetic, Semantic }

fn name_divergence(entry: &TargetEntry) -> NameDivergence {
    // folder_name absent => None (nothing to compare, FR-017)
    // else normalize both entry.name and folder_name via handle::normalize()
    // equal, or one a substring of the other => Cosmetic
    // otherwise => Semantic
}
```

`install_presence` feeds `render_table`'s SENSITIVITIES-prefix decision and the hero
listing's "next command" selection (skips a `Missing` row, Clarifications). `render_table`
already resolves `capture_readiness`, `engine_summary`, `sensitivities_summary` per row;
`install_presence` is the fourth per-row derivation added, following the same pattern.

`name_divergence` feeds `targets show`'s divergence note (`print_target` in
`crates/fragcap-cli/src/commands/targets.rs`), printed only for `Semantic`.

## Colocated color helper (`crates/fragcap-cli/src/color.rs`, new, `pub(crate)`)

```text
pub(crate) fn use_color() -> bool  // moved from commands/doctor.rs, same body
pub(crate) const WARN: &str = "\x1b[33m";   // moved from doctor/mod.rs's Status::Warn arm
pub(crate) const RESET: &str = "\x1b[0m";   // moved from doctor/mod.rs's ANSI_RESET
```

`doctor.rs` and `doctor/mod.rs` are updated to reference this module instead of their
own private copies (zero behavior change, single source for the palette, R-9).
`targets.rs` imports the same module for the missing-install-root note.

## `docs/fragcap-specification.md` / glossary

- Schema version reference (section 15, wherever version 7 is currently cited) moves
  to 8.
- No new term is introduced by this slice that lacks an existing glossary entry
  ("handle", "anchor", "fidelity", "install root" are all already glossary terms); the
  handle-derivation reference vectors (Appendix A) are updated for the `&` decision
  (R-11).
