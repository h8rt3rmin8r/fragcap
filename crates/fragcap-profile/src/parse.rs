// SPDX-License-Identifier: Apache-2.0

//! Reading a profile from JSON, and every fault found doing it.
//!
//! A profile is a JSON document conforming to the master schema's `profile`
//! variant (issue #76 moved the format here from TOML). Specification section
//! 15.4 requires that validation report every problem rather than stopping at
//! the first, so nothing on a diagnostic path uses `?`: a fault is pushed and
//! the walk continues.
//!
//! # Two validation layers, one report
//!
//! Loading runs two non-overlapping layers that both accumulate into one
//! [`Diagnostics`]:
//!
//! - **Structural**, delegated to [`crate::jsonschema::validate_value`] (the
//!   S025 validator), which owns types, required keys, enum ranges, unknown-key
//!   refusal, and the `schema` and `kind` discriminators. There is one
//!   structural implementation, bound to the published schema, so the profile a
//!   run loads is structurally identical to what the schema accepts.
//! - **fragcap-specific**, a lenient pass here that extracts an all-optional
//!   [`Draft`] from the parsed value and runs only what a schema cannot express:
//!   compiling the `exe` glob, the `path_regex`, and the `capture.duration`, the
//!   stage-count limit, and the semantic graph checks in [`crate::validate`].
//!
//! The two responsibility sets do not overlap, so nothing is reported twice.
//!
//! Two places stop accumulation. A JSON syntax fault yields one diagnostic and
//! nothing else, because a document that did not parse has no structure to
//! check. An unsupported `schema` version suppresses the rest, because every
//! other fault is then likely a consequence of reading a later schema under this
//! one's rules.
//!
//! # Locations
//!
//! Diagnostics locate faults by JSON pointer (for example `/stage/1/match/exe`).
//! serde_json exposes no per-value byte span, so a line and column is not
//! available; the pointer names the exact value instead. This is the tradeoff
//! recorded in the S026 research.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use serde_json::{Map, Value};

use fragcap_core::duration;

use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics};
use crate::glob::ImagePattern;
use crate::jsonschema::{self, SchemaCode, SchemaDiagnostic};
use crate::schema::{
    CaptureDefaults, CaptureMode, FidelityTier, Game, GameId, Kind, Lifecycle, MatchPredicates,
    PathRegex, Profile, Provenance, Stage,
};
use crate::validate;

/// The largest profile this crate will read.
///
/// A profile is tens of lines, so no legitimate one approaches this. The limit
/// exists because reading an arbitrary quantity of bytes because they were in the
/// right directory is a fault with no upside, and it is checked against the
/// file's metadata before the contents are read rather than after.
pub const MAX_PROFILE_BYTES: u64 = 1024 * 1024;

/// The most stages one profile may declare.
///
/// A launcher chain is a handful of processes: the focal titles of specification
/// section 5.4 declare two and three. This is two orders of magnitude beyond any
/// plausible topology and exists for a mechanical reason rather than a modelling
/// one. The ambiguity check of section 15.4 compares every unordered pair of
/// stages, so the pass is quadratic in this count, and [`MAX_PROFILE_BYTES`]
/// alone does not bound it: a one mebibyte profile can declare thousands of
/// stages. The master schema does not express this fragcap-specific limit, so it
/// is checked here.
pub const MAX_STAGES: usize = 64;

/// Why a profile could not be loaded from a path.
#[derive(Debug)]
pub enum LoadError {
    /// The file could not be read: absent, not a regular file, or permission
    /// denied.
    Read(io::Error),
    /// The file was read and is not an acceptable profile. Carries every
    /// problem found, including the size refusal, which is
    /// [`DiagnosticCode::FileTooLarge`].
    Invalid(Diagnostics),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Read(e) => write!(f, "cannot read profile: {e}"),
            LoadError::Invalid(d) => write!(f, "{d}"),
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadError::Read(e) => Some(e),
            LoadError::Invalid(d) => Some(d),
        }
    }
}

