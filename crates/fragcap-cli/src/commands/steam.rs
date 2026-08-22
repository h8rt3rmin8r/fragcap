// SPDX-License-Identifier: Apache-2.0

//! `steam`: Steam-specific inspection (specification section 16.3).
//!
//! `steam list` discovers the Steam installation and prints the installed titles
//! it can enumerate (FR-019: command results go to stdout), joined against the
//! local store by the exact `steam:<app_id>` anchor (never by name) so each row
//! carries the three-state identity a reader needs: registered and positioned in
//! the most recent `fragcap targets` listing snapshot, registered but not
//! positioned, or not registered at all (issue #171). The join only reads the
//! listing snapshot; `steam list` never writes it, so it can never change what
//! `fragcap capture <n>` resolves to.
//!
//! The listing honors the global `--json` flag (issue #172), emitting one
//! newline-delimited record per title carrying the install directory the human
//! table has never shown, plus the same identity fields under the same
//! presence/absence rules.
//!
//! Enumeration diagnostics (a skipped malformed manifest, a duplicate app id) and
//! the store-unavailable fallback warning go through the emitter to standard
//! error so they never contaminate the listing on stdout, in either mode.
//!
//! Registering an installed title as a capture target is `targets add --steam
//! <app_id>` (it lands in the user store); the retired `steam profile <app_id>`
//! scaffolding is gone.

use std::io::Write;

use fragcap::steam::{self, InstalledTitle, SteamError};
use fragcap::targets::{Store, TargetsError};
use fragcap::write_json_string;

use crate::cli::{SteamArgs, SteamCommand};
use crate::commands::targets::default_local_store;
use crate::emit::Emitter;
use crate::exit::{CliError, Exit};

/// Run a `steam` subcommand, writing its result to `out` and any diagnostics
/// through `emitter`. `json` is the global `--json` flag, honored here the same
/// way `doctor` honors it.
pub fn run(
    args: &SteamArgs,
    json: bool,
    out: &mut dyn Write,
    emitter: &mut Emitter,
) -> Result<Exit, CliError> {
    match &args.command {
        SteamCommand::List => list(json, out, emitter),
    }
}

/// The joined identity of an installed title against the local store: whether it
/// is registered, and if so whether it appears in the most recent listing
/// snapshot. `stable_id` rides along on both registered variants for the JSON
/// record (FR-011); the human table only ever renders the handle and position.
#[derive(Clone, Debug, PartialEq, Eq)]
enum SteamListingIdentity {
    /// Registered, and its stable id appears in the most recent listing
    /// snapshot at `position` (1-based).
    Positioned {
        stable_id: i64,
        handle: String,
        position: usize,
    },
    /// Registered, but absent from the most recent listing snapshot.
    Unpositioned { stable_id: i64, handle: String },
    /// No local registration resolves for this title's anchor. Also the
    /// fallback rendering for every title when the store cannot be checked at
    /// all (a warning distinguishes that case from a genuine absence).
    Unregistered,
}

/// Resolve one installed title's identity against an open store, by its exact
/// `steam:<app_id>` anchor (never by name: two different app ids can share a
/// display name, as Steam's soundtrack and redistributable entries do).
fn resolve_identity(store: &Store, app_id: &str) -> Result<SteamListingIdentity, TargetsError> {
    let anchor = format!("steam:{app_id}");
    match store.target_by_anchor(&anchor)? {
        None => Ok(SteamListingIdentity::Unregistered),
        Some(entry) => match store.listing_snapshot_position(entry.stable_id)? {
            Some(position) => Ok(SteamListingIdentity::Positioned {
                stable_id: entry.stable_id,
                handle: entry.handle,
                position,
            }),
            None => Ok(SteamListingIdentity::Unpositioned {
                stable_id: entry.stable_id,
                handle: entry.handle,
            }),
        },
    }
}

/// One installed title paired with its resolved local identity, the unit both
/// renderers (human and JSON) consume, so the two output modes cannot describe a
/// title's identity differently.
struct ListingRow<'a> {
    title: &'a InstalledTitle,
    identity: SteamListingIdentity,
}

