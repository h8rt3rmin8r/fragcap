// SPDX-License-Identifier: Apache-2.0

//! Publication licensing check.
//!
//! A published crate is distributed on its own, detached from the repository,
//! so the Apache-2.0 license text and the `NOTICE` file have to travel inside
//! each crate's package. Cargo will only include files that live under the
//! package directory, so every publishable crate carries its own copy.
//!
//! Copies drift. This check exists so that drift is a failed build rather
//! than a licensing defect discovered by someone reading a published crate,
//! at which point the published version can be yanked but never corrected.
//!
//! The comparison is byte for byte against the repository root originals.
//! Anything else would let a whitespace edit or a line ending change split the
//! copies apart without saying so.

use std::fs;
use std::path::Path;

/// Files that every publishable crate must carry, byte identical to the copy
/// at the repository root.
const REQUIRED: &[&str] = &["LICENSE", "NOTICE", "README.md"];

/// Files compared against a root original. `README.md` is per crate and has no
/// root counterpart, so it is required to exist but not required to match.
const MIRRORED: &[&str] = &["LICENSE", "NOTICE"];

/// True when the crate at this directory is published to a registry.
///
/// Reads the manifest rather than hardcoding a list, so a crate added later is
/// covered without anyone remembering to update this file.
fn is_published(manifest: &str) -> bool {
    !manifest
        .lines()
        .map(str::trim)
        .any(|l| l.starts_with("publish") && l.contains("false"))
}

/// Compare one crate's files against the root originals.
///
/// Pure over already-read bytes, so the rule can be tested without a fixture
/// tree on disk. `found` is `None` when the file is absent.
pub fn check_file(
    crate_name: &str,
    file: &str,
    root: Option<&[u8]>,
    found: Option<&[u8]>,
) -> Option<String> {
    match found {
        None => Some(format!("{crate_name}: {file} is missing")),
        Some(bytes) => match root {
            Some(original) if bytes != original => Some(format!(
                "{crate_name}: {file} differs from the repository root copy"
            )),
            _ => None,
        },
    }
}

pub fn run(root: &Path) -> std::io::Result<usize> {
    let mut originals = Vec::new();
    for name in MIRRORED {
        originals.push((*name, fs::read(root.join(name))?));
    }

    let mut problems: Vec<String> = Vec::new();

    for entry in fs::read_dir(root.join("crates"))? {
        let dir = entry?.path();
        if !dir.is_dir() {
            continue;
        }
        let name = match dir.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let manifest = fs::read_to_string(dir.join("Cargo.toml"))?;
        if !is_published(&manifest) {
            continue;
        }

        for file in REQUIRED {
            let found = fs::read(dir.join(file)).ok();
            let original = originals
                .iter()
                .find(|(n, _)| n == file)
                .map(|(_, b)| b.as_slice());
            if let Some(p) = check_file(&name, file, original, found.as_deref()) {
                problems.push(p);
            }
        }
    }

    problems.sort();
    for p in &problems {
        eprintln!("license: {p}");
    }
    Ok(problems.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_copy_is_clean() {
        assert_eq!(
            check_file("c", "LICENSE", Some(b"text"), Some(b"text")),
            None
        );
    }

    #[test]
    fn drifted_copy_is_reported() {
        let finding = check_file("c", "LICENSE", Some(b"text"), Some(b"text ")).unwrap();
        assert!(finding.contains("differs"));
        assert!(finding.contains("LICENSE"));
    }

    #[test]
    fn missing_copy_is_reported() {
        let finding = check_file("c", "NOTICE", Some(b"text"), None).unwrap();
        assert!(finding.contains("missing"));
    }

    #[test]
    fn per_crate_file_needs_no_root_original() {
        assert_eq!(check_file("c", "README.md", None, Some(b"anything")), None);
        assert!(check_file("c", "README.md", None, None).is_some());
    }

    #[test]
    fn unpublished_crate_is_skipped() {
        assert!(!is_published("[package]\npublish     = false\n"));
        assert!(is_published("[package]\nname = \"x\"\n"));
    }
}
