// SPDX-License-Identifier: Apache-2.0

//! The profile schema of specification section 15.2.
//!
//! Every type here is constructible only through [`crate::Profile::parse`].
//! There is no public constructor, no public field, and no `Default`, because a
//! default profile would be a profile that passed no check. Section 15.4
//! requires that validation run before every capture, and making the type
//! unobtainable otherwise is what turns that requirement into something a later
//! caller cannot forget.
//!
//! Optional values are held as declared-or-absent rather than as defaults
//! already applied. The distinction matters to the command line: a profile that
//! chose `payload = true` and a profile that said nothing are different inputs
//! to an override, and collapsing them here would destroy information the
//! operator supplied.

use std::fmt;
use std::time::Duration;

use regex::Regex;

use crate::glob::ImagePattern;

/// The only schema version this build supports.
///
/// A profile declaring anything else is refused with a version diagnostic rather
/// than read under these rules. That refusal is what makes strict key checking
/// safe: a profile written for a later fragcap says so, and gets told so.
pub const SCHEMA_VERSION: u32 = 1;

/// A game identifier: the slug section 15.2 requires.
///
/// A newtype rather than a validated `String` because section 15.3 joins it to a
/// directory during resolution, which makes it a filename component. A type is
/// what stops a later slice from interpolating an unvalidated reference into a
/// path. The character set is lowercase ASCII alphanumerics, hyphen, and
/// underscore, which excludes a path separator, a parent reference, and a drive
/// prefix by construction.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GameId(String);

impl GameId {
    /// Whether a string is a valid slug.
    ///
    /// Public because resolution has to answer the same question about a
    /// reference that never passed through validation, and two implementations
    /// of one rule is how the two places drift apart.
    pub fn is_valid(s: &str) -> bool {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    }

    /// Build an identifier, or report that the string is not a slug.
    pub fn new(s: &str) -> Option<GameId> {
        if GameId::is_valid(s) {
            Some(GameId(s.to_string()))
        } else {
            None
        }
    }

    /// The identifier as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// How fragcap treats a process on exit, per specification section 10.4.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    /// Expected to exit during the session. Its exit is normal and does not
    /// affect capture.
    Transient,
    /// Expected to live for the session. Its exit is a significant event, and
    /// if the stage is terminal its exit ends the capture.
    Session,
    /// Expected to outlive the session and may have been running before it
    /// began. Never awaited during acquisition, because waiting for something
    /// already running deadlocks.
    Service,
}

impl Lifecycle {
    /// The accepted spellings, for a diagnostic that has to list them.
    pub const ACCEPTED: &'static [&'static str] = &["transient", "session", "service"];

    /// Parse the TOML spelling.
    pub fn parse(s: &str) -> Option<Lifecycle> {
        match s {
            "transient" => Some(Lifecycle::Transient),
            "session" => Some(Lifecycle::Session),
            "service" => Some(Lifecycle::Service),
            _ => None,
        }
    }
}

/// A capture mode, per specification section 17.2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureMode {
    /// Bounded capture written to a file.
    File,
    /// Live streaming to a sink transport.
    Stream,
    /// Rolling in-memory window written on trigger.
    Ring,
}

impl CaptureMode {
    /// The accepted spellings, for a diagnostic that has to list them.
    pub const ACCEPTED: &'static [&'static str] = &["file", "stream", "ring"];

    /// Parse the TOML spelling.
    pub fn parse(s: &str) -> Option<CaptureMode> {
        match s {
            "file" => Some(CaptureMode::File),
            "stream" => Some(CaptureMode::Stream),
            "ring" => Some(CaptureMode::Ring),
            _ => None,
        }
    }
}

/// A compiled `path_regex` predicate and the text it came from.
///
/// Both, because the compile is the validation and discarding the result would
/// mean S12 compiles again: the same engine on the same input, so no divergence,
/// but work done twice for no reason.
///
/// Equality and formatting are defined on the source text. A compiled automaton
/// has no useful notion of either, and a derived `PartialEq` would not compile.
#[derive(Clone, Debug)]
pub struct PathRegex {
    source: String,
    compiled: Regex,
}

