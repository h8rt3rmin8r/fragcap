// SPDX-License-Identifier: Apache-2.0

//! Shared helpers for the CLI tier-1 tests.
//!
//! Every test drives the library entry [`fragcap_cli::run_with`] directly,
//! never a spawned process, capturing the command-result stream and the
//! diagnostics stream separately. Goldens are compared byte for byte and
//! regenerated with `FRAGCAP_UPDATE_GOLDENS=1`, the same discipline the corpus
//! goldens follow.

#![allow(dead_code)]

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

/// Build an argv, prepending the program name clap expects at position zero.
pub fn argv(list: &[&str]) -> Vec<OsString> {
    std::iter::once("fragcap".to_string())
        .chain(list.iter().map(|s| s.to_string()))
        .map(OsString::from)
        .collect()
}

/// Run the command surface, returning the exit code, the stdout result stream,
/// and the stderr diagnostics stream.
pub fn run(list: &[&str]) -> (u8, String, String) {
    let mut out: Vec<u8> = Vec::new();
    let mut err: Vec<u8> = Vec::new();
    let exit = fragcap_cli::run_with(argv(list), &mut out, &mut err);
    (
        exit.code(),
        String::from_utf8(out).expect("stdout is UTF-8"),
        String::from_utf8(err).expect("stderr is UTF-8"),
    )
}

/// The committed capture fixtures shared with the rest of the workspace.
pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
}

/// The CLI test data (profiles, process scripts) committed beside these tests.
pub fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
}

/// The CLI golden directory.
pub fn goldens_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("goldens")
}

/// A fixture path as a string, for passing on the command line.
pub fn fixture(name: &str) -> String {
    fixtures_dir().join(name).to_string_lossy().into_owned()
}

/// A data path as a string, for passing on the command line.
pub fn data(name: &str) -> String {
    data_dir().join(name).to_string_lossy().into_owned()
}

/// Compare `produced` against the named golden, or write it when
/// `FRAGCAP_UPDATE_GOLDENS` is set.
pub fn assert_golden(name: &str, produced: &[u8]) {
    let path = goldens_dir().join(name);
    if std::env::var_os("FRAGCAP_UPDATE_GOLDENS").is_some() {
        fs::create_dir_all(goldens_dir()).expect("golden directory");
        fs::write(&path, produced).expect("write golden");
        return;
    }
    let want = fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "golden {} must exist ({e}); regenerate with FRAGCAP_UPDATE_GOLDENS=1",
            path.display()
        )
    });
    if want != produced {
        panic!(
            "{name}: output differs from the committed golden. If the change is intended, \
             regenerate with FRAGCAP_UPDATE_GOLDENS=1 and read the diff.\n\
             golden {} bytes, produced {} bytes",
            want.len(),
            produced.len()
        );
    }
}

/// Replace every RFC3339 timestamp value with a placeholder so an event stream
/// is comparable to a golden without depending on the wall clock.
pub fn normalize_timestamps(stream: &str) -> String {
    let mut out = String::with_capacity(stream.len());
    for line in stream.lines() {
        if let Some(rest) = line.strip_prefix("{\"ts\":\"") {
            if let Some(end) = rest.find('"') {
                out.push_str("{\"ts\":\"<ts>\"");
                out.push_str(&rest[end + 1..]);
                out.push('\n');
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
