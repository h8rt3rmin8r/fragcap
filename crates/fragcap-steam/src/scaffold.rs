// SPDX-License-Identifier: Apache-2.0

//! Profile scaffolding (specification section 16.3).
//!
//! Scan an installed title's directory for executable images, classify them
//! heuristically, and render a profile skeleton. The skeleton is built as TOML
//! text and then parsed back through [`fragcap_profile::Profile::parse`] before
//! it is returned, so a scaffold that would fail section 15.4 validation is a
//! bug caught here rather than shipped (S17 D4; FR-008).
//!
//! The classifier first drops obvious non-game executables (installers,
//! redistributables, crash handlers, helper stubs, hash-named temp installers),
//! then proposes launcher stages for launcher-suggestive images and the largest
//! remaining image as the client. Dropping the non-game images first is what
//! fixes issue #64: the ESO launcher directory is full of installers and
//! redistributables, and without the denylist they became stages and the largest
//! of them, `setup.exe`, became the terminal client. Launcher detection stays
//! path-aware, per specification section 16.3: a launcher-suggestive token in
//! either the file name or a directory on the path marks a launcher. It never
//! infers process ancestry (`descends_from`) from a static scan, because runtime
//! topology is invisible on disk (S17 D7). Where two proposals would share an
//! image basename, it adds a `path_contains` predicate so the output satisfies
//! the ambiguous-image-match check.

use std::path::{Path, PathBuf};

use fragcap_profile::Profile;

use crate::library::InstalledTitle;
use crate::SteamError;

/// Substrings that mark an executable as launcher-suggestive, strongest first so
/// the best match becomes `role = "launcher"` and the rest fall to `launcher-2`.
const LAUNCHER_TOKENS: &[&str] = &["launcher", "launch", "starter", "bootstrap", "boot"];

/// Basename substrings that mark an executable as not a game process: installers,
/// redistributables, crash handlers, and helper stubs. These are dropped before
/// classification so they never become stages or the terminal client (issue #64).
const NON_GAME_TOKENS: &[&str] = &[
    "setup",
    "installer",
    "uninstall",
    "unins",
    "vc_redist",
    "vcredist",
    "redist",
    "dxsetup",
    "directx",
    "dotnetfx",
    "dotnet",
    "crashhandler",
    "crashreporter",
    "crashreport",
    "crashpad",
    "helper",
    "oalinst",
];

/// An executable image found by scanning an install directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableImage {
    /// Full path under the install directory.
    pub path: PathBuf,
    /// The basename, e.g. `eso64.exe`.
    pub file_name: String,
    /// Byte size; the client-selection tiebreak.
    pub size: u64,
}

/// The role a proposed stage carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    Launcher,
    Client,
}

/// A classified image, ready to render as a profile stage.
#[derive(Clone, Debug, PartialEq, Eq)]
struct StageProposal {
    role: Role,
    image: ExecutableImage,
    /// A `path_contains` value, set only when another proposal shares this
    /// basename, so the emitted profile passes the ambiguous-image check.
    path_disambiguator: Option<String>,
}

/// Scaffold a validated profile skeleton for an installed title.
pub fn scaffold(title: &InstalledTitle) -> Result<String, SteamError> {
    let images = scan(&title.install_dir)?;
    if images.is_empty() {
        return Err(SteamError::NoExecutables {
            install_dir: title.install_dir.clone(),
        });
    }
    let proposals = classify(images, &title.install_dir);
    let text = render(title, &proposals);

    // Validity by construction (D4): never emit a profile the validator rejects.
    match Profile::parse(&text) {
        Ok(_) => Ok(text),
        Err(diags) => Err(SteamError::Scaffold(diags.to_string())),
    }
}