impl PathRegex {
    /// Compile a pattern.
    ///
    /// # Errors
    ///
    /// The engine's own error, unaltered. That includes the compiled size limit,
    /// which is how a pathological pattern is refused: fragcap forms no second
    /// opinion about which patterns are too large, because a second opinion is a
    /// thing to keep in step with the first.
    pub fn new(source: &str) -> Result<PathRegex, regex::Error> {
        Ok(PathRegex {
            compiled: Regex::new(source)?,
            source: source.to_string(),
        })
    }

    /// The pattern as the author wrote it.
    pub fn as_str(&self) -> &str {
        &self.source
    }

    /// The compiled expression, for the slice that evaluates it.
    pub fn regex(&self) -> &Regex {
        &self.compiled
    }
}

impl PartialEq for PathRegex {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
    }
}

impl Eq for PathRegex {}

impl fmt::Display for PathRegex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.source)
    }
}

/// The match predicates of specification section 10.3.
///
/// All specified predicates must hold for a stage to bind. Nothing here
/// evaluates one; S12 does. At least one must be present, because an empty
/// predicate set matches every process on the system.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MatchPredicates {
    exe: Option<ImagePattern>,
    path_contains: Option<String>,
    path_regex: Option<PathRegex>,
    cmdline_contains: Option<String>,
    descends_from: Option<String>,
}

impl MatchPredicates {
    /// The accepted key names, for a diagnostic that has to list them.
    pub const ACCEPTED: &'static [&'static str] = &[
        "exe",
        "path_contains",
        "path_regex",
        "cmdline_contains",
        "descends_from",
    ];

    /// Executable file name glob.
    pub fn exe(&self) -> Option<&ImagePattern> {
        self.exe.as_ref()
    }

    /// Substring of the full image path.
    pub fn path_contains(&self) -> Option<&str> {
        self.path_contains.as_deref()
    }

    /// Regular expression against the full image path.
    pub fn path_regex(&self) -> Option<&PathRegex> {
        self.path_regex.as_ref()
    }

    /// Substring of the command line.
    pub fn cmdline_contains(&self) -> Option<&str> {
        self.cmdline_contains.as_deref()
    }

    /// Role of an ancestor in the synthetic process tree.
    pub fn descends_from(&self) -> Option<&str> {
        self.descends_from.as_deref()
    }

    /// Whether any predicate is present.
    pub fn is_empty(&self) -> bool {
        self.exe.is_none()
            && self.path_contains.is_none()
            && self.path_regex.is_none()
            && self.cmdline_contains.is_none()
            && self.descends_from.is_none()
    }

    /// Whether this stage carries a predicate other than `exe`.
    ///
    /// The ambiguity check of specification section 15.4 consumes this. A method
    /// rather than a stored flag, so that adding a sixth predicate cannot leave
    /// a stale value behind.
    pub fn is_pinned(&self) -> bool {
        self.path_contains.is_some()
            || self.path_regex.is_some()
            || self.cmdline_contains.is_some()
            || self.descends_from.is_some()
    }

    pub(crate) fn set_exe(&mut self, v: ImagePattern) {
        self.exe = Some(v);
    }

    pub(crate) fn set_path_contains(&mut self, v: String) {
        self.path_contains = Some(v);
    }

    pub(crate) fn set_path_regex(&mut self, v: PathRegex) {
        self.path_regex = Some(v);
    }

    pub(crate) fn set_cmdline_contains(&mut self, v: String) {
        self.cmdline_contains = Some(v);
    }

    pub(crate) fn set_descends_from(&mut self, v: String) {
        self.descends_from = Some(v);
    }
}

/// A named position in the launcher chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stage {
    role: String,
    lifecycle: Lifecycle,
    terminal: bool,
    predicates: MatchPredicates,
}