/// Read and validate a profile from a path.
///
/// The size limit is applied to the file's metadata before the contents are
/// read, so an enormous file is refused without being loaded.
///
/// # Errors
///
/// [`LoadError::Read`] if the path cannot be read or is not a regular file, and
/// [`LoadError::Invalid`] if the contents are not an acceptable profile.
pub fn load(path: &Path) -> Result<Profile, LoadError> {
    let meta = fs::metadata(path).map_err(LoadError::Read)?;
    if !meta.is_file() {
        return Err(LoadError::Read(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not a regular file",
        )));
    }
    if meta.len() > MAX_PROFILE_BYTES {
        let mut d = Diagnostics::new();
        d.push(Diagnostic::whole_file(
            DiagnosticCode::FileTooLarge,
            "<file>",
            format!(
                "profile is {} bytes, above the {MAX_PROFILE_BYTES} byte limit",
                meta.len()
            ),
        ));
        return Err(LoadError::Invalid(d.finish()));
    }
    let text = fs::read_to_string(path).map_err(LoadError::Read)?;
    Profile::parse(&text).map_err(LoadError::Invalid)
}

impl Profile {
    /// Parse and validate a profile from JSON text.
    ///
    /// The only way to obtain a [`Profile`]. Section 15.4's requirement that
    /// validation run before every capture therefore costs nothing to honor and
    /// cannot be forgotten by a later caller.
    ///
    /// # Errors
    ///
    /// Every problem found, never only the first, and never empty.
    pub fn parse(text: &str) -> Result<Profile, Diagnostics> {
        let mut d = Diagnostics::new();

        let value: Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => {
                // The one place syntax accumulation stops: a document that did
                // not parse has no structure to check.
                d.push(Diagnostic::located(
                    DiagnosticCode::Syntax,
                    "",
                    format!("not valid JSON: {e}"),
                ));
                return Err(d.finish());
            }
        };

        // Structural layer: the S025 validator, mapped into this crate's
        // diagnostics. An unsupported schema version suppresses everything else,
        // because every other fault is then likely a consequence of reading a
        // later schema under this build's rules, so only that one is reported.
        let structural = jsonschema::validate_value(&value);
        if let Some(version) = structural
            .iter()
            .find(|sd| sd.code == SchemaCode::UnsupportedSchema)
        {
            d.push(map_schema_diagnostic(version));
            return Err(d.finish());
        }
        for sd in structural.iter() {
            d.push(map_schema_diagnostic(sd));
        }

        // The profile-load path accepts only the strict, authoritative kinds. A
        // loose hint or export is a structurally valid artifact but not a capture
        // profile, so it is refused here. A missing or unrecognized kind is
        // already reported by the structural layer, so it is not repeated.
        match value.get("kind").and_then(Value::as_str) {
            Some("hint") | Some("export") => d.push(Diagnostic::located(
                DiagnosticCode::WrongType,
                "/kind",
                "a hint or export cannot be loaded as a capture profile; kind must be `profile`",
            )),
            _ => {}
        }

        // A capture profile may not declare `fidelity: observed`. The observed
        // tier is a runtime result the observation provider stamps, not a trust
        // level an author claims; allowing it on a profile would let the
        // top-precedence provider return an answer below the fidelity of a
        // lower-precedence one, inverting the section 15.7 rank. The structural
        // layer accepts `observed` on the shared enum, so this semantic
        // constraint is checked here.
        if value.get("fidelity").and_then(Value::as_str) == Some("observed") {
            d.push(Diagnostic::located(
                DiagnosticCode::ObservedProfileFidelity,
                "/fidelity",
                "fidelity `observed` is a runtime result and cannot be declared on a \
                 capture profile; use `authored`, `verified`, or `heuristic-unverified`",
            ));
        }

        // fragcap-specific layer: extract leniently, compile, semantic-check.
        let draft = draft_from_value(&value, &mut d);
        validate::check(&draft, &mut d);

        if !d.is_empty() {
            return Err(d.finish());
        }