/// Recursively collect `.exe` images under a directory.
///
/// Strict about read failures: a `read_dir` error, a directory-entry iterator
/// error, or a per-entry metadata error is surfaced as [`SteamError::Io`] rather
/// than skipped. An incomplete scan is not the same as an exhaustive one; the
/// platform walker (S030) declines when the scan cannot complete so it never
/// resolves a false single client from a partial view, and the scaffold likewise
/// does not build a skeleton from one.
pub(crate) fn scan(dir: &Path) -> Result<Vec<ExecutableImage>, SteamError> {
    fn walk(dir: &Path, out: &mut Vec<ExecutableImage>) -> Result<(), SteamError> {
        let entries = std::fs::read_dir(dir).map_err(|source| SteamError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| SteamError::Io {
                path: dir.to_path_buf(),
                source,
            })?;
            let path = entry.path();
            let meta = entry.metadata().map_err(|source| SteamError::Io {
                path: path.clone(),
                source,
            })?;
            if meta.is_dir() {
                walk(&path, out)?;
            } else if is_exe(&path) {
                let file_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                out.push(ExecutableImage {
                    path,
                    file_name,
                    size: meta.len(),
                });
            }
        }
        Ok(())
    }

    let mut out = Vec::new();
    walk(dir, &mut out)?;
    // Deterministic order regardless of directory iteration order.
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn is_exe(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("exe"))
        .unwrap_or(false)
}

/// The lowercased full path a launcher-token match runs over. Path-aware per
/// specification section 16.3: a launcher indication may be in the file name or a
/// directory on the path. Non-game images are removed before this runs, so an
/// installer sitting under a `Launcher` directory no longer reaches it.
fn launcher_haystack(image: &ExecutableImage) -> String {
    image.path.to_string_lossy().to_ascii_lowercase()
}

/// Whether an image is launcher-suggestive.
pub(crate) fn is_launcher(image: &ExecutableImage) -> bool {
    let hay = launcher_haystack(image);
    LAUNCHER_TOKENS.iter().any(|t| hay.contains(t))
}

/// A launcher's rank, lower being a stronger match, used to order the launcher
/// stages so the most launcher-like image is `role = "launcher"`.
fn launcher_rank(image: &ExecutableImage) -> usize {
    let hay = launcher_haystack(image);
    LAUNCHER_TOKENS
        .iter()
        .position(|t| hay.contains(t))
        .unwrap_or(LAUNCHER_TOKENS.len())
}

/// Whether an image is an obvious non-game executable: an installer,
/// redistributable, crash handler, helper stub, or a hash-named temp installer.
pub(crate) fn is_non_game(file_name: &str) -> bool {
    let name = file_name.to_ascii_lowercase();
    if NON_GAME_TOKENS.iter().any(|t| name.contains(t)) {
        return true;
    }
    // Hash-named temp installers, e.g. f5f0755f8afc2b40b7ceb0cc8fed2e30.exe: a
    // long run of hex digits and nothing else.
    let stem = name.strip_suffix(".exe").unwrap_or(&name);
    stem.len() >= 16 && stem.chars().all(|c| c.is_ascii_hexdigit())
}

/// Classify images into launcher and client stage proposals.
///
/// Obvious non-game executables are dropped first. Of what remains, launcher-token
/// images become launcher stages, ordered strongest-match first, and the largest
/// non-launcher image becomes the client. If every remaining image is
/// launcher-tokened (or there is only one), the largest is still promoted to
/// client so a client stage always exists. If the denylist would remove
/// everything, the original set is kept rather than emit an empty scaffold.
fn classify(images: Vec<ExecutableImage>, install_dir: &Path) -> Vec<StageProposal> {
    let mut candidates: Vec<ExecutableImage> = images
        .iter()
        .filter(|i| !is_non_game(&i.file_name))
        .cloned()
        .collect();
    if candidates.is_empty() {
        candidates = images;
    }

    let (mut launchers, others): (Vec<_>, Vec<_>) = candidates.into_iter().partition(is_launcher);

    // The client is the largest non-launcher, or, in the degenerate all-launcher
    // case, the largest launcher promoted out of the launcher set.
    let client = if let Some(largest) = largest(&others) {
        others[largest].clone()
    } else {
        let idx = largest(&launchers).expect("at least one candidate exists");
        launchers.remove(idx)
    };

    // Order launchers so the strongest launcher-token match is role = "launcher",
    // then by size, so the likely real launcher leads (issue #64).
    launchers.sort_by(|a, b| {
        launcher_rank(a)
            .cmp(&launcher_rank(b))
            .then(b.size.cmp(&a.size))
            .then(a.file_name.cmp(&b.file_name))
    });

    let mut proposals: Vec<StageProposal> = launchers
        .into_iter()
        .map(|image| StageProposal {
            role: Role::Launcher,
            image,
            path_disambiguator: None,
        })
        .collect();
    proposals.push(StageProposal {
        role: Role::Client,
        image: client,
        path_disambiguator: None,
    });

    disambiguate(&mut proposals, install_dir);
    proposals
}

