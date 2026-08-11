// SPDX-License-Identifier: Apache-2.0

//! Profile scaffolding (specification section 16.3).
//!
//! Scan an installed title's directory for executable images, classify them
//! heuristically, and render a profile skeleton. The skeleton is built as TOML
//! text and then parsed back through [`fragcap_profile::Profile::parse`] before
//! it is returned, so a scaffold that would fail section 15.4 validation is a
//! bug caught here rather than shipped (S17 D4; FR-008).
//!
//! The classifier proposes launcher stages for launcher-suggestive images and
//! the largest remaining image as the client. It never infers process ancestry
//! (`descends_from`) from a static scan, because runtime topology is invisible on
//! disk (S17 D7). Where two proposals would share an image basename, it adds a
//! `path_contains` predicate so the output satisfies the ambiguous-image-match
//! check.

use std::path::{Path, PathBuf};

use fragcap_profile::Profile;

use crate::library::InstalledTitle;
use crate::SteamError;

/// Substrings that mark an executable as launcher-suggestive.
const LAUNCHER_TOKENS: &[&str] = &["launcher", "launch", "starter", "bootstrap", "boot"];

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
    let proposals = classify(images);
    let text = render(title, &proposals);

    // Validity by construction (D4): never emit a profile the validator rejects.
    match Profile::parse(&text) {
        Ok(_) => Ok(text),
        Err(diags) => Err(SteamError::Scaffold(diags.to_string())),
    }
}