        // Every check passed, so every field the schema requires is present and
        // in its domain. The expects below cannot fire; each is guarded by a
        // diagnostic that would have returned above.
        let game = draft.game;
        let stages = draft
            .stages
            .into_iter()
            .map(|s| {
                Stage::new(
                    s.role.expect("role present after validation"),
                    s.lifecycle.expect("lifecycle present after validation"),
                    s.terminal,
                    s.predicates,
                )
            })
            .collect();

        // The section 15.6 metadata. `kind` and `fidelity` are required by the
        // schema, so validation passing means both are present and in their
        // domain; `provenance` and `notes` are optional and read as declared.
        let kind = value
            .get("kind")
            .and_then(Value::as_str)
            .and_then(Kind::parse)
            .expect("kind present and valid after validation");
        let fidelity = value
            .get("fidelity")
            .and_then(Value::as_str)
            .and_then(FidelityTier::parse)
            .expect("fidelity present and valid after validation");
        let provenance = read_provenance(&value);
        let notes = value
            .get("notes")
            .and_then(Value::as_str)
            .map(str::to_string);

        Ok(Profile::new(
            Game::new(
                game.id.expect("game.id present after validation"),
                game.name.expect("game.name present after validation"),
                game.platform,
                game.app_id,
            ),
            draft.capture,
            stages,
            kind,
            fidelity,
            provenance,
            notes,
        ))
    }
}

/// Read the optional top-level `provenance` object.
///
/// Structural validity is the schema's job: when `provenance` is present it
/// requires a non-empty `source`, so a present object always yields a source.
/// `seeded_at` is optional.
fn read_provenance(value: &Value) -> Option<Provenance> {
    let obj = value.get("provenance")?.as_object()?;
    let source = obj.get("source").and_then(Value::as_str)?.to_string();
    let seeded_at = obj
        .get("seeded_at")
        .and_then(Value::as_str)
        .map(str::to_string);
    Some(Provenance::new(source, seeded_at))
}

/// Map one structural diagnostic from the schema validator into this crate's
/// diagnostic vocabulary.
///
/// The schema's codes overlap this crate's; a few schema-specific distinctions
/// (a missing or wrong `kind` or `fidelity`) map onto the nearest existing code,
/// and the schema's precise message is carried through unchanged.
fn map_schema_diagnostic(sd: &SchemaDiagnostic) -> Diagnostic {
    let code = match sd.code {
        SchemaCode::NotAnObject => DiagnosticCode::WrongType,
        SchemaCode::UnsupportedSchema => DiagnosticCode::UnsupportedSchema,
        SchemaCode::MissingKind => DiagnosticCode::MissingField,
        SchemaCode::UnknownKind => DiagnosticCode::WrongType,
        SchemaCode::MissingFidelity => DiagnosticCode::MissingField,
        SchemaCode::InvalidFidelity => DiagnosticCode::WrongType,
        SchemaCode::MissingField => DiagnosticCode::MissingField,
        SchemaCode::MissingProvenance => DiagnosticCode::MissingField,
        SchemaCode::WrongType => DiagnosticCode::WrongType,
        SchemaCode::UnknownKey => DiagnosticCode::UnknownKey,
        SchemaCode::InvalidSlug => DiagnosticCode::InvalidSlug,
        SchemaCode::InvalidLifecycle => DiagnosticCode::InvalidLifecycle,
        SchemaCode::InvalidMode => DiagnosticCode::InvalidMode,
        SchemaCode::EmptyMatch => DiagnosticCode::EmptyMatch,
        SchemaCode::EmptyStages => DiagnosticCode::NoStages,
        SchemaCode::EmptyString => DiagnosticCode::MissingField,
        SchemaCode::InvalidCategory => DiagnosticCode::WrongType,
    };
    // Keep the pointer exactly as the validator reported it, including the empty
    // string for a root-level fault: the profile-load contract locates faults by
    // JSON pointer, and a consumer must be able to apply it as one. The
    // syntax-error path uses the same empty pointer for a root fault.
    Diagnostic::located(code, sd.pointer.clone(), sd.message.clone())
}