fn largest(images: &[ExecutableImage]) -> Option<usize> {
    images
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.size.cmp(&b.size).then(b.file_name.cmp(&a.file_name)))
        .map(|(i, _)| i)
}

/// Give any two proposals that share a basename a `path_contains` disambiguator
/// that actually distinguishes them, so the emitted profile both passes the
/// ambiguous-image-match check and binds each stage to only its own process
/// (D7; Codex review of PR #31).
///
/// The immediate parent directory name alone is not enough:
/// `launcher/bin/game.exe` and `client/bin/game.exe` would both get `bin`, so
/// each stage would still match both processes. Instead each proposal takes the
/// path of its executable relative to the install directory's parent, for
/// example `TheGame\launcher\bin\game.exe`. That tail is distinct per file
/// (two files with the same relative path would be the same file) and, because
/// it is anchored above the install directory and carries the basename, a
/// shorter tail is not a substring of a deeper sibling's path. In the rare case
/// the anchored tail is still a substring of another proposal's path (a
/// subdirectory named like the install directory), fall back to the full,
/// drive-anchored path, which is unique.
fn disambiguate(proposals: &mut [StageProposal], install_dir: &Path) {
    let n = proposals.len();
    let anchor = install_dir.parent().unwrap_or(install_dir);
    let paths: Vec<String> = proposals
        .iter()
        .map(|p| p.image.path.to_string_lossy().into_owned())
        .collect();

    for i in 0..n {
        let shares = (0..n).any(|j| {
            j != i
                && proposals[j]
                    .image
                    .file_name
                    .eq_ignore_ascii_case(&proposals[i].image.file_name)
        });
        if !shares {
            continue;
        }

        let tail = proposals[i]
            .image
            .path
            .strip_prefix(anchor)
            .unwrap_or(&proposals[i].image.path)
            .to_string_lossy()
            .into_owned();
        // `path_contains` matches case-insensitively (see the matcher), so the
        // uniqueness check must too.
        let distinguishes = |candidate: &str| {
            !candidate.is_empty()
                && (0..n)
                    .all(|j| j == i || !paths[j].to_lowercase().contains(&candidate.to_lowercase()))
        };
        let value = if distinguishes(&tail) {
            tail
        } else {
            paths[i].clone()
        };
        proposals[i].path_disambiguator = Some(value);
    }
}

/// The load-bearing warning carried on every scaffold, as structured data.
///
/// It was a TOML header comment before the JSON migration (#76); a comment is
/// stripped by every parser and the resolver cannot act on it, so it is now the
/// profile's `notes` field alongside a `fidelity` of `heuristic-unverified`. A
/// machine can then refuse to treat the guess as verified.
const HEURISTIC_NOTE: &str = "Scaffolded by `fragcap steam profile`. The stage \
classification here is HEURISTIC and must be verified against an observed capture \
session before you rely on it: image names alone cannot distinguish a launcher \
from a client, and a title may run several processes sharing one image name. See \
specification section 16.3.";

