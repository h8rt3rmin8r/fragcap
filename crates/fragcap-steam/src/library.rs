// SPDX-License-Identifier: Apache-2.0

//! Steam library discovery: read `libraryfolders.vdf` and every
//! `appmanifest_*.acf` to enumerate installed titles.
//!
//! The whole of this module is portable. The only Windows-specific step in
//! discovery is finding the Steam root through the registry, which lives in
//! [`crate::steam_root`]; everything here operates on a root directory, so it is
//! exercised on any host against a fixture tree ([`discover_in`]).

use std::path::{Path, PathBuf};

use crate::vdf;
use crate::SteamError;

/// A Steam library: a directory Steam installs titles into.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SteamLibrary {
    /// The library root (contains `steamapps/`).
    pub path: PathBuf,
}

/// An installed Steam title.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledTitle {
    /// The Steam application identifier. A string even when numeric, matching
    /// the profile schema's `game.app_id`.
    pub app_id: String,
    /// The human title name.
    pub name: String,
    /// The resolved absolute installation directory: `steamapps/common/<installdir>`
    /// for every type but `Music`, which resolves under `steamapps/music/` instead
    /// (slice S066, issue #166).
    pub install_dir: PathBuf,
    /// The raw manifest `installdir` value, verbatim, independent of which
    /// subdirectory it was joined into. Kept alongside `install_dir` (never
    /// reconstructed from it) so a consumer can recover the folder name Explorer
    /// shows even when it diverges from `name` (issue #173).
    pub installdir: String,
    /// The appinfo `common/type` value, verbatim (for example `Game`, `Music`), or
    /// `None` when no appinfo entry exists for this app, the cache is absent, or the
    /// value could not be read. Never guessed (P-9): an unknown type falls back to
    /// the `common/` assumption rather than asserting a type.
    pub app_type: Option<String>,
    /// The first appinfo `config/launch` entry's executable, verbatim, or `None`
    /// when no launch entry was observed. A findability hint only (issue #173); it
    /// never promotes to a resolved capture chain.
    pub launch_executable: Option<String>,
}

/// A resolved Steam installation and the titles installed across its libraries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SteamInstallation {
    /// The Steam install directory.
    pub root: PathBuf,
    /// Every configured library, including the root library.
    pub libraries: Vec<SteamLibrary>,
    /// Every installed title, deduplicated by `app_id` (first discovered wins).
    pub titles: Vec<InstalledTitle>,
    /// Non-fatal diagnostics: manifests skipped as malformed and duplicate
    /// app_id collisions. Reported, never silently dropped (FR-004).
    pub warnings: Vec<String>,
    /// The count of application manifests that were present but could not be
    /// parsed, each a title omitted from `titles`. A structured tally distinct from
    /// the human `warnings`, so a consumer that keeps a conserved discovery account
    /// can count the omission rather than infer it from warning strings (P-4).
    pub malformed_manifests: u64,
}

impl SteamInstallation {
    /// The installed title with the given app_id, if present.
    pub fn find(&self, app_id: &str) -> Option<&InstalledTitle> {
        self.titles.iter().find(|t| t.app_id == app_id)
    }
}

/// The outcome of looking up a title's install directory.
///
/// Carries the install directory when the title is installed, and always the
/// non-fatal enumeration warnings (a malformed manifest, an unreadable configured
/// library, a duplicate app id). A malformed manifest for the requested app makes
/// `install_dir` `None`, but the warning is preserved rather than making the title
/// silently indistinguishable from an uninstalled one, so the caller can surface
/// it (FR-008, P-4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallLookup {
    /// The resolved install directory, or `None` if the title is not installed.
    pub install_dir: Option<PathBuf>,
    /// Non-fatal enumeration diagnostics, never silently dropped.
    pub warnings: Vec<String>,
}

/// Look up the install directory of the installed title with `app_id` under a
/// given Steam root, carrying any enumeration warnings.
///
/// Portable (takes the root as a value), so the platform-walker's enumeration is
/// testable without a registry. The caller enriches a resolution request with
/// `install_dir` so the engine rule and the walker can resolve from it (S030), and
/// surfaces `warnings` rather than swallowing them.
pub fn install_root_in(root: &Path, app_id: &str) -> Result<InstallLookup, SteamError> {
    let installation = discover_in(root)?;
    let install_dir = installation
        .find(app_id)
        .map(|title| title.install_dir.clone());
    Ok(InstallLookup {
        install_dir,
        warnings: installation.warnings,
    })
}

