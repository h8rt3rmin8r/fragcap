// SPDX-License-Identifier: Apache-2.0

//! Reading a profile, and every structural fault found doing it.
//!
//! Specification section 15.4 requires that validation report every problem
//! rather than stopping at the first, which rules out the shape this module
//! would otherwise have. Nothing on a diagnostic path uses `?`: a fault is
//! pushed, the field is left absent, and the walk continues. The private
//! [`Draft`] is what makes that possible, because it is the same shape as the
//! schema with every field optional, so a fault in one field does not prevent
//! checking another.
//!
//! Two places stop accumulation, and both are deliberate. A TOML syntax fault
//! yields one diagnostic and nothing else, because a document that did not parse
//! has no tables to check and recovering into a guess would report faults
//! against a file the author did not write. An unsupported schema version yields
//! one diagnostic and suppresses the semantic set, because every other fault is
//! then likely a consequence of reading a later schema under this one's rules.
//!
//! # A known divergence in the parser
//!
//! The parser this module uses does not implement TOML datetimes, which its own
//! documentation states. No key in schema version 1 has a datetime type, so a
//! datetime can appear only in a profile that is invalid anyway, and the effect
//! is on the message rather than the verdict: a syntax diagnostic instead of a
//! type diagnostic located at the key. Slice S05 research R-1 records the
//! measurement and why the alternative parser is unavailable at this workspace's
//! minimum toolchain. `datetime_is_a_syntax_fault_not_a_type_fault` pins the
//! behavior so the next reader finds a decision rather than a surprise.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use fragcap_core::duration;
use toml_span::value::{Table, Value};

use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics};
use crate::glob::ImagePattern;
use crate::schema::{
    CaptureDefaults, CaptureMode, Game, GameId, Lifecycle, MatchPredicates, PathRegex, Profile,
    Stage, SCHEMA_VERSION,
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
/// stages, and thousands squared is not a number to spend on a file that has
/// already been refused.
///
/// This reverses a decision the slice first wrote down. See the S05 decisions
/// changelog fragment for why the original reasoning did not hold.
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
    /// Parse and validate a profile from TOML text.
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

        let root = match toml_span::parse(text) {
            Ok(v) => v,
            Err(e) => {
                // The one place accumulation stops. A document that did not
                // parse has no tables to check.
                d.push(Diagnostic::at(
                    DiagnosticCode::Syntax,
                    "<document>",
                    text,
                    e.span.start,
                    e.to_string(),
                ));
                return Err(d.finish());
            }
        };

        let Some(table) = root.as_table() else {
            d.push(Diagnostic::at(
                DiagnosticCode::Syntax,
                "<document>",
                text,
                root.span.start,
                "profile must be a table",
            ));
            return Err(d.finish());
        };

        // The schema version gate comes first, before even the top level key
        // check. An unsupported version suppresses everything else, and a key
        // this build does not know is the most likely thing a later schema
        // added, so reporting it alongside the version fault would be reporting
        // a consequence of the fault as though it were a second problem.
        match table.get("schema") {
            None => d.push(Diagnostic::whole_file(
                DiagnosticCode::MissingField,
                "schema",
                format!("missing `schema`; this build supports version {SCHEMA_VERSION}"),
            )),
            Some(v) => match v.as_integer() {
                None => d.push(wrong_type("schema", "integer", v, text)),
                Some(n) if n == i64::from(SCHEMA_VERSION) => {}
                Some(n) => {
                    d.push(Diagnostic::at(
                        DiagnosticCode::UnsupportedSchema,
                        "schema",
                        text,
                        v.span.start,
                        format!("schema version {n} is not supported; this build supports {SCHEMA_VERSION}"),
                    ));
                    return Err(d.finish());
                }
            },
        }

        unknown_keys(table, Profile::ACCEPTED, "", text, &mut d);

        let draft = draft(table, text, &mut d);
        validate::check(&draft, text, &mut d);

        if !d.is_empty() {
            return Err(d.finish());
        }

        // Every check passed, so every field the schema requires is present and
        // every value is in its domain. The unwraps below cannot fire; each is
        // guarded by a diagnostic that would have returned above.
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
        Ok(Profile::new(
            Game::new(
                game.id.expect("game.id present after validation"),
                game.name.expect("game.name present after validation"),
                game.platform,
                game.app_id,
            ),
            draft.capture,
            stages,
        ))
    }
}