impl Stage {
    /// The accepted key names, for a diagnostic that has to list them.
    pub const ACCEPTED: &'static [&'static str] = &["role", "lifecycle", "terminal", "match"];

    pub(crate) fn new(
        role: String,
        lifecycle: Lifecycle,
        terminal: bool,
        predicates: MatchPredicates,
    ) -> Stage {
        Stage {
            role,
            lifecycle,
            terminal,
            predicates,
        }
    }

    /// The role name, unique within the profile.
    pub fn role(&self) -> &str {
        &self.role
    }

    /// How an exit is treated.
    pub fn lifecycle(&self) -> Lifecycle {
        self.lifecycle
    }

    /// Whether this stage's exit ends the capture. At most one stage per
    /// profile, and its lifecycle is always `session`.
    pub fn is_terminal(&self) -> bool {
        self.terminal
    }

    /// The predicates that must all hold.
    pub fn predicates(&self) -> &MatchPredicates {
        &self.predicates
    }
}

/// The identity of the game a profile describes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    id: GameId,
    name: String,
    platform: Option<String>,
    app_id: Option<String>,
}

impl Game {
    /// The accepted key names, for a diagnostic that has to list them.
    pub const ACCEPTED: &'static [&'static str] = &["id", "name", "platform", "app_id"];

    pub(crate) fn new(
        id: GameId,
        name: String,
        platform: Option<String>,
        app_id: Option<String>,
    ) -> Game {
        Game {
            id,
            name,
            platform,
            app_id,
        }
    }

    /// The slug, used for profile resolution.
    pub fn id(&self) -> &GameId {
        &self.id
    }

    /// The display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The platform for managed launch, if the profile declared one.
    ///
    /// Left unconstrained in schema version 1. Section 16 gives it meaning, and
    /// constraining it to a known set now would be guessing at that slice's
    /// vocabulary from outside it.
    pub fn platform(&self) -> Option<&str> {
        self.platform.as_deref()
    }

    /// The platform application identifier, if the profile declared one.
    ///
    /// A string even when it looks numeric, because a platform identifier is an
    /// opaque token and leading zeros would matter if one ever had them.
    pub fn app_id(&self) -> Option<&str> {
        self.app_id.as_deref()
    }
}

/// Capture defaults, overridable on the command line.
///
/// Every field is optional and none carries a substituted default, so that a
/// caller can tell a profile that chose a value from one that said nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CaptureDefaults {
    mode: Option<CaptureMode>,
    duration: Option<Duration>,
    roles: Option<Vec<String>>,
    loopback: Option<bool>,
    payload: Option<bool>,
}

impl CaptureDefaults {
    /// The accepted key names, for a diagnostic that has to list them.
    ///
    /// Exactly the five specification section 15.2 declares. Section 17.2 lists
    /// more capture options on the command line, and a profile key with no
    /// consumer is a key whose behavior is untested and whose meaning is set by
    /// whoever first reads it. S14 owns the command line and adds the keys it
    /// can honor.
    pub const ACCEPTED: &'static [&'static str] =
        &["mode", "duration", "roles", "loopback", "payload"];

    /// The declared capture mode.
    pub fn mode(&self) -> Option<CaptureMode> {
        self.mode
    }

    /// The declared duration bound, as a parsed span.
    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }

    /// The declared roles to capture.
    pub fn roles(&self) -> Option<&[String]> {
        self.roles.as_deref()
    }

    /// Whether the loopback adapter is included.
    pub fn loopback(&self) -> Option<bool> {
        self.loopback
    }

    /// Whether payloads are captured.
    pub fn payload(&self) -> Option<bool> {
        self.payload
    }

    pub(crate) fn set_mode(&mut self, v: CaptureMode) {
        self.mode = Some(v);
    }

    pub(crate) fn set_duration(&mut self, v: Duration) {
        self.duration = Some(v);
    }

    pub(crate) fn set_roles(&mut self, v: Vec<String>) {
        self.roles = Some(v);
    }

    pub(crate) fn set_loopback(&mut self, v: bool) {
        self.loopback = Some(v);
    }

    pub(crate) fn set_payload(&mut self, v: bool) {
        self.payload = Some(v);
    }
}