/// Discover libraries and installed titles under a given Steam root.
///
/// Portable: takes the root as a value so the whole enumeration is testable
/// without a registry or a real Steam install. A malformed manifest is recorded
/// in `warnings` and skipped; the well-formed ones survive (FR-004).
pub fn discover_in(root: &Path) -> Result<SteamInstallation, SteamError> {
    let mut warnings = Vec::new();
    let mut libraries = vec![SteamLibrary {
        path: root.to_path_buf(),
    }];

    for lib in read_library_folders(root, &mut warnings) {
        if !libraries.iter().any(|l| l.path == lib) {
            libraries.push(SteamLibrary { path: lib });
        }
    }

    // Read the appinfo cache once per call and build an appid lookup, rather than
    // once per title: a missing cache (no Steam has ever fetched metadata) is not a
    // discovery error, matching `read_appinfo`'s own "no cache is not an error"
    // contract. A present-but-unreadable cache is surfaced as a warning (it degrades
    // every title's type resolution to the `common/` fallback, which is worth
    // knowing) but does not fail the whole walk (slice S066).
    let appinfo_index = match crate::appinfo::read_appinfo(root) {
        Ok(parse) => {
            // A section that failed to decode is one app whose type could not be
            // read; it is absent from the index (so that one app falls back to
            // `common`, matching an app_id absent from the cache entirely), but the
            // failure is still named rather than silently swallowed (P-4). Every
            // other section decoded cleanly is unaffected: a per-section fault is
            // isolated by `parse_appinfo` and does not degrade the rest of the
            // cache (review of PR #193).
            for failure in &parse.failures {
                warnings.push(match failure.appid {
                    Some(appid) => format!(
                        "could not read appinfo section for app {appid}: {}; \
                         its app type defaults to common",
                        failure.reason
                    ),
                    None => format!(
                        "could not read appinfo cache: {}; app types default to common",
                        failure.reason
                    ),
                });
            }
            parse
                .apps
                .into_iter()
                .map(|a| {
                    (
                        a.appid,
                        (
                            a.common_type,
                            a.launch.first().map(|l| l.executable.clone()),
                        ),
                    )
                })
                .collect::<std::collections::HashMap<u32, (Option<String>, Option<String>)>>()
        }
        Err(e) => {
            warnings.push(format!(
                "could not read appinfo cache: {e}; app types default to common"
            ));
            std::collections::HashMap::new()
        }
    };

    let mut titles: Vec<InstalledTitle> = Vec::new();
    let mut malformed_manifests: u64 = 0;
    for (i, lib) in libraries.iter().enumerate() {
        // The implicit root (index 0) is not a configured library; a missing
        // `steamapps` there is benign. Every other library came from
        // `libraryfolders.vdf`, so if its `steamapps` cannot be read (a
        // disconnected drive, a permission error) that is a title-omitting fault
        // worth reporting rather than swallowing (Codex review of PR #31).
        let warn_unreadable = i != 0;
        for title in read_library_titles(
            &lib.path,
            &mut warnings,
            &mut malformed_manifests,
            warn_unreadable,
            &appinfo_index,
        ) {
            if let Some(existing) = titles.iter().find(|t| t.app_id == title.app_id) {
                warnings.push(format!(
                    "app_id {} found in more than one library; keeping {} and ignoring {}",
                    title.app_id,
                    existing.install_dir.display(),
                    title.install_dir.display()
                ));
            } else {
                titles.push(title);
            }
        }
    }

    Ok(SteamInstallation {
        root: root.to_path_buf(),
        libraries,
        titles,
        warnings,
        malformed_manifests,
    })
}