/// Resolve every title's identity against `store`, or, when `store` is `None`
/// (resolution or open failed), fall back to `Unregistered` for every row and
/// warn that identity information could not be joined (FR-008). A per-title
/// store-query error that is not "not found" is likewise folded into
/// `Unregistered` for rendering, with its own warning, so a real error is never
/// silently reported as a registration fact (P-9).
fn resolve_rows<'a>(
    titles: &'a [InstalledTitle],
    store: Option<&Store>,
    emitter: &mut Emitter,
) -> Vec<ListingRow<'a>> {
    let Some(store) = store else {
        emitter.warn("local store unavailable; showing installation state only");
        return titles
            .iter()
            .map(|title| ListingRow {
                title,
                identity: SteamListingIdentity::Unregistered,
            })
            .collect();
    };
    titles
        .iter()
        .map(|title| {
            let identity = resolve_identity(store, &title.app_id).unwrap_or_else(|e| {
                emitter.warn(&format!(
                    "could not resolve local identity for app {}: {e}",
                    title.app_id
                ));
                SteamListingIdentity::Unregistered
            });
            ListingRow { title, identity }
        })
        .collect()
}

/// Sort rows by title name (case-insensitive ordinal), tie-broken by app id, so
/// the order is deterministic across runs (FR-007). Two different app ids can
/// share a display name (Steam's soundtrack and redistributable entries often
/// do); the app id tiebreak keeps the order total in that case.
fn sort_rows(rows: &mut [ListingRow<'_>]) {
    rows.sort_by(|a, b| {
        a.title
            .name
            .to_lowercase()
            .cmp(&b.title.name.to_lowercase())
            .then_with(|| a.title.app_id.cmp(&b.title.app_id))
    });
}

/// List the installed Steam titles this machine can enumerate, joined against
/// the local store, in either human or JSON mode.
fn list(json: bool, out: &mut dyn Write, emitter: &mut Emitter) -> Result<Exit, CliError> {
    let installation = steam::discover().map_err(map_steam_error)?;

    for warning in &installation.warnings {
        emitter.warn(warning);
    }

    let opened_store = match default_local_store() {
        Some(path) => match Store::open(&path) {
            Ok(store) => Some(store),
            Err(e) => {
                emitter.warn(&format!(
                    "local store at {} unavailable: {e}",
                    path.display()
                ));
                None
            }
        },
        None => None,
    };

    let mut rows = resolve_rows(&installation.titles, opened_store.as_ref(), emitter);
    sort_rows(&mut rows);

    if json {
        render_json(&rows, out);
    } else {
        render_human(&rows, out);
    }
    Ok(Exit::SUCCESS)
}

/// Render the human table: a header, then one row per title with a textually
/// distinct `TARGET` cell per identity state (contract:
/// `specs/067-steam-list-identity-json/contracts/steam-list-cli.md`).
fn render_human(rows: &[ListingRow<'_>], out: &mut dyn Write) {
    if rows.is_empty() {
        let _ = writeln!(out, "no installed titles enumerated");
        return;
    }
    let _ = writeln!(out, "APP ID\tNAME\tSTATE\tTARGET");
    for row in rows {
        let (state, target) = match &row.identity {
            SteamListingIdentity::Positioned {
                handle, position, ..
            } => ("registered", format!("{handle} (#{position})")),
            SteamListingIdentity::Unpositioned { handle, .. } => {
                ("registered", format!("{handle} (no position)"))
            }
            SteamListingIdentity::Unregistered => ("unregistered", String::new()),
        };
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}",
            row.title.app_id, row.title.name, state, target
        );
    }
}

