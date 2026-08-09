// SPDX-License-Identifier: Apache-2.0

//! The resolution order of specification section 15.3.
//!
//! A profile reference resolves in four steps, first match winning:
//!
//! 1. A path to an existing file.
//! 2. `<ref>.toml` in a profile directory given on the command line.
//! 3. `<ref>.toml` in the user profile directory.
//! 4. A bundled profile whose `game.id` matches.
//!
//! User profiles shadow bundled ones by design, so a bundled profile that has
//! drifted from a game update is corrected locally without waiting for a release.
//!
//! # The search path comes from the caller
//!
//! This module never asks the operating system where a user's configuration
//! lives. It takes an ordered [`SearchPath`] and a [`BundledSet`] and implements
//! the order over them. That keeps a platform-directories dependency out of the
//! workspace, keeps the ordering testable against directories a test builds, and
//! leaves the platform question to the command line, which is the layer allowed
//! to have an opinion about it.
//!
//! # Why the slug rule is applied here as well as during validation
//!
//! A reference arrives from a command line argument, and steps two through four
//! join it to a directory. A reference of `../../../windows/system32/drivers` is
//! refused before any join happens, because a check that relies on the open
//! failing is a check that depends on what happens to be at the target.
//!
//! Step one is exempt on purpose. An operator who types a path has named a file,
//! and refusing an absolute path there would break the ordinary case section 15.3
//! puts first. The distinction is between naming a file and interpolating a name
//! into a search path.

use std::fmt;
use std::path::{Path, PathBuf};

use crate::parse::{load, LoadError};
use crate::schema::{GameId, Profile};

/// Where resolution should look, in section 15.3's order.
///
/// Both fields are supplied by the caller. Modelled as the two steps rather than
/// as one flat list, so that a successful resolution can say which step supplied
/// the profile without the caller having to remember what an index meant.
#[derive(Clone, Debug, Default)]
pub struct SearchPath {
    /// Profile directories given on the command line, in the order given.
    /// Section 15.3 step 2.
    pub command_line: Vec<PathBuf>,
    /// The user profile directory. Section 15.3 step 3.
    pub user: Option<PathBuf>,
}

impl SearchPath {
    /// An empty search path, which resolves only step one and step four.
    pub fn new() -> SearchPath {
        SearchPath::default()
    }
}

/// Two bundled profiles declared the same `game.id`.
///
/// Refused at construction, because section 15.3 step four selects on that
/// identifier and a duplicate makes the step ambiguous.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DuplicateGameId(pub String);

impl fmt::Display for DuplicateGameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "two bundled profiles declare game.id `{}`; resolution step four \
             selects on it and cannot choose",
            self.0
        )
    }
}

impl std::error::Error for DuplicateGameId {}

/// The profiles bundled with a fragcap distribution. Section 15.3 step 4.
///
/// Holds already-validated profiles rather than text, so an invalid bundled
/// profile cannot exist in the set at all. The set this slice ships is empty:
/// section 15.5 ships profiles for the two focal titles, and a bundled profile is
/// a claim about a specific game's current process topology, which the slices
/// that can verify such a claim own.
#[derive(Clone, Debug, Default)]
pub struct BundledSet {
    profiles: Vec<Profile>,
}

impl BundledSet {
    /// An empty set.
    pub fn empty() -> BundledSet {
        BundledSet::default()
    }

    /// Build a set, refusing a duplicate `game.id`.
    ///
    /// # Errors
    ///
    /// [`DuplicateGameId`] naming the identifier that appeared twice.
    pub fn new(profiles: Vec<Profile>) -> Result<BundledSet, DuplicateGameId> {
        for (i, a) in profiles.iter().enumerate() {
            for b in profiles.iter().skip(i + 1) {
                if a.game().id() == b.game().id() {
                    return Err(DuplicateGameId(a.game().id().to_string()));
                }
            }
        }
        Ok(BundledSet { profiles })
    }

    /// The profile with this identifier, if the set has one.
    pub fn get(&self, id: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.game().id().as_str() == id)
    }

    /// How many profiles are bundled.
    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }
}

/// Which of section 15.3's four steps supplied a profile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileSource {
    /// Step 1: the reference named this existing file.
    ExplicitPath(PathBuf),
    /// Step 2: found in a command line profile directory.
    CommandLineDirectory(PathBuf),
    /// Step 3: found in the user profile directory.
    UserDirectory(PathBuf),
    /// Step 4: bundled with the distribution.
    Bundled,
}

impl fmt::Display for ProfileSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfileSource::ExplicitPath(p) => write!(f, "path {}", p.display()),
            ProfileSource::CommandLineDirectory(p) => {
                write!(f, "command line file {}", p.display())
            }
            ProfileSource::UserDirectory(p) => write!(f, "user file {}", p.display()),
            ProfileSource::Bundled => write!(f, "bundled profile"),
        }
    }
}