/// The targeting trust tier of specification section 15.6.
///
/// How trustworthy a target definition is, which the resolver of section 15.7
/// ranks by. Entirely separate from the attribution fidelity of section 13.4
/// (`fragcap_core::attribution::Fidelity`, one of `Live`, `Retained`, `None`),
/// which is how a captured packet was attributed. The two are different axes on
/// different types, and neither is derived from the other; the `Observed` tier
/// here is not the same thing as a live attribution.
///
/// The variants are declared in ascending trust order on purpose, so the derived
/// [`Ord`] makes the more trusted tier the greater value:
/// `Authored > Verified > HeuristicUnverified > Observed`. A resolver that ranks
/// by trust then compares tiers the way it reads, and a provider precedence that
/// inverted the order would be caught by comparing against this.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FidelityTier {
    /// A confirmed live result: a running process the tool observed itself. The
    /// bottom of the cascade and the arbiter of last resort.
    Observed,
    /// A machine's guess (a shipped hint, an engine rule, a platform walker) that
    /// has not been confirmed against a live capture.
    HeuristicUnverified,
    /// A curated definition that has been verified against a live capture.
    Verified,
    /// A definition a human authored directly. The highest trust.
    Authored,
}

impl FidelityTier {
    /// The accepted spellings, highest trust first, for a diagnostic or listing.
    pub const ACCEPTED: &'static [&'static str] =
        &["authored", "verified", "heuristic-unverified", "observed"];

    /// Parse the schema spelling.
    pub fn parse(s: &str) -> Option<FidelityTier> {
        match s {
            "authored" => Some(FidelityTier::Authored),
            "verified" => Some(FidelityTier::Verified),
            "heuristic-unverified" => Some(FidelityTier::HeuristicUnverified),
            "observed" => Some(FidelityTier::Observed),
            _ => None,
        }
    }

    /// The schema spelling, so `parse(t.as_str()) == Some(t)` for every tier.
    pub fn as_str(&self) -> &'static str {
        match self {
            FidelityTier::Authored => "authored",
            FidelityTier::Verified => "verified",
            FidelityTier::HeuristicUnverified => "heuristic-unverified",
            FidelityTier::Observed => "observed",
        }
    }
}

/// The artifact form of specification section 15.6, the `kind` discriminator.
///
/// The profile-load path accepts the two authoritative forms, [`Kind::Profile`]
/// and [`Kind::Package`] (an authored target package is structurally a profile at
/// the highest fidelity). It refuses the loose forms [`Kind::Hint`] and
/// [`Kind::Export`], which are not capture profiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A curated or verified capture profile.
    Profile,
    /// A user-authored target package. Loadable as a capture profile.
    Package,
    /// A heuristic guess. Not loadable as a capture profile.
    Hint,
    /// An export envelope of loose records. Not loadable as a capture profile.
    Export,
}

impl Kind {
    /// Parse the schema spelling.
    pub fn parse(s: &str) -> Option<Kind> {
        match s {
            "profile" => Some(Kind::Profile),
            "package" => Some(Kind::Package),
            "hint" => Some(Kind::Hint),
            "export" => Some(Kind::Export),
            _ => None,
        }
    }

    /// The schema spelling.
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::Profile => "profile",
            Kind::Package => "package",
            Kind::Hint => "hint",
            Kind::Export => "export",
        }
    }
}

/// Where a target artifact came from, per the schema's `provenance`.
///
/// The `source` is an opaque label a provider stamps (for example `user`,
/// `steam-appinfo`, `engine-rule`, `runtime-observation`); `seeded_at` records
/// when a generated record was last refreshed. Carried, never inferred (P-9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provenance {
    source: String,
    seeded_at: Option<String>,
}

impl Provenance {
    pub(crate) fn new(source: String, seeded_at: Option<String>) -> Provenance {
        Provenance { source, seeded_at }
    }

    /// The origin label.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// When the record was generated or last refreshed, if declared.
    pub fn seeded_at(&self) -> Option<&str> {
        self.seeded_at.as_deref()
    }
}

