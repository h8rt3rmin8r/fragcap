// SPDX-License-Identifier: Apache-2.0

//! The value types the store exchanges.
//!
//! The types make an invalid hint unrepresentable where the schema makes it
//! invalid: a [`LaunchEntry`] cannot be built with an empty executable, and an
//! [`Engine`] cannot be built without both a source and a confidence. The store
//! then cannot hold a row it could not export, which is the storage-layer half
//! of P-9.

use crate::TargetsError;

/// Where an engine attribution came from. The closed set the schema's
/// `engine.source` enum defines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineSource {
    Pcgamingwiki,
    ExeHeuristic,
    DepotFilenameRules,
}

impl EngineSource {
    /// The schema token for this source.
    pub fn as_str(self) -> &'static str {
        match self {
            EngineSource::Pcgamingwiki => "pcgamingwiki",
            EngineSource::ExeHeuristic => "exe_heuristic",
            EngineSource::DepotFilenameRules => "depot_filename_rules",
        }
    }

    /// Parse a schema token, rejecting anything out of the set.
    pub fn parse(s: &str) -> Result<Self, TargetsError> {
        match s {
            "pcgamingwiki" => Ok(EngineSource::Pcgamingwiki),
            "exe_heuristic" => Ok(EngineSource::ExeHeuristic),
            "depot_filename_rules" => Ok(EngineSource::DepotFilenameRules),
            other => Err(TargetsError::Model(format!(
                "engine source out of set: {other:?}"
            ))),
        }
    }
}

/// Confidence in an engine attribution. A within-field grading of one heuristic
/// field, deliberately not a fidelity tier (P-9): a low confidence does not
/// lower the record's overall trust.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineConfidence {
    Confirmed,
    High,
    Medium,
    Low,
    Unknown,
}

impl EngineConfidence {
    /// The schema token for this confidence.
    pub fn as_str(self) -> &'static str {
        match self {
            EngineConfidence::Confirmed => "confirmed",
            EngineConfidence::High => "high",
            EngineConfidence::Medium => "medium",
            EngineConfidence::Low => "low",
            EngineConfidence::Unknown => "unknown",
        }
    }

    /// Parse a schema token, rejecting anything out of the set.
    pub fn parse(s: &str) -> Result<Self, TargetsError> {
        match s {
            "confirmed" => Ok(EngineConfidence::Confirmed),
            "high" => Ok(EngineConfidence::High),
            "medium" => Ok(EngineConfidence::Medium),
            "low" => Ok(EngineConfidence::Low),
            "unknown" => Ok(EngineConfidence::Unknown),
            other => Err(TargetsError::Model(format!(
                "engine confidence out of set: {other:?}"
            ))),
        }
    }
}

/// A technology category. The closed set the schema's `technology.category` enum
/// defines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TechCategory {
    Engine,
    AntiCheat,
    Sdk,
    Framework,
    Emulator,
    Container,
    Runtime,
    Launcher,
}

impl TechCategory {
    /// The schema token for this category.
    pub fn as_str(self) -> &'static str {
        match self {
            TechCategory::Engine => "engine",
            TechCategory::AntiCheat => "anti_cheat",
            TechCategory::Sdk => "sdk",
            TechCategory::Framework => "framework",
            TechCategory::Emulator => "emulator",
            TechCategory::Container => "container",
            TechCategory::Runtime => "runtime",
            TechCategory::Launcher => "launcher",
        }
    }

    /// Parse a schema token, rejecting anything out of the set.
    pub fn parse(s: &str) -> Result<Self, TargetsError> {
        match s {
            "engine" => Ok(TechCategory::Engine),
            "anti_cheat" => Ok(TechCategory::AntiCheat),
            "sdk" => Ok(TechCategory::Sdk),
            "framework" => Ok(TechCategory::Framework),
            "emulator" => Ok(TechCategory::Emulator),
            "container" => Ok(TechCategory::Container),
            "runtime" => Ok(TechCategory::Runtime),
            "launcher" => Ok(TechCategory::Launcher),
            other => Err(TargetsError::Model(format!(
                "technology category out of set: {other:?}"
            ))),
        }
    }
}

/// Which of the three seeding tiers a piece of state belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeedTier {
    /// The public catalog: appid and name.
    Catalog,
    /// The launch metadata: the launch array, launcher_mediated, token_required.
    Launch,
    /// The community engine data.
    Engine,
}

impl SeedTier {
    /// The stored token for this tier.
    pub fn as_str(self) -> &'static str {
        match self {
            SeedTier::Catalog => "catalog",
            SeedTier::Launch => "launch",
            SeedTier::Engine => "engine",
        }
    }

    /// Parse a stored token.
    pub fn parse(s: &str) -> Result<Self, TargetsError> {
        match s {
            "catalog" => Ok(SeedTier::Catalog),
            "launch" => Ok(SeedTier::Launch),
            "engine" => Ok(SeedTier::Engine),
            other => Err(TargetsError::Model(format!(
                "seed tier out of set: {other:?}"
            ))),
        }
    }
}

/// An engine attribution. Source and confidence are mandatory (the schema
/// requires both); the name is optional, absent when the lookup did not settle
/// on one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Engine {
    pub name: Option<String>,
    pub source: EngineSource,
    pub confidence: EngineConfidence,
}

/// One launch configuration, carried whole. The executable is required and
/// non-empty; every filter is an optional free string, as Steam declares them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchEntry {
    pub os: Option<String>,
    pub osarch: Option<String>,
    pub launch_type: Option<String>,
    pub beta_branch: Option<String>,
    executable: String,
    pub arguments: Option<String>,
    pub description: Option<String>,
}

impl LaunchEntry {
    /// Build a launch entry, rejecting an empty executable. The one required
    /// field is validated here so no invalid entry ever reaches the store or the
    /// export.
    pub fn new(executable: impl Into<String>) -> Result<Self, TargetsError> {
        let executable = executable.into();
        if executable.is_empty() {
            return Err(TargetsError::Model(
                "launch entry executable must not be empty".to_string(),
            ));
        }
        Ok(LaunchEntry {
            os: None,
            osarch: None,
            launch_type: None,
            beta_branch: None,
            executable,
            arguments: None,
            description: None,
        })
    }

    /// The invoked binary. Non-empty by construction.
    pub fn executable(&self) -> &str {
        &self.executable
    }
}

/// A detected technology present for a title.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Technology {
    pub category: TechCategory,
    pub name: String,
    pub marker_path: Option<String>,
}

/// One target title. Identity is a Steam application id. Every non-identity
/// field is optional, so a Tier-1-only row (appid and name) is a valid, if
/// sparse, game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    pub appid: u32,
    pub name: Option<String>,
    pub review_count: Option<u64>,
    pub owners: Option<u64>,
    pub peak_ccu: Option<u64>,
    pub launcher_mediated: Option<bool>,
    pub token_required: Option<bool>,
    pub engine: Option<Engine>,
    pub launch: Vec<LaunchEntry>,
    pub technologies: Vec<Technology>,
}

impl Game {
    /// A new game with only its identity set.
    pub fn new(appid: u32) -> Self {
        Game {
            appid,
            name: None,
            review_count: None,
            owners: None,
            peak_ccu: None,
            launcher_mediated: None,
            token_required: None,
            engine: None,
            launch: Vec::new(),
            technologies: Vec::new(),
        }
    }
}

/// Per-tier seeding progress. Structural this slice; no fetch writes it yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedState {
    pub tier: SeedTier,
    pub last_run_at: Option<String>,
    pub resume_cursor: Option<String>,
}