/// The profile as read, before the semantic checks, with every field optional.
///
/// Private on purpose. It exists so that a fault in one field does not stop
/// another being checked, and it is not a shape any caller should see.
pub(crate) struct Draft {
    pub(crate) game: DraftGame,
    pub(crate) capture: CaptureDefaults,
    /// How many entries `capture.roles` declared, whatever their types. Kept so
    /// a list whose elements failed to parse is not mistaken for an empty list.
    pub(crate) roles_declared: Option<usize>,
    pub(crate) stages: Vec<DraftStage>,
}

pub(crate) struct DraftGame {
    pub(crate) id: Option<GameId>,
    pub(crate) name: Option<String>,
    pub(crate) platform: Option<String>,
    pub(crate) app_id: Option<String>,
}

pub(crate) struct DraftStage {
    pub(crate) index: usize,
    pub(crate) role: Option<String>,
    pub(crate) lifecycle: Option<Lifecycle>,
    pub(crate) terminal: bool,
    pub(crate) predicates: MatchPredicates,
}

impl DraftStage {
    /// How this stage is named in a diagnostic location, as a JSON pointer.
    ///
    /// By index rather than by role, because the role may be the thing that is
    /// missing or wrong, and a location that depends on the value it is
    /// reporting on is a location that disappears when it is most needed.
    pub(crate) fn loc(&self, suffix: &str) -> String {
        if suffix.is_empty() {
            format!("/stage/{}", self.index)
        } else {
            let path = suffix.replace('.', "/");
            format!("/stage/{}/{path}", self.index)
        }
    }
}

fn draft_from_value(value: &Value, d: &mut Diagnostics) -> Draft {
    let mut out = Draft {
        game: DraftGame {
            id: None,
            name: None,
            platform: None,
            app_id: None,
        },
        capture: CaptureDefaults::default(),
        roles_declared: None,
        stages: Vec::new(),
    };

    let Some(obj) = value.as_object() else {
        // A non-object root is already a structural fault; nothing to extract.
        return out;
    };

    if let Some(game) = obj.get("game").and_then(Value::as_object) {
        read_game(game, &mut out.game);
    }
    if let Some(capture) = obj.get("capture").and_then(Value::as_object) {
        read_capture(capture, d, &mut out);
    }
    if let Some(items) = obj.get("stage").and_then(Value::as_array) {
        if items.len() > MAX_STAGES {
            d.push(Diagnostic::located(
                DiagnosticCode::TooManyStages,
                "/stage",
                format!(
                    "profile declares {} stages, above the {MAX_STAGES} stage limit; \
                     the ambiguity check of section 15.4 compares every pair of stages, \
                     so the pass is quadratic in this count",
                    items.len()
                ),
            ));
            // Do not populate the semantic draft past the limit. The ambiguity
            // check is quadratic in the stage count, and the profile is already
            // refused; extracting thousands of stages only to compare every pair
            // spends work on a file that has been rejected.
        } else {
            for (index, item) in items.iter().enumerate() {
                if let Some(stage) = item.as_object() {
                    out.stages.push(read_stage(index, stage, d));
                }
            }
        }
    }

    out
}

fn read_game(table: &Map<String, Value>, out: &mut DraftGame) {
    // Structural validity (types, required keys, the slug pattern) is the
    // schema's job; here we only extract. A field the schema flagged is left
    // absent, and the schema's diagnostic stands alone.
    if let Some(s) = table.get("id").and_then(Value::as_str) {
        out.id = GameId::new(s);
    }
    if let Some(s) = table.get("name").and_then(Value::as_str) {
        if !s.is_empty() {
            out.name = Some(s.to_string());
        }
    }
    out.platform = table
        .get("platform")
        .and_then(Value::as_str)
        .map(str::to_string);
    out.app_id = table
        .get("app_id")
        .and_then(Value::as_str)
        .map(str::to_string);
}