/// A validated game profile.
///
/// Every invariant the validation set of specification section 15.4 checks holds
/// of any value of this type. In particular the stage set is non-empty, in
/// declaration order, contains at least one non-service stage and at most one
/// terminal stage, has unique role names, has a `descends_from` relation that is
/// acyclic and resolves within the set, and contains no ambiguous image match.
///
/// It also carries the section 15.6 metadata the resolver reads: the `kind`, the
/// `fidelity` tier, an optional `provenance`, and optional `notes`. These are
/// validated structurally at load and retained rather than discarded, so a later
/// consumer can read how trustworthy the definition is without re-parsing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    game: Game,
    capture: CaptureDefaults,
    stages: Vec<Stage>,
    kind: Kind,
    fidelity: FidelityTier,
    provenance: Option<Provenance>,
    notes: Option<String>,
}

impl Profile {
    /// The accepted top level key names, for a diagnostic that has to list them.
    ///
    /// The master schema (section 15.6) governs unknown-key refusal; this list
    /// stays in step with it so a fragcap-side message names the same keys.
    pub const ACCEPTED: &'static [&'static str] = &[
        "schema",
        "kind",
        "fidelity",
        "provenance",
        "notes",
        "game",
        "capture",
        "stage",
    ];

    pub(crate) fn new(
        game: Game,
        capture: CaptureDefaults,
        stages: Vec<Stage>,
        kind: Kind,
        fidelity: FidelityTier,
        provenance: Option<Provenance>,
        notes: Option<String>,
    ) -> Profile {
        Profile {
            game,
            capture,
            stages,
            kind,
            fidelity,
            provenance,
            notes,
        }
    }

    /// The schema version this profile was validated against.
    ///
    /// Always [`SCHEMA_VERSION`]. Present as an accessor rather than a field so
    /// that a later build supporting two versions does not change the shape of
    /// every caller.
    pub fn schema(&self) -> u32 {
        SCHEMA_VERSION
    }

    /// The game's identity.
    pub fn game(&self) -> &Game {
        &self.game
    }

    /// The capture defaults.
    pub fn capture(&self) -> &CaptureDefaults {
        &self.capture
    }

    /// The stages, in declaration order.
    pub fn stages(&self) -> &[Stage] {
        &self.stages
    }

    /// The stage whose exit ends the capture, if the profile declares one.
    pub fn terminal_stage(&self) -> Option<&Stage> {
        self.stages.iter().find(|s| s.is_terminal())
    }

    /// The stage with the given role, if there is one.
    pub fn stage(&self, role: &str) -> Option<&Stage> {
        self.stages.iter().find(|s| s.role() == role)
    }

    /// The artifact form this profile declared, one of [`Kind::Profile`] or
    /// [`Kind::Package`] (the load path refuses the loose kinds).
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// The declared trust tier the resolver ranks by (section 15.6).
    pub fn fidelity(&self) -> FidelityTier {
        self.fidelity
    }

    /// Where this profile came from, if it declared a provenance. A profile is
    /// not required to declare one; only the loose artifacts are.
    pub fn provenance(&self) -> Option<&Provenance> {
        self.provenance.as_ref()
    }

    /// Human-readable context carried as data, if the profile declared any. The
    /// scaffold uses this to carry its heuristic-verification warning.
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slug_charset_is_accepted() {
        for s in ["eso", "div2", "a", "a-b", "a_b", "eso64", "0"] {
            assert!(GameId::is_valid(s), "expected {s:?} to be a valid slug");
        }
    }

    #[test]
    fn a_slug_that_could_escape_a_search_directory_is_refused() {
        // The reason this is a character set rather than a length check. A
        // reference is joined to a directory during resolution, and every value
        // here would reach outside it or name something other than a profile.
        for s in [
            "",
            "..",
            "../etc",
            "a/b",
            "a\\b",
            "C:",
            "C:\\Windows",
            "ESO",
            "e s o",
            "eso.toml",
            "eso*",
        ] {
            assert!(!GameId::is_valid(s), "expected {s:?} to be refused");
            assert_eq!(GameId::new(s), None);
        }
    }

    #[test]
    fn lifecycle_and_mode_accept_only_their_declared_spellings() {
        assert_eq!(Lifecycle::parse("transient"), Some(Lifecycle::Transient));
        assert_eq!(Lifecycle::parse("session"), Some(Lifecycle::Session));
        assert_eq!(Lifecycle::parse("service"), Some(Lifecycle::Service));
        assert_eq!(Lifecycle::parse("Session"), None);
        assert_eq!(Lifecycle::parse("persistent"), None);

        assert_eq!(CaptureMode::parse("file"), Some(CaptureMode::File));
        assert_eq!(CaptureMode::parse("stream"), Some(CaptureMode::Stream));
        assert_eq!(CaptureMode::parse("ring"), Some(CaptureMode::Ring));
        assert_eq!(CaptureMode::parse("File"), None);
    }

    #[test]
    fn a_path_regex_compiles_and_keeps_its_source() {
        let r = PathRegex::new(r"(?i)elder\s+scrolls").expect("compiles");
        assert_eq!(r.as_str(), r"(?i)elder\s+scrolls");
        assert!(r.regex().is_match("Elder  Scrolls Online"));
    }

    #[test]
    fn path_regex_equality_is_on_the_source() {
        let a = PathRegex::new("a+").expect("compiles");
        let b = PathRegex::new("a+").expect("compiles");
        let c = PathRegex::new("b+").expect("compiles");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn fidelity_tier_parses_only_its_schema_spellings_and_round_trips() {
        for (s, t) in [
            ("authored", FidelityTier::Authored),
            ("verified", FidelityTier::Verified),
            ("heuristic-unverified", FidelityTier::HeuristicUnverified),
            ("observed", FidelityTier::Observed),
        ] {
            assert_eq!(FidelityTier::parse(s), Some(t));
            assert_eq!(t.as_str(), s);
            assert_eq!(FidelityTier::parse(t.as_str()), Some(t));
        }
        assert_eq!(FidelityTier::parse("Authored"), None);
        assert_eq!(FidelityTier::parse("heuristic"), None);
        assert_eq!(FidelityTier::parse(""), None);
    }

    #[test]
    fn fidelity_tier_orders_more_trusted_as_greater() {
        // The whole point of the ascending declaration: a resolver ranking by
        // trust compares the way it reads. Authored is the most trusted.
        assert!(FidelityTier::Authored > FidelityTier::Verified);
        assert!(FidelityTier::Verified > FidelityTier::HeuristicUnverified);
        assert!(FidelityTier::HeuristicUnverified > FidelityTier::Observed);
        let mut tiers = [
            FidelityTier::Verified,
            FidelityTier::Observed,
            FidelityTier::Authored,
            FidelityTier::HeuristicUnverified,
        ];
        tiers.sort();
        assert_eq!(
            tiers,
            [
                FidelityTier::Observed,
                FidelityTier::HeuristicUnverified,
                FidelityTier::Verified,
                FidelityTier::Authored,
            ]
        );
    }

    #[test]
    fn kind_parses_only_its_schema_spellings_and_round_trips() {
        for (s, k) in [
            ("profile", Kind::Profile),
            ("package", Kind::Package),
            ("hint", Kind::Hint),
            ("export", Kind::Export),
        ] {
            assert_eq!(Kind::parse(s), Some(k));
            assert_eq!(k.as_str(), s);
        }
        assert_eq!(Kind::parse("Profile"), None);
        assert_eq!(Kind::parse("bundle"), None);
    }

    #[test]
    fn predicates_report_emptiness_and_pinning_separately() {
        let mut p = MatchPredicates::default();
        assert!(p.is_empty());
        assert!(!p.is_pinned());

        p.set_exe(ImagePattern::new("eso64.exe").expect("pattern"));
        assert!(!p.is_empty());
        assert!(
            !p.is_pinned(),
            "a stage matching on exe alone is not pinned, which is what the \
             ambiguity check of section 15.4 turns on"
        );

        p.set_descends_from("anticheat".to_string());
        assert!(p.is_pinned());
    }
}