/// The profile as read, before the semantic checks, with every field optional.
///
/// Private on purpose. It exists so that a fault in one field does not stop
/// another being checked, which is what FR-013 requires, and it is not a shape
/// any caller should see.
pub(crate) struct Draft {
    pub(crate) game: DraftGame,
    pub(crate) capture: CaptureDefaults,
    pub(crate) capture_span: Option<usize>,
    pub(crate) roles_span: Option<usize>,
    /// How many entries `capture.roles` declared, whatever their types.
    ///
    /// Kept beside the surviving entries so that a list whose elements failed to
    /// parse is not mistaken for an empty list.
    pub(crate) roles_declared: Option<usize>,
    pub(crate) stages: Vec<DraftStage>,
    pub(crate) stage_key_span: Option<usize>,
}

pub(crate) struct DraftGame {
    pub(crate) id: Option<GameId>,
    pub(crate) name: Option<String>,
    pub(crate) platform: Option<String>,
    pub(crate) app_id: Option<String>,
}

pub(crate) struct DraftStage {
    pub(crate) index: usize,
    pub(crate) span: usize,
    pub(crate) role: Option<String>,
    pub(crate) role_span: Option<usize>,
    pub(crate) lifecycle: Option<Lifecycle>,
    pub(crate) lifecycle_span: Option<usize>,
    pub(crate) terminal: bool,
    pub(crate) terminal_span: Option<usize>,
    pub(crate) predicates: MatchPredicates,
    pub(crate) match_span: Option<usize>,
    pub(crate) exe_span: Option<usize>,
    pub(crate) descends_from_span: Option<usize>,
}

impl DraftStage {
    /// How this stage is named in a diagnostic location.
    ///
    /// By index rather than by role, because the role may be the thing that is
    /// missing or wrong, and a location that depends on the value it is
    /// reporting on is a location that disappears when it is most needed.
    pub(crate) fn loc(&self, suffix: &str) -> String {
        if suffix.is_empty() {
            format!("stage[{}]", self.index)
        } else {
            format!("stage[{}].{suffix}", self.index)
        }
    }
}

fn draft(table: &Table<'_>, text: &str, d: &mut Diagnostics) -> Draft {
    let mut out = Draft {
        game: DraftGame {
            id: None,
            name: None,
            platform: None,
            app_id: None,
        },
        capture: CaptureDefaults::default(),
        capture_span: None,
        roles_span: None,
        roles_declared: None,
        stages: Vec::new(),
        stage_key_span: None,
    };

    match table.get("game") {
        None => d.push(Diagnostic::whole_file(
            DiagnosticCode::MissingField,
            "game",
            "missing `[game]` table",
        )),
        Some(v) => match v.as_table() {
            None => d.push(wrong_type("game", "table", v, text)),
            Some(game) => read_game(game, text, d, &mut out.game),
        },
    }

    if let Some(v) = table.get("capture") {
        out.capture_span = Some(v.span.start);
        match v.as_table() {
            None => d.push(wrong_type("capture", "table", v, text)),
            Some(capture) => read_capture(capture, text, d, &mut out),
        }
    }

    match table.get("stage") {
        None => d.push(Diagnostic::whole_file(
            DiagnosticCode::NoStages,
            "stage",
            "profile declares no `[[stage]]`; at least one is required",
        )),
        Some(v) => {
            out.stage_key_span = Some(v.span.start);
            match v.as_array() {
                None => d.push(wrong_type("stage", "array of tables", v, text)),
                Some(items) if items.is_empty() => d.push(Diagnostic::at(
                    DiagnosticCode::NoStages,
                    "stage",
                    text,
                    v.span.start,
                    "profile declares no `[[stage]]`; at least one is required",
                )),
                Some(items) if items.len() > MAX_STAGES => d.push(Diagnostic::at(
                    DiagnosticCode::TooManyStages,
                    "stage",
                    text,
                    v.span.start,
                    format!(
                        "profile declares {} stages, above the {MAX_STAGES} stage \
                         limit; the ambiguity check of section 15.4 compares every \
                         pair of stages, so the pass is quadratic in this count",
                        items.len()
                    ),
                )),
                Some(items) => {
                    for (index, item) in items.iter().enumerate() {
                        let loc = format!("stage[{index}]");
                        match item.as_table() {
                            None => d.push(wrong_type(&loc, "table", item, text)),
                            Some(stage) => {
                                out.stages
                                    .push(read_stage(index, item.span.start, stage, text, d));
                            }
                        }
                    }
                }
            }
        }
    }

    out
}