/// Render a profile skeleton to JSON text.
///
/// Built as a [`serde_json::Value`] and serialized, so escaping is correct by
/// construction and the output re-parses (the caller asserts it validates). The
/// stage classification is a heuristic, which the `fidelity` and `notes` fields
/// declare.
fn render(title: &InstalledTitle, proposals: &[StageProposal]) -> String {
    use serde_json::{json, Map, Value};

    let mut stages: Vec<Value> = Vec::with_capacity(proposals.len());
    // The renderer proposes at most one launcher per distinct image, but several
    // launchers need unique role names; the first keeps `launcher`, later ones
    // become `launcher-2`, `launcher-3`, and so on.
    let mut launcher_count = 0u32;
    for p in proposals {
        let mut stage = Map::new();
        match p.role {
            Role::Launcher => {
                launcher_count += 1;
                let role = if launcher_count == 1 {
                    "launcher".to_string()
                } else {
                    format!("launcher-{launcher_count}")
                };
                stage.insert("role".to_string(), json!(role));
                stage.insert("lifecycle".to_string(), json!("transient"));
            }
            Role::Client => {
                stage.insert("role".to_string(), json!("client"));
                stage.insert("lifecycle".to_string(), json!("session"));
                stage.insert("terminal".to_string(), json!(true));
            }
        }
        let mut predicates = Map::new();
        predicates.insert("exe".to_string(), json!(p.image.file_name));
        if let Some(d) = &p.path_disambiguator {
            predicates.insert("path_contains".to_string(), json!(d));
        }
        stage.insert("match".to_string(), Value::Object(predicates));
        stages.push(Value::Object(stage));
    }

    let profile = json!({
        "schema": 1,
        "kind": "profile",
        "fidelity": "heuristic-unverified",
        "notes": HEURISTIC_NOTE,
        "game": {
            "id": slug(&title.app_id),
            "name": title.name,
            "platform": "steam",
            "app_id": title.app_id,
        },
        "stage": stages,
    });

    let mut out = serde_json::to_string_pretty(&profile).expect("a scaffold is serializable");
    out.push('\n');
    out
}

