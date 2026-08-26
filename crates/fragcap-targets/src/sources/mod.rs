// SPDX-License-Identifier: Apache-2.0

//! The discovery sources (slice S052).
//!
//! Tier 2 is [`known_roots::KnownRootsSource`], which walks a fixed list of
//! game-only directories across every eligible fixed volume. Tier 3 (the
//! user-pointed directory and interactive sources) lands alongside it. Every
//! source enumerates directories through the [`DirectoryLister`] seam, injected so
//! the walk is a pure decision over a value in tests (FR-019); the real filesystem
//! implementation is [`FsDirectoryLister`].

pub mod directory;
pub mod interactive;
pub mod known_roots;

/// The result of listing a directory's immediate children.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirListing {
    /// The directory does not exist. For a known root this is normal, not an error
    /// (FR-010): it contributes no candidate and is not counted.
    Absent,
    /// The immediate subdirectory paths under the directory.
    Present(Vec<String>),
    /// The directory exists but could not be read (a permission or I/O error). The
    /// walk counts this `access_error`, named by directory, never silent (P-4).
    AccessError,
}

/// A source of a directory's immediate subdirectories. Injected so the walk needs
/// no filesystem in tests; the real implementation is [`FsDirectoryLister`].
pub trait DirectoryLister {
    /// The immediate subdirectories of `dir`.
    fn subdirectories(&self, dir: &str) -> DirListing;
}

/// The real filesystem lister over `std::fs` (portable).
pub struct FsDirectoryLister;

impl DirectoryLister for FsDirectoryLister {
    fn subdirectories(&self, dir: &str) -> DirListing {
        // Known roots are stored with `/` so fixtures and live discovery share one
        // list. Convert that neutral form exactly at the Windows filesystem
        // boundary; otherwise `ReadDir::path` preserves the mixed prefix and appends
        // children with `\`, producing a mixed durable candidate identity.
        #[cfg(windows)]
        let native_dir = dir.replace('/', "\\");
        #[cfg(not(windows))]
        let native_dir = dir.to_string();
        let read = match std::fs::read_dir(&native_dir) {
            Ok(read) => read,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return DirListing::Absent,
            Err(_) => return DirListing::AccessError,
        };
        let mut out = Vec::new();
        for entry in read {
            // An entry that cannot be read, or whose type cannot be determined,
            // means the directory was not fully enumerated. Reporting the partial
            // list as complete would present an interrupted walk as a finished one,
            // so the whole directory is an access error, counted rather than a
            // silently truncated success (P-4).
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => return DirListing::AccessError,
            };
            let is_dir = match entry.file_type() {
                Ok(file_type) => file_type.is_dir(),
                Err(_) => return DirListing::AccessError,
            };
            // A subdirectory is a candidate location; a file is not.
            if is_dir {
                out.push(entry.path().to_string_lossy().into_owned());
            }
        }
        DirListing::Present(out)
    }
}

/// The final path segment of a directory path, tolerant of both separators. Used
/// as a candidate's display name when nothing richer is available.
pub(crate) fn base_name(path: &str) -> String {
    path.trim_end_matches(['/', '\\'])
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(path)
        .to_string()
}

/// A fixture directory tree: a map of directory path to its immediate
/// subdirectories, plus a set of paths that report an access error. A path absent
/// from both is [`DirListing::Absent`]. Keeps the walk testable with no filesystem.
pub struct FixtureTree {
    dirs: std::collections::HashMap<String, Vec<String>>,
    access_errors: std::collections::HashSet<String>,
}

impl FixtureTree {
    /// An empty tree (every path is absent).
    pub fn new() -> Self {
        FixtureTree {
            dirs: std::collections::HashMap::new(),
            access_errors: std::collections::HashSet::new(),
        }
    }

    /// Record `dir` as present with the given immediate subdirectories.
    pub fn with_dir(mut self, dir: &str, subdirs: &[&str]) -> Self {
        self.dirs.insert(
            dir.to_string(),
            subdirs.iter().map(|s| s.to_string()).collect(),
        );
        self
    }

    /// Record `dir` as present but unreadable (a permission or I/O error).
    pub fn with_access_error(mut self, dir: &str) -> Self {
        self.access_errors.insert(dir.to_string());
        self
    }
}

impl Default for FixtureTree {
    fn default() -> Self {
        Self::new()
    }
}

impl DirectoryLister for FixtureTree {
    fn subdirectories(&self, dir: &str) -> DirListing {
        if self.access_errors.contains(dir) {
            return DirListing::AccessError;
        }
        match self.dirs.get(dir) {
            Some(subdirs) => DirListing::Present(subdirs.clone()),
            None => DirListing::Absent,
        }
    }
}