fn read_game(table: &Table<'_>, text: &str, d: &mut Diagnostics, out: &mut DraftGame) {
    unknown_keys(table, Game::ACCEPTED, "game", text, d);

    match table.get("id") {
        None => d.push(Diagnostic::whole_file(
            DiagnosticCode::MissingField,
            "game.id",
            "missing `game.id`",
        )),
        Some(v) => {
            if let Some(s) = want_str("game.id", v, text, d) {
                match GameId::new(s) {
                    Some(id) => out.id = Some(id),
                    None => d.push(Diagnostic::at(
                        DiagnosticCode::InvalidSlug,
                        "game.id",
                        text,
                        v.span.start,
                        format!(
                            "`{s}` is not a valid id; use lowercase letters, digits, \
                             hyphen, and underscore"
                        ),
                    )),
                }
            }
        }
    }

    match table.get("name") {
        None => d.push(Diagnostic::whole_file(
            DiagnosticCode::MissingField,
            "game.name",
            "missing `game.name`",
        )),
        Some(v) => {
            if let Some(s) = want_str("game.name", v, text, d) {
                if s.is_empty() {
                    d.push(Diagnostic::at(
                        DiagnosticCode::MissingField,
                        "game.name",
                        text,
                        v.span.start,
                        "`game.name` is empty",
                    ));
                } else {
                    out.name = Some(s.to_string());
                }
            }
        }
    }

    if let Some(v) = table.get("platform") {
        out.platform = want_str("game.platform", v, text, d).map(str::to_string);
    }
    if let Some(v) = table.get("app_id") {
        out.app_id = want_str("game.app_id", v, text, d).map(str::to_string);
    }
}

fn read_capture(table: &Table<'_>, text: &str, d: &mut Diagnostics, out: &mut Draft) {
    unknown_keys(table, CaptureDefaults::ACCEPTED, "capture", text, d);

    if let Some(v) = table.get("mode") {
        if let Some(s) = want_str("capture.mode", v, text, d) {
            match CaptureMode::parse(s) {
                Some(m) => out.capture.set_mode(m),
                None => d.push(Diagnostic::at(
                    DiagnosticCode::InvalidMode,
                    "capture.mode",
                    text,
                    v.span.start,
                    format!(
                        "`{s}` is not a capture mode; expected one of {}",
                        CaptureMode::ACCEPTED.join(", ")
                    ),
                )),
            }
        }
    }

    if let Some(v) = table.get("duration") {
        if let Some(s) = want_str("capture.duration", v, text, d) {
            match duration::parse(s) {
                Ok(span) => out.capture.set_duration(span),
                Err(e) => d.push(Diagnostic::at(
                    DiagnosticCode::InvalidDuration,
                    "capture.duration",
                    text,
                    v.span.start,
                    e.to_string(),
                )),
            }
        }
    }

    if let Some(v) = table.get("roles") {
        out.roles_span = Some(v.span.start);
        match v.as_array() {
            None => d.push(wrong_type("capture.roles", "array of strings", v, text)),
            Some(items) => {
                // Every entry that parsed is kept, even when a sibling did not.
                // A list of `["ghost", 1]` carries two independent faults: the
                // second element's type, and the first naming a role no stage
                // declares. Discarding the list on the first fault would report
                // one and hide the other, which is what FR-013 forbids.
                let mut roles = Vec::with_capacity(items.len());
                for (i, item) in items.iter().enumerate() {
                    let loc = format!("capture.roles[{i}]");
                    if let Some(s) = want_str(&loc, item, text, d) {
                        roles.push(s.to_string());
                    }
                }
                // How many entries the author wrote, which is what decides
                // whether the list was empty. Using the surviving count would
                // report `["ghost", 1]` as an empty list, which it is not.
                out.roles_declared = Some(items.len());
                out.capture.set_roles(roles);
            }
        }
    }

    if let Some(v) = table.get("loopback") {
        if let Some(b) = want_bool("capture.loopback", v, text, d) {
            out.capture.set_loopback(b);
        }
    }
    if let Some(v) = table.get("payload") {
        if let Some(b) = want_bool("capture.payload", v, text, d) {
            out.capture.set_payload(b);
        }
    }
}