fn read_capture(table: &Map<String, Value>, d: &mut Diagnostics, out: &mut Draft) {
    if let Some(s) = table.get("mode").and_then(Value::as_str) {
        if let Some(m) = CaptureMode::parse(s) {
            out.capture.set_mode(m);
        }
    }
    if let Some(s) = table.get("duration").and_then(Value::as_str) {
        match duration::parse(s) {
            Ok(span) => out.capture.set_duration(span),
            Err(e) => d.push(Diagnostic::located(
                DiagnosticCode::InvalidDuration,
                "/capture/duration",
                e.to_string(),
            )),
        }
    }
    if let Some(items) = table.get("roles").and_then(Value::as_array) {
        let mut roles = Vec::with_capacity(items.len());
        for item in items {
            if let Some(s) = item.as_str() {
                roles.push(s.to_string());
            }
        }
        // The declared count decides emptiness; the surviving count would report
        // `["ghost", 1]` as empty, which it is not.
        out.roles_declared = Some(items.len());
        out.capture.set_roles(roles);
    }
    if let Some(b) = table.get("loopback").and_then(Value::as_bool) {
        out.capture.set_loopback(b);
    }
    if let Some(b) = table.get("payload").and_then(Value::as_bool) {
        out.capture.set_payload(b);
    }
}

fn read_stage(index: usize, table: &Map<String, Value>, d: &mut Diagnostics) -> DraftStage {
    let mut out = DraftStage {
        index,
        role: None,
        lifecycle: None,
        terminal: false,
        predicates: MatchPredicates::default(),
    };

    if let Some(s) = table.get("role").and_then(Value::as_str) {
        if !s.is_empty() {
            out.role = Some(s.to_string());
        }
    }
    if let Some(s) = table.get("lifecycle").and_then(Value::as_str) {
        out.lifecycle = Lifecycle::parse(s);
    }
    if let Some(b) = table.get("terminal").and_then(Value::as_bool) {
        out.terminal = b;
    }
    if let Some(m) = table.get("match").and_then(Value::as_object) {
        read_predicates(index, m, d, &mut out);
    }

    out
}

fn read_predicates(
    index: usize,
    table: &Map<String, Value>,
    d: &mut Diagnostics,
    out: &mut DraftStage,
) {
    let loc = |key: &str| format!("/stage/{index}/match/{key}");

    if let Some(s) = table.get("exe").and_then(Value::as_str) {
        match ImagePattern::new(s) {
            Ok(p) => out.predicates.set_exe(p),
            Err(e) => d.push(Diagnostic::located(
                DiagnosticCode::InvalidGlob,
                loc("exe"),
                e.to_string(),
            )),
        }
    }
    if let Some(s) = table.get("path_contains").and_then(Value::as_str) {
        out.predicates.set_path_contains(s.to_string());
    }
    if let Some(s) = table.get("path_regex").and_then(Value::as_str) {
        match PathRegex::new(s) {
            Ok(r) => out.predicates.set_path_regex(r),
            // The engine's own message, including its compiled size limit.
            Err(e) => d.push(Diagnostic::located(
                DiagnosticCode::InvalidRegex,
                loc("path_regex"),
                e.to_string(),
            )),
        }
    }
    if let Some(s) = table.get("cmdline_contains").and_then(Value::as_str) {
        out.predicates.set_cmdline_contains(s.to_string());
    }
    if let Some(s) = table.get("descends_from").and_then(Value::as_str) {
        out.predicates.set_descends_from(s.to_string());
    }
}

#[cfg(test)]
mod tests {
    use crate::diagnostic::DiagnosticCode;
    use crate::schema::{FidelityTier, Kind, Profile};

    fn parse(body: &str) -> Profile {
        Profile::parse(body).unwrap_or_else(|d| {
            panic!(
                "profile did not validate: {:?}",
                d.iter().map(|x| x.message.clone()).collect::<Vec<_>>()
            )
        })
    }

