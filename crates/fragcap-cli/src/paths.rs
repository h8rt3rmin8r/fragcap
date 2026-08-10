// SPDX-License-Identifier: Apache-2.0

//! Where profiles are looked for.
//!
//! Specification section 15.3 resolves a reference over a user profile
//! directory and any directories given on the command line, then a bundled set.
//! The resolution order itself lives in `fragcap-profile`, which takes an
//! ordered [`SearchPath`]; this module only decides what the user directory is
//! on this platform and assembles the search path from it and the repeatable
//! `--profile-dir` values.
//!
//! The user directory is the platform per-user configuration location,
//! `%APPDATA%\fragcap\profiles`, read from the environment so no
//! platform-directories crate is pulled in. An override, `FRAGCAP_PROFILE_DIR`,
//! exists so a test can point the user directory at a scratch directory without
//! depending on the developer's real one.
//!
//! The bundled set is empty in this release: a bundled profile is a claim about
//! a specific game's current process topology, and the slices that can verify
//! such a claim own shipping one (specification section 15.5).

use std::env;
use std::path::PathBuf;

use fragcap::profile::{BundledSet, SearchPath};

/// The environment variable that overrides the user profile directory.
pub const PROFILE_DIR_ENV: &str = "FRAGCAP_PROFILE_DIR";

/// The user profile directory, or `None` when the platform location cannot be
/// determined.
pub fn user_profile_dir() -> Option<PathBuf> {
    if let Some(dir) = env::var_os(PROFILE_DIR_ENV) {
        return Some(PathBuf::from(dir));
    }
    env::var_os("APPDATA").map(|base| PathBuf::from(base).join("fragcap").join("profiles"))
}

/// Assemble the section 15.3 search path from the command-line directories and
/// the user directory.
pub fn search_path(command_line: &[PathBuf]) -> SearchPath {
    SearchPath {
        command_line: command_line.to_vec(),
        user: user_profile_dir(),
    }
}

/// The bundled profile set for this release: empty.
pub fn bundled() -> BundledSet {
    BundledSet::empty()
}
