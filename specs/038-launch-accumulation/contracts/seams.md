# Phase 1 Contracts: crate-boundary seams

The internal API each crate exposes. No external/network interface; these are the
seams that keep the sibling rule intact.

## fragcap-steam (Steam file-format domain)

```rust
// appinfo.rs
pub struct SteamLaunchEntry { /* fields per data-model.md */ }
pub struct AppInfoApp { pub appid: u32, pub change_number: u32, pub launch: Vec<SteamLaunchEntry> }
pub struct AppInfoFailure { pub appid: Option<u32>, pub reason: String }
pub struct AppInfoParse { pub apps: Vec<AppInfoApp>, pub failures: Vec<AppInfoFailure> }

/// Parse raw appinfo.vdf bytes. Pure and portable; the whole parser is tested
/// against synthetic bytes. A recognized-but-truncated file yields the apps read
/// before the fault plus a trailing failure; an unrecognized magic is a single
/// failure with appid None and no apps.
pub fn parse_appinfo(bytes: &[u8]) -> AppInfoParse;

/// Read and parse the appinfo cache under a Steam root: root/appcache/appinfo.vdf.
/// Portable (takes the root), so it is testable against a fixture tree. A missing
/// file is Ok with an empty parse (no appinfo cache is not an error), matching the
/// library-walk convention. An unreadable-but-present file is a SteamError::Io.
pub fn read_appinfo(root: &Path) -> Result<AppInfoParse, SteamError>;
```

- Contract: opens no process handle; reads one file. No transmit. Reuses
  `vdf::VdfValue` for the parsed tree.
- Reused unchanged: `library::discover_in(root) -> SteamInstallation` gives the
  installed app-id set (`titles[].app_id`, strings) that bounds the considered
  set.

## fragcap-targets (column owner)

```rust
// store.rs
impl Store {
    /// Merge one app's launch tier: ensure the row exists, set
    /// appinfo_change_number, and replace this appid's launch_entries with `entries`
    /// in order. Touches no Tier 1 (name/metrics), Tier 3 (engine),
    /// launcher_mediated, or token_required column. One transaction.
    pub fn merge_launch(
        &mut self,
        appid: u32,
        change_number: u32,
        entries: &[LaunchEntry],
    ) -> Result<(), TargetsError>;

    /// The change-number recorded the last time launch data was stored for this
    /// app, or None if never (or an unseen appid). The staleness comparison input.
    pub fn stored_change_number(&self, appid: u32) -> Result<Option<u32>, TargetsError>;
}
```

- Contract: `merge_launch` is the launch analogue of `merge_catalog`/
  `merge_engine`; it never performs the wholesale `upsert_game` replace.
- Migration: `SCHEMA_VERSION = 2`; `from_connection` gains the v1 to v2 path
  (`ALTER TABLE games ADD COLUMN appinfo_change_number INTEGER`).

## fragcap (facade, behind `targets`)

```rust
// accumulate.rs  (cfg(feature = "targets"))
pub struct LaunchAccumulationSummary {
    pub considered: u64, pub written: u64, pub skipped: u64,
    pub failed: u64, pub empty: u64,
}
impl LaunchAccumulationSummary {
    /// considered == written + skipped + failed + empty
    pub fn is_conserved(&self) -> bool;
}

/// Walk the installed library under `root`, and for each installed app, learn or
/// refresh its launch data from the appinfo cache into `store`, reporting progress
/// through `report`. Owns the honest account. Never prunes. Portable (takes a
/// root), so it is tested against a fixture Steam-root tree and an in-memory store.
pub fn accumulate_launch_data(
    root: &Path,
    store: &mut Store,
    report: &mut dyn FnMut(AccumulationProgress),
) -> Result<LaunchAccumulationSummary, AccumulationError>;
```

- Contract: computes staleness by comparing each `AppInfoApp::change_number`
  against `Store::stored_change_number`; maps `SteamLaunchEntry` to
  `LaunchEntry`; classifies every considered app into exactly one summary bucket;
  a per-app parse failure (from `AppInfoParse::failures`) is counted, not fatal.
- `AccumulationProgress` carries a considered/total count so the CLI can print a
  bounded progress line (FR-010). `report` is a sink the CLI supplies; the library
  prints nothing itself.

## fragcap-cli (run.rs, capture start)

- Contract: when a hint database path is present (the S037 `--hint-db` /
  `FRAGCAP_HINT_DB`) and this build carries `targets`, open that store, call
  `accumulate_launch_data` with a progress printer, close, then proceed to
  `build_resolver` as today. When no hint database is configured, do nothing (no
  accumulation, unchanged behavior). A Steam-not-installed / no-appinfo condition
  yields a zero-considered summary and the capture proceeds.
