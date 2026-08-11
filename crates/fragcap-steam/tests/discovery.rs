// SPDX-License-Identifier: Apache-2.0

//! Integration coverage for Steam library discovery through the public
//! [`fragcap_steam::discover_in`] API, against a fixture Steam tree laid out in a
//! temporary directory. No Steam installation is required.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use fragcap_steam::discover_in;

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A throwaway directory tree, removed on drop.
struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new() -> TempTree {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("fragcap-steam-it-{}-{}", std::process::id(), n));
        std::fs::create_dir_all(&root).unwrap();
        TempTree { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn write(&self, path: &Path, contents: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn manifest(app_id: &str, name: &str, installdir: &str) -> String {
    format!(
        "\"AppState\"\n{{\n  \"appid\" \"{app_id}\"\n  \"name\" \"{name}\"\n  \
         \"installdir\" \"{installdir}\"\n}}\n"
    )
}

fn escaped(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}

#[test]
fn enumerates_every_title_across_every_library() {
    let tree = TempTree::new();
    let root = tree.path();
    let lib_b = root.join("SteamLibrary");

    tree.write(
        &root.join("steamapps").join("libraryfolders.vdf"),
        &format!(
            "\"libraryfolders\"\n{{\n  \"0\" {{ \"path\" \"{}\" }}\n  \
             \"1\" {{ \"path\" \"{}\" }}\n  \"contentstatsid\" \"12345\"\n}}\n",
            escaped(root),
            escaped(&lib_b),
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
    assert!(inst.find("900883").is_some());
    assert!(inst.find("2221490").is_some());
    // The non-numeric `contentstatsid` key is not a library and is ignored.
    assert!(inst.libraries.iter().any(|l| l.path == lib_b));
}

#[test]
fn a_malformed_manifest_is_reported_and_skipped() {
    let tree = TempTree::new();
    let root = tree.path();
    tree.write(
        &root.join("steamapps").join("appmanifest_1.acf"),
        &manifest("1", "Good", "Good"),
    );
    tree.write(
        &root.join("steamapps").join("appmanifest_2.acf"),
        "\"AppState\" { \"appid\" \"2\"",
    );

    let inst = discover_in(root).unwrap();

    assert_eq!(inst.titles.len(), 1);
    assert_eq!(inst.titles[0].app_id, "1");
    assert!(inst
        .warnings
        .iter()
        .any(|w| w.contains("appmanifest_2.acf")));
}