/// Read the library paths declared in `libraryfolders.vdf`.
///
/// Handles both the modern nested form (`"0" { "path" "..." }`) and the older
/// flat form (`"0" "..."`). Non-numeric keys (`contentstatsid` and friends) are
/// ignored. A missing or malformed file yields no extra libraries and a warning.
fn read_library_folders(root: &Path, warnings: &mut Vec<String>) -> Vec<PathBuf> {
    let candidates = [
        root.join("steamapps").join("libraryfolders.vdf"),
        root.join("config").join("libraryfolders.vdf"),
    ];
    let Some(path) = candidates.iter().find(|p| p.exists()) else {
        return Vec::new();
    };

    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            warnings.push(format!("could not read {}: {e}", path.display()));
            return Vec::new();
        }
    };
    let doc = match vdf::parse(&text) {
        Ok(d) => d,
        Err(e) => {
            warnings.push(format!("skipping malformed {}: {e}", path.display()));
            return Vec::new();
        }
    };
    let Some(folders) = doc.get("libraryfolders") else {
        return Vec::new();
    };
    let Some(entries) = folders.entries() else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for (key, value) in entries {
        if !key.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let path = match value {
            vdf::VdfValue::Str(p) => Some(p.clone()),
            vdf::VdfValue::Obj(_) => value
                .get("path")
                .and_then(|p| p.as_str())
                .map(str::to_string),
        };
        if let Some(p) = path {
            out.push(PathBuf::from(p));
        }
    }
    out
}

/// Read every `appmanifest_*.acf` in one library's `steamapps` directory.
///
/// `warn_unreadable` reports a read failure (the library came from the manifest
/// and should be enumerable); the implicit root passes `false`, since a root
/// without `steamapps` is benign.
fn read_library_titles(
    library: &Path,
    warnings: &mut Vec<String>,
    malformed: &mut u64,
    warn_unreadable: bool,
    appinfo_index: &std::collections::HashMap<u32, (Option<String>, Option<String>)>,
) -> Vec<InstalledTitle> {
    let steamapps = library.join("steamapps");
    let entries = match std::fs::read_dir(&steamapps) {
        Ok(e) => e,
        Err(e) => {
            if warn_unreadable {
                warnings.push(format!(
                    "skipping library {}: cannot read {}: {e}",
                    library.display(),
                    steamapps.display()
                ));
            }
            // Nonfatal: a library with no readable steamapps holds no titles here.
            return Vec::new();
        }
    };

    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !(name.starts_with("appmanifest_") && name.ends_with(".acf")) {
            continue;
        }
        match read_manifest(&path, &steamapps, appinfo_index) {
            Ok(title) => out.push(title),
            Err(reason) => {
                // A present manifest that will not parse is one title omitted:
                // counted structurally as well as reported, so a consumer's
                // discovery account can reflect the loss (P-4).
                *malformed += 1;
                warnings.push(format!("skipping {}: {reason}", path.display()));
            }
        }
    }
    // Deterministic order regardless of directory iteration order.
    out.sort_by(|a, b| a.app_id.cmp(&b.app_id));
    out
}