/// Recursively collect `.exe` images under a directory.
fn scan(dir: &Path) -> Result<Vec<ExecutableImage>, SteamError> {
    fn walk(dir: &Path, out: &mut Vec<ExecutableImage>) -> Result<(), SteamError> {
        let entries = std::fs::read_dir(dir).map_err(|source| SteamError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        for entry in entries.flatten() {
            let path = entry.path();
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
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

fn is_launcher(image: &ExecutableImage) -> bool {
    let hay = image.path.to_string_lossy().to_ascii_lowercase();
    LAUNCHER_TOKENS.iter().any(|t| hay.contains(t))
}

/// Classify images into launcher and client stage proposals.
///
/// Launcher-token images become launcher stages; the largest non-launcher image
/// becomes the client. If every image is launcher-tokened (or there is only one
/// image), the largest overall is still promoted to client so a client stage
/// always exists.
fn classify(images: Vec<ExecutableImage>) -> Vec<StageProposal> {
    let (launchers, others): (Vec<_>, Vec<_>) = images.into_iter().partition(is_launcher);

    // The client is the largest non-launcher, or, in the degenerate all-launcher
    // case, the largest launcher promoted out of the launcher set.
    let mut launchers = launchers;
    let client = if let Some(largest) = largest(&others) {
        others[largest].clone()
    } else {
        let idx = largest(&launchers).expect("at least one image exists");
        launchers.remove(idx)
    };

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

    disambiguate(&mut proposals);
    proposals
}

fn largest(images: &[ExecutableImage]) -> Option<usize> {
    images
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.size.cmp(&b.size).then(b.file_name.cmp(&a.file_name)))
        .map(|(i, _)| i)
}

/// Give any two proposals that share a basename a `path_contains` disambiguator,
/// so the emitted profile passes the ambiguous-image-match check (D7).
fn disambiguate(proposals: &mut [StageProposal]) {
    let n = proposals.len();
    for i in 0..n {
        let shares = (0..n).any(|j| {
            j != i
                && proposals[j]
                    .image
                    .file_name
                    .eq_ignore_ascii_case(&proposals[i].image.file_name)
        });
        if shares {
            proposals[i].path_disambiguator = parent_component(&proposals[i].image.path);
        }
    }
}

/// The immediate parent directory name of a path, as a `path_contains` value.
fn parent_component(path: &Path) -> Option<String> {
    path.parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
}

/// Render a profile skeleton to TOML text.
fn render(title: &InstalledTitle, proposals: &[StageProposal]) -> String {
    let mut s = String::new();
    s.push_str(
        "# Scaffolded by `fragcap steam profile`. The stage classification below is\n\
         # HEURISTIC and must be verified against an observed capture session before you\n\
         # rely on it: image names alone cannot distinguish a launcher from a client, and\n\
         # a title may run several processes sharing one image name. See specification\n\
         # section 16.3.\n\n",
    );
    s.push_str("schema = 1\n\n");
    s.push_str("[game]\n");
    s.push_str(&format!("id = \"{}\"\n", slug(&title.app_id)));
    s.push_str(&format!("name = \"{}\"\n", toml_escape(&title.name)));
    s.push_str("platform = \"steam\"\n");
    s.push_str(&format!("app_id = \"{}\"\n", toml_escape(&title.app_id)));

    for p in proposals {
        s.push_str("\n[[stage]]\n");
        match p.role {
            Role::Launcher => {
                s.push_str("role = \"launcher\"\n");
                s.push_str("lifecycle = \"transient\"\n");
            }
            Role::Client => {
                s.push_str("role = \"client\"\n");
                s.push_str("lifecycle = \"session\"\n");
                s.push_str("terminal = true\n");
            }
        }
        let mut predicates = format!("exe = \"{}\"", toml_escape(&p.image.file_name));
        if let Some(d) = &p.path_disambiguator {
            predicates.push_str(&format!(", path_contains = \"{}\"", toml_escape(d)));
        }
        s.push_str(&format!("match = {{ {predicates} }}\n"));
    }

    // A single launcher role name is unique; multiple launcher stages need unique
    // roles, which render() below never emits, so at most one launcher is
    // proposed per distinct role. Roles are made unique in-place here:
    make_roles_unique(&mut s);
    s
}

/// Ensure launcher role names are unique by suffixing repeats.
///
/// The renderer writes every launcher stage with role `launcher`; the validator
/// requires unique role names. Rather than thread state through the renderer,
/// rewrite the emitted text so the first `launcher` keeps its name and later ones
/// become `launcher-2`, `launcher-3`, and so on.
fn make_roles_unique(text: &mut String) {
    let mut count = 0u32;
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        if line == "role = \"launcher\"" {
            count += 1;
            if count == 1 {
                out.push_str(line);
            } else {
                out.push_str(&format!("role = \"launcher-{count}\""));
            }
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    *text = out;
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

/// Escape a string for a TOML basic (double-quoted) string.
fn toml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out
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
        let props = classify(vec![
            img("game/Bethesda.net_Launcher.exe", 10),
            img("game/eso64.exe", 100),
            img("game/small.exe", 5),
        ]);
        let client = props.iter().find(|p| p.role == Role::Client).unwrap();
        assert_eq!(client.image.file_name, "eso64.exe");
        assert!(props
            .iter()
            .any(|p| p.role == Role::Launcher && p.image.file_name == "Bethesda.net_Launcher.exe"));
    }

    #[test]
    fn the_largest_non_launcher_wins_the_client_role() {
        let props = classify(vec![
            img("g/a.exe", 1),
            img("g/b.exe", 9),
            img("g/c.exe", 3),
        ]);
        let client = props.iter().find(|p| p.role == Role::Client).unwrap();
        assert_eq!(client.image.file_name, "b.exe");
    }

    #[test]
    fn an_all_launcher_scan_still_yields_a_client() {
        let props = classify(vec![
            img("g/launcher.exe", 5),
            img("g/GameLauncher.exe", 20),
        ]);
        assert_eq!(props.iter().filter(|p| p.role == Role::Client).count(), 1);
        let client = props.iter().find(|p| p.role == Role::Client).unwrap();
        assert_eq!(client.image.file_name, "GameLauncher.exe");
    }

    #[test]
    fn a_single_image_becomes_the_client() {
        let props = classify(vec![img("g/only.exe", 1)]);
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].role, Role::Client);
    }

    #[test]
    fn shared_basenames_get_a_path_disambiguator() {
        let props = classify(vec![
            img("g/bin/TheDivision2.exe", 50),
            img("g/launch/TheDivision2.exe", 5),
        ]);
        // Both share a basename; both must carry a disambiguator so the pair is
        // pinned and passes the ambiguous-image check.
        for p in &props {
            if p.image.file_name == "TheDivision2.exe" {
                assert!(
                    p.path_disambiguator.is_some(),
                    "expected a disambiguator for {p:?}"
                );
            }
        }
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
        Profile::parse(&text).expect("scaffold must validate");
        assert!(text.contains("HEURISTIC"), "missing heuristic header");
        assert!(text.contains("platform = \"steam\""));
        assert!(text.contains("app_id = \"900883\""));
        assert!(text.contains("role = \"client\""));
        assert!(text.contains("eso64.exe"));
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