/// A valid `game.id` slug derived from the app_id.
///
/// The slug charset is lowercase ASCII alphanumerics, hyphen, and underscore. A
/// `steam-` prefix keeps it non-empty and stable, and any out-of-charset byte in
/// the app_id becomes a hyphen.
fn slug(app_id: &str) -> String {
    let mut s = String::from("steam-");
    for c in app_id.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_' {
            s.push(c);
        } else if c.is_ascii_uppercase() {
            s.push(c.to_ascii_lowercase());
        } else {
            s.push('-');
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(path: &str, size: u64) -> ExecutableImage {
        let p = PathBuf::from(path);
        let file_name = p.file_name().unwrap().to_string_lossy().into_owned();
        ExecutableImage {
            path: p,
            file_name,
            size,
        }
    }

    #[test]
    fn a_launcher_token_image_is_a_launcher_and_the_largest_other_is_the_client() {
        let props = classify(
            vec![
                img("game/Bethesda.net_Launcher.exe", 10),
                img("game/eso64.exe", 100),
                img("game/small.exe", 5),
            ],
            Path::new("game"),
        );
        let client = props.iter().find(|p| p.role == Role::Client).unwrap();
        assert_eq!(client.image.file_name, "eso64.exe");
        assert!(props
            .iter()
            .any(|p| p.role == Role::Launcher && p.image.file_name == "Bethesda.net_Launcher.exe"));
    }

    #[test]
    fn the_largest_non_launcher_wins_the_client_role() {
        let props = classify(
            vec![img("g/a.exe", 1), img("g/b.exe", 9), img("g/c.exe", 3)],
            Path::new("g"),
        );
        let client = props.iter().find(|p| p.role == Role::Client).unwrap();
        assert_eq!(client.image.file_name, "b.exe");
    }

    #[test]
    fn an_all_launcher_scan_still_yields_a_client() {
        let props = classify(
            vec![img("g/launcher.exe", 5), img("g/GameLauncher.exe", 20)],
            Path::new("g"),
        );
        assert_eq!(props.iter().filter(|p| p.role == Role::Client).count(), 1);
        let client = props.iter().find(|p| p.role == Role::Client).unwrap();
        assert_eq!(client.image.file_name, "GameLauncher.exe");
    }

    #[test]
    fn a_single_image_becomes_the_client() {
        let props = classify(vec![img("g/only.exe", 1)], Path::new("g"));
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].role, Role::Client);
    }

    #[test]
    fn shared_basenames_get_disambiguators_that_actually_distinguish() {
        // Two same-basename executables under sibling `bin` directories,
        // distinguished by a launcher-token directory on the path (the classifier
        // is path-aware). A disambiguator of just the parent (`bin`) would match
        // both; each must get a value that matches its own path and not the
        // other's.
        let props = classify(
            vec![
                img("Game/launcher/bin/TheDivision2.exe", 5),
                img("Game/client/bin/TheDivision2.exe", 50),
            ],
            Path::new("Game"),
        );
        let matches =
            |needle: &str, path: &str| path.to_lowercase().contains(&needle.to_lowercase());
        for p in &props {
            let d = p
                .path_disambiguator
                .as_ref()
                .unwrap_or_else(|| panic!("expected a disambiguator for {p:?}"));
            // The disambiguator matches this proposal's own path...
            let own = p.image.path.to_string_lossy();
            assert!(matches(d, &own), "{d} should match own path {own}");
            // ...and no other same-basename proposal's path.
            for q in &props {
                if !std::ptr::eq(p, q) {
                    let other = q.image.path.to_string_lossy();
                    assert!(
                        !matches(d, &other),
                        "disambiguator {d} must not also match {other}"
                    );
                }
            }
        }
    }

    #[test]
    fn installers_and_redistributables_are_never_stages() {
        // The ESO-306130 shape: a real launcher, redistributables, an installer,
        // a hash-named temp installer, a helper stub, and the game client.
        let props = classify(
            vec![
                img("g/Bethesda.net_Launcher.exe", 20),
                img("g/vc_redist.2022.x64.exe", 900),
                img("g/vc_redist.2022.x86.exe", 800),
                img("g/setup.exe", 1000),
                img("g/f5f0755f8afc2b40b7ceb0cc8fed2e30.exe", 500),
                img("g/RestartHelper.exe", 30),
                img("g/eso64.exe", 400),
            ],
            Path::new("g"),
        );
        let names: Vec<&str> = props.iter().map(|p| p.image.file_name.as_str()).collect();
        for junk in [
            "vc_redist.2022.x64.exe",
            "vc_redist.2022.x86.exe",
            "setup.exe",
            "f5f0755f8afc2b40b7ceb0cc8fed2e30.exe",
            "RestartHelper.exe",
        ] {
            assert!(
                !names.contains(&junk),
                "{junk} must not be a stage: {names:?}"
            );
        }
        let client = props.iter().find(|p| p.role == Role::Client).unwrap();
        assert_eq!(client.image.file_name, "eso64.exe");
        assert!(props
            .iter()
            .any(|p| p.role == Role::Launcher && p.image.file_name == "Bethesda.net_Launcher.exe"));
    }

    #[test]
    fn an_installer_is_not_chosen_as_the_client_even_when_largest() {
        // setup.exe dwarfs the game, but an installer must never be the client.
        let props = classify(
            vec![img("g/setup.exe", 10_000), img("g/eso64.exe", 100)],
            Path::new("g"),
        );
        let client = props.iter().find(|p| p.role == Role::Client).unwrap();
        assert_eq!(client.image.file_name, "eso64.exe");
        assert!(!props.iter().any(|p| p.image.file_name == "setup.exe"));
    }

    #[test]
    fn installers_under_a_launcher_directory_are_dropped_not_promoted_to_client() {
        // The ESO shape: the launcher directory holds the launcher plus an
        // installer and a redistributable, and the installer is the largest image.
        // The denylist drops the installer and redist before classification, so
        // the largest of them is never promoted to the terminal client; the real
        // game client (elsewhere on disk, no launcher token on its path) wins
        // (issue #64).
        let props = classify(
            vec![
                img("ESO/Launcher/Bethesda.net_Launcher.exe", 20),
                img("ESO/Launcher/setup.exe", 5000),
                img("ESO/Launcher/vc_redist.2022.x64.exe", 900),
                img("ESO/game/client/eso64.exe", 400),
            ],
            Path::new("ESO"),
        );
        let client = props.iter().find(|p| p.role == Role::Client).unwrap();
        assert_eq!(client.image.file_name, "eso64.exe");
        assert!(!props.iter().any(|p| p.image.file_name == "setup.exe"));
        assert!(!props
            .iter()
            .any(|p| p.image.file_name == "vc_redist.2022.x64.exe"));
    }

    #[test]
    fn an_all_denylisted_scan_still_yields_a_client() {
        // If every image is an installer or redistributable, keep the set rather
        // than error, so the operator gets a flagged starting point, not nothing.
        let props = classify(
            vec![img("g/setup.exe", 10), img("g/vc_redist.x64.exe", 20)],
            Path::new("g"),
        );
        assert_eq!(props.iter().filter(|p| p.role == Role::Client).count(), 1);
    }

    #[test]
    fn launchers_are_ordered_strongest_token_first() {
        let props = classify(
            vec![
                img("g/GameBoot.exe", 5),
                img("g/MainLauncher.exe", 5),
                img("g/eso64.exe", 100),
            ],
            Path::new("g"),
        );
        let launchers: Vec<&str> = props
            .iter()
            .filter(|p| p.role == Role::Launcher)
            .map(|p| p.image.file_name.as_str())
            .collect();
        assert_eq!(
            launchers.first(),
            Some(&"MainLauncher.exe"),
            "the strongest launcher-token match leads: {launchers:?}"
        );
    }

    #[test]
    fn slug_sanitizes_to_the_valid_charset() {
        assert_eq!(slug("900883"), "steam-900883");
        assert_eq!(slug("Ab.C"), "steam-ab-c");
    }

    #[test]
    fn a_scaffold_validates_and_carries_the_heuristic_header() {
        use crate::test_support::TempTree;
        let tree = TempTree::new();
        let install = tree.path().join("Zenimax Online");
        tree.write_exe(&install.join("Bethesda.net_Launcher.exe"), 10);
        tree.write_exe(&install.join("eso64.exe"), 100);
        let title = InstalledTitle {
            app_id: "900883".to_string(),
            name: "The Elder Scrolls Online".to_string(),
            install_dir: install,
        };
        let text = scaffold(&title).unwrap();
        // scaffold() validates internally; assert it again and check the shape.
        let p = Profile::parse(&text).expect("scaffold must validate");
        // The heuristic warning survives as structured data, not a comment.
        assert!(text.contains("HEURISTIC"), "missing heuristic note");
        assert!(
            text.contains("\"fidelity\": \"heuristic-unverified\""),
            "a scaffold is stamped heuristic-unverified: {text}"
        );
        assert!(text.contains("\"notes\""), "the warning is a notes field");
        assert!(text.contains("\"platform\": \"steam\""));
        assert!(text.contains("\"app_id\": \"900883\""));
        assert!(text.contains("\"role\": \"client\""));
        assert!(text.contains("eso64.exe"));
        assert_eq!(p.game().name(), "The Elder Scrolls Online");
    }

    #[test]
    fn a_scaffold_with_a_shared_basename_still_validates() {
        use crate::test_support::TempTree;
        let tree = TempTree::new();
        let install = tree.path().join("game");
        // The same image name in two directories: one under a launcher-token
        // path, one not. The renderer must pin both so validation passes.
        tree.write_exe(&install.join("bin").join("TheDivision2.exe"), 100);
        tree.write_exe(&install.join("launch").join("TheDivision2.exe"), 5);
        let title = InstalledTitle {
            app_id: "2221490".to_string(),
            name: "The Division 2".to_string(),
            install_dir: install,
        };
        let text = scaffold(&title).unwrap();
        Profile::parse(&text).expect("shared-basename scaffold must validate");
        assert!(text.contains("path_contains"), "expected a disambiguator");
    }

    #[test]
    fn an_empty_install_directory_is_a_named_error() {
        use crate::test_support::TempTree;
        let tree = TempTree::new();
        let install = tree.path().join("empty");
        std::fs::create_dir_all(&install).unwrap();
        let title = InstalledTitle {
            app_id: "1".to_string(),
            name: "Empty".to_string(),
            install_dir: install,
        };
        assert!(matches!(
            scaffold(&title),
            Err(SteamError::NoExecutables { .. })
        ));
    }
}