/// Parse one application manifest into an [`InstalledTitle`], joining its install
/// directory under `steamapps/music/` when the appinfo cache names this app's type
/// `Music` (case-insensitively), else under `steamapps/common/` as before (slice
/// S066, issue #166). An app_id absent from `appinfo_index`, or with no `common/type`
/// recorded, keeps the `common/` default: an unknown type is never guessed into
/// either bucket (P-9).
fn read_manifest(
    path: &Path,
    steamapps: &Path,
    appinfo_index: &std::collections::HashMap<u32, (Option<String>, Option<String>)>,
) -> Result<InstalledTitle, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("read error: {e}"))?;
    let doc = vdf::parse(&text).map_err(|e| e.to_string())?;
    let state = doc.get("AppState").ok_or("no AppState block")?;

    let app_id = state
        .get("appid")
        .and_then(|v| v.as_str())
        .ok_or("no appid")?
        .to_string();
    let name = state
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let installdir = state
        .get("installdir")
        .and_then(|v| v.as_str())
        .ok_or("no installdir")?
        .to_string();

    let (app_type, launch_executable) = app_id
        .parse::<u32>()
        .ok()
        .and_then(|id| appinfo_index.get(&id))
        .cloned()
        .unwrap_or((None, None));

    let subdir = match app_type.as_deref() {
        Some(t) if t.eq_ignore_ascii_case("music") => "music",
        _ => "common",
    };

    Ok(InstalledTitle {
        app_id,
        name,
        install_dir: steamapps.join(subdir).join(&installdir),
        installdir,
        app_type,
        launch_executable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempTree;

    fn manifest(app_id: &str, name: &str, installdir: &str) -> String {
        format!(
            "\"AppState\"\n{{\n  \"appid\" \"{app_id}\"\n  \"name\" \"{name}\"\n  \
             \"installdir\" \"{installdir}\"\n}}\n"
        )
    }

    #[test]
    fn discovers_titles_across_two_libraries() {
        let tree = TempTree::new();
        let root = tree.path();
        // Library A is the steam root; library B is a second folder.
        let lib_b = root.join("LibraryB");
        tree.write(
            &root
                .join("steamapps")
                .join("libraryfolders.vdf"),
            &format!(
                "\"libraryfolders\"\n{{\n  \"0\" {{ \"path\" \"{}\" }}\n  \"1\" {{ \"path\" \"{}\" }}\n}}\n",
                root.display().to_string().replace('\\', "\\\\"),
                lib_b.display().to_string().replace('\\', "\\\\"),
            ),
        );
        tree.write(
            &root.join("steamapps").join("appmanifest_900883.acf"),
            &manifest("900883", "ESO", "Zenimax Online"),
        );
        tree.write(
            &lib_b.join("steamapps").join("appmanifest_2221490.acf"),
            &manifest("2221490", "The Division 2", "Tom Clancy's The Division 2"),
        );

        let inst = discover_in(root).unwrap();
        assert_eq!(inst.titles.len(), 2, "warnings: {:?}", inst.warnings);
        let eso = inst.find("900883").unwrap();
        assert_eq!(eso.name, "ESO");
        assert_eq!(
            eso.install_dir,
            root.join("steamapps").join("common").join("Zenimax Online")
        );
        let div2 = inst.find("2221490").unwrap();
        assert_eq!(
            div2.install_dir,
            lib_b
                .join("steamapps")
                .join("common")
                .join("Tom Clancy's The Division 2")
        );
    }

    #[test]
    fn a_malformed_manifest_is_skipped_and_the_rest_survive() {
        let tree = TempTree::new();
        let root = tree.path();
        tree.write(
            &root.join("steamapps").join("appmanifest_1.acf"),
            &manifest("1", "Good", "Good"),
        );
        tree.write(
            &root.join("steamapps").join("appmanifest_2.acf"),
            "\"AppState\" { \"appid\" \"2\" ", // unterminated
        );

        let inst = discover_in(root).unwrap();
        assert_eq!(inst.titles.len(), 1);
        assert_eq!(inst.titles[0].app_id, "1");
        assert!(
            inst.warnings
                .iter()
                .any(|w| w.contains("appmanifest_2.acf")),
            "expected a skip warning, got {:?}",
            inst.warnings
        );
    }

    #[test]
    fn app_type_and_launch_executable_cross_reference_the_appinfo_cache() {
        use crate::appinfo::fixtures::{appinfo_bytes, FixtureApp, FixtureLaunch, V29};

        let tree = TempTree::new();
        let root = tree.path();
        tree.write(
            &root.join("steamapps").join("appmanifest_100.acf"),
            &manifest("100", "Some Soundtrack", "Some Soundtrack"),
        );
        tree.write(
            &root.join("steamapps").join("appmanifest_200.acf"),
            &manifest("200", "A Game", "A Game Folder"),
        );
        tree.write(
            &root.join("steamapps").join("appmanifest_300.acf"),
            &manifest("300", "No Appinfo Entry", "No Appinfo Entry"),
        );
        let bytes = appinfo_bytes(
            V29,
            &[
                FixtureApp {
                    appid: 100,
                    change_number: 1,
                    launch: vec![],
                    common_type: Some("Music".to_string()),
                },
                FixtureApp {
                    appid: 200,
                    change_number: 1,
                    launch: vec![FixtureLaunch::windows("game.exe")],
                    common_type: Some("Game".to_string()),
                },
            ],
        );
        tree.write_bytes(&root.join("appcache").join("appinfo.vdf"), &bytes);

        let inst = discover_in(root).unwrap();
        assert_eq!(inst.titles.len(), 3, "warnings: {:?}", inst.warnings);

        let music = inst.find("100").unwrap();
        assert_eq!(music.installdir, "Some Soundtrack");
        assert_eq!(music.app_type.as_deref(), Some("Music"));
        assert_eq!(music.launch_executable, None);
        assert_eq!(
            music.install_dir,
            root.join("steamapps").join("music").join("Some Soundtrack"),
            "a Music-typed app resolves under steamapps/music/"
        );

        let game = inst.find("200").unwrap();
        assert_eq!(game.installdir, "A Game Folder");
        assert_eq!(game.app_type.as_deref(), Some("Game"));
        assert_eq!(game.launch_executable.as_deref(), Some("game.exe"));
        assert_eq!(
            game.install_dir,
            root.join("steamapps").join("common").join("A Game Folder"),
            "an ordinary Game-typed app still resolves under steamapps/common/"
        );

        let no_entry = inst.find("300").unwrap();
        assert_eq!(no_entry.installdir, "No Appinfo Entry");
        assert_eq!(
            no_entry.app_type, None,
            "an app_id absent from the appinfo cache is unknown, not guessed"
        );
        assert_eq!(no_entry.launch_executable, None);
        assert_eq!(
            no_entry.install_dir,
            root.join("steamapps")
                .join("common")
                .join("No Appinfo Entry"),
            "an unknown type falls back to steamapps/common/"
        );
    }

    #[test]
    fn a_corrupt_appinfo_section_is_warned_about_and_does_not_affect_other_titles() {
        // Review of PR #193 (Codex): a per-section decode failure was previously
        // discarded entirely, so a title whose appinfo section is corrupt silently
        // degraded to app_type: None with no indication anything went wrong. The
        // fault is isolated to that one app (`parse_appinfo` resyncs to the next
        // section without breaking), so every other title's app_type is unaffected
        // regardless of which position the corrupt section occupies.
        use crate::appinfo::fixtures::{appinfo_bytes_with_bad_section, FixtureApp, V29};

        let tree = TempTree::new();
        let root = tree.path();
        tree.write(
            &root.join("steamapps").join("appmanifest_1.acf"),
            &manifest("1", "Before", "Before"),
        );
        tree.write(
            &root.join("steamapps").join("appmanifest_2.acf"),
            &manifest("2", "Corrupt", "Corrupt"),
        );
        tree.write(
            &root.join("steamapps").join("appmanifest_3.acf"),
            &manifest("3", "After", "After"),
        );
        let apps = [
            FixtureApp {
                appid: 1,
                change_number: 1,
                launch: vec![],
                common_type: Some("Music".to_string()),
            },
            FixtureApp {
                appid: 2,
                change_number: 1,
                launch: vec![],
                common_type: Some("Music".to_string()),
            },
            FixtureApp {
                appid: 3,
                change_number: 1,
                launch: vec![],
                common_type: Some("Music".to_string()),
            },
        ];
        // The middle section (index 1, app 2) is corrupted; the other two decode
        // cleanly.
        let bytes = appinfo_bytes_with_bad_section(V29, &apps, 1);
        tree.write_bytes(&root.join("appcache").join("appinfo.vdf"), &bytes);

        let inst = discover_in(root).unwrap();
        assert_eq!(inst.titles.len(), 3, "warnings: {:?}", inst.warnings);

        let before = inst.find("1").unwrap();
        assert_eq!(
            before.app_type.as_deref(),
            Some("Music"),
            "a title before the corrupt section is unaffected"
        );
        let after = inst.find("3").unwrap();
        assert_eq!(
            after.app_type.as_deref(),
            Some("Music"),
            "a title after the corrupt section is unaffected"
        );
        let corrupt = inst.find("2").unwrap();
        assert_eq!(
            corrupt.app_type, None,
            "the corrupt section's app type is unknown, not guessed"
        );
        assert_eq!(
            corrupt.install_dir,
            root.join("steamapps").join("common").join("Corrupt"),
            "an unknown type falls back to common/"
        );
        assert!(
            inst.warnings.iter().any(|w| w.contains('2')),
            "the corrupt section is named in a warning, not silently swallowed: {:?}",
            inst.warnings
        );
    }

    #[test]
    fn a_missing_appinfo_cache_leaves_every_title_unknown_with_no_error() {
        let tree = TempTree::new();
        let root = tree.path();
        tree.write(
            &root.join("steamapps").join("appmanifest_1.acf"),
            &manifest("1", "Good", "Good"),
        );
        // No appcache/appinfo.vdf under the root at all.
        let inst = discover_in(root).unwrap();
        let title = inst.find("1").unwrap();
        assert_eq!(title.app_type, None);
        assert_eq!(title.launch_executable, None);
        assert_eq!(
            title.install_dir,
            root.join("steamapps").join("common").join("Good")
        );
    }

    #[test]
    fn install_root_in_resolves_and_carries_warnings() {
        let tree = TempTree::new();
        let root = tree.path();
        tree.write(
            &root.join("steamapps").join("appmanifest_1.acf"),
            &manifest("1", "Good", "Good"),
        );
        tree.write(
            &root.join("steamapps").join("appmanifest_2.acf"),
            "\"AppState\" { \"appid\" \"2\" ", // malformed, discovery skips it
        );

        // The installed title resolves, and the malformed-manifest warning is
        // carried rather than swallowed (FR-008).
        let found = install_root_in(root, "1").unwrap();
        assert_eq!(
            found.install_dir,
            Some(root.join("steamapps").join("common").join("Good"))
        );
        assert!(
            found
                .warnings
                .iter()
                .any(|w| w.contains("appmanifest_2.acf")),
            "the malformed manifest is surfaced, got {:?}",
            found.warnings
        );

        // A title that is not installed still carries the warnings, so a malformed
        // manifest is never silently indistinguishable from an uninstalled title.
        let missing = install_root_in(root, "999").unwrap();
        assert!(missing.install_dir.is_none());
        assert!(!missing.warnings.is_empty());
    }

    #[test]
    fn a_configured_library_that_cannot_be_read_is_reported() {
        let tree = TempTree::new();
        let root = tree.path();
        tree.write(
            &root.join("steamapps").join("appmanifest_1.acf"),
            &manifest("1", "Good", "Good"),
        );
        // A configured library whose directory does not exist (a disconnected
        // drive): its titles are omitted, but the omission is reported.
        let gone = root.join("GoneDrive");
        tree.write(
            &root.join("steamapps").join("libraryfolders.vdf"),
            &format!(
                "\"libraryfolders\" {{ \"1\" {{ \"path\" \"{}\" }} }}",
                gone.display().to_string().replace('\\', "\\\\"),
            ),
        );

        let inst = discover_in(root).unwrap();
        assert_eq!(inst.titles.len(), 1);
        assert!(
            inst.warnings.iter().any(|w| w.contains("GoneDrive")),
            "expected an unreadable-library warning, got {:?}",
            inst.warnings
        );
    }

    #[test]
    fn a_duplicate_app_id_keeps_the_first_and_reports_the_collision() {
        let tree = TempTree::new();
        let root = tree.path();
        let lib_b = root.join("LibraryB");
        tree.write(
            &root.join("steamapps").join("libraryfolders.vdf"),
            &format!(
                "\"libraryfolders\" {{ \"1\" {{ \"path\" \"{}\" }} }}",
                lib_b.display().to_string().replace('\\', "\\\\"),
            ),
        );
        tree.write(
            &root.join("steamapps").join("appmanifest_5.acf"),
            &manifest("5", "First", "First"),
        );
        tree.write(
            &lib_b.join("steamapps").join("appmanifest_5.acf"),
            &manifest("5", "Second", "Second"),
        );

        let inst = discover_in(root).unwrap();
        assert_eq!(inst.titles.len(), 1);
        assert_eq!(inst.find("5").unwrap().name, "First");
        assert!(inst
            .warnings
            .iter()
            .any(|w| w.contains("more than one library")));
    }
}