/// Render one newline-delimited JSON record per title (FR-009 through FR-012),
/// matching `doctor --json`'s hand-rolled construction style.
fn render_json(rows: &[ListingRow<'_>], out: &mut dyn Write) {
    for row in rows {
        let mut line = String::from("{\"app_id\":");
        write_json_string(&row.title.app_id, &mut line);
        line.push_str(",\"name\":");
        write_json_string(&row.title.name, &mut line);
        line.push_str(",\"install_dir\":");
        write_json_string(&row.title.install_dir.display().to_string(), &mut line);
        match &row.identity {
            SteamListingIdentity::Positioned {
                stable_id,
                handle,
                position,
            } => {
                line.push_str(",\"handle\":");
                write_json_string(handle, &mut line);
                line.push_str(&format!(
                    ",\"stable_id\":{stable_id},\"position\":{position}"
                ));
            }
            SteamListingIdentity::Unpositioned { stable_id, handle } => {
                line.push_str(",\"handle\":");
                write_json_string(handle, &mut line);
                line.push_str(&format!(",\"stable_id\":{stable_id}"));
            }
            SteamListingIdentity::Unregistered => {}
        }
        line.push('}');
        let _ = writeln!(out, "{line}");
    }
}

/// Map a Steam error to the CLI exit contract.
///
/// A missing Steam installation or an unsupported platform is a configuration
/// problem (exit 2); a filesystem failure is an expected runtime failure (exit 1).
/// Shared with `targets add --steam` so the two Steam entry points classify the
/// same error the same way.
pub(crate) fn map_steam_error(error: SteamError) -> CliError {
    match error {
        SteamError::NotInstalled
        | SteamError::UnsupportedPlatform
        | SteamError::TitleNotFound { .. } => CliError::usage(error.to_string()),
        SteamError::Io { .. }
        | SteamError::NoExecutables { .. }
        | SteamError::Scaffold(_)
        | SteamError::LaunchFailed { .. } => CliError::failure(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fragcap::profile::FidelityTier;
    use fragcap::targets::{ClassificationSource, TargetClassification, TargetEntry};

    use super::*;
    use crate::emit::{Format, Verbosity};

    fn title(app_id: &str, name: &str) -> InstalledTitle {
        InstalledTitle {
            app_id: app_id.to_string(),
            name: name.to_string(),
            install_dir: PathBuf::from(format!("C:/Steam/steamapps/common/{name}")),
            installdir: name.to_string(),
            app_type: None,
            launch_executable: None,
        }
    }

    /// Insert a target anchored to `steam:<app_id>` with a deterministic stable
    /// id derived the same way the real registration path does, so
    /// `target_by_anchor` finds it.
    fn register(store: &mut Store, app_id: &str, handle: &str) -> i64 {
        let anchor = format!("steam:{app_id}");
        let stable_id = fragcap::targets::identifier::anchored_id(&anchor);
        store
            .insert_target(&TargetEntry {
                id: None,
                stable_id,
                handle: handle.to_string(),
                name: handle.to_string(),
                classification: TargetClassification::Unknown,
                classification_source: ClassificationSource::Platform,
                fidelity: FidelityTier::HeuristicUnverified,
                provenance: None,
                anchor: Some(anchor),
                launch_entries: None,
                install_root: None,
                evidence: None,
                detection_scan: None,
                folder_name: None,
                executable_hint: None,
            })
            .expect("insert target");
        stable_id
    }

    fn emitter(buf: &mut Vec<u8>) -> Emitter<'_> {
        Emitter::new(buf, Format::Human, Verbosity::Normal)
    }

    #[test]
    fn resolve_identity_covers_all_three_states() {
        let mut store = Store::open_in_memory().expect("store");
        let positioned_id = register(&mut store, "100", "positioned_handle");
        let unpositioned_id = register(&mut store, "200", "unpositioned_handle");
        store
            .write_listing_snapshot(&[(positioned_id, "positioned_handle")])
            .expect("snapshot");

        assert_eq!(
            resolve_identity(&store, "100").expect("resolve"),
            SteamListingIdentity::Positioned {
                stable_id: positioned_id,
                handle: "positioned_handle".to_string(),
                position: 1,
            }
        );
        assert_eq!(
            resolve_identity(&store, "200").expect("resolve"),
            SteamListingIdentity::Unpositioned {
                stable_id: unpositioned_id,
                handle: "unpositioned_handle".to_string(),
            }
        );
        assert_eq!(
            resolve_identity(&store, "300").expect("resolve"),
            SteamListingIdentity::Unregistered
        );
    }

    #[test]
    fn human_render_distinguishes_all_three_states_and_leads_with_a_header() {
        let mut store = Store::open_in_memory().expect("store");
        let positioned_id = register(&mut store, "100", "positioned_handle");
        register(&mut store, "200", "unpositioned_handle");
        store
            .write_listing_snapshot(&[(positioned_id, "positioned_handle")])
            .expect("snapshot");

        let titles = vec![
            title("100", "Positioned Title"),
            title("200", "Unpositioned Title"),
            title("300", "Unregistered Title"),
        ];
        let mut buf: Vec<u8> = Vec::new();
        let mut e = emitter(&mut buf);
        let mut rows = resolve_rows(&titles, Some(&store), &mut e);
        sort_rows(&mut rows);

        let mut out: Vec<u8> = Vec::new();
        render_human(&rows, &mut out);
        let text = String::from_utf8(out).unwrap();
        let mut lines = text.lines();
        assert_eq!(
            lines.next(),
            Some("APP ID\tNAME\tSTATE\tTARGET"),
            "a header leads every human render"
        );
        assert!(text.contains("positioned_handle (#1)"));
        assert!(text.contains("unpositioned_handle (no position)"));
        assert!(text.contains("300\tUnregistered Title\tunregistered\t"));
        assert!(
            !text.contains("Unregistered Title\tunregistered\thandle"),
            "an unregistered row never carries a handle"
        );
    }

    #[test]
    fn rows_sort_by_name_case_insensitive_then_app_id() {
        let titles = vec![title("2", "beta"), title("1", "Alpha"), title("3", "alpha")];
        let mut buf: Vec<u8> = Vec::new();
        let mut e = emitter(&mut buf);
        let mut rows = resolve_rows(&titles, None, &mut e);
        sort_rows(&mut rows);
        let order: Vec<&str> = rows.iter().map(|r| r.title.app_id.as_str()).collect();
        assert_eq!(order, vec!["1", "3", "2"], "Alpha(1) < alpha(3) < beta(2)");
    }

    #[test]
    fn steam_list_never_writes_the_listing_snapshot() {
        let mut store = Store::open_in_memory().expect("store");
        let positioned_id = register(&mut store, "100", "positioned_handle");
        store
            .write_listing_snapshot(&[(positioned_id, "positioned_handle")])
            .expect("snapshot");

        let titles = vec![title("100", "Positioned Title"), title("999", "New Title")];
        let mut buf: Vec<u8> = Vec::new();
        let mut e = emitter(&mut buf);
        let mut rows = resolve_rows(&titles, Some(&store), &mut e);
        sort_rows(&mut rows);
        let mut out: Vec<u8> = Vec::new();
        render_human(&rows, &mut out);

        // The snapshot still names exactly the one row it named before: a new,
        // unregistered title never appears in it, and the positioned title's
        // position is unchanged.
        assert_eq!(
            store.listing_snapshot_position(positioned_id).expect("pos"),
            Some(1)
        );
        assert_eq!(store.listing_snapshot_len().expect("len"), 1);
    }

    #[test]
    fn absent_store_falls_back_to_unregistered_and_warns() {
        let titles = vec![title("100", "Some Title")];
        let mut buf: Vec<u8> = Vec::new();
        let mut e = emitter(&mut buf);
        let rows = resolve_rows(&titles, None, &mut e);
        assert_eq!(rows[0].identity, SteamListingIdentity::Unregistered);
        let warnings = String::from_utf8(buf).unwrap();
        assert!(
            warnings.contains("local store unavailable"),
            "a store-absent fallback warns rather than silently reporting unregistered: {warnings}"
        );
    }

    #[test]
    fn json_render_carries_identity_fields_only_when_present() {
        let mut store = Store::open_in_memory().expect("store");
        let positioned_id = register(&mut store, "100", "positioned_handle");
        let unpositioned_id = register(&mut store, "200", "unpositioned_handle");
        store
            .write_listing_snapshot(&[(positioned_id, "positioned_handle")])
            .expect("snapshot");

        let titles = vec![
            title("100", "Positioned Title"),
            title("200", "Unpositioned Title"),
            title("300", "Unregistered Title"),
        ];
        let mut buf: Vec<u8> = Vec::new();
        let mut e = emitter(&mut buf);
        let mut rows = resolve_rows(&titles, Some(&store), &mut e);
        sort_rows(&mut rows);

        let mut out: Vec<u8> = Vec::new();
        render_json(&rows, &mut out);
        let text = String::from_utf8(out).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        for line in &lines {
            assert!(line.starts_with('{') && line.ends_with('}'));
            assert!(line.contains("\"app_id\":"));
            assert!(line.contains("\"name\":"));
            assert!(line.contains("\"install_dir\":"));
        }

        let positioned_line = lines
            .iter()
            .find(|l| l.contains(&format!("\"stable_id\":{positioned_id}")))
            .expect("positioned record present");
        assert!(positioned_line.contains("\"handle\":\"positioned_handle\""));
        assert!(positioned_line.contains("\"position\":1"));

        let unpositioned_line = lines
            .iter()
            .find(|l| l.contains(&format!("\"stable_id\":{unpositioned_id}")))
            .expect("unpositioned record present");
        assert!(unpositioned_line.contains("\"handle\":\"unpositioned_handle\""));
        assert!(
            !unpositioned_line.contains("\"position\""),
            "an unpositioned record carries no position key"
        );

        let unregistered_line = lines
            .iter()
            .find(|l| l.contains("Unregistered Title"))
            .expect("unregistered record present");
        assert!(!unregistered_line.contains("\"handle\""));
        assert!(!unregistered_line.contains("\"stable_id\""));
        assert!(!unregistered_line.contains("\"position\""));
    }

    #[test]
    fn json_render_of_zero_titles_is_zero_bytes() {
        let mut out: Vec<u8> = Vec::new();
        render_json(&[], &mut out);
        assert!(out.is_empty(), "no titles yields no JSON records");
    }
}