    #[test]
    fn a_loaded_profile_exposes_its_kind_and_fidelity() {
        let p = parse(
            r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"eso","name":"ESO"},"stage":[{"role":"client","lifecycle":"session","match":{"exe":"eso64.exe"}}]}"#,
        );
        assert_eq!(p.kind(), Kind::Profile);
        assert_eq!(p.fidelity(), FidelityTier::Verified);
        assert!(p.provenance().is_none(), "no provenance was declared");
        assert!(p.notes().is_none(), "no notes were declared");
        // The pre-existing fields are unchanged by the metadata surfacing.
        assert_eq!(p.game().id().as_str(), "eso");
        assert_eq!(p.stages().len(), 1);
    }

    #[test]
    fn a_loaded_profile_exposes_declared_provenance_and_notes() {
        let p = parse(
            r#"{"schema":1,"kind":"profile","fidelity":"heuristic-unverified","notes":"verify against a live capture","provenance":{"source":"steam-appinfo","seeded_at":"2026-08-12"},"game":{"id":"eso","name":"ESO"},"stage":[{"role":"client","lifecycle":"session","match":{"exe":"eso64.exe"}}]}"#,
        );
        assert_eq!(p.fidelity(), FidelityTier::HeuristicUnverified);
        assert_eq!(p.notes(), Some("verify against a live capture"));
        let prov = p.provenance().expect("provenance declared");
        assert_eq!(prov.source(), "steam-appinfo");
        assert_eq!(prov.seeded_at(), Some("2026-08-12"));
    }

    #[test]
    fn a_package_loads_as_a_capture_profile() {
        let p = parse(
            r#"{"schema":1,"kind":"package","fidelity":"authored","game":{"id":"eso","name":"ESO"},"stage":[{"role":"client","lifecycle":"session","match":{"exe":"eso64.exe"}}]}"#,
        );
        assert_eq!(p.kind(), Kind::Package);
        assert_eq!(p.fidelity(), FidelityTier::Authored);
    }

    #[test]
    fn a_hint_is_refused_as_a_capture_profile() {
        // A structurally valid hint is not a capture profile; parse refuses it.
        let err = Profile::parse(
            r#"{"schema":1,"kind":"hint","fidelity":"heuristic-unverified","provenance":{"source":"steam-appinfo"},"game":{"id":"eso","name":"ESO"},"stage":[{"role":"client","lifecycle":"session","match":{"exe":"eso64.exe"}}]}"#,
        )
        .expect_err("a hint is not a capture profile");
        assert!(
            err.iter()
                .any(|d| d.message.contains("kind must be `profile`")),
            "the refusal names the kind requirement"
        );
    }

    #[test]
    fn a_profile_declaring_observed_fidelity_is_refused() {
        // `observed` is the observation provider's runtime stamp, not an authored
        // trust level. Allowing it on a profile would let the top-precedence
        // provider answer below a lower one's fidelity, inverting the rank.
        let err = Profile::parse(
            r#"{"schema":1,"kind":"profile","fidelity":"observed","game":{"id":"eso","name":"ESO"},"stage":[{"role":"client","lifecycle":"session","match":{"exe":"eso64.exe"}}]}"#,
        )
        .expect_err("an observed capture profile is refused");
        assert!(
            err.iter()
                .any(|d| d.code == DiagnosticCode::ObservedProfileFidelity),
            "the refusal carries the observed-profile-fidelity code"
        );
    }

    #[test]
    fn targeting_fidelity_is_distinct_from_attribution_fidelity() {
        // FR-010: the targeting FidelityTier and the attribution Fidelity are
        // different types on different axes. The compiler enforces the
        // separation: a `targeting == attribution` comparison does not compile.
        // Observed targeting is not the same as a live attribution.
        use fragcap_core::attribution::Fidelity as AttributionFidelity;
        let targeting = FidelityTier::Observed;
        let attribution = AttributionFidelity::Live;
        assert_eq!(targeting.as_str(), "observed");
        assert_eq!(format!("{attribution:?}"), "Live");
    }
}