fn read_stage(
    index: usize,
    span: usize,
    table: &Table<'_>,
    text: &str,
    d: &mut Diagnostics,
) -> DraftStage {
    let mut out = DraftStage {
        index,
        span,
        role: None,
        role_span: None,
        lifecycle: None,
        lifecycle_span: None,
        terminal: false,
        terminal_span: None,
        predicates: MatchPredicates::default(),
        match_span: None,
        exe_span: None,
        descends_from_span: None,
    };
    let at = |suffix: &str| {
        if suffix.is_empty() {
            format!("stage[{index}]")
        } else {
            format!("stage[{index}].{suffix}")
        }
    };

    unknown_keys(table, Stage::ACCEPTED, &at(""), text, d);

    match table.get("role") {
        None => d.push(Diagnostic::at(
            DiagnosticCode::MissingField,
            at("role"),
            text,
            span,
            "missing `role`",
        )),
        Some(v) => {
            out.role_span = Some(v.span.start);
            if let Some(s) = want_str(&at("role"), v, text, d) {
                if s.is_empty() {
                    d.push(Diagnostic::at(
                        DiagnosticCode::MissingField,
                        at("role"),
                        text,
                        v.span.start,
                        "`role` is empty",
                    ));
                } else {
                    out.role = Some(s.to_string());
                }
            }
        }
    }

    match table.get("lifecycle") {
        None => d.push(Diagnostic::at(
            DiagnosticCode::MissingField,
            at("lifecycle"),
            text,
            span,
            "missing `lifecycle`",
        )),
        Some(v) => {
            out.lifecycle_span = Some(v.span.start);
            if let Some(s) = want_str(&at("lifecycle"), v, text, d) {
                match Lifecycle::parse(s) {
                    Some(l) => out.lifecycle = Some(l),
                    None => d.push(Diagnostic::at(
                        DiagnosticCode::InvalidLifecycle,
                        at("lifecycle"),
                        text,
                        v.span.start,
                        format!(
                            "`{s}` is not a lifecycle; expected one of {}",
                            Lifecycle::ACCEPTED.join(", ")
                        ),
                    )),
                }
            }
        }
    }

    if let Some(v) = table.get("terminal") {
        out.terminal_span = Some(v.span.start);
        if let Some(b) = want_bool(&at("terminal"), v, text, d) {
            out.terminal = b;
        }
    }

    match table.get("match") {
        None => d.push(Diagnostic::at(
            DiagnosticCode::MissingField,
            at("match"),
            text,
            span,
            "missing `match`",
        )),
        Some(v) => {
            out.match_span = Some(v.span.start);
            match v.as_table() {
                None => d.push(wrong_type(&at("match"), "table", v, text)),
                Some(m) => read_predicates(m, &at("match"), text, d, &mut out),
            }
        }
    }

    out
}

