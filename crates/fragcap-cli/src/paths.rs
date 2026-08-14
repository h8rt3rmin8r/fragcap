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
use std::path::{Path, PathBuf};

use fragcap::profile::{BundledSet, SearchPath};

/// The environment variable that overrides the user profile directory.
pub const PROFILE_DIR_ENV: &str = "FRAGCAP_PROFILE_DIR";

/// The environment variable that supplies a targets hint database for resolution.
pub const HINT_DB_ENV: &str = "FRAGCAP_HINT_DB";

/// The environment variable that overrides the analyzer extcap directory.
pub const EXTCAP_DIR_ENV: &str = "FRAGCAP_EXTCAP_DIR";

/// The analyzer's personal extcap directory, or `None` when the platform
/// location cannot be determined.
///
/// This is where the fragcap binary is copied to register it as an extcap
/// capture source (specification section 14.5). `doctor` reports it read-only and
/// installs nothing. On Windows it is `%APPDATA%\Wireshark\extcap`; elsewhere it
/// is the XDG or HOME Wireshark configuration location. An override,
/// `FRAGCAP_EXTCAP_DIR`, lets a test point it at a scratch directory.
pub fn extcap_dir() -> Option<PathBuf> {
    if let Some(dir) = env::var_os(EXTCAP_DIR_ENV) {
        return Some(PathBuf::from(dir));
    }
    #[cfg(windows)]
    {
        env::var_os("APPDATA").map(|base| PathBuf::from(base).join("Wireshark").join("extcap"))
    }
    #[cfg(not(windows))]
    {
        if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg).join("wireshark").join("extcap"));
        }
        env::var_os("HOME").map(|home| {
            PathBuf::from(home)
                .join(".config")
                .join("wireshark")
                .join("extcap")
        })
    }
}

/// The user profile directory, or `None` when the platform location cannot be
/// determined.
pub fn user_profile_dir() -> Option<PathBuf> {
    if let Some(dir) = env::var_os(PROFILE_DIR_ENV) {
        return Some(PathBuf::from(dir));
    }
    env::var_os("APPDATA").map(|base| PathBuf::from(base).join("fragcap").join("profiles"))
}

/// The targets hint database to consult during resolution, or `None` when the
/// operator supplied neither the `--hint-db` flag nor the `FRAGCAP_HINT_DB`
/// override.
///
/// The flag takes precedence over the environment variable, mirroring how the
/// profile directory resolves. Returning a path here does not assert the file
/// exists; the caller decides that a missing file means no provider (not an
/// error), while a present-but-unopenable one is surfaced loudly.
pub fn hint_db_path(flag: Option<&Path>) -> Option<PathBuf> {
    if let Some(path) = flag {
        return Some(path.to_path_buf());
    }
    env::var_os(HINT_DB_ENV).map(PathBuf::from)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flag_supplies_the_hint_db_path() {
        // The explicit flag is returned as-is, independent of the environment.
        let flag = Path::new("C:/scratch/hint.db");
        assert_eq!(hint_db_path(Some(flag)), Some(flag.to_path_buf()));
    }
}
