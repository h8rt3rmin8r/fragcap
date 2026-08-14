// SPDX-License-Identifier: Apache-2.0

//! The targets hint database (issue #78).
//!
//! An embedded SQLite store of known game binaries and launch patterns, and its
//! schema-conformant JSON export. The store is the provider at precedence 2 of
//! the resolution cascade (issue #77): every hint it emits is stamped
//! `heuristic-unverified`, never a source of truth, always overridable by a live
//! runtime observation (P-9).
//!
//! This slice (S034) builds the foundation: the [`Store`], the three-tier
//! seeding model (which of the catalog, launch, and engine tiers owns which
//! columns), and the [`export`] path. There is no network fetching here; the
//! store is populated offline through [`Store::upsert_game`] or [`import`], and
//! the seeders that fill it from the Steam Web API, PICS, and PCGamingWiki are
//! later slices.
//!
//! # The store cannot lie
//!
//! The store cannot hold a row it could not export. Enum sets, the engine
//! both-or-neither invariant, and the non-empty executable are enforced by
//! SQLite CHECK constraints and by the value types in [`model`], and the
//! [`export`] path validates its own output against the published schema before
//! returning it. A malformed [`import`] fails whole rather than writing a partial
//! store (P-4).

pub mod export;
pub mod import;
pub mod model;
pub mod schema;
pub mod store;

pub use export::export;
pub use import::{import, ImportSummary};
pub use model::{
    Engine, EngineConfidence, EngineSource, Game, LaunchEntry, SeedState, SeedTier, TechCategory,
    Technology,
};
pub use store::Store;

use std::fmt;

/// Everything that can go wrong in the targets store.
///
/// The variants map onto the CLI's exit contract: [`TargetsError::Model`] and
/// [`TargetsError::Seed`] are input problems (exit 1), [`TargetsError::Sqlite`]
/// and [`TargetsError::SchemaVersion`] are operational (exit 1), and
/// [`TargetsError::ExportInvalid`] is an internal-invariant failure surfaced
/// rather than emitted (exit 1). Usage errors (missing arguments) are the CLI's
/// own concern and never reach here (exit 2).
#[derive(Debug)]
pub enum TargetsError {
    /// A value violates the model's rules (empty executable, out-of-set engine
    /// value). Rejected before any write.
    Model(String),
    /// A seed document could not be parsed or carried an invalid structure
    /// (missing appid, duplicate appid within the document).
    Seed(String),
    /// A store I/O or SQL error.
    Sqlite(rusqlite::Error),
    /// The store file carries a schema version newer than this build understands.
    SchemaVersion {
        /// The version found on the file.
        found: i64,
    },
    /// The exporter produced a document its own schema validator rejected. An
    /// internal invariant failure: the exporter is wrong, and the document is
    /// surfaced as an error rather than emitted.
    ExportInvalid(String),
}

impl fmt::Display for TargetsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetsError::Model(m) => write!(f, "invalid target value: {m}"),
            TargetsError::Seed(m) => write!(f, "invalid seed: {m}"),
            TargetsError::Sqlite(e) => write!(f, "store error: {e}"),
            TargetsError::SchemaVersion { found } => write!(
                f,
                "store schema version {found} is newer than this build understands (version {})",
                schema::SCHEMA_VERSION
            ),
            TargetsError::ExportInvalid(m) => {
                write!(f, "export failed self-validation (internal error): {m}")
            }
        }
    }
}

impl std::error::Error for TargetsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TargetsError::Sqlite(e) => Some(e),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for TargetsError {
    fn from(e: rusqlite::Error) -> Self {
        TargetsError::Sqlite(e)
    }
}