/// A resolved profile and where it came from.
///
/// Returned together so that a caller cannot report a capture without being able
/// to say which file configured it.
#[derive(Debug)]
pub struct Resolved {
    /// The validated profile.
    pub profile: Profile,
    /// Which step supplied it.
    pub source: ProfileSource,
}

/// Why a reference could not be resolved.
#[derive(Debug)]
pub enum ResolveError {
    /// The reference is neither an existing file nor a valid slug, so it cannot
    /// be joined to a search directory.
    InvalidReference {
        /// The reference as given.
        reference: String,
    },
    /// Nothing was found, and here is everywhere that was looked.
    NotFound {
        /// The reference as given.
        reference: String,
        /// Every location consulted, in the order consulted.
        searched: Vec<PathBuf>,
    },
    /// A candidate was found and could not be used.
    Load {
        /// The file that won its step.
        path: PathBuf,
        /// Why it could not be used.
        source: LoadError,
    },
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveError::InvalidReference { reference } => write!(
                f,
                "`{reference}` is not an existing file and is not a valid profile id; \
                 an id uses lowercase letters, digits, hyphen, and underscore"
            ),
            ResolveError::NotFound {
                reference,
                searched,
            } => {
                write!(f, "no profile `{reference}` found; searched")?;
                if searched.is_empty() {
                    write!(f, " nowhere: no profile directories were given")
                } else {
                    for p in searched {
                        write!(f, "\n  {}", p.display())?;
                    }
                    Ok(())
                }
            }
            ResolveError::Load { path, source } => {
                write!(f, "profile {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for ResolveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ResolveError::Load { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Resolve a profile reference, per specification section 15.3.
///
/// # Errors
///
/// [`ResolveError::InvalidReference`] when the reference cannot be used in steps
/// two through four and is not an existing file, [`ResolveError::NotFound`] with
/// every location searched, and [`ResolveError::Load`] when a candidate won its
/// step and could not be used.
pub fn resolve(
    reference: &str,
    search: &SearchPath,
    bundled: &BundledSet,
) -> Result<Resolved, ResolveError> {
    // Step 1. An explicit path to an existing regular file. No slug check: the
    // operator named a file. A directory does not satisfy this step.
    let direct = Path::new(reference);
    if direct.is_file() {
        let profile = load(direct).map_err(|source| ResolveError::Load {
            path: direct.to_path_buf(),
            source,
        })?;
        return Ok(Resolved {
            profile,
            source: ProfileSource::ExplicitPath(direct.to_path_buf()),
        });
    }

    // Steps 2 through 4 interpolate the reference into a path or match it
    // against an identifier, so it must be a slug. Refused before any join.
    if !GameId::is_valid(reference) {
        return Err(ResolveError::InvalidReference {
            reference: reference.to_string(),
        });
    }

    let file_name = format!("{reference}.toml");
    let mut searched: Vec<PathBuf> = Vec::new();

    // Step 2, then step 3. Written as two loops rather than one over a chained
    // iterator, so that each step names its own `ProfileSource` at the point it
    // succeeds and neither needs a constructor threaded through a tuple.
    for dir in &search.command_line {
        if let Some((profile, path)) = try_directory(dir, &file_name, &mut searched)? {
            return Ok(Resolved {
                profile,
                source: ProfileSource::CommandLineDirectory(path),
            });
        }
    }
    if let Some(dir) = &search.user {
        if let Some((profile, path)) = try_directory(dir, &file_name, &mut searched)? {
            return Ok(Resolved {
                profile,
                source: ProfileSource::UserDirectory(path),
            });
        }
    }

    // Step 4. A bundled profile whose game.id matches.
    if let Some(profile) = bundled.get(reference) {
        return Ok(Resolved {
            profile: profile.clone(),
            source: ProfileSource::Bundled,
        });
    }

    Err(ResolveError::NotFound {
        reference: reference.to_string(),
        searched,
    })
}

/// Look for one candidate in one search directory.
///
/// A directory that is absent or unreadable is skipped, because a missing user
/// configuration directory is the ordinary state of a fresh install rather than
/// an error. A candidate that is present has won its step, so a failure to use it
/// is an error rather than a skip: falling through would silently select a
/// profile the operator did not choose.
fn try_directory(
    dir: &Path,
    file_name: &str,
    searched: &mut Vec<PathBuf>,
) -> Result<Option<(Profile, PathBuf)>, ResolveError> {
    if !dir.is_dir() {
        return Ok(None);
    }
    let candidate = dir.join(file_name);
    searched.push(candidate.clone());
    if !candidate.is_file() {
        return Ok(None);
    }
    let profile = load(&candidate).map_err(|source| ResolveError::Load {
        path: candidate.clone(),
        source,
    })?;
    Ok(Some((profile, candidate)))
}