fn read_predicates(
    table: &Table<'_>,
    loc: &str,
    text: &str,
    d: &mut Diagnostics,
    out: &mut DraftStage,
) {
    unknown_keys(table, MatchPredicates::ACCEPTED, loc, text, d);

    if let Some(v) = table.get("exe") {
        out.exe_span = Some(v.span.start);
        let key = format!("{loc}.exe");
        if let Some(s) = want_str(&key, v, text, d) {
            match ImagePattern::new(s) {
                Ok(p) => out.predicates.set_exe(p),
                Err(e) => d.push(Diagnostic::at(
                    DiagnosticCode::InvalidGlob,
                    key,
                    text,
                    v.span.start,
                    e.to_string(),
                )),
            }
        }
    }

    if let Some(v) = table.get("path_contains") {
        let key = format!("{loc}.path_contains");
        if let Some(s) = want_str(&key, v, text, d) {
            out.predicates.set_path_contains(s.to_string());
        }
    }

    if let Some(v) = table.get("path_regex") {
        let key = format!("{loc}.path_regex");
        if let Some(s) = want_str(&key, v, text, d) {
            match PathRegex::new(s) {
                Ok(r) => out.predicates.set_path_regex(r),
                Err(e) => d.push(Diagnostic::at(
                    DiagnosticCode::InvalidRegex,
                    key,
                    text,
                    v.span.start,
                    // The engine's own message, including its compiled size
                    // limit. fragcap forms no second opinion about which
                    // patterns are too large.
                    e.to_string(),
                )),
            }
        }
    }

    if let Some(v) = table.get("cmdline_contains") {
        let key = format!("{loc}.cmdline_contains");
        if let Some(s) = want_str(&key, v, text, d) {
            out.predicates.set_cmdline_contains(s.to_string());
        }
    }

    if let Some(v) = table.get("descends_from") {
        out.descends_from_span = Some(v.span.start);
        let key = format!("{loc}.descends_from");
        if let Some(s) = want_str(&key, v, text, d) {
            out.predicates.set_descends_from(s.to_string());
        }
    }
}

/// Report every key in a table that is not in the accepted set.
///
/// Rejecting rather than ignoring, because ignoring is the silent failure: an
/// author who writes `payloads = false` intending `payload = false` gets a
/// capture containing contents they meant to exclude, and nothing says so.
fn unknown_keys(
    table: &Table<'_>,
    accepted: &[&str],
    prefix: &str,
    text: &str,
    d: &mut Diagnostics,
) {
    for key in table.keys() {
        let name = key.name.as_ref();
        if accepted.contains(&name) {
            continue;
        }
        let loc = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{prefix}.{name}")
        };
        d.push(Diagnostic::at(
            DiagnosticCode::UnknownKey,
            loc,
            text,
            key.span.start,
            format!(
                "unknown key `{name}`; accepted here: {}",
                accepted.join(", ")
            ),
        ));
    }
}

/// The TOML type of a value, for a diagnostic that has to name what it found.
fn type_name(v: &Value<'_>) -> &'static str {
    if v.as_table().is_some() {
        "table"
    } else if v.as_array().is_some() {
        "array"
    } else if v.as_str().is_some() {
        "string"
    } else if v.as_bool().is_some() {
        "boolean"
    } else if v.as_integer().is_some() {
        "integer"
    } else if v.as_float().is_some() {
        "float"
    } else {
        "value"
    }
}

fn wrong_type(loc: &str, expected: &str, v: &Value<'_>, text: &str) -> Diagnostic {
    Diagnostic::at(
        DiagnosticCode::WrongType,
        loc,
        text,
        v.span.start,
        format!("expected {expected}, found {}", type_name(v)),
    )
}

fn want_str<'a>(loc: &str, v: &'a Value<'a>, text: &str, d: &mut Diagnostics) -> Option<&'a str> {
    match v.as_str() {
        Some(s) => Some(s),
        None => {
            d.push(wrong_type(loc, "string", v, text));
            None
        }
    }
}

fn want_bool(loc: &str, v: &Value<'_>, text: &str, d: &mut Diagnostics) -> Option<bool> {
    match v.as_bool() {
        Some(b) => Some(b),
        None => {
            d.push(wrong_type(loc, "boolean", v, text));
            None
        }
    }
}
